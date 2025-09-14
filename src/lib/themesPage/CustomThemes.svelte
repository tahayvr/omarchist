<script>
	import ThemeCard from '$lib/themesPage/ThemeCard.svelte';
	import { themeCache } from '$lib/stores/themeCache.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { onMount } from 'svelte';

	let customThemes = $state([]);
	let localError = $state(null);
	let hasLoaded = $state(false);

	// Use the global loading state from theme cache
	const { loading, error } = themeCache;

	async function loadCustomThemes() {
		try {
			const themes = await themeCache.getCustomThemes(true);

			function sortKey(t) {
				return (t.title || t.dir || '').toLowerCase();
			}

			themes.sort((a, b) => {
				const keyA = sortKey(a);
				const keyB = sortKey(b);
				return keyA.localeCompare(keyB);
			});

			customThemes = themes;
			localError = null;
		} catch (err) {
			console.error('Failed to load custom themes:', err);
			localError = err.message || 'Failed to load themes';
		} finally {
			hasLoaded = true;
		}
	}

	onMount(() => {
		loadCustomThemes();
	});

	// Use either global error or local error
	const displayError = $derived(error.value || localError);
</script>

{#if loading.value}
	<div class="flex items-center justify-center p-8">
		<Skeleton />
	</div>
{:else if displayError}
	<div class="flex items-center justify-center p-8">
		<div class="text-red-500">Error loading custom themes: {displayError}</div>
	</div>
{:else}
	<div class="flex flex-col gap-6">
		{#if customThemes.length > 0}
			<section>
				<div class="grid grid-cols-1 gap-4 lg:grid-cols-2 xl:grid-cols-3">
					{#each customThemes as theme}
						<ThemeCard
							dir={theme.dir}
							title={theme.title}
							imageUrl={theme.image}
							is_system={theme.is_system}
							is_custom={theme.is_custom}
							colors={theme.colors}
							onDeleted={loadCustomThemes}
						/>
					{/each}
				</div>
			</section>
		{:else if hasLoaded && !loading.value}
			<div class="flex items-center justify-center p-8">
				<div class="text-gray-500">No custom themes found</div>
			</div>
		{/if}
	</div>
{/if}
