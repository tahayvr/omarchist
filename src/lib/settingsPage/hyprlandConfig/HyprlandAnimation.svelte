<script>
	import { onDestroy, onMount } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { toast } from 'svelte-sonner';
	import {
		initializeHyprlandAnimationState,
		loadHyprlandAnimation,
		saveHyprlandAnimation,
		resetHyprlandAnimationToDefaults,
		recomputeDirty,
		validateHyprlandAnimationForm
	} from '$lib/utils/hyprlandAnimationUtils.js';
	import Explainer from '$lib/components/Explainer.svelte';

	const hyprlandAnimation = $state(initializeHyprlandAnimationState());

	const AUTO_SAVE_DELAY = 800;
	const AUTO_SAVE_SUCCESS_TOAST_COOLDOWN = 2000;

	let lastValidationToastSignature = null;
	let lastAutoSaveSuccessToastAt = 0;
	let lastFormSignature = '';
	let lastSavedFormSignature = '';

	function clearAutoSaveTimer() {
		if (hyprlandAnimation.autoSaveHandle) {
			clearTimeout(hyprlandAnimation.autoSaveHandle);
			hyprlandAnimation.autoSaveHandle = null;
		}
	}

	onMount(async () => {
		await loadHyprlandAnimation(hyprlandAnimation);
	});

	$effect(() => {
		hyprlandAnimation.validation = validateHyprlandAnimationForm(hyprlandAnimation.form);
	});

	$effect(() => {
		if (!hyprlandAnimation.hasHydrated) {
			lastFormSignature = '';
			lastSavedFormSignature = '';
			return;
		}

		const formSignature = JSON.stringify(hyprlandAnimation.form ?? {});
		const savedSignature = JSON.stringify(hyprlandAnimation.lastSavedForm ?? {});

		if (formSignature === lastFormSignature && savedSignature === lastSavedFormSignature) {
			return;
		}

		lastFormSignature = formSignature;
		lastSavedFormSignature = savedSignature;
		recomputeDirty(hyprlandAnimation, {
			currentSignature: formSignature,
			lastSavedSignature: savedSignature
		});
	});

	$effect(() => {
		const { validation, dirty, hasHydrated, isLoading } = hyprlandAnimation;
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
				toast.error('Hyprland animation settings need attention.', {
					description
				});
			}
			return;
		}

		lastValidationToastSignature = null;
	});

	$effect(() => {
		if (hyprlandAnimation.error) {
			toast(hyprlandAnimation.error);
			hyprlandAnimation.error = null;
		}
	});

	$effect(() => {
		if (hyprlandAnimation.success) {
			toast(hyprlandAnimation.success);
			hyprlandAnimation.success = null;
		}
	});

	$effect(() => {
		const { autoSaveHandle, dirty, hasHydrated, isLoading, isSaving, validation } =
			hyprlandAnimation;
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
		hyprlandAnimation.autoSaveHandle = setTimeout(async () => {
			hyprlandAnimation.autoSaveHandle = null;
			const saved = await saveHyprlandAnimation(hyprlandAnimation, { silent: true });
			if (saved) {
				const now = Date.now();
				if (now - lastAutoSaveSuccessToastAt >= AUTO_SAVE_SUCCESS_TOAST_COOLDOWN) {
					lastAutoSaveSuccessToastAt = now;
					toast.success('Hyprland animation settings saved.');
				}
			}
		}, AUTO_SAVE_DELAY);
	});

	async function handleReset() {
		clearAutoSaveTimer();
		resetHyprlandAnimationToDefaults(hyprlandAnimation);
		await saveHyprlandAnimation(hyprlandAnimation, {
			message: 'Hyprland animation settings reset to defaults.'
		});
	}

	onDestroy(() => {
		clearAutoSaveTimer();
	});
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Animation Settings</Card.Title>
	</Card.Header>
	<Card.Content class="space-y-6 uppercase">
		<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
			<div class="flex items-center justify-between gap-4">
				<Label for="enabled">Enable Animations</Label>
				<Switch
					id="enabled"
					bind:checked={hyprlandAnimation.form.enabled}
					disabled={hyprlandAnimation.isSaving}
				/>
			</div>
			<!-- Workspace Wraparound -->
			<div class="flex items-center justify-between space-x-4">
				<div class="flex-1 space-y-1">
					<div class="flex items-center gap-2">
						<Label for="workspace_wraparound">Workspace Wraparound</Label>
						<Explainer
							explainerText="Enable or disable all animations in Hyprland. When disabled, windows and workspaces
							will transition instantly without any animation effects."
						/>
					</div>
				</div>
				<Switch
					id="workspace_wraparound"
					bind:checked={hyprlandAnimation.form.workspace_wraparound}
					disabled={hyprlandAnimation.isSaving}
				/>
			</div>
		</div>
	</Card.Content>
	<Card.Footer class="flex justify-end">
		<Button
			variant="outline"
			disabled={hyprlandAnimation.isLoading || hyprlandAnimation.isSaving}
			onclick={handleReset}
			class="uppercase"
		>
			Reset
		</Button>
	</Card.Footer>
</Card.Root>
