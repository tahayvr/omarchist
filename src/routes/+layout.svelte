<script>
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { startThemePreloading } from '../lib/services/themePreloader.js';
	import { loadSettings } from '../lib/utils/settingsUtils.js';

	let { children } = $props();
	let themeRefreshUnlisten = null;

	// App settings state using Svelte 5 runes
	const appSettings = $state({
		autoApplyTheme: true,
		isLoading: false,
		error: null,
		isInitialized: false
	});

	/**
	 * Apply theme colors to CSS custom properties
	 * @param {Object} colors - Theme colors object
	 * @param {string} colors.background - Background color
	 * @param {string} colors.foreground - Foreground color
	 * @returns {boolean} - Success status
	 */
	function applyTheme(colors) {
		try {
			// Validate color values
			if (!colors || typeof colors !== 'object') {
				throw new Error('Invalid colors object provided');
			}

			if (!colors.background || !colors.foreground) {
				throw new Error('Missing required color properties (background, foreground)');
			}

			// Validate color format (basic check for hex or color function)
			const colorPattern = /^(#[0-9a-fA-F]{6}|#[0-9a-fA-F]{3}|oklch\(.*\)|rgb\(.*\)|hsl\(.*\))$/;
			if (!colorPattern.test(colors.background) && !colorPattern.test(colors.foreground)) {
				console.warn('Color values may not be in expected format:', colors);
			}

			// Apply theme colors to CSS custom properties
			const root = document.documentElement;
			root.style.setProperty('--background', colors.background);
			root.style.setProperty('--foreground', colors.foreground);

			// Update related theme variables that depend on background/foreground
			root.style.setProperty('--card', colors.background);
			root.style.setProperty('--card-foreground', colors.foreground);
			root.style.setProperty('--popover', colors.background);
			root.style.setProperty('--popover-foreground', colors.foreground);
			root.style.setProperty('--primary', colors.background);
			root.style.setProperty('--sidebar', colors.background);
			root.style.setProperty('--sidebar-foreground', colors.foreground);
			root.style.setProperty('--sidebar-primary', colors.background);
			root.style.setProperty('--sidebar-primary-foreground', colors.foreground);
			root.style.setProperty('--sidebar-border', colors.foreground);

			return true;
		} catch (error) {
			console.error('Failed to apply theme colors:', error);
			return false;
		}
	}

	/**
	 * Load initial theme colors from backend
	 */
	async function loadInitialTheme() {
		try {
			const colors = await invoke('get_system_theme_colors');

			if (colors && colors.background && colors.foreground) {
				const success = applyTheme(colors);
				if (!success) {
					console.error('Failed to apply initial theme colors');
				}
			} else {
				console.warn('Invalid or incomplete theme colors received:', colors);
			}
		} catch (error) {
			console.error('Could not load initial system theme colors:', error);
			// Application continues with default theme
		}
	}

	/**
	 * Handle theme refresh events from backend
	 * @param {Object} event - Tauri event object
	 */
	function handleThemeRefresh(event) {
		try {
			const colors = event.payload;

			const success = applyTheme(colors);
			if (!success) {
				console.error('Theme refresh failed - maintaining current theme');
			}
		} catch (error) {
			console.error('Error handling theme refresh event:', error);
			// Maintain current theme on error
		}
	}

	onMount(async () => {
		// Initialize app settings first - critical for app functionality
		try {
			const settingsLoaded = await loadSettings(appSettings);
			if (!settingsLoaded) {
				console.warn('App settings initialization failed, using defaults');
			}
		} catch (error) {
			console.error('Critical error during settings initialization:', error);
			// Application continues with default settings
		}

		// Load initial theme using the same mechanism as CLI refresh
		await loadInitialTheme();

		// Start background theme preloading
		startThemePreloading()
			.then((success) => {
				if (!success) {
					console.warn('Theme preloading failed, themes will load on demand');
				}
			})
			.catch((error) => {
				console.error('Unexpected error during theme preloading:', error);
			});

		// Initialize theme refresh event listener
		try {
			const { listen } = await import('@tauri-apps/api/event');

			themeRefreshUnlisten = await listen('theme-refresh', handleThemeRefresh);
		} catch (error) {
			console.error('Failed to initialize theme refresh event listener:', error);
			// Application continues to work without live theme updates
		}
	});

	onDestroy(() => {
		// Cleanup theme refresh listener
		if (themeRefreshUnlisten) {
			try {
				themeRefreshUnlisten();
			} catch (error) {
				console.error('Error during theme refresh event listener cleanup:', error);
			}
		}
	});
</script>

<ModeWatcher />

{@render children()}
