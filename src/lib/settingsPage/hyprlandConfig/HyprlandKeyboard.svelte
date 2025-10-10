<script>
	import { onDestroy, onMount } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as Accordion from '$lib/components/ui/accordion/index.js';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { toast } from 'svelte-sonner';
	import Explainer from '$lib/components/Explainer.svelte';
	import {
		initializeHyprlandInputState,
		loadHyprlandInput,
		saveHyprlandInput,
		resetHyprlandInputToDefaults,
		recomputeDirty,
		validateHyprlandInputForm,
		getModels,
		getLayouts,
		getVariantsForLayout,
		getOptionGroups,
		getSelectedOptionsSet
	} from '$lib/utils/hyprlandInputUtils.js';
	import SettingsFilterToggle from '../SettingsFilterToggle.svelte';

	const hyprlandInput = $state(initializeHyprlandInputState());

	const modelOptions = $derived(getModels(hyprlandInput));
	const layoutOptions = $derived(getLayouts(hyprlandInput));
	const variantOptions = $derived(
		getVariantsForLayout(hyprlandInput, hyprlandInput.form.kb_layout)
	);
	const optionGroups = $derived(getOptionGroups(hyprlandInput));
	const selectedOptions = $derived(getSelectedOptionsSet(hyprlandInput.form));
	const selectedModelLabel = $derived.by(() => {
		if (!hyprlandInput.form.kb_model) {
			return 'Default';
		}
		const match = modelOptions.find((model) => model.name === hyprlandInput.form.kb_model);
		return match?.description || hyprlandInput.form.kb_model;
	});

	const AUTO_SAVE_DELAY = 800;
	const AUTO_SAVE_SUCCESS_TOAST_COOLDOWN = 2000;

	let lastValidationToastSignature = null;
	let lastAutoSaveSuccessToastAt = 0;
	let lastFormSignature = '';
	let lastSavedFormSignature = '';

	function clearAutoSaveTimer() {
		if (hyprlandInput.autoSaveHandle) {
			clearTimeout(hyprlandInput.autoSaveHandle);
			hyprlandInput.autoSaveHandle = null;
		}
	}

	onMount(async () => {
		await loadHyprlandInput(hyprlandInput);
	});

	$effect(() => {
		hyprlandInput.validation = validateHyprlandInputForm(hyprlandInput.form, hyprlandInput.catalog);
	});

	$effect(() => {
		if (!hyprlandInput.hasHydrated) {
			lastFormSignature = '';
			lastSavedFormSignature = '';
			return;
		}

		const formSignature = JSON.stringify(hyprlandInput.form ?? {});
		const savedSignature = JSON.stringify(hyprlandInput.lastSavedForm ?? {});

		if (formSignature === lastFormSignature && savedSignature === lastSavedFormSignature) {
			return;
		}

		lastFormSignature = formSignature;
		lastSavedFormSignature = savedSignature;
		recomputeDirty(hyprlandInput, {
			currentSignature: formSignature,
			lastSavedSignature: savedSignature
		});
	});

	$effect(() => {
		const { validation, dirty, hasHydrated, isLoading } = hyprlandInput;
		if (!hasHydrated || isLoading) {
			lastValidationToastSignature = null;
			return;
		}

		if (!validation?.isValid && dirty) {
			const signature = JSON.stringify(validation.fieldErrors ?? {});
			if (signature && signature !== lastValidationToastSignature) {
				lastValidationToastSignature = signature;
				const messages = Object.values(validation.fieldErrors ?? {});
				const description = messages.length
					? messages.join(' ')
					: 'Please resolve the highlighted fields.';
				toast.error('Hyprland input settings need attention.', {
					description
				});
			}
			return;
		}

		lastValidationToastSignature = null;
	});

	$effect(() => {
		if (hyprlandInput.error) {
			toast(hyprlandInput.error);
			hyprlandInput.error = null;
		}
	});

	$effect(() => {
		if (hyprlandInput.catalogError) {
			toast(hyprlandInput.catalogError);
			hyprlandInput.catalogError = null;
		}
	});

	$effect(() => {
		if (hyprlandInput.success) {
			toast.success(hyprlandInput.success);
			hyprlandInput.success = null;
		}
	});

	$effect(() => {
		const { autoSaveHandle, dirty, hasHydrated, isLoading, isSaving, validation } = hyprlandInput;
		void autoSaveHandle;

		if (!hasHydrated) {
			clearAutoSaveTimer();
			return;
		}

		if (isLoading || isSaving) {
			clearAutoSaveTimer();
			return;
		}

		if (!dirty || !validation?.isValid) {
			clearAutoSaveTimer();
			return;
		}

		clearAutoSaveTimer();
		hyprlandInput.autoSaveHandle = setTimeout(async () => {
			hyprlandInput.autoSaveHandle = null;
			const saved = await saveHyprlandInput(hyprlandInput, { silent: true });
			if (saved) {
				const now = Date.now();
				if (now - lastAutoSaveSuccessToastAt >= AUTO_SAVE_SUCCESS_TOAST_COOLDOWN) {
					lastAutoSaveSuccessToastAt = now;
					toast.success('Hyprland input settings saved.');
				}
			}
		}, AUTO_SAVE_DELAY);
	});

	$effect(() => {
		const variants = getVariantsForLayout(hyprlandInput, hyprlandInput.form.kb_layout);
		if (!variants.some((variant) => variant.name === hyprlandInput.form.kb_variant)) {
			hyprlandInput.form.kb_variant = '';
		}
	});

	onDestroy(() => {
		clearAutoSaveTimer();
	});

	function handleModelChange(value) {
		hyprlandInput.form.kb_model = value === '__none__' ? '' : value;
	}

	function handleLayoutChange(value) {
		hyprlandInput.form.kb_layout = value;
		hyprlandInput.form.kb_variant = '';
	}

	function handleVariantChange(value) {
		hyprlandInput.form.kb_variant = value === '__none__' ? '' : value;
	}

	function handleOptionToggle(code, checked) {
		const nextState = checked === 'indeterminate' ? true : Boolean(checked);
		const current = getSelectedOptionsSet(hyprlandInput.form);
		if (nextState) {
			current.add(code);
		} else {
			current.delete(code);
		}
		hyprlandInput.form.kb_options = Array.from(current).join(',');
	}

	async function handleReset() {
		clearAutoSaveTimer();
		resetHyprlandInputToDefaults(hyprlandInput);
		hyprlandInput.form.kb_layout = 'us';
		hyprlandInput.form.kb_variant = '';
		hyprlandInput.form.kb_model = '';
		hyprlandInput.form.kb_options = '';
		hyprlandInput.form.kb_rules = '';
		hyprlandInput.form.kb_file = '';
		hyprlandInput.form.numlock_by_default = false;
		hyprlandInput.form.resolve_binds_by_sym = false;
		hyprlandInput.form.repeat_rate = 25;
		hyprlandInput.form.repeat_delay = 600;
		await saveHyprlandInput(hyprlandInput, {
			message: 'Hyprland input settings reset to defaults.'
		});
	}
