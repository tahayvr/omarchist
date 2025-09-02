<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	let { themeName = $bindable() } = $props();
	let isLightMode = $state(false);
	let isLoading = $state(false);
	let previousLightMode = $state(false);

	// Load light mode status when theme name changes
	$effect(async () => {
		if (themeName && themeName.trim()) {
			try {
				const lightMode = await invoke('is_theme_light_mode', { theme_name: themeName });
				isLightMode = lightMode;
				previousLightMode = lightMode;
			} catch (error) {
				console.error('Failed to check light mode status:', error);
				isLightMode = false;
				previousLightMode = false;
			}
		}
	});

	// Handle light mode changes when checkbox is toggled
	$effect(async () => {
		// Skip if this is the initial load or if we're currently loading
		if (!themeName || !themeName.trim() || isLoading) return;

		// Skip if the value hasn't actually changed
		if (isLightMode === previousLightMode) return;

		isLoading = true;
		try {
			await invoke('set_theme_light_mode', {
				theme_name: themeName,
				is_light: isLightMode
			});
			previousLightMode = isLightMode;
			console.log(`Light mode ${isLightMode ? 'enabled' : 'disabled'} for theme: ${themeName}`);
		} catch (error) {
			console.error('Failed to set light mode:', error);
			// Revert checkbox state on error
			isLightMode = previousLightMode;
		} finally {
			isLoading = false;
		}
	});
</script>

<div class="flex w-full flex-col">
	<Card.Root class="w-lg max-w-4xl">
		<Card.Content class="space-y-8">
			<div class="flex items-center gap-3">
				<Checkbox id="light-mode" bind:checked={isLightMode} disabled={isLoading} />
				<Label for="light-mode" class="uppercase">Light Mode</Label>
			</div>
		</Card.Content>
	</Card.Root>
</div>
