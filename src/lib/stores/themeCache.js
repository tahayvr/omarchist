import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// UI state only - backend cache is the single source of truth
const _loading = writable(false);
const _error = writable(null);

export const loading = derived(_loading, ($loading) => $loading);
export const error = derived(_error, ($error) => $error);

// Internal state for request deduplication
let activeRequest = null;

/**
 * Loads themes from the backend cache (single source of truth)
 * @returns {Promise<Array>} Array of theme objects
 */
async function loadThemesFromBackend() {
	try {
		// Always use the cached backend command - backend cache is the source of truth
		const themes = await invoke('get_themes_cached');
		return Array.isArray(themes) ? themes : [];
	} catch (error) {
		console.error('Failed to load themes from backend cache:', error);
		throw new Error(`Failed to load themes: ${error.message || error}`);
	}
}

/**
 * Gets themes from backend cache
 * @param {boolean} forceRefresh - Force refresh of backend cache
 * @param {boolean} optimistic - If true, don't show loading state immediately
 * @returns {Promise<Array>}
 */
export async function getThemes(forceRefresh = false, optimistic = false) {
	// Deduplicate concurrent requests
	if (activeRequest) {
		return activeRequest;
	}

	let loadingTimeout = null;
	if (!optimistic) {
		_loading.set(true);
	} else {
		loadingTimeout = setTimeout(() => {
			_loading.set(true);
		}, 100);
	}

	_error.set(null);

	activeRequest = (async () => {
		try {
			if (forceRefresh) {
				return await invoke('invalidate_and_refresh_cache');
			} else {
				return await loadThemesFromBackend();
			}
		} catch (error) {
			_error.set(error.message || 'Failed to load themes');
			throw error;
		}
	})().finally(() => {
		if (loadingTimeout) {
			clearTimeout(loadingTimeout);
		}
		_loading.set(false);
		activeRequest = null;
	});

	return activeRequest;
}

/**
 * Refreshes the backend cache and returns fresh themes
 * @param {boolean} silent - If true, doesn't show loading state
 * @returns {Promise<Array>} Array of theme objects
 */
export async function refreshThemes(silent = false) {
	if (!silent) {
		_loading.set(true);
	}

	try {
		const themes = await invoke('invalidate_and_refresh_cache');
		_error.set(null);
		return themes;
	} catch (error) {
		_error.set(error.message || 'Failed to refresh themes');
		throw error;
	} finally {
		if (!silent) {
			_loading.set(false);
		}
	}
}

// Invalidates the backend cache
export async function invalidateCache() {
	try {
		await invoke('refresh_theme_cache');
		_error.set(null);
		console.log('Backend theme cache invalidated');
	} catch (error) {
		console.error('Failed to invalidate backend cache:', error);
		_error.set(error.message || 'Failed to invalidate cache');
	}
}

// Invalidates cache for a specific theme
export async function invalidateTheme(themeDir) {
	try {
		await invoke('invalidate_theme_cache', { themeDir });
		console.log('Backend cache invalidated for theme:', themeDir);
	} catch (error) {
		console.error('Failed to invalidate theme cache:', error);
	}
}

// Invalidates cache for multiple themes
export async function invalidateThemes(themeDirs) {
	try {
		await invoke('invalidate_themes_cache', { themeDirs });
		console.log('Backend cache invalidated for themes:', themeDirs);
	} catch (error) {
		console.error('Failed to invalidate themes cache:', error);
	}
}

/**
 * Preloads themes in the backend cache without affecting UI state
 * @returns {Promise<void>}
 */
export async function preloadThemes() {
	try {
		await invoke('preload_themes');
		console.log('Backend themes preloaded successfully');
	} catch (error) {
		console.warn('Failed to preload themes in backend:', error);
		// Don't throw error for preload failures
	}
}

/**
 * Gets system themes only (filtered from backend cache)
 * @param {boolean} optimistic - If true, use optimistic loading (no immediate loading state)
 * @returns {Promise<Array>} Array of system theme objects
 */
export async function getSystemThemes(optimistic = true) {
	const allThemes = await getThemes(false, optimistic);
	// System themes are explicitly flagged by the backend.
	// (Community/unknown themes may have both flags false.)
	return allThemes.filter((theme) => theme?.is_system === true);
}

/**
 * Gets custom themes only (filtered from backend cache)
 * @param {boolean} optimistic - If true, use optimistic loading (no immediate loading state)
 * @returns {Promise<Array>} Array of custom theme objects
 */
export async function getCustomThemes(optimistic = true) {
	const allThemes = await getThemes(false, optimistic);
	// Custom/user themes are explicitly flagged by the backend.
	return allThemes.filter((theme) => theme?.is_custom === true);
}

/**
 * Gets backend cache information for debugging
 * @returns {Promise<Object>} Cache information object
 */
export async function getCacheInfo() {
	try {
		const stats = await invoke('get_cache_stats');
		return stats;
	} catch (error) {
		console.error('Failed to get cache info:', error);
		return { error: error.message };
	}
}

export const themeCache = {
	loading,
	error,
	get: getThemes,
	refresh: refreshThemes,
	invalidate: invalidateCache,
	invalidateTheme,
	invalidateThemes,
	preload: preloadThemes,
	getSystemThemes,
	getCustomThemes,
	getCacheInfo
};
