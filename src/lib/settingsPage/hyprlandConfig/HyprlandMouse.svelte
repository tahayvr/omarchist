<script>
	import { onDestroy, onMount } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import Explainer from '$lib/components/Explainer.svelte';
	import { toast } from 'svelte-sonner';
	import SettingsFilterToggle from '../SettingsFilterToggle.svelte';
	import {
		initializeHyprlandInputState,
		loadHyprlandInput,
		saveHyprlandInput,
		resetHyprlandInputToDefaults,
		recomputeDirty,
		validateHyprlandInputForm
	} from '$lib/utils/hyprlandInputUtils.js';

	const hyprlandInput = $state(initializeHyprlandInputState());
	let settingsFilter = $state('basic');
	const isBasicMode = $derived(settingsFilter === 'basic');

	function shouldHideInBasic(isBasic = false) {
		return isBasicMode && !isBasic;
	}

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

	onDestroy(() => {
		clearAutoSaveTimer();
	});

	async function handleReset() {
		clearAutoSaveTimer();
		resetHyprlandInputToDefaults(hyprlandInput);
		hyprlandInput.form.sensitivity = 0;
		hyprlandInput.form.accel_profile = '';
		hyprlandInput.form.force_no_accel = false;
		hyprlandInput.form.left_handed = false;
		hyprlandInput.form.scroll_points = '';
		hyprlandInput.form.scroll_method = '';
		hyprlandInput.form.scroll_button = 0;
		hyprlandInput.form.scroll_button_lock = false;
		hyprlandInput.form.scroll_factor = 1;
		hyprlandInput.form.natural_scroll = false;
		hyprlandInput.form.follow_mouse = 1;
		hyprlandInput.form.follow_mouse_threshold = 0;
		hyprlandInput.form.focus_on_close = 0;
		hyprlandInput.form.mouse_refocus = true;
		hyprlandInput.form.float_switch_override_focus = 1;
		hyprlandInput.form.special_fallthrough = false;
		hyprlandInput.form.off_window_axis_events = 1;
		hyprlandInput.form.emulate_discrete_scroll = 1;
		hyprlandInput.form.touchpad = {
			disable_while_typing: true,
			natural_scroll: false,
			scroll_factor: 1,
			middle_button_emulation: false,
			tap_button_map: '',
			clickfinger_behavior: false,
			tap_to_click: true,
			drag_lock: 0,
			tap_and_drag: true,
			flip_x: false,
			flip_y: false,
			drag_3fg: 0
		};
		await saveHyprlandInput(hyprlandInput, {
			message: 'Mouse and touchpad settings reset to defaults.'
		});
	}
</script>

