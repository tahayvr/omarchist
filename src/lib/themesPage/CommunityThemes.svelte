<script>
	import ThemeCard from '$lib/themesPage/ThemeCard.svelte';
	import { themeCache } from '$lib/stores/themeCache.js';
	import { onMount } from 'svelte';

	let communityThemes = $state([]);
	let localError = $state(null);
	let hasLoaded = $state(false);

	// Use the global loading state from theme cache
	const { loading, error } = themeCache;

	async function loadCommunityThemes() {
		try {
			// Use optimistic loading - don't show loading immediately for cache hits
			const allThemes = await themeCache.get(false, true);
			// Community themes are neither system nor custom
			communityThemes = (allThemes || []).filter((t) => !t?.is_system && !t?.is_custom);
			localError = null;
		} catch (err) {
			console.error('Failed to load community themes:', err);
			localError = err?.message ?? String(err);
		} finally {
			hasLoaded = true;
		}
	}

	onMount(() => {
		loadCommunityThemes();
	});

	// Use either global error or local error
	const displayError = $derived(error.value || localError);
</script>

{#if loading.value}
	<div class="flex items-center justify-center p-8">
		<div class="text-lg">Loading community themes...</div>
	</div>
{:else if displayError}
	<div class="flex items-center justify-center p-8">
		<div class="text-red-500">Error loading community themes: {displayError}</div>
	</div>
{:else}
	<div class="flex flex-col gap-6">
		{#if communityThemes.length > 0}
			<section>
				<div class="grid grid-cols-1 gap-4 lg:grid-cols-2 xl:grid-cols-3">
					{#each communityThemes as theme (theme.dir || theme.title)}
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
		{:else if hasLoaded && !loading.value}
			<div class="flex items-center justify-center p-8">
				<div class="text-gray-500">No community themes found</div>
			</div>
		{/if}
	</div>
{/if}
