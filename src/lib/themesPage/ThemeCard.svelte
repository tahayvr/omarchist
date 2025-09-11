<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { warn } from '@tauri-apps/plugin-log';
	import ColorPalette from './ColorPalette.svelte';
	import { goto } from '$app/navigation';
	import { themeApplyState } from '$lib/stores/themeApplyState';
	import { get } from 'svelte/store';

	let { dir, title, imageUrl = '', is_system, is_custom, colors = null } = $props();

	const isApplying = $derived($themeApplyState === dir);

	async function applyTheme(themeDir) {
		let applying_theme = get(themeApplyState);
		if (applying_theme != null) {
			await warn("Skipped applying theme for lock. Already applying `", applying_theme, "`");
			return;
		}
		themeApplyState.set(themeDir);
		try {
			await invoke('apply_theme', { dir: themeDir });
		} catch (e) {
			// Handle error
		} finally {
			themeApplyState.set(null);
		}
	}

	async function applyThemeIfEnabled(themeDir) {
		try {
			const settings = await invoke('get_app_settings');
			if (settings.auto_apply_theme) {
				await applyTheme(themeDir);
			}
		} catch (err) {
			console.error('Failed to check auto_apply_theme setting:', err);
			// If we can't get settings, don't apply theme to be safe
		}
	}

	async function editTheme(themeDir) {
		await applyThemeIfEnabled(themeDir);
		// Navigate to the theme editor page
		goto(`/themes/${themeDir}`);
	}
</script>

<div class="flex w-full items-center justify-center">
	<Card.Root class="h-full w-full">
		<Card.Header class="flex flex-row items-center justify-between gap-4">
			<div class="flex items-end gap-8">
				<div class="flex flex-col gap-2 uppercase">
					<Card.Title>{title}</Card.Title>
				</div>
				<div>
					<!-- <Button variant="link">Install</Button> -->
				</div>
			</div>
			<div class="flex flex-row gap-2">
				{#if is_custom}
					<Badge variant="primary" class="text-muted-foreground bg-muted dark:bg-muted/50">
						Custom
					</Badge>
				{:else if is_system}
					<Badge variant="secondary" class="text-muted-foreground">System</Badge>
				{:else}
					<Badge variant="primary" class="text-muted-foreground">Community</Badge>
				{/if}
			</div>
		</Card.Header>
		<Card.Content>
			{#if imageUrl}
				<img src={imageUrl} alt={title} class="aspect-video w-full bg-black" />
			{:else}
				<ColorPalette {colors} />
			{/if}
		</Card.Content>
		<Card.Footer class="flex items-center justify-between uppercase">
			<Button
				variant={isApplying ? "outline" : "ghost"}
				size="sm"
				class="uppercase"
				onclick={() => applyTheme(dir)}
				disabled={isApplying}
			>
				{isApplying ? 'Applying...' : 'Apply Theme'}
			</Button>
			{#if is_custom}
				<Button variant="ghost" size="sm" class="uppercase" onclick={() => editTheme(dir)}>
					Edit Theme
				</Button>
			{/if}
		</Card.Footer>
	</Card.Root>
</div>
