<script>
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import MoreIcon from '@lucide/svelte/icons/ellipsis-vertical';
	import { Button } from '$lib/components/ui/button/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { refreshThemes } from '$lib/stores/themeCache.js';

	import { Command } from '@tauri-apps/plugin-shell';
	import { join, homeDir } from '@tauri-apps/api/path';
	import DropdownMenuItem from '$lib/components/ui/dropdown-menu/dropdown-menu-item.svelte';

	export let themeDir;
	export let onDeleted = null;

	async function handleDelete() {
		if (!themeDir) return;
		try {
			await invoke('delete_custom_theme', { name: themeDir });
			if (typeof refreshThemes === 'function') {
				await refreshThemes();
			}
			window.dispatchEvent(
				new CustomEvent('themes:changed', { detail: { action: 'deleted', theme: themeDir } })
			);
			if (onDeleted) onDeleted();
		} catch (err) {
			alert('Failed to delete theme: ' + (err?.message || err));
		}
	}

	async function handleOpenFolder() {
		if (!themeDir) return;
		try {
			const home = await homeDir();
			const themePath = await join(home, '.config', 'omarchy', 'themes', themeDir);
			await Command.create('nautilus', [themePath]).execute();
		} catch (err) {
			alert('Failed to open folder: ' + (err?.message || err));
		}
	}
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon" class="h-8 w-8 p-0">
				<MoreIcon class="h-4 w-4" />
				<span class="sr-only">Open options</span>
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end" class="uppercase">
		<DropdownMenu.Group>
			<DropdownMenu.Item>Share Theme</DropdownMenu.Item>
			<DropdownMenu.Item onclick={handleOpenFolder}>Open Folder</DropdownMenu.Item>
			<DropdownMenu.Separator />
			<DropdownMenu.Item
				onclick={handleDelete}
				class="bg-red-500/15 text-red-500 hover:text-red-500">Delete Theme</DropdownMenu.Item
			>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>
