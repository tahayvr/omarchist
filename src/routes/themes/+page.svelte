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
	import { open } from '@tauri-apps/plugin-dialog';
	import { homeDir, join } from '@tauri-apps/api/path';
	import { refreshThemes } from '$lib/stores/themeCache.js';
	import { toast } from 'svelte-sonner';

	let isCreating = $state(false);
	let themeName = $state('');
	let error = $state('');

	// Import dialog state
	let showImportDialog = $state(false);
	let showConflictDialog = $state(false);
	let conflictInfo = $state(null);
	let importFilePath = $state('');
	let isImporting = $state(false);
	let importError = $state('');

	function resetForm() {
		themeName = '';
		error = '';
	}

	function resetImportDialog() {
		showImportDialog = false;
		showConflictDialog = false;
		conflictInfo = null;
		importFilePath = '';
		isImporting = false;
		importError = '';
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

			resetForm();
			goto(`/themes/${encodeURIComponent(trimmedName)}`);
		} catch (err) {
			error = err.toString();
		} finally {
			isCreating = false;
		}
	}

	async function handleImportClick() {
		try {
			const options = {
				filters: [
					{
						name: 'Omarchist File',
						extensions: ['omarchy']
					},
					{
						name: 'Legacy JSON Theme',
						extensions: ['json']
					}
				],
				multiple: false
			};

			try {
				const downloadsDir = await join(await homeDir(), 'Downloads');
				options.defaultPath = downloadsDir;
			} catch (pathErr) {
				console.warn('Falling back to dialog default location:', pathErr);
			}

			const file = await open(options);

			if (!file) {
				// User cancelled
				return;
			}

			importFilePath = file;

			// Validate the theme file first
			const validation = await invoke('validate_theme_file', {
				filePath: file
			});

			if (!validation.valid) {
				importError = validation.errors.join(', ');
				showImportDialog = true;
				return;
			}

			// Attempt import
			await performImport(false);
		} catch (err) {
			importError = err?.message || err.toString();
			showImportDialog = true;
		}
	}

	async function performImport(renameOnConflict) {
		isImporting = true;
		importError = '';

		try {
			const result = await invoke('import_custom_theme', {
				filePath: importFilePath,
				renameOnConflict: renameOnConflict
			});

			if (result.success) {
				// Refresh theme cache
				if (typeof refreshThemes === 'function') {
					await refreshThemes();
				}

				// Notify listeners so UI components reload without a manual refresh
				if (typeof window !== 'undefined') {
					window.dispatchEvent(new CustomEvent('themes:changed'));
				}

				// Notify success
				toast.success('Theme imported', {
					description: `${result.theme_name} is ready to use.`
				});

				// Reset and close
				resetImportDialog();
			} else if (result.conflict) {
				// Show conflict dialog
				conflictInfo = result.conflict;
				showConflictDialog = true;
			}
		} catch (err) {
			importError = err?.message || err.toString();
			showImportDialog = true;
		} finally {
			isImporting = false;
		}
	}

	async function handleConflictResolve(rename) {
		showConflictDialog = false;
		if (rename) {
			await performImport(true);
		} else {
			resetImportDialog();
		}
	}
</script>

<div class="w-full">
	<!-- Header with create theme button -->
	<div class="absolute top-4 right-6">
		<Button class="uppercase" variant="outline" onclick={handleImportClick}>Import theme</Button>
		<Dialog.Root>
			<Dialog.Trigger class={buttonVariants({ variant: 'secondary' })}
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

	<!-- Import Error Dialog -->
	<Dialog.Root bind:open={showImportDialog}>
		<Dialog.Content class="uppercase">
			<Dialog.Header>
				<Dialog.Title>Import Failed</Dialog.Title>
			</Dialog.Header>
			<div class="space-y-2">
				<p class="text-sm text-red-500">{importError}</p>
			</div>
			<Dialog.Footer>
				<Button
					variant="ghost"
					onclick={() => {
						showImportDialog = false;
					}}
					class="uppercase">Close</Button
				>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<!-- Conflict Resolution Dialog -->
	<Dialog.Root bind:open={showConflictDialog}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title class="uppercase">Theme Already Exists</Dialog.Title>
			</Dialog.Header>
			<div class="space-y-2">
				<p class="text-sm">
					A theme named "{conflictInfo?.existing_theme}" already exists.
				</p>
				<p class="text-sm">Would you like to import with a new name?</p>
				{#if conflictInfo?.suggested_name}
					<p class="text-sm font-semibold">
						New name: {conflictInfo.suggested_name}
					</p>
				{/if}
			</div>
			<Dialog.Footer>
				<Button variant="ghost" onclick={() => handleConflictResolve(false)}>Cancel</Button>
				<Button
					variant="secondary"
					onclick={() => handleConflictResolve(true)}
					disabled={isImporting}
					class="uppercase"
				>
					Rename and Import
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<div class="px-6">
		<!-- Themes content -->
		<Tabs.Root value="all" class="w-full">
			<Tabs.List class="mt-4 mb-4">
				<Tabs.Trigger value="all" class="text-sm uppercase">System</Tabs.Trigger>
				<Tabs.Trigger value="custom" class="text-sm uppercase">Omarchist</Tabs.Trigger>
			</Tabs.List>
			<Tabs.Content value="all"><SystemThemes /></Tabs.Content>
			<Tabs.Content value="custom"><CustomThemes /></Tabs.Content>
		</Tabs.Root>
	</div>
</div>
