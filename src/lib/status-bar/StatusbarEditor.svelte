<script>
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import StatusbarHeader from './StatusbarHeader.svelte';
	import StatusbarLayout from './layout/StatusbarLayout.svelte';
	import StatusbarModules from './modules/StatusbarModules.svelte';
	import {
		KNOWN_MODULES,
		initializeWaybarConfigState,
		loadWaybarConfig,
		saveWaybarConfig,
		resetWaybarConfigToDefaults,
		getModuleRegion,
		setModuleRegion,
		getModuleFields,
		setModuleField,
		setModuleConfig,
		updateWaybarLayoutSection,
		getGlobalFieldDefinitions,
		updateWaybarGlobals,
		sanitizeGlobalInput,
		applySnapshotToState,
		listWaybarProfiles,
		createWaybarProfile,
		selectWaybarProfile,
		deleteWaybarProfile
	} from '$lib/utils/waybar/waybarConfigUtils.js';
	import ColorPickerField from '$lib/themeDesigner/ColorPickerField.svelte';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import ColorPickerWaybar from './ColorPickerWaybar.svelte';

	const config = $state(initializeWaybarConfigState());
	const moduleDefinitions = KNOWN_MODULES;
	const globalFields = getGlobalFieldDefinitions();

	const profileState = $state({
		items: [],
		isLoading: false,
		isBusy: false
	});

	const isProfileInteractionLocked = $derived(
		config.isLoading || config.isSaving || profileState.isLoading || profileState.isBusy
	);

	const AUTO_SAVE_DELAY = 800;
	const AUTO_SAVE_SUCCESS_TOAST_COOLDOWN = 2000;

	let autoSaveHandle = null;
	let lastValidationToastSignature = null;
	let lastAutoSaveSuccessToastAt = 0;

	function clearAutoSaveTimer() {
		if (autoSaveHandle) {
			clearTimeout(autoSaveHandle);
			autoSaveHandle = null;
		}
	}

	async function refreshProfiles() {
		profileState.isLoading = true;
		try {
			const response = await listWaybarProfiles();
			const items = Array.isArray(response?.profiles) ? response.profiles : [];
			profileState.items = items;
		} catch (error) {
			console.error('Failed to load Waybar profiles:', error);
			toast.error('Unable to load Waybar configurations.', {
				description:
					error?.message ?? 'Please ensure the Waybar configuration directory is accessible.'
			});
		} finally {
			profileState.isLoading = false;
		}
	}

	async function handleProfileSelect(profileId) {
		if (!profileId) {
			return false;
		}
		if (profileState.isBusy) {
			return false;
		}
		profileState.isBusy = true;
		try {
			const response = await selectWaybarProfile(profileId);
			if (response?.snapshot) {
				applySnapshotToState(config, response.snapshot);
			}
			if (Array.isArray(response?.profiles)) {
				profileState.items = response.profiles;
			}
			return true;
		} catch (error) {
			console.error('Failed to switch Waybar configuration:', error);
			toast.error('Unable to switch Waybar configuration.', {
				description: error?.message ?? 'Please try again.'
			});
			return false;
		} finally {
			profileState.isBusy = false;
		}
	}

	async function handleProfileCreate(name) {
		const trimmed = name?.trim();
		if (!trimmed) {
			toast.error('Please provide a configuration name.');
			return false;
		}
		if (profileState.isBusy) {
			return false;
		}
		profileState.isBusy = true;
		try {
			const response = await createWaybarProfile(trimmed);
			if (response?.snapshot) {
				applySnapshotToState(config, response.snapshot);
			}
			if (Array.isArray(response?.profiles)) {
				profileState.items = response.profiles;
			}
			toast.success('Waybar configuration created.');
			return true;
		} catch (error) {
			console.error('Failed to create Waybar configuration:', error);
			toast.error('Unable to create Waybar configuration.', {
				description: error?.message ?? 'Please try again.'
			});
			return false;
		} finally {
			profileState.isBusy = false;
		}
	}

	async function handleProfileDelete(profileId) {
		if (!profileId) {
			return false;
		}
		if (profileState.isBusy) {
			return false;
		}
		profileState.isBusy = true;
		try {
			const response = await deleteWaybarProfile(profileId);
			if (response?.snapshot) {
				applySnapshotToState(config, response.snapshot);
			}
			if (Array.isArray(response?.profiles)) {
				profileState.items = response.profiles;
			}
			toast.success('Waybar configuration deleted.');
			return true;
		} catch (error) {
			console.error('Failed to delete Waybar configuration:', error);
			toast.error('Unable to delete Waybar configuration.', {
				description: error?.message ?? 'Please try again.'
			});
			return false;
		} finally {
			profileState.isBusy = false;
		}
	}

	function handleProfileShare(profileId) {
		if (!profileId) {
			toast.info('Select a configuration to share first.');
			return;
		}
		toast.info('Waybar configuration sharing is coming soon.');
	}

	onMount(async () => {
		await loadWaybarConfig(config);
		await refreshProfiles();
	});

	$effect(() => {
		if (config.error) {
			toast.error('Waybar configuration error', {
				description: config.error
			});
			config.error = null;
		}
	});

	$effect(() => {
		if (config.success) {
			toast.success(config.success);
			config.success = null;
		}
	});

	$effect(() => {
		const { validation, dirty, hasHydrated, isLoading, isSaving } = config;

		if (!hasHydrated) {
			clearAutoSaveTimer();
			return;
		}

		if (isLoading || isSaving) {
			clearAutoSaveTimer();
			return;
		}

		if (!dirty) {
			clearAutoSaveTimer();
			return;
		}

		if (!validation?.isValid) {
			clearAutoSaveTimer();
			const signature = JSON.stringify(validation?.fieldErrors ?? {});
			if (signature && signature !== lastValidationToastSignature) {
				lastValidationToastSignature = signature;
				const messages = Object.values(validation?.fieldErrors ?? {});
				const description = messages.length
					? messages.join(' ')
					: 'Please resolve highlighted Waybar settings.';
				toast.error('Waybar configuration needs attention.', { description });
			}
			return;
		}

		lastValidationToastSignature = null;
		clearAutoSaveTimer();
		autoSaveHandle = setTimeout(async () => {
			autoSaveHandle = null;
			const saved = await saveWaybarConfig(config, { silent: true });
			if (saved) {
				const now = Date.now();
				if (now - lastAutoSaveSuccessToastAt >= AUTO_SAVE_SUCCESS_TOAST_COOLDOWN) {
					lastAutoSaveSuccessToastAt = now;
					toast.success('Waybar configuration saved.');
				}
			}
		}, AUTO_SAVE_DELAY);
	});

	function getRegion(moduleId) {
		return getModuleRegion(config, moduleId);
	}

	function getModuleConfig(moduleId) {
		return config.modules?.[moduleId] ?? {};
	}

	function getModuleFieldsFor(moduleId) {
		return getModuleFields(moduleId);
	}

	function handleRegionChange(moduleId, region) {
		setModuleRegion(config, moduleId, region);
	}

	function handleModuleFieldChange(moduleId, fieldKey, value) {
		if (!moduleId || !fieldKey) {
			return;
		}
		setModuleField(config, moduleId, fieldKey, value);
	}

	function handleModuleConfigChange(moduleId, moduleConfig) {
		if (!moduleId || !moduleConfig || typeof moduleConfig !== 'object') {
			return;
		}
		setModuleConfig(config, moduleId, moduleConfig);
	}

	function handleLayoutReorder(event) {
		const { section, modules } = event.detail ?? event ?? {};
		if (!section || !Array.isArray(modules)) {
			return;
		}
		updateWaybarLayoutSection(config, section, modules);
	}

	function handleGlobalValueChange(fieldKey, rawValue) {
		const sanitized = sanitizeGlobalInput(fieldKey, rawValue);
		if (config.globals?.[fieldKey] === sanitized) {
			return;
		}
		updateWaybarGlobals(config, fieldKey, sanitized);
	}

	function handleGlobalInput(field, event) {
		const target = event.currentTarget ?? event.target;
		const value = target?.value ?? '';
		handleGlobalValueChange(field.key, value);
	}

	function handleGlobalNumber(field, event) {
		const target = event.currentTarget ?? event.target;
		const value = target?.value ?? '';
		handleGlobalValueChange(field.key, value);
	}

	async function handleReset() {
		clearAutoSaveTimer();
		resetWaybarConfigToDefaults(config);
		await saveWaybarConfig(config, {
			message: 'Waybar configuration reset to defaults.'
		});
	}

	function getGlobalError(key) {
		return config.validation?.fieldErrors?.[`globals.${key}`];
	}

	function getSelectLabel(field, value) {
		const label = field.options?.find((option) => option.value === value)?.label;
		return label ?? field.placeholder ?? 'Select an option';
	}

	const isBusy = $derived(config.isLoading || config.isSaving);
	const isValid = $derived(config.validation?.isValid ?? true);

	onDestroy(() => {
		clearAutoSaveTimer();
	});
