<script>
	import { onDestroy, onMount } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Accordion from '$lib/components/ui/accordion/index.js';
	import { toast } from 'svelte-sonner';
	import {
		initializeHyprlandGeneralState,
		loadHyprlandGeneral,
		saveHyprlandGeneral,
		resetHyprlandGeneralToDefaults,
		recomputeDirty,
		validateHyprlandGeneralForm
	} from '$lib/utils/hyprlandGeneralUtils.js';
	import Explainer from '$lib/components/Explainer.svelte';
	import HyprlandGeneralSnap from './HyprlandGeneralSnap.svelte';

	const hyprlandGeneral = $state(initializeHyprlandGeneralState());

	const layoutOptions = [
		{ label: 'MASTER', value: 'master' },
		{ label: 'DWINDLE', value: 'dwindle' }
	];

	const resizeCornerOptions = [
		{ label: 'Disable', value: 0 },
		{ label: 'Top left', value: 1 },
		{ label: 'Top right', value: 2 },
		{ label: 'Bottom right', value: 3 },
		{ label: 'Bottom left', value: 4 }
	];

	let lastValidationToastSignature = null;
	let lastAutoSaveSuccessToastAt = 0;

	function getResizeCornerLabel(value) {
		const option = resizeCornerOptions.find((entry) => entry.value === value);
		return option ? option.label : 'Select';
	}

	onMount(async () => {
		await loadHyprlandGeneral(hyprlandGeneral);
	});

	const AUTO_SAVE_DELAY = 800;
	const AUTO_SAVE_SUCCESS_TOAST_COOLDOWN = 2000;

	function clearAutoSaveTimer() {
		if (hyprlandGeneral.autoSaveHandle) {
			clearTimeout(hyprlandGeneral.autoSaveHandle);
			hyprlandGeneral.autoSaveHandle = null;
		}
	}

	$effect(() => {
		hyprlandGeneral.validation = validateHyprlandGeneralForm(hyprlandGeneral.form);
	});

	let lastFormSignature = '';
	let lastSavedFormSignature = '';

	$effect(() => {
		if (!hyprlandGeneral.hasHydrated) {
			lastFormSignature = '';
			lastSavedFormSignature = '';
			return;
		}

		const formSignature = JSON.stringify(hyprlandGeneral.form ?? {});
		const savedSignature = JSON.stringify(hyprlandGeneral.lastSavedForm ?? {});

		if (formSignature === lastFormSignature && savedSignature === lastSavedFormSignature) {
			return;
		}

		lastFormSignature = formSignature;
		lastSavedFormSignature = savedSignature;
		recomputeDirty(hyprlandGeneral, {
			currentSignature: formSignature,
			lastSavedSignature: savedSignature
		});
	});

	$effect(() => {
		const { validation, dirty, hasHydrated, isLoading } = hyprlandGeneral;
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
				toast.error('Hyprland general settings need attention.', {
					description
				});
			}
			return;
		}

		lastValidationToastSignature = null;
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

	$effect(() => {
		const { autoSaveHandle, dirty, hasHydrated, isLoading, isSaving, validation } = hyprlandGeneral;
		void autoSaveHandle;

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
			return;
		}

		clearAutoSaveTimer();
		hyprlandGeneral.autoSaveHandle = setTimeout(async () => {
			hyprlandGeneral.autoSaveHandle = null;
			const saved = await saveHyprlandGeneral(hyprlandGeneral, { silent: true });
			if (saved) {
				const now = Date.now();
				if (now - lastAutoSaveSuccessToastAt >= AUTO_SAVE_SUCCESS_TOAST_COOLDOWN) {
					lastAutoSaveSuccessToastAt = now;
					toast.success('Hyprland general settings saved.');
				}
			}
		}, AUTO_SAVE_DELAY);
	});

	async function handleReset() {
		clearAutoSaveTimer();
		resetHyprlandGeneralToDefaults(hyprlandGeneral);
		await saveHyprlandGeneral(hyprlandGeneral, {
			message: 'Hyprland general settings reset to defaults.'
		});
	}

	onDestroy(() => {
		clearAutoSaveTimer();
	});
</script>

