<script>
	import ThemeCard from '$lib/themesPage/ThemeCard.svelte';
	import { themeCache } from '$lib/stores/themeCache.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { onMount } from 'svelte';

	let systemThemes = $state([]);
	let localError = $state(null);
	let hasLoaded = $state(false);

	const { loading, error } = themeCache;

	async function loadSystemThemes() {
		try {
			// Use optimistic loading - don't show loading immediately for cache hits
			const themes = await themeCache.getSystemThemes(true);
			systemThemes = themes;
			localError = null;
		} catch (err) {
			console.error('Failed to load system themes:', err);
			localError = err.message || 'Failed to load themes';
		} finally {
			hasLoaded = true;
		}
	}

	onMount(() => {
		loadSystemThemes();
	});

	const displayError = $derived(error.value || localError);
</script>

{#if loading.value}
	<div class="flex items-center justify-center p-8">
		<Skeleton />
	</div>
{:else if displayError}
	<div class="flex items-center justify-center p-8">
		<div class="text-red-500">Error loading themes: {displayError}</div>
	</div>
{:else}
	<div class="flex flex-col gap-6">
		<!-- List System Themes -->
		{#if systemThemes.length > 0}
			<section>
				<div class="grid grid-cols-1 gap-4 lg:grid-cols-2 xl:grid-cols-3">
					{#each systemThemes as theme}
						<ThemeCard
							dir={theme.dir}
							title={theme.title}
							imageUrl={theme.image}
							is_system={theme.is_system}
							is_custom={theme.is_custom}
							colors={theme.colors}
						/>
					{/each}
				</div>
			</section>
		{/if}

		<!-- No themes found - only show after loading is complete -->
		{#if systemThemes.length === 0 && hasLoaded && !loading.value}
			<div class="flex items-center justify-center p-8">
				<div class="text-gray-500">No themes found</div>
			</div>
		{/if}
	</div>
{/if}
