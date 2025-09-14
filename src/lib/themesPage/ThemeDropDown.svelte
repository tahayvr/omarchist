<script>
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import MoreIcon from '@lucide/svelte/icons/ellipsis-vertical';
	import { Button } from '$lib/components/ui/button/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { refreshThemes } from '$lib/stores/themeCache.js';

	export let themeDir;
	export let themeTitle;
	export let onDeleted = null;

	async function handleDelete() {
		if (!themeDir) return;
		const confirmed = confirm(
			`Are you sure you want to delete the theme '${themeTitle || themeDir}'? This cannot be undone.`
		);
		if (!confirmed) return;
		try {
			await invoke('delete_custom_theme', { name: themeDir });
			await refreshThemes();
			if (onDeleted) onDeleted();
		} catch (err) {
			alert('Failed to delete theme: ' + (err?.message || err));
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
	<DropdownMenu.Content align="end">
		<DropdownMenu.Group>
			<DropdownMenu.Item onclick={handleDelete}>Delete Theme</DropdownMenu.Item>
			<DropdownMenu.Item>Open Folder</DropdownMenu.Item>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>
