<script>
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { startThemePreloading } from '../lib/services/themePreloader.js';
	import { themeCache } from '../lib/stores/themeCache.js';
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

	async function loadInitialTheme() {
		try {
			await invoke('refresh_theme_adjustments');
			console.log('Initial theme refresh requested successfully');
		} catch (error) {
			console.error('Failed to refresh theme adjustments on startup:', error);
		}
	}

	async function handleThemeRefresh() {
		try {
			await themeCache.refresh(true);
			console.log('Theme cache refreshed after backend update');
		} catch (error) {
			console.error('Failed to refresh theme cache after event:', error);
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
