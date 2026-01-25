<script>
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import MoreIcon from '@lucide/svelte/icons/ellipsis-vertical';
	import { Button } from '$lib/components/ui/button/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { refreshThemes } from '$lib/stores/themeCache.js';
	import { save } from '@tauri-apps/plugin-dialog';
	import { toast } from 'svelte-sonner';
	import { Command } from '@tauri-apps/plugin-shell';
	import { join, homeDir, documentDir } from '@tauri-apps/api/path';

	export let themeDir;
	export let is_system = false;
	export let is_custom = false;
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

			const userThemePath = await join(home, '.config', 'omarchy', 'themes', themeDir);
			const systemThemePath = await join(home, '.local', 'share', 'omarchy', 'themes', themeDir);

			const preferredPath = is_custom
				? userThemePath
				: is_system
					? systemThemePath
					: systemThemePath;
			const fallbackPath = preferredPath === userThemePath ? systemThemePath : userThemePath;

			const result = await Command.create('nautilus', [preferredPath]).execute();
			if (result?.code && result.code !== 0) {
				await Command.create('nautilus', [fallbackPath]).execute();
			}
		} catch (err) {
			alert('Failed to open folder: ' + (err?.message || err));
		}
	}

	async function handleShareTheme() {
		if (!themeDir) return;
		try {
			const defaultFilename = `${themeDir}.omarchy`;
			const dialogOptions = {
				defaultPath: defaultFilename,
				filters: [
					{
						name: 'Omarchist File',
						extensions: ['omarchy']
					},
					{
						name: 'Legacy JSON Theme',
						extensions: ['json']
					}
				]
			};

			try {
				const documents = await documentDir();
				if (documents) {
					dialogOptions.defaultPath = await join(documents, defaultFilename);
				}
			} catch (pathErr) {
				console.warn('Unable to determine Documents directory, using fallback path.', pathErr);
			}

			const destination = await save(dialogOptions);

			if (!destination) {
				// User cancelled
				return;
			}

			// Export the theme
			await invoke('export_custom_theme', {
				themeName: themeDir,
				destination: destination
			});

			toast.success('Theme exported', {
				description: `${themeDir} has been exported successfully.`
			});
		} catch (err) {
			alert('Failed to export theme: ' + (err?.message || err));
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
			{#if is_custom}
				<DropdownMenu.Item onclick={handleShareTheme}>Share Theme</DropdownMenu.Item>
			{/if}
			<DropdownMenu.Item onclick={handleOpenFolder}>Open Folder</DropdownMenu.Item>
			{#if is_custom}
				<DropdownMenu.Separator />
				<DropdownMenu.Item
					onclick={handleDelete}
					class="bg-red-500/15 text-red-500 hover:text-red-500">Delete Theme</DropdownMenu.Item
				>
			{/if}
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>