<Card.Root class="space-y-4">
	<Card.Header>
		<Card.Title class="uppercase">
			<div class="flex items-center justify-between">
				<span class="text-accent-foreground">Mouse & Touchpad</span>
				<SettingsFilterToggle bind:value={settingsFilter} />
			</div>
		</Card.Title>
	</Card.Header>
	<Card.Content class="space-y-8 uppercase">
		<section class="basic space-y-4">
			<h3 class="text-accent-foreground/70 text-sm font-semibold tracking-wide">
				Pointer behaviour
			</h3>
			<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
				<div class="flex items-center justify-between gap-2" class:hidden={shouldHideInBasic(true)}>
					<Label for="sensitivity" class="flex items-center gap-2">
						<span>Pointer sensitivity</span>
						<Explainer
							explainerText="-1 disables acceleration, 0 keeps default, 1 doubles sensitivity."
						/>
					</Label>
					<Input
						id="sensitivity"
						type="number"
						class="w-32 uppercase"
						bind:value={hyprlandInput.form.sensitivity}
						disabled={hyprlandInput.isLoading}
						min="-1"
						max="1"
						step="0.05"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.sensitivity}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.sensitivity}
						</p>
					{/if}
				</div>
				<div class="flex flex-col gap-2" class:hidden={shouldHideInBasic(true)}>
					<Label for="accel_profile" class="flex items-center gap-2">
						<span>Acceleration profile</span>
						<Explainer
							explainerText="Typical values: adaptive, flat, custom. Leave blank for Hyprland defaults."
						/>
					</Label>
					<Input
						id="accel_profile"
						type="text"
						class="uppercase"
						bind:value={hyprlandInput.form.accel_profile}
						disabled={hyprlandInput.isLoading}
						spellcheck={false}
						placeholder="adaptive"
					/>
				</div>
			</div>
			<div class="grid gap-4 md:grid-cols-2">
				<div
					class="flex items-center justify-between gap-4"
					class:hidden={shouldHideInBasic(false)}
				>
					<Label for="force_no_accel" class="flex items-center gap-2">
						<span>Disable libinput accel</span>
						<Explainer
							explainerText="Forces a flat acceleration curve from libinput by disabling acceleration."
						/>
					</Label>
					<Switch
						id="force_no_accel"
						bind:checked={hyprlandInput.form.force_no_accel}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(true)}>
					<Label for="left_handed" class="flex items-center gap-2">
						<span>Left-handed mode</span>
						<Explainer explainerText="Swaps primary and secondary mouse buttons." />
					</Label>
					<Switch
						id="left_handed"
						bind:checked={hyprlandInput.form.left_handed}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
			</div>
		</section>

		<section class="space-y-4">
			<h3 class="text-accent-foreground/70 text-sm font-semibold tracking-wide">
				Touchpad controls
			</h3>
			<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
				<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(true)}>
					<Label for="touchpad_disable_while_typing" class="flex items-center gap-2">
						<span>Disable while typing</span>
						<Explainer
							explainerText="Temporarily pauses touchpad input while keys are pressed to avoid accidental gestures."
						/>
					</Label>
					<Switch
						id="touchpad_disable_while_typing"
						bind:checked={hyprlandInput.form.touchpad.disable_while_typing}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(true)}>
					<Label for="touchpad_natural_scroll" class="flex items-center gap-2">
						<span>Natural scrolling (touchpad)</span>
						<Explainer
							explainerText="Reverses touchpad scroll direction to mimic touch interfaces."
						/>
					</Label>
					<Switch
						id="touchpad_natural_scroll"
						bind:checked={hyprlandInput.form.touchpad.natural_scroll}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div class="flex items-center justify-between gap-2" class:hidden={shouldHideInBasic(true)}>
					<Label for="touchpad_scroll_factor" class="flex items-center gap-2">
						<span>Scroll factor (touchpad)</span>
						<Explainer
							explainerText="Scales scroll distance generated by the touchpad. Values above 1 speed it up."
						/>
					</Label>
					<Input
						id="touchpad_scroll_factor"
						type="number"
						class="w-32 uppercase"
						bind:value={hyprlandInput.form.touchpad.scroll_factor}
						disabled={hyprlandInput.isLoading}
						min="0"
						step="0.1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.['touchpad.scroll_factor']}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors['touchpad.scroll_factor']}
						</p>
					{/if}
				</div>
				<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(true)}>
					<Label for="touchpad_middle_button_emulation" class="flex items-center gap-2">
						<span>Middle button emulation</span>
						<Explainer
							explainerText="Simulates a middle click by combining left and right clicks on the touchpad."
						/>
					</Label>
					<Switch
						id="touchpad_middle_button_emulation"
						bind:checked={hyprlandInput.form.touchpad.middle_button_emulation}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div class="flex flex-col gap-2" class:hidden={shouldHideInBasic(true)}>
					<Label for="touchpad_tap_button_map" class="flex items-center gap-2">
						<span>Tap button map</span>
						<Explainer explainerText="Accepts lrm or lmr to map three-finger taps." />
					</Label>
					<Input
						id="touchpad_tap_button_map"
						type="text"
						class="uppercase"
						bind:value={hyprlandInput.form.touchpad.tap_button_map}
						disabled={hyprlandInput.isLoading}
						spellcheck={false}
						placeholder="lrm"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.['touchpad.tap_button_map']}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors['touchpad.tap_button_map']}
						</p>
					{/if}
				</div>
				<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(true)}>
					<Label for="touchpad_clickfinger_behavior" class="flex items-center gap-2">
						<span>Clickfinger behaviour</span>
						<Explainer
							explainerText="Treats finger combinations as different buttons instead of tapping zones."
						/>
					</Label>
					<Switch
						id="touchpad_clickfinger_behavior"
						bind:checked={hyprlandInput.form.touchpad.clickfinger_behavior}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(true)}>
					<Label for="touchpad_tap_to_click" class="flex items-center gap-2">
						<span>Tap to click</span>
						<Explainer explainerText="Allow single-finger taps to register as clicks." />
					</Label>
					<Switch
						id="touchpad_tap_to_click"
						bind:checked={hyprlandInput.form.touchpad.tap_to_click}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div class="flex items-center justify-between gap-2" class:hidden={shouldHideInBasic(true)}>
					<Label for="touchpad_drag_lock" class="flex items-center gap-2">
						<span>Drag lock</span>
						<Explainer
							explainerText="Controls lock behaviour for tap-and-hold gestures. Accepts 0-2."
						/>
					</Label>
					<Input
						id="touchpad_drag_lock"
						type="number"
						class="w-24 uppercase"
						bind:value={hyprlandInput.form.touchpad.drag_lock}
						disabled={hyprlandInput.isLoading}
						min="0"
						max="2"
						step="1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.['touchpad.drag_lock']}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors['touchpad.drag_lock']}
						</p>
					{/if}
				</div>
				<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(true)}>
					<Label for="touchpad_tap_and_drag" class="flex items-center gap-2">
						<span>Tap and drag</span>
						<Explainer explainerText="Allows dragging windows with tap-and-hold gestures." />
					</Label>
					<Switch
						id="touchpad_tap_and_drag"
						bind:checked={hyprlandInput.form.touchpad.tap_and_drag}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div
					class="flex items-center justify-between gap-4"
					class:hidden={shouldHideInBasic(false)}
				>
					<Label for="touchpad_flip_x" class="flex items-center gap-2">
						<span>Flip X axis</span>
						<Explainer explainerText="Mirrors horizontal gesture direction." />
					</Label>
					<Switch
						id="touchpad_flip_x"
						bind:checked={hyprlandInput.form.touchpad.flip_x}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div
					class="flex items-center justify-between gap-4"
					class:hidden={shouldHideInBasic(false)}
				>
					<Label for="touchpad_flip_y" class="flex items-center gap-2">
						<span>Flip Y axis</span>
						<Explainer explainerText="Mirrors vertical gesture direction." />
					</Label>
					<Switch
						id="touchpad_flip_y"
						bind:checked={hyprlandInput.form.touchpad.flip_y}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div
					class="flex items-center justify-between gap-2"
					class:hidden={shouldHideInBasic(false)}
				>
					<Label for="touchpad_drag_3fg" class="flex items-center gap-2">
						<span>3-finger drag mode</span>
						<Explainer
							explainerText="Configure how three-finger drags behave. Accepts 0-2 per Hyprland docs."
						/>
					</Label>
					<Input
						id="touchpad_drag_3fg"
						type="number"
						class="w-24 uppercase"
						bind:value={hyprlandInput.form.touchpad.drag_3fg}
						disabled={hyprlandInput.isLoading}
						min="0"
						max="2"
						step="1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.['touchpad.drag_3fg']}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors['touchpad.drag_3fg']}
						</p>
					{/if}
				</div>
			</div>
		</section>

		<section class="space-y-4">
			<h3 class="text-accent-foreground/70 text-sm font-semibold tracking-wide">Scroll settings</h3>
			<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
				<div class="flex flex-col gap-2" class:hidden={shouldHideInBasic(false)}>
					<Label for="scroll_points" class="flex items-center gap-2">
						<span>Scroll points</span>
						<Explainer
							explainerText="Override scroll points configuration. Leave blank to inherit Hyprland defaults."
						/>
					</Label>
					<Input
						id="scroll_points"
						type="text"
						class="uppercase"
						bind:value={hyprlandInput.form.scroll_points}
						disabled={hyprlandInput.isLoading}
						spellcheck={false}
						placeholder=""
					/>
				</div>
				<div class="flex flex-col gap-2" class:hidden={shouldHideInBasic(true)}>
					<Label for="scroll_method" class="flex items-center gap-2">
						<span>Scroll method</span>
						<Explainer explainerText="Examples: two_finger, edge, no_scroll." />
					</Label>
					<Input
						id="scroll_method"
						type="text"
						class="uppercase"
						bind:value={hyprlandInput.form.scroll_method}
						disabled={hyprlandInput.isLoading}
						spellcheck={false}
						placeholder="two_finger"
					/>
				</div>
				<div class="flex items-center justify-between gap-2" class:hidden={shouldHideInBasic(true)}>
					<Label for="scroll_button" class="flex items-center gap-2">
						<span>Scroll button</span>
						<Explainer explainerText="Mouse button to hold for edge scrolling. Use 0 to disable." />
					</Label>
					<Input
						id="scroll_button"
						type="number"
						class="w-28 uppercase"
						bind:value={hyprlandInput.form.scroll_button}
						disabled={hyprlandInput.isLoading}
						step="1"
						min="0"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.scroll_button}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.scroll_button}
						</p>
					{/if}
				</div>
				<div class="flex items-center justify-between gap-2" class:hidden={shouldHideInBasic(true)}>
					<Label for="scroll_factor" class="flex items-center gap-2">
						<span>Scroll factor</span>
						<Explainer
							explainerText="Scales scroll distance. Values greater than 1 speed up scrolling."
						/>
					</Label>
					<Input
						id="scroll_factor"
						type="number"
						class="w-32 uppercase"
						bind:value={hyprlandInput.form.scroll_factor}
						disabled={hyprlandInput.isLoading}
						step="0.1"
						min="0"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.scroll_factor}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.scroll_factor}
						</p>
					{/if}
				</div>
			</div>
			<div class="grid gap-4 md:grid-cols-2">
				<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(true)}>
					<Label for="scroll_button_lock" class="flex items-center gap-2">
						<span>Lock scroll button</span>
						<Explainer explainerText="Keeps scrolling active until the button is pressed again." />
					</Label>
					<Switch
						id="scroll_button_lock"
						bind:checked={hyprlandInput.form.scroll_button_lock}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(true)}>
					<Label for="natural_scroll" class="flex items-center gap-2">
						<span>Natural scrolling</span>
						<Explainer explainerText="Reverses scroll direction to mimic touch interfaces." />
					</Label>
					<Switch
						id="natural_scroll"
						bind:checked={hyprlandInput.form.natural_scroll}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
			</div>
		</section>

		<section class="space-y-4" class:hidden={shouldHideInBasic(false)}>
			<h3 class="text-accent-foreground/70 text-sm font-semibold tracking-wide">
				Focus & window follow
			</h3>
			<div class="grid gap-4 md:grid-cols-3 md:gap-x-6 md:gap-y-4">
				<div class="flex flex-col gap-2">
					<Label for="follow_mouse" class="flex items-center gap-2">
						<span>Follow mouse level</span>
						<Explainer
							explainerText="0 disables focus follows mouse. Refer to Hyprland docs for mode meanings."
						/>
					</Label>
					<Input
						id="follow_mouse"
						type="number"
						class="w-24 uppercase"
						bind:value={hyprlandInput.form.follow_mouse}
						disabled={hyprlandInput.isLoading}
						min="0"
						max="3"
						step="1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.follow_mouse}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.follow_mouse}
						</p>
					{/if}
				</div>
				<div class="flex flex-col gap-2">
					<Label for="follow_mouse_threshold" class="flex items-center gap-2">
						<span>Follow threshold (ms)</span>
						<Explainer
							explainerText="Delay before focus changes when the pointer moves between windows."
						/>
					</Label>
					<Input
						id="follow_mouse_threshold"
						type="number"
						class="w-28 uppercase"
						bind:value={hyprlandInput.form.follow_mouse_threshold}
						disabled={hyprlandInput.isLoading}
						min="0"
						step="1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.follow_mouse_threshold}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.follow_mouse_threshold}
						</p>
					{/if}
				</div>
				<div class="flex flex-col gap-2">
					<Label for="focus_on_close" class="flex items-center gap-2">
						<span>Focus on close</span>
						<Explainer
							explainerText="0 keeps current behaviour, 1 focuses newly available window."
						/>
					</Label>
					<Input
						id="focus_on_close"
						type="number"
						class="w-24 uppercase"
						bind:value={hyprlandInput.form.focus_on_close}
						disabled={hyprlandInput.isLoading}
						min="0"
						max="1"
						step="1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.focus_on_close}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.focus_on_close}
						</p>
					{/if}
				</div>
				<div class="flex items-center justify-between gap-4">
					<Label for="mouse_refocus" class="flex items-center gap-2">
						<span>Refocus on pointer return</span>
						<Explainer
							explainerText="Re-focuses windows when the pointer re-enters them after leaving."
						/>
					</Label>
					<Switch
						id="mouse_refocus"
						bind:checked={hyprlandInput.form.mouse_refocus}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
				<div class="flex flex-col gap-2">
					<Label for="float_switch_override_focus" class="flex items-center gap-2">
						<span>Float switch override focus</span>
						<Explainer
							explainerText="Controls how floating windows override focus when toggled. Allowed values: 0-2."
						/>
					</Label>
					<Input
						id="float_switch_override_focus"
						type="number"
						class="w-28 uppercase"
						bind:value={hyprlandInput.form.float_switch_override_focus}
						disabled={hyprlandInput.isLoading}
						min="0"
						max="2"
						step="1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.float_switch_override_focus}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.float_switch_override_focus}
						</p>
					{/if}
				</div>
				<div class="flex items-center justify-between gap-4">
					<Label for="special_fallthrough" class="flex items-center gap-2">
						<span>Special workspace fallthrough</span>
						<Explainer
							explainerText="Allows focusing floating layers even when special workspaces are active."
						/>
					</Label>
					<Switch
						id="special_fallthrough"
						bind:checked={hyprlandInput.form.special_fallthrough}
						disabled={hyprlandInput.isLoading}
					/>
				</div>
			</div>
		</section>

		<section class="space-y-4" class:hidden={shouldHideInBasic(false)}>
			<h3 class="text-accent-foreground/70 text-sm font-semibold tracking-wide">
				Advanced behaviour
			</h3>
			<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
				<div class="flex flex-col gap-2">
					<Label for="off_window_axis_events" class="flex items-center gap-2">
						<span>Off-window axis events</span>
						<Explainer
							explainerText="Controls delivery of scroll events outside window bounds. Valid range: 0-3."
						/>
					</Label>
					<Input
						id="off_window_axis_events"
						type="number"
						class="w-28 uppercase"
						bind:value={hyprlandInput.form.off_window_axis_events}
						disabled={hyprlandInput.isLoading}
						min="0"
						max="3"
						step="1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.off_window_axis_events}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.off_window_axis_events}
						</p>
					{/if}
				</div>
				<div class="flex flex-col gap-2">
					<Label for="emulate_discrete_scroll" class="flex items-center gap-2">
						<span>Emulate discrete scroll</span>
						<Explainer
							explainerText="Useful on touchpads for generating wheel-style scroll events. Values: 0-2."
						/>
					</Label>
					<Input
						id="emulate_discrete_scroll"
						type="number"
						class="w-28 uppercase"
						bind:value={hyprlandInput.form.emulate_discrete_scroll}
						disabled={hyprlandInput.isLoading}
						min="0"
						max="2"
						step="1"
					/>
					{#if hyprlandInput.validation?.fieldErrors?.emulate_discrete_scroll}
						<p class="text-destructive text-[0.625rem] normal-case">
							{hyprlandInput.validation.fieldErrors.emulate_discrete_scroll}
						</p>
					{/if}
				</div>
			</div>
		</section>
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
