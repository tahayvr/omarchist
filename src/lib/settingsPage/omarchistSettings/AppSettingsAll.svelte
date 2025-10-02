<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { onMount } from 'svelte';
	import {
		loadSettings,
		updateSetting,
		clearError,
		validateSettingsState,
		getSettingDescription
	} from '$lib/utils/settingsUtils.js';

	const appSettings = $state({
		autoApplyTheme: true,
		isLoading: false,
		error: null,
		isInitialized: false
	});

	onMount(async () => {
		await loadSettings(appSettings);
	});
	// TODO: Refactor for future expansion
	// Handle checkbox change
	async function handleAutoApplyChange(checked) {
		console.log('🔧 Settings Page: Updating autoApplyTheme to:', checked);

		// Validate the new value before attempting to save
		if (typeof checked !== 'boolean') {
			console.error('🔧 Settings Page: Invalid value type for autoApplyTheme:', typeof checked);
			appSettings.error = 'Invalid setting value. Please refresh the page and try again.';
			return;
		}

		const success = await updateSetting(appSettings, 'autoApplyTheme', checked);
		console.log('🔧 Settings Page: Update result:', success);

		// Validate state after update
		const validation = validateSettingsState(appSettings);
		if (!validation.isValid) {
			console.warn('🔧 Settings Page: State validation failed after update:', validation.errors);
		}
	}

	// Watch for changes to autoApplyTheme and trigger save
	let previousValue = appSettings.autoApplyTheme;
	$effect(() => {
		// Skip the initial load
		if (!appSettings.isInitialized) return;

		// trigger if the value changes
		if (appSettings.autoApplyTheme !== previousValue) {
			console.log(
				'🔧 Checkbox value changed from',
				previousValue,
				'to',
				appSettings.autoApplyTheme
			);
			handleAutoApplyChange(appSettings.autoApplyTheme);
			previousValue = appSettings.autoApplyTheme;
		}
	});

	// TODO: Clear error message
	function handleClearError() {
		clearError(appSettings);
	}

	// Get setting description for tooltip
	function getTooltipText() {
		return getSettingDescription('autoApplyTheme');
	}
</script>

<div class="flex flex-col items-center justify-center">
	<Card.Root class="w-full max-w-lg">
		<Card.Header>
			<Card.Title class="uppercase">Theme Designer</Card.Title>
		</Card.Header>
		<Card.Content class="space-y-4">
			<!-- Settings controls -->
			<div class="flex items-center gap-3">
				<Checkbox
					id="auto-apply"
					bind:checked={appSettings.autoApplyTheme}
					disabled={appSettings.isLoading}
					class={appSettings.isLoading ? 'opacity-50' : ''}
				/>
				<Label
					for="auto-apply"
					class={`${appSettings.isLoading ? 'opacity-50' : ''} cursor-pointer`}
					title={getTooltipText()}
				>
					Apply theme when entering Edit-Mode
				</Label>
			</div>
		</Card.Content>
	</Card.Root>
</div>