<Card.Root class="space-y-4">
	<Card.Header>
		<Card.Title class="uppercase">General</Card.Title>
	</Card.Header>
	<Card.Content class="space-y-6 uppercase">
		<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="border_size" class="flex-1">
						Border size
						<Explainer explainerText="size of the border around windows" />
					</Label>
					<Input
						id="border_size"
						type="number"
						class="w-24"
						bind:value={hyprlandGeneral.form.border_size}
						disabled={hyprlandGeneral.isLoading}
						min="0"
					></Input>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="gaps_in" class="flex-1">
						Gaps in
						<Explainer
							explainerText="gaps between windows; supports CSS-style values like 5,10,15,20"
							docUrl="https://wiki.hypr.land/Configuring/Variables/#general"
						/>
					</Label>
					<Input
						id="gaps_in"
						type="text"
						class="w-full max-w-[200px]"
						bind:value={hyprlandGeneral.form.gaps_in}
						disabled={hyprlandGeneral.isLoading}
						spellcheck={false}
					></Input>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="gaps_out" class="flex-1">
						Gaps out
						<Explainer
							explainerText="gaps between windows and monitor edges; supports CSS-style values"
							docUrl="https://wiki.hypr.land/Configuring/Variables/#general"
						/>
					</Label>
					<Input
						id="gaps_out"
						type="text"
						class="w-full max-w-[200px]"
						bind:value={hyprlandGeneral.form.gaps_out}
						disabled={hyprlandGeneral.isLoading}
						spellcheck={false}
					></Input>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="float_gaps" class="flex-1">
						Float gaps
						<Explainer
							explainerText="gaps for floating windows; use -1 to revert to Hyprland defaults"
							docUrl="https://wiki.hypr.land/Configuring/Variables/#general"
						/>
					</Label>
					<Input
						id="float_gaps"
						type="text"
						class="w-full max-w-[200px]"
						bind:value={hyprlandGeneral.form.float_gaps}
						disabled={hyprlandGeneral.isLoading}
						spellcheck={false}
					></Input>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="gaps_workspaces" class="flex-1">
						Workspace gaps
						<Explainer explainerText="additional gaps between workspaces. Stacks with 'gaps out'" />
					</Label>
					<Input
						id="gaps_workspaces"
						type="number"
						class="w-24"
						bind:value={hyprlandGeneral.form.gaps_workspaces}
						disabled={hyprlandGeneral.isLoading}
						min="0"
					></Input>
				</div>
			</div>
			<div class="flex items-center justify-between gap-4">
				<Label for="no_border_on_floating" class="flex-1">
					No border on floating windows
					<Explainer explainerText="disable borders for floating windows" />
				</Label>
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
							{#each layoutOptions as option (option.value)}
								<Select.Item value={option.value}>{option.label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
			</div>
			<div class="flex items-center justify-between gap-4">
				<Label for="no_focus_fallback" class="flex-1">
					No focus fallback
					<Explainer
						explainerText="if true, will not fall back to the next available window when moving focus in a direction where no window was found"
					/>
				</Label>
				<Switch
					id="no_focus_fallback"
					bind:checked={hyprlandGeneral.form.no_focus_fallback}
					disabled={hyprlandGeneral.isLoading}
				/>
			</div>
			<div class="flex items-center justify-between gap-4">
				<Label for="resize_on_border" class="flex-1">
					Resize on border
					<Explainer
						explainerText="enables resizing windows by clicking and dragging on borders and gaps"
					/>
				</Label>
				<Switch
					id="resize_on_border"
					bind:checked={hyprlandGeneral.form.resize_on_border}
					disabled={hyprlandGeneral.isLoading}
				/>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="extend_border_grab_area" class="flex-1">
						Extend border grab area
						<Explainer
							explainerText="extends the area around the border where you can click and drag on, only used when 'resize_on_border' is on."
						/>
					</Label>
					<Input
						id="extend_border_grab_area"
						type="number"
						class="w-24"
						bind:value={hyprlandGeneral.form.extend_border_grab_area}
						disabled={hyprlandGeneral.isLoading}
						min="0"
					></Input>
				</div>
			</div>
			<div class="flex items-center justify-between gap-4">
				<Label for="hover_icon_on_border" class="flex-1">
					Hover icon on border
					<Explainer
						explainerText="show a cursor icon when hovering over borders, only used when 'resize_on_border' is on."
					/>
				</Label>
				<Switch
					id="hover_icon_on_border"
					bind:checked={hyprlandGeneral.form.hover_icon_on_border}
					disabled={hyprlandGeneral.isLoading}
				/>
			</div>
			<div class="flex items-center justify-between gap-4">
				<Label for="allow_tearing" class="flex-1">
					Allow tearing
					<Explainer
						explainerText="master switch for allowing screen tearing."
						docUrl="https://wiki.hypr.land/Configuring/Tearing/"
					/>
				</Label>
				<Switch
					id="allow_tearing"
					bind:checked={hyprlandGeneral.form.allow_tearing}
					disabled={hyprlandGeneral.isLoading}
				/>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<Label for="resize_corner" class="flex-1">
						Resize corner
						<Explainer
							explainerText="forces floating windows to use a specific corner when being resized."
						/>
					</Label>
					<Select.Root
						type="single"
						name="resize_corner"
						value={String(hyprlandGeneral.form.resize_corner ?? 0)}
						onValueChange={(value) => {
							hyprlandGeneral.form.resize_corner = Number(value);
						}}
						disabled={hyprlandGeneral.isLoading}
					>
						<Select.Trigger class="w-[180px]">
							{getResizeCornerLabel(hyprlandGeneral.form.resize_corner)}
						</Select.Trigger>
						<Select.Content>
							{#each resizeCornerOptions as option (option.value)}
								<Select.Item value={String(option.value)}>{option.label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
			</div>
		</div>
		<Accordion.Root type="single">
			<Accordion.Item>
				<Accordion.Trigger class="uppercase">Snap</Accordion.Trigger>
				<Accordion.Content>
					<HyprlandGeneralSnap {hyprlandGeneral} />
				</Accordion.Content>
			</Accordion.Item>
		</Accordion.Root>
	</Card.Content>
	<Card.Footer class="flex justify-end">
		<Button
			variant="outline"
			onclick={handleReset}
			disabled={hyprlandGeneral.isLoading || hyprlandGeneral.isSaving}
			class="uppercase"
		>
			Reset
		</Button>
	</Card.Footer>
</Card.Root>
