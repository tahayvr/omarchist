<script>
	import { onMount } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { toast } from 'svelte-sonner';
	import {
		initializeHyprlandGeneralState,
		loadHyprlandGeneral,
		saveHyprlandGeneral,
		resetHyprlandGeneralToDefaults,
		recomputeDirty,
		validateHyprlandGeneralForm
	} from '$lib/utils/hyprlandGeneralUtils.js';

	const hyprlandGeneral = $state(initializeHyprlandGeneralState());

	const layoutOptions = [
		{ label: 'MASTER', value: 'master' },
		{ label: 'DWINDLE', value: 'dwindle' }
	];

	onMount(async () => {
		await loadHyprlandGeneral(hyprlandGeneral);
	});

	$effect(() => {
		if (!hyprlandGeneral.snapshot) return;
		recomputeDirty(hyprlandGeneral);
	});

	$effect(() => {
		hyprlandGeneral.validation = validateHyprlandGeneralForm(hyprlandGeneral.form);
	});

	$effect(() => {
		if (hyprlandGeneral.error) {
			toast(hyprlandGeneral.error);
			hyprlandGeneral.error = null;
		}
	});

	$effect(() => {
		if (hyprlandGeneral.success) {
			toast(hyprlandGeneral.success);
			hyprlandGeneral.success = null;
		}
	});

	async function handleSave() {
		await saveHyprlandGeneral(hyprlandGeneral);
	}

	function handleReset() {
		resetHyprlandGeneralToDefaults(hyprlandGeneral);
	}
</script>

<Card.Root class="space-y-4">
	<Card.Header>
		<Card.Title class="uppercase">General</Card.Title>
		<Card.Description class="text-muted-foreground text-xs tracking-wide uppercase">
			Manage Hyprland windowing defaults. Values are applied through Omarchist overrides.
		</Card.Description>
	</Card.Header>
	<Card.Content class="space-y-6 uppercase">
		<div class="grid gap-4 md:grid-cols-2">
			<div class="flex items-center justify-between gap-4">
				<Label for="no_border_on_floating" class="flex-1">No border on floating windows</Label>
				<Switch
					id="no_border_on_floating"
					bind:checked={hyprlandGeneral.form.no_border_on_floating}
					disabled={hyprlandGeneral.isLoading}
				/>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="layout" class="flex-1">Layout</Label>
					<Select.Root
						type="single"
						name="layout"
						bind:value={hyprlandGeneral.form.layout}
						disabled={hyprlandGeneral.isLoading}
					>
						<Select.Trigger class="w-[180px]">
							{hyprlandGeneral.form.layout?.toUpperCase() ?? 'SELECT'}
						</Select.Trigger>
						<Select.Content>
							{#each layoutOptions as option}
								<Select.Item value={option.value}>{option.label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
				{#if hyprlandGeneral.validation.fieldErrors.layout}
					<p class="text-destructive text-[10px] font-semibold tracking-wide uppercase">
						{hyprlandGeneral.validation.fieldErrors.layout}
					</p>
				{/if}
			</div>
			<div class="flex items-center justify-between gap-4">
				<Label for="no_focus_fallback" class="flex-1">No focus fallback</Label>
				<Switch
					id="no_focus_fallback"
					bind:checked={hyprlandGeneral.form.no_focus_fallback}
					disabled={hyprlandGeneral.isLoading}
				/>
			</div>
			<div class="flex items-center justify-between gap-4">
				<Label for="resize_on_border" class="flex-1">Resize on border</Label>
				<Switch
					id="resize_on_border"
					bind:checked={hyprlandGeneral.form.resize_on_border}
					disabled={hyprlandGeneral.isLoading}
				/>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="extend_border_grab_area" class="flex-1">Extend border grab area</Label>
					<Input
						id="extend_border_grab_area"
						type="number"
						class="w-32"
						bind:value={hyprlandGeneral.form.extend_border_grab_area}
						disabled={hyprlandGeneral.isLoading}
						min="0"
					></Input>
				</div>
				{#if hyprlandGeneral.validation.fieldErrors.extend_border_grab_area}
					<p class="text-destructive text-[10px] font-semibold tracking-wide uppercase">
						{hyprlandGeneral.validation.fieldErrors.extend_border_grab_area}
					</p>
				{/if}
			</div>
			<div class="flex items-center justify-between gap-4">
				<Label for="hover_icon_on_border" class="flex-1">Hover icon on border</Label>
				<Switch
					id="hover_icon_on_border"
					bind:checked={hyprlandGeneral.form.hover_icon_on_border}
					disabled={hyprlandGeneral.isLoading}
				/>
			</div>
			<div class="flex items-center justify-between gap-4">
				<Label for="allow_tearing" class="flex-1">Allow tearing</Label>
				<Switch
					id="allow_tearing"
					bind:checked={hyprlandGeneral.form.allow_tearing}
					disabled={hyprlandGeneral.isLoading}
				/>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="resize_corner" class="flex-1">Resize corner</Label>
					<Input
						id="resize_corner"
						type="number"
						class="w-32"
						bind:value={hyprlandGeneral.form.resize_corner}
						disabled={hyprlandGeneral.isLoading}
						min="0"
						max="4"
					></Input>
				</div>
				{#if hyprlandGeneral.validation.fieldErrors.resize_corner}
					<p class="text-destructive text-[10px] font-semibold tracking-wide uppercase">
						{hyprlandGeneral.validation.fieldErrors.resize_corner}
					</p>
				{/if}
			</div>
		</div>
	</Card.Content>
	<Card.Footer class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
		<div class="flex gap-2">
			<Button
				onclick={handleSave}
				disabled={!hyprlandGeneral.dirty ||
					hyprlandGeneral.isSaving ||
					hyprlandGeneral.isLoading ||
					!hyprlandGeneral.validation.isValid}
				class="uppercase"
			>
				{hyprlandGeneral.isSaving ? 'Saving…' : 'Save changes'}
			</Button>
			<Button
				variant="outline"
				onclick={handleReset}
				disabled={hyprlandGeneral.isLoading}
				class="uppercase"
			>
				Reset to defaults
			</Button>
		</div>
		{#if hyprlandGeneral.isLoading}
			<span class="text-muted-foreground text-xs tracking-wide uppercase">Loading settings…</span>
		{/if}
	</Card.Footer>
</Card.Root>