</script>

<div class="space-y-6">
	<StatusbarHeader
		isLoading={config.isLoading}
		isSaving={config.isSaving}
		dirty={config.dirty}
		{isValid}
		profiles={profileState.items}
		selectedProfileId={config.profileId}
		profilesLoading={profileState.isLoading}
		profileBusy={profileState.isBusy}
		profileInteractionLocked={isProfileInteractionLocked}
		onProfileSelect={handleProfileSelect}
		onProfileCreate={handleProfileCreate}
		onProfileDelete={handleProfileDelete}
		onProfileShare={handleProfileShare}
		onReset={handleReset}
	/>

	<div class="flex w-full flex-col gap-6 xl:flex-row">
		<div class="flex w-full flex-col gap-6 xl:w-1/2">
			<StatusbarLayout
				layout={config.layout}
				modules={moduleDefinitions}
				disabled={isBusy}
				onReorder={handleLayoutReorder}
			/>
			<Card.Root>
				<Card.Header>
					<Card.Title class="text-accent-foreground uppercase">Bar Appearance</Card.Title>
					<Card.Description class="text-muted-foreground text-xs tracking-wide uppercase">
						Applies to the whole status bar.
					</Card.Description>
				</Card.Header>
				<Card.Content class="grid gap-4 md:grid-cols-2">
					{#each globalFields as field (field.key)}
						<div class="space-y-2">
							<Label for={`global-${field.key}`} class="text-[0.65rem] font-semibold uppercase">
								{field.label}
							</Label>
							{#if field.type === 'select'}
								<Select.Root
									value={config.globals?.[field.key] ?? ''}
									onValueChange={(value) => handleGlobalValueChange(field.key, value)}
									disabled={isBusy}
									type="single"
								>
									<Select.Trigger id={`global-${field.key}`} class="w-32 uppercase">
										{getSelectLabel(field, config.globals?.[field.key] ?? '')}
									</Select.Trigger>
									<Select.Content>
										{#each field.options ?? [] as option (option.value)}
											<Select.Item value={option.value}>
												{option.label}
											</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							{:else if field.type === 'number'}
								<Input
									id={`global-${field.key}`}
									type="number"
									class="w-24 uppercase"
									value={config.globals?.[field.key] ?? ''}
									min={field.min}
									max={field.max}
									step={field.step}
									placeholder={field.placeholder}
									disabled={isBusy}
									oninput={(event) => handleGlobalNumber(field, event)}
								/>
							{:else}
								<Input
									id={`global-${field.key}`}
									type="text"
									class="uppercase"
									value={config.globals?.[field.key] ?? ''}
									placeholder={field.placeholder}
									disabled={isBusy}
									oninput={(event) => handleGlobalInput(field, event)}
								/>
							{/if}
							{#if getGlobalError(field.key)}
								<p class="text-destructive text-[0.65rem] tracking-wide uppercase">
									{getGlobalError(field.key)}
								</p>
							{/if}
						</div>
					{/each}

					<div class="col-span-2">
						<Separator class="my-4" />
					</div>

					<!-- Bar Colors -->
					<div class="space-y-2">
						<ColorPickerWaybar
							label="Bar Background"
							bind:color={config.globals.background}
							onChange={(color) => handleGlobalValueChange('background', color)}
						/>
					</div>

					<div class="space-y-2">
						<ColorPickerField
							label="Bar Foreground"
							bind:color={config.globals.foreground}
							onChange={(color) => handleGlobalValueChange('foreground', color)}
						/>
					</div>

					<div class="col-span-2">
						<Separator class="my-4" />
						<h3 class="text-accent-foreground mb-3 text-sm font-semibold uppercase">
							Module Section Styling
						</h3>
					</div>

					<!-- Left Section -->
					<div class="col-span-2">
						<Card.Root class="border-muted/50">
							<Card.Header>
								<Card.Title class="text-accent-foreground/70 text-xs uppercase">
									Left Section
								</Card.Title>
							</Card.Header>
							<Card.Content class="space-y-3">
								<div class="flex items-center gap-4">
									<div class="flex-1 space-y-2">
										<Label class="text-[0.65rem] font-semibold uppercase">Margin (px)</Label>
										<Input
											type="number"
											class="w-24 uppercase"
											value={config.globals.leftMargin ?? 8}
											min={0}
											max={64}
											step={1}
											disabled={isBusy}
											oninput={(event) => handleGlobalValueChange('leftMargin', event.target.value)}
										/>
									</div>
									<div class="flex-1 space-y-2">
										<Label class="text-[0.65rem] font-semibold uppercase">Padding (px)</Label>
										<Input
											type="number"
											class="w-24 uppercase"
											value={config.globals.leftPadding ?? 0}
											min={0}
											max={64}
											step={1}
											disabled={isBusy}
											oninput={(event) =>
												handleGlobalValueChange('leftPadding', event.target.value)}
										/>
									</div>
								</div>
								<div>
									<ColorPickerWaybar
										label="Background"
										bind:color={config.globals.leftBackground}
										onChange={(color) => handleGlobalValueChange('leftBackground', color)}
									/>
								</div>
							</Card.Content>
						</Card.Root>
					</div>

					<!-- Center Section -->
					<div class="col-span-2">
						<Card.Root class="border-muted/50">
							<Card.Header>
								<Card.Title class="text-accent-foreground/70 text-xs uppercase">
									Center Section
								</Card.Title>
							</Card.Header>
							<Card.Content class="space-y-3">
								<div class="flex items-center gap-4">
									<div class="flex-1 space-y-2">
										<Label class="text-[0.65rem] font-semibold uppercase">Margin (px)</Label>
										<Input
											type="number"
											class="w-24 uppercase"
											value={config.globals.centerMargin ?? 0}
											min={0}
											max={64}
											step={1}
											disabled={isBusy}
											oninput={(event) =>
												handleGlobalValueChange('centerMargin', event.target.value)}
										/>
									</div>
									<div class="flex-1 space-y-2">
										<Label class="text-[0.65rem] font-semibold uppercase">Padding (px)</Label>
										<Input
											type="number"
											class="w-24 uppercase"
											value={config.globals.centerPadding ?? 0}
											min={0}
											max={64}
											step={1}
											disabled={isBusy}
											oninput={(event) =>
												handleGlobalValueChange('centerPadding', event.target.value)}
										/>
									</div>
								</div>
								<div>
									<ColorPickerWaybar
										label="Background"
										bind:color={config.globals.centerBackground}
										onChange={(color) => handleGlobalValueChange('centerBackground', color)}
									/>
								</div>
							</Card.Content>
						</Card.Root>
					</div>

					<!-- Right Section -->
					<div class="col-span-2">
						<Card.Root class="border-muted/50">
							<Card.Header>
								<Card.Title class="text-accent-foreground/70 text-xs uppercase">
									Right Section
								</Card.Title>
							</Card.Header>
							<Card.Content class="space-y-3">
								<div class="flex items-center gap-4">
									<div class="flex-1 space-y-2">
										<Label class="text-[0.65rem] font-semibold uppercase">Margin (px)</Label>
										<Input
											type="number"
											class="w-24 uppercase"
											value={config.globals.rightMargin ?? 8}
											min={0}
											max={64}
											step={1}
											disabled={isBusy}
											oninput={(event) =>
												handleGlobalValueChange('rightMargin', event.target.value)}
										/>
									</div>
									<div class="flex-1 space-y-2">
										<Label class="text-[0.65rem] font-semibold uppercase">Padding (px)</Label>
										<Input
											type="number"
											class="w-24 uppercase"
											value={config.globals.rightPadding ?? 0}
											min={0}
											max={64}
											step={1}
											disabled={isBusy}
											oninput={(event) =>
												handleGlobalValueChange('rightPadding', event.target.value)}
										/>
									</div>
								</div>
								<div>
									<ColorPickerWaybar
										label="Background"
										bind:color={config.globals.rightBackground}
										onChange={(color) => handleGlobalValueChange('rightBackground', color)}
									/>
								</div>
							</Card.Content>
						</Card.Root>
					</div>
				</Card.Content>
			</Card.Root>
		</div>
		<div class="flex w-full flex-col gap-6 xl:w-1/2">
			<StatusbarModules
				modules={moduleDefinitions}
				{getRegion}
				getFields={getModuleFieldsFor}
				getConfig={getModuleConfig}
				onRegionChange={handleRegionChange}
				onFieldChange={handleModuleFieldChange}
				onConfigChange={handleModuleConfigChange}
				disabled={isBusy}
			/>
		</div>
	</div>
</div>
