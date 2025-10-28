<script>
	import CommunityThemeCard from '$lib/themesPage/CommunityThemeCard.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	let themeData = $state({
		items: [],
		error: null,
		isLoading: false,
		isRefreshing: false
	});

	function transformTheme(theme) {
		if (!theme) return null;
		const detailUrl = theme.detail_url ?? theme.detailUrl ?? '';
		const imageUrl = theme.image_url ?? theme.imageUrl ?? '';
		const author = theme.author ?? '';
		const slug = theme.slug ?? detailUrl;
		const githubUrl = theme.github_url ?? theme.githubUrl ?? '';
		const installCommand = theme.install_command ?? theme.installCommand ?? '';
		const installUrl = theme.install_url ?? theme.installUrl ?? '';

		return {
			title: theme.title ?? slug ?? 'Community Theme',
			author,
			imageUrl,
			detailUrl,
			slug,
			githubUrl,
			installCommand,
			installUrl
		};
	}

	function formatErrorDetails(err) {
		if (!err) return 'Unknown error';
		if (typeof err === 'string') return err;
		if (err.message) return err.message;
		try {
			return JSON.stringify(err);
		} catch {
			return String(err);
		}
	}

	async function loadCommunityThemes(forceRefresh = false) {
		if (themeData.isLoading || themeData.isRefreshing) return;

	themeData.isLoading = !forceRefresh;
	themeData.isRefreshing = forceRefresh;
	themeData.error = null;

	try {
		const themes = await invoke('get_community_themes', { forceRefresh });
		const mapped = (themes || []).map(transformTheme).filter(Boolean);
		console.info('Community themes loaded', { count: mapped.length });
		themeData.items = mapped;
	} catch (err) {
		console.error('Failed to load community themes:', err);
		themeData.error = formatErrorDetails(err);
	} finally {
		themeData.isLoading = false;
		themeData.isRefreshing = false;
	}
}

	onMount(() => {
		loadCommunityThemes(false);
	});
</script>

{#if themeData.isLoading}
	<div class="flex items-center justify-center p-8">
		<Skeleton class="h-32 w-full max-w-xl" />
	</div>
{:else if themeData.error}
	<div class="flex flex-col items-center justify-center gap-4 p-8 text-center">
		<div class="text-red-500">Error loading community themes: {themeData.error}</div>
		<Button variant="outline" disabled={themeData.isRefreshing} onclick={() => loadCommunityThemes(true)}>
			{#if themeData.isRefreshing}
				Refreshing...
			{:else}
				Try Again
			{/if}
		</Button>
	</div>
{:else}
	<div class="flex flex-col gap-6">
		{#if themeData.items.length > 0}
			<section>
				<div class="grid grid-cols-1 gap-4 lg:grid-cols-2 xl:grid-cols-3">
					{#each themeData.items as theme (theme.slug)}
						<CommunityThemeCard
							title={theme.title}
							author={theme.author}
							imageUrl={theme.imageUrl}
							detailUrl={theme.detailUrl}
							githubUrl={theme.githubUrl}
							installCommand={theme.installCommand}
							installUrl={theme.installUrl}
						/>
					{/each}
				</div>
			</section>
		{:else}
			<div class="flex items-center justify-center p-8">
				<div class="text-gray-500">No community themes found</div>
			</div>
		{/if}
	</div>
{/if}
