<script>
	import { onDestroy, onMount } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Accordion from '$lib/components/ui/accordion/index.js';
	import { toast } from 'svelte-sonner';
	import {
		initializeHyprlandDecorationState,
		loadHyprlandDecoration,
		saveHyprlandDecoration,
		resetHyprlandDecorationToDefaults,
		recomputeDirty,
		validateHyprlandDecorationForm
	} from '$lib/utils/hyprlandDecorationUtils.js';
	import Explainer from '$lib/components/Explainer.svelte';
	import SettingsFilterToggle from '../SettingsFilterToggle.svelte';

	const hyprlandDecoration = $state(initializeHyprlandDecorationState());
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
		if (hyprlandDecoration.autoSaveHandle) {
			clearTimeout(hyprlandDecoration.autoSaveHandle);
			hyprlandDecoration.autoSaveHandle = null;
		}
	}

	onMount(async () => {
		await loadHyprlandDecoration(hyprlandDecoration);
	});

	$effect(() => {
		hyprlandDecoration.validation = validateHyprlandDecorationForm(hyprlandDecoration.form);
	});

	$effect(() => {
		if (!hyprlandDecoration.hasHydrated) {
			lastFormSignature = '';
			lastSavedFormSignature = '';
			return;
		}

		const formSignature = JSON.stringify(hyprlandDecoration.form ?? {});
		const savedSignature = JSON.stringify(hyprlandDecoration.lastSavedForm ?? {});

		if (formSignature === lastFormSignature && savedSignature === lastSavedFormSignature) {
			return;
		}

		lastFormSignature = formSignature;
		lastSavedFormSignature = savedSignature;
		recomputeDirty(hyprlandDecoration, {
			currentSignature: formSignature,
			lastSavedSignature: savedSignature
		});
	});

	$effect(() => {
		const { validation, dirty, hasHydrated, isLoading } = hyprlandDecoration;
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
				toast.error('Hyprland decoration settings need attention.', {
					description
				});
			}
			return;
		}

		lastValidationToastSignature = null;
	});

	$effect(() => {
		if (hyprlandDecoration.error) {
			toast(hyprlandDecoration.error);
			hyprlandDecoration.error = null;
		}
	});

	$effect(() => {
		if (hyprlandDecoration.success) {
			toast(hyprlandDecoration.success);
			hyprlandDecoration.success = null;
		}
	});

	$effect(() => {
		const { autoSaveHandle, dirty, hasHydrated, isLoading, isSaving, validation } =
			hyprlandDecoration;
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
		hyprlandDecoration.autoSaveHandle = setTimeout(async () => {
			hyprlandDecoration.autoSaveHandle = null;
			const saved = await saveHyprlandDecoration(hyprlandDecoration, { silent: true });
			if (saved) {
				const now = Date.now();
				if (now - lastAutoSaveSuccessToastAt >= AUTO_SAVE_SUCCESS_TOAST_COOLDOWN) {
					lastAutoSaveSuccessToastAt = now;
					toast.success('Hyprland decoration settings saved.');
				}
			}
		}, AUTO_SAVE_DELAY);
	});

	async function handleReset() {
		clearAutoSaveTimer();
		resetHyprlandDecorationToDefaults(hyprlandDecoration);
		await saveHyprlandDecoration(hyprlandDecoration, {
			message: 'Hyprland decoration settings reset to defaults.'
		});
	}

	onDestroy(() => {
		clearAutoSaveTimer();
	});
</script>

