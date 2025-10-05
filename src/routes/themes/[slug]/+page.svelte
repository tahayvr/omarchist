<script>
	import { Button } from '$lib/components/ui/button/index.js';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/core';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as Accordion from '$lib/components/ui/accordion/index.js';
	import { page } from '$app/stores';
	import SchemaForm from '$lib/themeDesigner/SchemaForm.svelte';
	import IconThemeSelector from '$lib/themeDesigner/IconThemeSelector.svelte';
	import NeovimTextArea from '$lib/themeDesigner/NeovimTextArea.svelte';
	import BackgroundImageSelector from '$lib/themeDesigner/BackgroundImageSelector.svelte';
	import GeneralTab from '$lib/themeDesigner/GeneralTab.svelte';
	import VsCodeTextArea from '$lib/themeDesigner/VSCodeTextArea.svelte';

	let themeName = $state('');
	let originalThemeName = $state('');
	let themeAuthor = $state('');
	let isSaving = $state(false);
	let appSchemas = $state(null);
	let alacrittySchema = $state(null);
	let btopSchema = $state(null);
	let hyprlandSchema = $state(null);
	let hyprlockSchema = $state(null);
	let makoSchema = $state(null);
	let walkerSchema = $state(null);
	let swayosdSchema = $state(null);
	let ghosttySchema = $state(null);
	let kittySchema = $state(null);

	let alacrittyData = $state({});
	let waybarData = $state({});
	let chromiumData = $state({});
	let btopData = $state({});
	let hyprlandData = $state({});
	let hyprlockData = $state({});
	let iconsData = $state({});
	let makoData = $state({});
	let walkerData = $state({});
	let swayosdData = $state({});
	let neovimData = $state({});
	let backgroundsData = $state([]);
	let vscodeData = $state({});
	let ghosttyData = $state({});
	let kittyData = $state({});

	// Helpers to get/set nested properties by path
	function setByPath(obj, path, value) {
		const parts = path.split('.');
		let ref = obj;
		for (let i = 0; i < parts.length - 1; i++) {
			const key = parts[i];
			if (typeof ref[key] !== 'object' || ref[key] === null) ref[key] = {};
			ref = ref[key];
		}
		ref[parts[parts.length - 1]] = value;
	}

	$effect(() => {
		function handleKeydown(e) {
			// Support both Ctrl+S (Linux/Windows) and Cmd+S (Mac)
			if ((e.ctrlKey || e.metaKey) && e.key === 's') {
				e.preventDefault();
				saveTheme();
			}
		}
		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
	});

	// Sanitize theme name to match Rust backend logic
	function sanitizeName(name) {
		let result = '';
		for (const char of name) {
			if (char === ' ') {
				result += '-';
			} else if (/[a-zA-Z0-9_-]/.test(char)) {
				result += char.toLowerCase();
			}
			// Skip invalid characters
		}
		return result;
	}

	function getSanitizedAuthor() {
		const trimmed = themeAuthor?.trim();
		if (!trimmed) return null;
		return trimmed.length ? trimmed : null;
	}

	// Load based on route slug
	$effect(() => {
		const slug = $page?.params?.slug;
		if (slug) {
			loadPage(slug);
		}
	});

	async function loadPage(slug) {
		try {
			// Fetch schemas and current theme in parallel
			const [schemas, theme] = await Promise.all([
				invoke('get_app_schemas'),
				invoke('get_custom_theme', { name: slug })
			]);

			appSchemas = schemas;
			alacrittySchema = schemas?.alacritty || null;
			btopSchema = schemas?.btop || null;
			hyprlandSchema = schemas?.hyprland || null;
			hyprlockSchema = schemas?.hyprlock || null;
			makoSchema = schemas?.mako || null;
			walkerSchema = schemas?.walker || null;
			swayosdSchema = schemas?.swayosd || null;
			ghosttySchema = schemas?.ghostty || null;
			kittySchema = schemas?.kitty || null;
			themeName = theme.name;
			originalThemeName = theme.name; // Store original name for rename detection
			themeAuthor = theme?.author || '';
			alacrittyData = theme?.apps?.alacritty || {};
			waybarData = theme?.apps?.waybar || {};
			chromiumData = theme?.apps?.chromium || {};
			btopData = theme?.apps?.btop || {};
			hyprlandData = theme?.apps?.hyprland || {};
			hyprlockData = theme?.apps?.hyprlock || {};
			iconsData = theme?.apps?.icons || {};
			makoData = theme?.apps?.mako || {};
			walkerData = theme?.apps?.walker || {};
			swayosdData = theme?.apps?.swayosd || {};
			neovimData = theme?.apps?.neovim || {};
			vscodeData = theme?.apps?.vscode || {};
			ghosttyData = theme?.apps?.ghostty || {};
			kittyData = theme?.apps?.kitty || {};
			// backgroundsData will be loaded by the BackgroundImageSelector component
		} catch (error) {
			console.error('Failed to load theme or schemas:', error);
		}
	}

	async function saveTheme() {
		if (!themeName.trim()) return;
		const sanitizedAuthor = getSanitizedAuthor();
		isSaving = true;
		try {
			// Check if theme name has changed
			const nameChanged = themeName !== originalThemeName;

			if (nameChanged) {
				try {
					// Step 1: Rename the theme directory and files first
					await invoke('rename_custom_theme', {
						old_name: originalThemeName,
						new_name: themeName
					});

					// Step 2: Save the theme data to the renamed theme
					await invoke('update_custom_theme_advanced', {
						name: themeName,
						theme_data: {
							alacritty: alacrittyData,
							waybar: waybarData,
							chromium: chromiumData,
							btop: btopData,
							hyprland: hyprlandData,
							hyprlock: hyprlockData,
							icons: iconsData,
							mako: makoData,
							walker: walkerData,
							swayosd: swayosdData,
							neovim: neovimData,
							vscode: vscodeData,
							ghostty: ghosttyData,
							kitty: kittyData
						}
					});

					await invoke('set_theme_author', {
						theme_name: themeName,
						author: sanitizedAuthor
					});
					themeAuthor = sanitizedAuthor ?? '';

					// Step 3: Update the original name reference
					originalThemeName = themeName;

					// Step 4: Small delay to ensure files are written to disk
					await new Promise((resolve) => setTimeout(resolve, 100));

					// Step 5: Apply the theme (this copies files to active locations)
					await invoke('apply_theme', { dir: themeName });

					// Step 6: Refresh theme adjustments
					await refreshThemeIfEnabled();

					// Step 7: Navigate to the new URL
					const sanitizedName = sanitizeName(themeName);
					goto(`/themes/${sanitizedName}`);
					return;
				} catch (renameError) {
					console.error('Failed to rename theme:', renameError);
					alert(`Failed to rename theme: ${renameError}`);
					return;
				}
			}

			// Normal save without rename
			await invoke('update_custom_theme_advanced', {
				name: themeName,
				theme_data: {
					alacritty: alacrittyData,
					waybar: waybarData,
					chromium: chromiumData,
					btop: btopData,
					hyprland: hyprlandData,
					hyprlock: hyprlockData,
					icons: iconsData,
					mako: makoData,
					walker: walkerData,
					swayosd: swayosdData,
					neovim: neovimData,
					vscode: vscodeData,
					ghostty: ghosttyData,
					kitty: kittyData
				}
			});

			await invoke('set_theme_author', {
				theme_name: themeName,
				author: sanitizedAuthor
			});
			themeAuthor = sanitizedAuthor ?? '';

			// Apply theme first, then refresh adjustments
			await invoke('apply_theme', { dir: themeName });
			await refreshThemeIfEnabled();

			// Re-fetch theme to verify persistence and refresh data
			const refreshed = await invoke('get_custom_theme', { name: themeName });
			themeAuthor = refreshed?.author || '';
			alacrittyData = refreshed?.apps?.alacritty || alacrittyData;
			waybarData = refreshed?.apps?.waybar || waybarData;
			chromiumData = refreshed?.apps?.chromium || chromiumData;
			btopData = refreshed?.apps?.btop || btopData;
			hyprlandData = refreshed?.apps?.hyprland || hyprlandData;
			hyprlockData = refreshed?.apps?.hyprlock || hyprlockData;
			iconsData = refreshed?.apps?.icons || iconsData;
			makoData = refreshed?.apps?.mako || makoData;
			walkerData = refreshed?.apps?.walker || walkerData;
			swayosdData = refreshed?.apps?.swayosd || swayosdData;
			neovimData = refreshed?.apps?.neovim || neovimData;
			vscodeData = refreshed?.apps?.vscode || vscodeData;
			ghosttyData = refreshed?.apps?.ghostty || ghosttyData;
			kittyData = refreshed?.apps?.kitty || kittyData;
		} catch (error) {
			console.error('Failed to save theme:', error);
			alert(`Failed to save theme: ${error}`);
		} finally {
			isSaving = false;
		}
	}

	async function refreshThemeIfEnabled() {
		try {
			const settings = await invoke('get_app_settings');
			if (settings.auto_apply_theme) {
				await invoke('refresh_theme_adjustments');
			}
		} catch (err) {
			console.error('Failed to check auto_apply_theme setting:', err);
			// If we can't get settings, don't refresh theme to be safe
		}
	}

	function cancelEditing() {
		goto('/themes');
	}

	async function launchApp(appName) {
		await invoke('execute_bash_command_async', {
			command: `uwsm app -- ${appName}`
		});
	}

	async function testNotification() {
		await invoke('execute_bash_command_async', {
			command: `notify-send "Test Notification" "This is a test notification"`
		});
	}
