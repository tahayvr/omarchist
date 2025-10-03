<script>
	import { onDestroy, onMount } from 'svelte';
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

	let lastValidationToastSignature = null;
	let lastAutoSaveSuccessToastAt = 0;

	onMount(async () => {
		await loadHyprlandGeneral(hyprlandGeneral);
	});

	$effect(() => {
		if (!hyprlandGeneral.snapshot) return;
		recomputeDirty(hyprlandGeneral);
	});

	const AUTO_SAVE_DELAY = 600;
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
		<Card.Description class="text-muted-foreground text-xs tracking-wide uppercase">
			Manage Hyprland windowing defaults. Values are applied through Omarchist overrides.
		</Card.Description>
	</Card.Header>
	<Card.Content class="space-y-6 uppercase">
		<div class="grid gap-x-8 gap-y-4 md:grid-cols-2">
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
							{#each layoutOptions as option (option.value)}
								<Select.Item value={option.value}>{option.label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
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
			</div>
		</div>
	</Card.Content>
	<Card.Footer class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
		<Button
			variant="outline"
			onclick={handleReset}
			disabled={hyprlandGeneral.isLoading || hyprlandGeneral.isSaving}
			class="uppercase"
		>
			Reset to defaults
		</Button>
	</Card.Footer>
</Card.Root>