<Card.Root class="space-y-4">
	<Card.Header>
		<Card.Title class="uppercase">
			<div class="flex items-center justify-between">
				Decoration <SettingsFilterToggle bind:value={settingsFilter} />
			</div>
		</Card.Title>
	</Card.Header>
	<Card.Content class="space-y-6 uppercase">
		<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
			<div class="basic flex flex-col gap-2" class:hidden={shouldHideInBasic(true)}>
				<div class="flex items-center justify-between gap-4">
					<Label for="rounding" class="flex-1">
						Rounding
						<Explainer explainerText="rounded corners’ radius" />
					</Label>
					<Input
						id="rounding"
						type="number"
						class="w-24"
						bind:value={hyprlandDecoration.form.rounding}
						disabled={hyprlandDecoration.isLoading}
						min="0"
					></Input>
				</div>
			</div>
			<div class="basic flex flex-col gap-2" class:hidden={shouldHideInBasic(true)}>
				<div class="flex items-center justify-between gap-4">
					<Label for="rounding_power" class="flex-1">
						Rounding power
						<Explainer
							explainerText="adjusts the curve used for rounding corners, larger is smoother, 2.0 is a circle, 4.0 is a squircle, 1.0 is a triangular corner. [1.0 - 10.0]"
						/>
					</Label>
					<Input
						id="rounding_power"
						type="number"
						class="w-full max-w-[200px]"
						bind:value={hyprlandDecoration.form.rounding_power}
						disabled={hyprlandDecoration.isLoading}
						min="1.0"
						max="10.0"
						step="0.1"
					></Input>
				</div>
			</div>
			<div class="basic flex flex-col gap-2" class:hidden={shouldHideInBasic(true)}>
				<div class="flex items-center justify-between gap-4">
					<Label for="active_opacity" class="flex-1">
						Active opacity
						<Explainer explainerText="opacity of active windows. [0.0 - 1.0]" />
					</Label>
					<Input
						id="active_opacity"
						type="number"
						class="w-full max-w-[200px]"
						bind:value={hyprlandDecoration.form.active_opacity}
						disabled={hyprlandDecoration.isLoading}
						min="0.0"
						max="1.0"
						step="0.1"
					></Input>
				</div>
			</div>
			<div class="basic flex flex-col gap-2" class:hidden={shouldHideInBasic(true)}>
				<div class="flex items-center justify-between gap-4">
					<Label for="inactive_opacity" class="flex-1">
						Inactive opacity
						<Explainer explainerText="opacity of inactive windows. [0.0 - 1.0]" />
					</Label>
					<Input
						id="inactive_opacity"
						type="number"
						class="w-full max-w-[200px]"
						bind:value={hyprlandDecoration.form.inactive_opacity}
						disabled={hyprlandDecoration.isLoading}
						min="0.0"
						max="1.0"
						step="0.1"
					></Input>
				</div>
			</div>
			<div class="flex flex-col gap-2" class:hidden={shouldHideInBasic(false)}>
				<div class="flex items-center justify-between gap-4">
					<Label for="fullscreen_opacity" class="flex-1">
						Fullscreen opacity
						<Explainer explainerText="opacity of fullscreen windows. [0.0 - 1.0]" />
					</Label>
					<Input
						id="fullscreen_opacity"
						type="number"
						class="w-full max-w-[200px]"
						bind:value={hyprlandDecoration.form.fullscreen_opacity}
						disabled={hyprlandDecoration.isLoading}
						min="0.0"
						max="1.0"
						step="0.1"
					></Input>
				</div>
			</div>
			<div
				class="basic flex items-center justify-between gap-4"
				class:hidden={shouldHideInBasic(true)}
			>
				<Label for="dim_modal" class="flex-1">
					Dim modal
					<Explainer explainerText="enables dimming of parents of modal windows" />
				</Label>
				<Switch
					id="dim_modal"
					bind:checked={hyprlandDecoration.form.dim_modal}
					disabled={hyprlandDecoration.isLoading}
				/>
			</div>
			<div
				class="basic flex items-center justify-between gap-4"
				class:hidden={shouldHideInBasic(true)}
			>
				<Label for="dim_inactive" class="flex-1">
					Dim inactive
					<Explainer explainerText="enables dimming of inactive windows" />
				</Label>
				<Switch
					id="dim_inactive"
					bind:checked={hyprlandDecoration.form.dim_inactive}
					disabled={hyprlandDecoration.isLoading}
				/>
			</div>
			<div class="flex flex-col gap-2" class:hidden={shouldHideInBasic(false)}>
				<div class="flex items-center justify-between gap-4">
					<Label for="dim_strength" class="flex-1">
						Dim strength
						<Explainer explainerText="how much inactive windows should be dimmed [0.0 - 1.0]" />
					</Label>
					<Input
						id="dim_strength"
						type="number"
						class="w-24"
						bind:value={hyprlandDecoration.form.dim_strength}
						disabled={hyprlandDecoration.isLoading}
						min="0.0"
						max="1.0"
						step="0.1"
					></Input>
				</div>
			</div>
			<div class="flex flex-col gap-2" class:hidden={shouldHideInBasic(false)}>
				<div class="flex items-center justify-between gap-4">
					<Label for="dim_special" class="flex-1">
						Dim special
						<Explainer
							explainerText="how much to dim the rest of the screen by when a special workspace is open. [0.0 - 1.0]"
						/>
					</Label>
					<Input
						id="dim_special"
						type="number"
						class="w-24"
						bind:value={hyprlandDecoration.form.dim_special}
						disabled={hyprlandDecoration.isLoading}
						min="0.0"
						max="1.0"
						step="0.1"
					></Input>
				</div>
			</div>
			<div class="flex flex-col gap-2" class:hidden={shouldHideInBasic(false)}>
				<div class="flex items-center justify-between gap-4">
					<Label for="dim_around" class="flex-1">
						Dim around
						<Explainer
							explainerText="how much the 'dimaround' window rule should dim by. [0.0 - 1.0]"
						/>
					</Label>
					<Input
						id="dim_around"
						type="number"
						class="w-24"
						bind:value={hyprlandDecoration.form.dim_around}
						disabled={hyprlandDecoration.isLoading}
						min="0.0"
						max="1.0"
						step="0.1"
					></Input>
				</div>
			</div>
			<div class="flex flex-col gap-2" class:hidden={shouldHideInBasic(false)}>
				<div class="flex items-center justify-between gap-4">
					<Label for="screen_shader" class="flex-1">
						Screen shader
						<Explainer
							explainerText="a path to a custom shader to be applied at the end of rendering."
							docUrl="https://wiki.hypr.land/Configuring/Variables/#decoration"
						/>
					</Label>
					<Input
						id="screen_shader"
						type="text"
						class="w-full max-w-[260px]"
						bind:value={hyprlandDecoration.form.screen_shader}
						disabled={hyprlandDecoration.isLoading}
						spellcheck={false}
					></Input>
				</div>
			</div>
			<div class="flex items-center justify-between gap-4" class:hidden={shouldHideInBasic(false)}>
				<Label for="border_part_of_window" class="flex-1">
					Border part of window
					<Explainer explainerText="whether the window border should be a part of the window" />
				</Label>
				<Switch
					id="border_part_of_window"
					bind:checked={hyprlandDecoration.form.border_part_of_window}
					disabled={hyprlandDecoration.isLoading}
				/>
			</div>
		</div>
		<Accordion.Root type="single" class={shouldHideInBasic(false) ? 'hidden' : ''}>
			<Accordion.Item>
				<Accordion.Trigger class="uppercase">Blur</Accordion.Trigger>
				<Accordion.Content>
					<section class="space-y-6 uppercase">
						<p class="text-muted-foreground -mt-3 text-xs tracking-wide uppercase">
							Configure blur effect for windows.
						</p>

						<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
							<div class="flex items-center justify-between gap-4">
								<Label for="blur_enabled" class="flex-1">
									Enable blur
									<Explainer explainerText="toggles Hyprland's kawase blur." />
								</Label>
								<Switch
									id="blur_enabled"
									bind:checked={hyprlandDecoration.form.blur.enabled}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="blur_size" class="flex-1">
										Kernel size
										<Explainer explainerText="strength of the blur kernel. [>= 1]" />
									</Label>
									<Input
										id="blur_size"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.blur.size}
										disabled={hyprlandDecoration.isLoading}
										min="1"
									></Input>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="blur_passes" class="flex-1">
										Passes
										<Explainer explainerText="number of kawase passes. [>= 1]" />
									</Label>
									<Input
										id="blur_passes"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.blur.passes}
										disabled={hyprlandDecoration.isLoading}
										min="1"
									></Input>
								</div>
							</div>
							<div class="flex items-center justify-between gap-4">
								<Label for="blur_ignore_opacity" class="flex-1">
									Ignore opacity
									<Explainer explainerText="disables respect for window opacity when blurring." />
								</Label>
								<Switch
									id="blur_ignore_opacity"
									bind:checked={hyprlandDecoration.form.blur.ignore_opacity}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex items-center justify-between gap-4">
								<Label for="blur_new_optimizations" class="flex-1">
									New optimizations
									<Explainer explainerText="use optimized blur implementation." />
								</Label>
								<Switch
									id="blur_new_optimizations"
									bind:checked={hyprlandDecoration.form.blur.new_optimizations}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex items-center justify-between gap-4">
								<Label for="blur_xray" class="flex-1">
									Xray
									<Explainer explainerText="renders blur above window contents." />
								</Label>
								<Switch
									id="blur_xray"
									bind:checked={hyprlandDecoration.form.blur.xray}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="blur_noise" class="flex-1">
										Noise
										<Explainer explainerText="adds dithering noise to blur. [0.0 - 1.0]" />
									</Label>
									<Input
										id="blur_noise"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.blur.noise}
										disabled={hyprlandDecoration.isLoading}
										min="0.0"
										max="1.0"
										step="0.01"
									></Input>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="blur_contrast" class="flex-1">
										Contrast
										<Explainer explainerText="adjusts blur contrast. [0.0 - 2.0]" />
									</Label>
									<Input
										id="blur_contrast"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.blur.contrast}
										disabled={hyprlandDecoration.isLoading}
										min="0.0"
										max="2.0"
										step="0.01"
									></Input>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="blur_brightness" class="flex-1">
										Brightness
										<Explainer explainerText="adjusts blur brightness. [0.0 - 2.0]" />
									</Label>
									<Input
										id="blur_brightness"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.blur.brightness}
										disabled={hyprlandDecoration.isLoading}
										min="0.0"
										max="2.0"
										step="0.01"
									></Input>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="blur_vibrancy" class="flex-1">
										Vibrancy
										<Explainer explainerText="adds saturation to blur. [0.0 - 1.0]" />
									</Label>
									<Input
										id="blur_vibrancy"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.blur.vibrancy}
										disabled={hyprlandDecoration.isLoading}
										min="0.0"
										max="1.0"
										step="0.01"
									></Input>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="blur_vibrancy_darkness" class="flex-1">
										Vibrancy darkness
										<Explainer
											explainerText="controls darkening when vibrancy is enabled. [0.0 - 1.0]"
										/>
									</Label>
									<Input
										id="blur_vibrancy_darkness"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.blur.vibrancy_darkness}
										disabled={hyprlandDecoration.isLoading}
										min="0.0"
										max="1.0"
										step="0.01"
									></Input>
								</div>
							</div>
							<div class="flex items-center justify-between gap-4">
								<Label for="blur_special" class="flex-1">
									Blur special workspace
									<Explainer explainerText="applies blur behind special workspace background." />
								</Label>
								<Switch
									id="blur_special"
									bind:checked={hyprlandDecoration.form.blur.special}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex items-center justify-between gap-4">
								<Label for="blur_popups" class="flex-1">
									Blur popups
									<Explainer explainerText="enable blur for popup windows." />
								</Label>
								<Switch
									id="blur_popups"
									bind:checked={hyprlandDecoration.form.blur.popups}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="blur_popups_ignorealpha" class="flex-1">
										Popups ignore alpha
										<Explainer explainerText="opacity to treat popup backgrounds as. [0.0 - 1.0]" />
									</Label>
									<Input
										id="blur_popups_ignorealpha"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.blur.popups_ignorealpha}
										disabled={hyprlandDecoration.isLoading}
										min="0.0"
										max="1.0"
										step="0.01"
									></Input>
								</div>
							</div>
							<div class="flex items-center justify-between gap-4">
								<Label for="blur_input_methods" class="flex-1">
									Blur input methods
									<Explainer explainerText="blur virtual keyboard/input method windows." />
								</Label>
								<Switch
									id="blur_input_methods"
									bind:checked={hyprlandDecoration.form.blur.input_methods}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="blur_input_methods_ignorealpha" class="flex-1">
										Input methods ignore alpha
										<Explainer explainerText="opacity baseline for input methods. [0.0 - 1.0]" />
									</Label>
									<Input
										id="blur_input_methods_ignorealpha"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.blur.input_methods_ignorealpha}
										disabled={hyprlandDecoration.isLoading}
										min="0.0"
										max="1.0"
										step="0.01"
									></Input>
								</div>
							</div>
						</div>
					</section>
				</Accordion.Content>
			</Accordion.Item>
			<Accordion.Item>
				<Accordion.Trigger class="uppercase">Shadow</Accordion.Trigger>
				<Accordion.Content
					><section class="space-y-6 uppercase">
						<p class="text-muted-foreground -mt-3 text-xs tracking-wide uppercase">
							Configure shadow effect for windows.
						</p>

						<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
							<div class="flex items-center justify-between gap-4">
								<Label for="shadow_enabled" class="flex-1">
									Enable shadows
									<Explainer explainerText="toggle Hyprland window shadows." />
								</Label>
								<Switch
									id="shadow_enabled"
									bind:checked={hyprlandDecoration.form.shadow.enabled}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="shadow_range" class="flex-1">
										Range
										<Explainer explainerText="shadow spread distance. [>= 0]" />
									</Label>
									<Input
										id="shadow_range"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.shadow.range}
										disabled={hyprlandDecoration.isLoading}
										min="0"
									></Input>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="shadow_render_power" class="flex-1">
										Render power
										<Explainer explainerText="quality/strength of shadow render. [1 - 4]" />
									</Label>
									<Input
										id="shadow_render_power"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.shadow.render_power}
										disabled={hyprlandDecoration.isLoading}
										min="1"
										max="4"
									></Input>
								</div>
							</div>
							<div class="flex items-center justify-between gap-4">
								<Label for="shadow_sharp" class="flex-1">
									Sharp shadows
									<Explainer explainerText="use a sharper shadow falloff." />
								</Label>
								<Switch
									id="shadow_sharp"
									bind:checked={hyprlandDecoration.form.shadow.sharp}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex items-center justify-between gap-4">
								<Label for="shadow_ignore_window" class="flex-1">
									Ignore window
									<Explainer explainerText="skip drawing shadows when window is fullscreen." />
								</Label>
								<Switch
									id="shadow_ignore_window"
									bind:checked={hyprlandDecoration.form.shadow.ignore_window}
									disabled={hyprlandDecoration.isLoading}
								/>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="shadow_color" class="flex-1">
										Shadow color
										<Explainer explainerText="RGBA color for active window shadows." />
									</Label>
									<Input
										id="shadow_color"
										type="text"
										class="w-full max-w-[220px]"
										bind:value={hyprlandDecoration.form.shadow.color}
										disabled={hyprlandDecoration.isLoading}
										spellcheck={false}
									></Input>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="shadow_color_inactive" class="flex-1">
										Inactive color
										<Explainer explainerText="shadows for unfocused windows." />
									</Label>
									<Input
										id="shadow_color_inactive"
										type="text"
										class="w-full max-w-[220px]"
										bind:value={hyprlandDecoration.form.shadow.color_inactive}
										disabled={hyprlandDecoration.isLoading}
										spellcheck={false}
									></Input>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="shadow_offset" class="flex-1">
										Offset
										<Explainer explainerText="shadow offset vector, e.g. '0 0'." />
									</Label>
									<Input
										id="shadow_offset"
										type="text"
										class="w-full max-w-[220px]"
										bind:value={hyprlandDecoration.form.shadow.offset}
										disabled={hyprlandDecoration.isLoading}
										spellcheck={false}
									></Input>
								</div>
							</div>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-4">
									<Label for="shadow_scale" class="flex-1">
										Scale
										<Explainer explainerText="shadow size multiplier. [0.0 - 1.0]" />
									</Label>
									<Input
										id="shadow_scale"
										type="number"
										class="w-24"
										bind:value={hyprlandDecoration.form.shadow.scale}
										disabled={hyprlandDecoration.isLoading}
										min="0.0"
										max="1.0"
										step="0.01"
									></Input>
								</div>
							</div>
						</div>
					</section>
				</Accordion.Content>
			</Accordion.Item>
		</Accordion.Root>
	</Card.Content>
	<Card.Footer class="flex justify-end">
		<Button
			variant="outline"
			onclick={handleReset}
			disabled={hyprlandDecoration.isLoading || hyprlandDecoration.isSaving}
			class="uppercase"
		>
			Reset
		</Button>
	</Card.Footer>
</Card.Root>