</script>

<div class="w-full px-6 py-4">
	<!-- Header -->
	<div class="mb-6 flex w-full items-center justify-between">
		<Button variant="ghost" onclick={cancelEditing} class="uppercase">Back</Button>
		<h1 class="text-lg font-medium uppercase">{themeName}</h1>
		<Button variant="outline" onclick={saveTheme} disabled={isSaving} class="uppercase"
			>Update Theme</Button
		>
	</div>

	<!-- Main content -->
	<div class="w-full">
		<Tabs.Root value="waybar" class="w-full">
			<Tabs.List class="h-auto flex-wrap gap-1">
				<Tabs.Trigger value="general" class="uppercase">General</Tabs.Trigger>
				<Tabs.Trigger value="waybar" class="uppercase">Status Bar</Tabs.Trigger>
				<Tabs.Trigger value="windows" class="uppercase">Windows</Tabs.Trigger>
				<Tabs.Trigger value="menu" class="uppercase">Omarchy Menu</Tabs.Trigger>
				<Tabs.Trigger value="terminal" class="uppercase">Terminal</Tabs.Trigger>
				<Tabs.Trigger value="browser" class="uppercase">Browser</Tabs.Trigger>
				<Tabs.Trigger value="file" class="uppercase">File Manager</Tabs.Trigger>
				<Tabs.Trigger value="hyprlock" class="uppercase">Lock Screen</Tabs.Trigger>
				<Tabs.Trigger value="notification" class="uppercase">Notification</Tabs.Trigger>
				<Tabs.Trigger value="editor" class="uppercase">Editor</Tabs.Trigger>
				<Tabs.Trigger value="btop" class="uppercase">Activity Monitor</Tabs.Trigger>
				<Tabs.Trigger value="swayosd" class="uppercase">On-Screen Display</Tabs.Trigger>
				<Tabs.Trigger value="backgrounds" class="uppercase">Backgrounds</Tabs.Trigger>
			</Tabs.List>
			<Tabs.Content value="general" class="max-w-[1200px]">
				<GeneralTab bind:themeName bind:author={themeAuthor} />
			</Tabs.Content>
			<Tabs.Content value="terminal" class="max-w-[1200px]">
				<Accordion.Root type="single">
					<Accordion.Item value="item-1">
						<Accordion.Trigger>Alacritty</Accordion.Trigger>
						<Accordion.Content>
							<SchemaForm
								schema={alacrittySchema}
								data={alacrittyData}
								on:field-change={(e) => {
									const { field, value } = e.detail;
									setByPath(alacrittyData, field, value);
									alacrittyData = { ...alacrittyData };
								}}
							/>
							<Button
								variant="outline"
								size="sm"
								class="mt-4 uppercase"
								onclick={() => launchApp('alacritty --working-directory=$HOME')}
								>Launch Alacritty</Button
							>
						</Accordion.Content>
					</Accordion.Item>
					<Accordion.Item value="item-2">
						<Accordion.Trigger>Ghostty</Accordion.Trigger>
						<Accordion.Content>
							<SchemaForm
								schema={ghosttySchema}
								data={ghosttyData}
								on:field-change={(e) => {
									const { field, value } = e.detail;
									setByPath(ghosttyData, field, value);
									ghosttyData = { ...ghosttyData };
								}}
							/>
							<Button
								variant="outline"
								size="sm"
								class="mt-4 uppercase"
								onclick={() => launchApp('ghostty')}>Launch Ghostty</Button
							>
						</Accordion.Content>
					</Accordion.Item>
					<Accordion.Item value="item-3">
						<Accordion.Trigger>Kitty</Accordion.Trigger>
						<Accordion.Content>
							<SchemaForm
								schema={kittySchema}
								data={kittyData}
								on:field-change={(e) => {
									const { field, value } = e.detail;
									setByPath(kittyData, field, value);
									kittyData = { ...kittyData };
								}}
							/>
							<Button
								variant="outline"
								size="sm"
								class="mt-4 uppercase"
								onclick={() => launchApp('kitty')}>Launch Kitty</Button
							>
						</Accordion.Content>
					</Accordion.Item>
				</Accordion.Root>
			</Tabs.Content>
			<Tabs.Content value="waybar" class="max-w-[1200px]">
				<SchemaForm
					schema={appSchemas?.waybar}
					data={waybarData}
					on:field-change={(e) => {
						const { field, value } = e.detail;
						setByPath(waybarData, field, value);
						waybarData = { ...waybarData };
					}}
				/>
			</Tabs.Content>
			<Tabs.Content value="browser" class="max-w-[1200px]">
				<p class="mt-4 uppercase">Chromium</p>
				<SchemaForm
					schema={appSchemas?.chromium}
					data={chromiumData}
					on:field-change={(e) => {
						const { field, value } = e.detail;
						setByPath(chromiumData, field, value);
						chromiumData = { ...chromiumData };
					}}
				/>
			</Tabs.Content>
			<Tabs.Content value="btop" class="max-w-[1200px]">
				<SchemaForm
					schema={btopSchema}
					data={btopData}
					on:field-change={(e) => {
						const { field, value } = e.detail;
						setByPath(btopData, field, value);
						btopData = { ...btopData };
					}}
				/>
				<Button
					variant="outline"
					size="sm"
					class="mt-4 uppercase"
					onclick={() => launchApp('alacritty -e btop')}>Launch btop</Button
				>
			</Tabs.Content>
			<Tabs.Content value="windows" class="max-w-[1200px]">
				<SchemaForm
					schema={hyprlandSchema}
					data={hyprlandData}
					on:field-change={(e) => {
						const { field, value } = e.detail;
						setByPath(hyprlandData, field, value);
						hyprlandData = { ...hyprlandData };
					}}
				/>
			</Tabs.Content>
			<Tabs.Content value="hyprlock" class="max-w-[1200px]">
				<SchemaForm
					schema={hyprlockSchema}
					data={hyprlockData}
					on:field-change={(e) => {
						const { field, value } = e.detail;
						setByPath(hyprlockData, field, value);
						hyprlockData = { ...hyprlockData };
					}}
				/>
			</Tabs.Content>
			<Tabs.Content value="file" class="max-w-[1200px]">
				<div>
					<IconThemeSelector
						value={iconsData?.theme_name || 'Yaru-red'}
						on:change={(e) => {
							iconsData = { ...iconsData, theme_name: e.detail.value };
						}}
					/>
					<Button
						variant="outline"
						size="sm"
						class="mt-4 uppercase"
						onclick={() => launchApp('nautilus --new-window')}>Launch Nautilus</Button
					>
				</div>
			</Tabs.Content>
			<Tabs.Content value="notification" class="max-w-[1200px]">
				<SchemaForm
					schema={makoSchema}
					data={makoData}
					on:field-change={(e) => {
						const { field, value } = e.detail;
						setByPath(makoData, field, value);
						makoData = { ...makoData };
					}}
				/>
				<Button
					variant="outline"
					size="sm"
					class="mt-4 uppercase"
					onclick={() => testNotification()}>Test Mako Notification</Button
				>
			</Tabs.Content>
			<Tabs.Content value="menu" class="max-w-[1200px]">
				<SchemaForm
					schema={walkerSchema}
					data={walkerData}
					on:field-change={(e) => {
						const { field, value } = e.detail;
						setByPath(walkerData, field, value);
						walkerData = { ...walkerData };
					}}
				/>
			</Tabs.Content>
			<Tabs.Content value="swayosd" class="max-w-[1200px]">
				<SchemaForm
					schema={swayosdSchema}
					data={swayosdData}
					on:field-change={(e) => {
						const { field, value } = e.detail;
						setByPath(swayosdData, field, value);
						swayosdData = { ...swayosdData };
					}}
				/>
			</Tabs.Content>
			<Tabs.Content value="editor" class="max-w-[1200px]">
				<Accordion.Root type="single" value="item-1">
					<Accordion.Item value="item-1">
						<Accordion.Trigger>Neovim</Accordion.Trigger>
						<Accordion.Content
							><NeovimTextArea
								value={neovimData?.raw_config || ''}
								on:change={(e) => {
									neovimData = { ...neovimData, raw_config: e.detail.value };
								}}
							/>
						</Accordion.Content>
					</Accordion.Item>
					<Accordion.Item value="item-2">
						<Accordion.Trigger>VSCode</Accordion.Trigger>
						<Accordion.Content
							><VsCodeTextArea
								value={vscodeData?.raw_config || ''}
								on:change={(e) => {
									vscodeData = { ...vscodeData, raw_config: e.detail.value };
								}}
							/>
						</Accordion.Content>
					</Accordion.Item>
				</Accordion.Root>
			</Tabs.Content>
			<Tabs.Content value="backgrounds">
				<BackgroundImageSelector {themeName} bind:backgrounds={backgroundsData} />
			</Tabs.Content>
		</Tabs.Root>
	</div>
</div>