</script>

<Card.Root class="space-y-4">
	<Card.Header>
		<Card.Title class="uppercase">
			<div class="flex items-center justify-between">
				Keyboard <SettingsFilterToggle />
			</div>
		</Card.Title>
	</Card.Header>
	<Card.Content class="space-y-6 uppercase">
		<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
			<div class="flex flex-col gap-2">
				<Label for="kb_model" class="flex items-center gap-2">
					<span>Keyboard model</span>
					<Explainer explainerText="Selects a specific hardware model from XKB definitions." />
				</Label>
				<Select.Root
					type="single"
					value={hyprlandInput.form.kb_model || '__none__'}
					onValueChange={handleModelChange}
					disabled={hyprlandInput.isLoading || !modelOptions.length}
				>
					<Select.Trigger class="w-full max-w-[280px] uppercase">
						{#if hyprlandInput.isLoading}
							Loading…
						{:else if modelOptions.length}
							{selectedModelLabel}
						{:else}
							No keyboard models detected
						{/if}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="__none__">Default</Select.Item>
						{#each modelOptions as model (model.name)}
							<Select.Item value={model.name}>{model.description || model.name}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
				{#if !hyprlandInput.isLoading && !modelOptions.length}
					<p class="text-muted-foreground text-[0.625rem] leading-relaxed normal-case">
						No keyboard models were detected. Omarchist will use Hyprland's defaults.
					</p>
				{/if}
			</div>
			<div class="flex flex-col gap-2">
				<Label for="kb_layout" class="flex items-center gap-2">
					<span>Keyboard layout</span>
					<Explainer explainerText="The primary keyboard layout Hyprland should apply." />
				</Label>
				<Select.Root
					type="single"
					value={hyprlandInput.form.kb_layout}
					onValueChange={handleLayoutChange}
					disabled={hyprlandInput.isLoading || !layoutOptions.length}
				>
					<Select.Trigger class="w-full max-w-[280px] uppercase">
						{hyprlandInput.form.kb_layout || 'Select layout'}
					</Select.Trigger>
					<Select.Content>
						{#each layoutOptions as layout (layout.name)}
							<Select.Item value={layout.name}>{layout.description || layout.name}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="flex flex-col gap-2 md:col-span-2">
				<Label for="kb_variant" class="flex items-center gap-2">
					<span>Keyboard variant</span>
					<Explainer
						explainerText="Optional variant applied on top of the layout. Only variants valid for the selected layout are listed."
					/>
				</Label>
				<Select.Root
					type="single"
					value={hyprlandInput.form.kb_variant || '__none__'}
					onValueChange={handleVariantChange}
					disabled={hyprlandInput.isLoading || !variantOptions.length}
				>
					<Select.Trigger class="w-full uppercase">
						{hyprlandInput.form.kb_variant
							? (variantOptions.find((variant) => variant.name === hyprlandInput.form.kb_variant)
									?.description ?? hyprlandInput.form.kb_variant)
							: variantOptions.length
								? 'Default'
								: 'No variants available'}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="__none__">Default</Select.Item>
						{#each variantOptions as variant (variant.name)}
							<Select.Item value={variant.name}>{variant.description || variant.name}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="flex flex-col gap-2 md:col-span-2">
				<Label for="kb_options" class="flex items-center gap-2">
					<span>Additional options</span>
					<Explainer
						explainerText="Comma-separated XKB option codes (group:option). Toggle common options below to populate this field."
					/>
				</Label>
				<Textarea
					id="kb_options"
					rows="3"
					bind:value={hyprlandInput.form.kb_options}
					disabled={hyprlandInput.isLoading}
					spellcheck={false}
				></Textarea>
			</div>
			<div class="flex flex-col gap-2 md:col-span-2">
				<Label for="kb_rules" class="flex items-center gap-2">
					<span>Keyboard rules</span>
					<Explainer explainerText="Optional XKB rules override string." />
				</Label>
				<Input
					id="kb_rules"
					type="text"
					class="uppercase"
					bind:value={hyprlandInput.form.kb_rules}
					disabled={hyprlandInput.isLoading}
					spellcheck={false}
					placeholder="Default"
				/>
			</div>
			<div class="flex flex-col gap-2 md:col-span-2">
				<Label for="kb_file" class="flex items-center gap-2">
					<span>Custom keymap file</span>
					<Explainer explainerText="Path to a custom .xkb file to load instead of XKB rules." />
				</Label>
				<Input
					id="kb_file"
					type="text"
					class="uppercase"
					bind:value={hyprlandInput.form.kb_file}
					disabled={hyprlandInput.isLoading}
					spellcheck={false}
					placeholder="/path/to/custom.xkb"
				/>
			</div>
			<div class="flex items-center justify-between gap-4 md:col-span-2">
				<Label for="numlock_by_default" class="flex items-center gap-2">
					<span>Enable num lock</span>
					<Explainer explainerText="Engage num lock by default when Hyprland starts." />
				</Label>
				<Switch
					id="numlock_by_default"
					bind:checked={hyprlandInput.form.numlock_by_default}
					disabled={hyprlandInput.isLoading}
				/>
			</div>
			<div class="flex items-center justify-between gap-4 md:col-span-2">
				<Label for="resolve_binds_by_sym" class="flex items-center gap-2">
					<span>Resolve binds by symbol</span>
					<Explainer
						explainerText="Make keybinds follow the currently active layout by matching symbols."
					/>
				</Label>
				<Switch
					id="resolve_binds_by_sym"
					bind:checked={hyprlandInput.form.resolve_binds_by_sym}
					disabled={hyprlandInput.isLoading}
				/>
			</div>
			<div class="grid gap-4 md:col-span-2 md:grid-cols-2">
				<div class="flex flex-col gap-2">
					<Label for="repeat_rate" class="flex items-center gap-2">
						<span>Repeat rate</span>
						<Explainer explainerText="Number of repeats per second when holding a key." />
					</Label>
					<Input
						id="repeat_rate"
						type="number"
						class="w-28 uppercase"
						bind:value={hyprlandInput.form.repeat_rate}
						disabled={hyprlandInput.isLoading}
						min="1"
						max="100"
						step="1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.repeat_rate}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.repeat_rate}
						</p>
					{/if}
				</div>
				<div class="flex flex-col gap-2">
					<Label for="repeat_delay" class="flex items-center gap-2">
						<span>Repeat delay</span>
						<Explainer explainerText="Delay in milliseconds before a held key begins repeating." />
					</Label>
					<Input
						id="repeat_delay"
						type="number"
						class="w-28 uppercase"
						bind:value={hyprlandInput.form.repeat_delay}
						disabled={hyprlandInput.isLoading}
						min="100"
						max="10000"
						step="50"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.repeat_delay}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.repeat_delay}
						</p>
					{/if}
				</div>
			</div>
		</div>
		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<p class="text-muted-foreground text-xs">Toggle XKB options by group</p>
				{#if selectedOptions.size}
					<p class="text-muted-foreground text-xs">
						{selectedOptions.size} option{selectedOptions.size === 1 ? '' : 's'} selected
					</p>
				{/if}
			</div>
			{#if !optionGroups.length}
				<p class="text-muted-foreground text-xs">No keyboard options were detected.</p>
			{:else}
				<Accordion.Root type="multiple" class="space-y-2">
					{#each optionGroups as group (group.name)}
						<Accordion.Item value={group.name} class="border-border/60 rounded-lg border">
							<Accordion.Trigger
								class="w-full px-4 py-2 text-left text-sm font-semibold capitalize"
							>
								{group.description || group.name}
							</Accordion.Trigger>
							<Accordion.Content class="px-4 pb-3">
								<div class="flex flex-col gap-2">
									{#each group.options as option (option.name)}
										<div class="flex items-start gap-3 text-xs">
											<Checkbox
												checked={selectedOptions.has(`${group.name}:${option.name}`)}
												onCheckedChange={(checked) =>
													handleOptionToggle(`${group.name}:${option.name}`, checked)}
												disabled={hyprlandInput.isLoading}
												class="mt-1"
											/>
											<div class="flex flex-col gap-1">
												<span class="font-semibold tracking-wide">
													{option.description || option.name}
												</span>
												<span class="text-muted-foreground lowercase">
													{group.name}:{option.name}
												</span>
											</div>
										</div>
									{/each}
								</div>
							</Accordion.Content>
						</Accordion.Item>
					{/each}
				</Accordion.Root>
			{/if}
		</div>
	</Card.Content>
	<Card.Footer class="flex justify-end">
		<Button
			variant="outline"
			disabled={hyprlandInput.isLoading || hyprlandInput.isSaving}
			onclick={handleReset}
			class="uppercase"
		>
			Reset
		</Button>
	</Card.Footer>
</Card.Root>
