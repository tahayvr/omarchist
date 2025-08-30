<script>
	import SystemThemes from '$lib/themesPage/SystemThemes.svelte';
	import CustomThemes from '$lib/themesPage/CustomThemes.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { themeCache } from '$lib/stores/themeCache.js';

	let isOpen = $state(false);
	let isCreating = $state(false);
	let themeName = $state('');
	let error = $state('');

	function resetForm() {
		themeName = '';
		error = '';
	}

	async function applyTheme(themeDir) {
		await invoke('apply_theme', { dir: themeDir });
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

	async function createTheme() {
		if (!themeName.trim()) {
			error = 'Theme name is required';
			return;
		}

		if (themeName.includes('/') || themeName.includes('\\')) {
			error = 'Theme name cannot contain slashes';
			return;
		}

		isCreating = true;
		error = '';

		try {
			const trimmedName = themeName.trim();
			await invoke('init_custom_theme', {
				name: trimmedName
			});

			await applyThemeIfEnabled(trimmedName);

			isOpen = false;
			resetForm();
			goto(`/themes/${encodeURIComponent(trimmedName)}`);
		} catch (err) {
			error = err.toString();
		} finally {
			isCreating = false;
		}
	}
</script>

<div class="w-full">
	<!-- Header with create theme button -->
	<div class="absolute top-4 right-6">
		<Dialog.Root>
			<Dialog.Trigger class={buttonVariants({ variant: 'outline' })}
				>CREATE NEW THEME</Dialog.Trigger
			>
			<Dialog.Content class="uppercase">
				<div class="space-y-2">
					<Label for="theme-name">Name your theme:</Label>
					<Input id="theme-name" bind:value={themeName} class="text-lg font-semibold" />
				</div>
				{#if error}
					<div class="mt-2 text-sm text-red-500">
						{error}
					</div>
				{/if}
				<Dialog.Footer>
					<Button variant="ghost" onclick={createTheme} disabled={isCreating || !themeName.trim()}
						>Create</Button
					>
				</Dialog.Footer>
			</Dialog.Content>
		</Dialog.Root>
	</div>
	<div class="px-6">
		<!-- Themes content -->
		<Tabs.Root value="all" class="w-full">
			<Tabs.List class="mt-4 mb-4">
				<Tabs.Trigger value="all" class="text-sm uppercase">System</Tabs.Trigger>
				<Tabs.Trigger value="custom" class="text-sm uppercase">Custom</Tabs.Trigger>
			</Tabs.List>
			<Tabs.Content value="all"><SystemThemes /></Tabs.Content>
			<Tabs.Content value="custom"><CustomThemes /></Tabs.Content>
		</Tabs.Root>
	</div>
</div>
