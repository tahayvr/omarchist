import { themeCache } from '../stores/themeCache.js';

/**
 * Configuration for theme preloader
 */
const PRELOADER_CONFIG = {
	// Delay before starting preload to avoid blocking app startup
	startupDelayMs: 100,
	// Maximum time to wait for preload before giving up
	timeoutMs: 10000,
	// Whether to retry on failure
	retryOnFailure: true,
	// Maximum number of retry attempts
	maxRetries: 2,
	// Delay between retry attempts
	retryDelayMs: 1000
};

/**
 * Theme preloader service that handles background theme loading on app startup
 */
export class ThemePreloader {
	constructor(config = {}) {
		this.config = { ...PRELOADER_CONFIG, ...config };
		this.isPreloading = false;
		this.preloadPromise = null;
		this.retryCount = 0;
		this.startTime = null;
	}

	/**
	 * Checks if preloading is needed based on cache state
	 * @returns {Promise<boolean>} True if preloading should be performed
	 */
	async shouldPreload() {
		// Don't preload if already in progress
		if (this.isPreloading) {
			return false;
		}

		try {
			// Check backend cache status
			const cacheInfo = await themeCache.getCacheInfo();
			if (cacheInfo.theme_cache_valid && cacheInfo.theme_cache_size > 0) {
				console.log('Theme preloader: Backend cache already valid, skipping preload');
				return false;
			}
		} catch {
			console.warn('Theme preloader: Could not check cache status, proceeding with preload');
		}

		return true;
	}

	/**
	 * Handles preload errors with graceful degradation
	 * @param {Error} error - The error that occurred
	 * @returns {boolean} True if should retry, false otherwise
	 */
	handlePreloadError(error) {
		console.warn(`Theme preloader error (attempt ${this.retryCount + 1}):`, error);

		// Check if we should retry
		if (this.config.retryOnFailure && this.retryCount < this.config.maxRetries) {
			this.retryCount++;
			console.log(
				`Theme preloader: Retrying in ${this.config.retryDelayMs}ms (attempt ${this.retryCount}/${this.config.maxRetries})`
			);
			return true;
		}

		// Max retries reached or retry disabled
		console.error('Theme preloader: Max retries reached or retry disabled, giving up');
		return false;
	}

	/**
	 * Performs the actual theme preloading with timeout and retry logic
	 * @returns {Promise<boolean>} True if successful, false otherwise
	 */
	async performPreload() {
		try {
			// Create timeout promise
			const timeoutPromise = new Promise((_, reject) => {
				setTimeout(() => reject(new Error('Preload timeout')), this.config.timeoutMs);
			});

			// Race between preload and timeout
			await Promise.race([themeCache.preload(), timeoutPromise]);

			console.log(
				`Theme preloader: Successfully preloaded themes in ${Date.now() - this.startTime}ms`
			);
			return true;
		} catch (error) {
			const shouldRetry = this.handlePreloadError(error);

			if (shouldRetry) {
				// Wait before retry
				await new Promise((resolve) => setTimeout(resolve, this.config.retryDelayMs));
				return this.performPreload(); // Recursive retry
			}

			return false;
		}
	}

	/**
	 * Starts theme preloading with startup delay
	 * @returns {Promise<boolean>} Promise that resolves to success status
	 */
	async preloadThemes() {
		// Check if preloading is needed
		if (!(await this.shouldPreload())) {
			return true;
		}

		// Return existing promise if already preloading
		if (this.preloadPromise) {
			return this.preloadPromise;
		}

		this.isPreloading = true;
		this.retryCount = 0;
		this.startTime = Date.now();

		console.log(`Theme preloader: Starting preload with ${this.config.startupDelayMs}ms delay`);

		this.preloadPromise = new Promise((resolve) => {
			const executePreload = async () => {
				try {
					// Add startup delay to avoid blocking app initialization
					if (this.config.startupDelayMs > 0) {
						await new Promise((resolve) => setTimeout(resolve, this.config.startupDelayMs));
					}

					const success = await this.performPreload();
					resolve(success);
				} catch (error) {
					console.error('Theme preloader: Unexpected error during preload:', error);
					resolve(false);
				} finally {
					this.isPreloading = false;
					this.preloadPromise = null;
				}
			};

			executePreload();
		});

		return this.preloadPromise;
	}

	/**
	 * Gets the current preload status
	 * @returns {Object} Status object with preload information
	 */
	getStatus() {
		return {
			isPreloading: this.isPreloading,
			retryCount: this.retryCount,
			maxRetries: this.config.maxRetries,
			startTime: this.startTime,
			elapsedTime: this.startTime ? Date.now() - this.startTime : 0
		};
	}

	/**
	 * Cancels ongoing preload operation
	 */
	cancel() {
		if (this.isPreloading) {
			console.log('Theme preloader: Cancelling preload operation');
			this.isPreloading = false;
			this.preloadPromise = null;
		}
	}
}

// Create and export singleton instance
export const themePreloader = new ThemePreloader();

/**
 * Convenience function to start theme preloading
 * @param {Object} config - Optional configuration overrides
 * @returns {Promise<boolean>} Promise that resolves to success status
 */
export async function startThemePreloading(config = {}) {
	if (config && Object.keys(config).length > 0) {
		// Create new instance with custom config
		const customPreloader = new ThemePreloader(config);
		return customPreloader.preloadThemes();
	}

	// Use singleton instance
	return themePreloader.preloadThemes();
}

/**
 * Gets preload status from singleton instance
 * @returns {Object} Status object
 */
