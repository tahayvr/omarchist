<script>
	import * as RadioGroup from '$lib/components/ui/radio-group/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { createEventDispatcher } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import CardContent from '$lib/components/ui/card/card-content.svelte';

	let { value = 'Yaru-red' } = $props();

	const dispatch = createEventDispatcher();

	// Available Yaru icon variants (the ones I found)
	const iconThemes = [
		{ value: 'Yaru-red', label: 'Yaru Red', color: '#e92020' },
		{ value: 'Yaru-blue', label: 'Yaru Blue', color: '#208fe9' },
		{ value: 'Yaru-olive', label: 'Yaru Olive', color: '#636B2F' },
		{ value: 'Yaru-yellow', label: 'Yaru Yellow', color: '#e9ba20' },
		{ value: 'Yaru-purple', label: 'Yaru Purple', color: '#5e2750' },
		{ value: 'Yaru-magenta', label: 'Yaru Magenta', color: '#FF00FF' },
		{ value: 'Yaru-sage', label: 'Yaru Sage', color: '#123d18' }
	];

	function handleValueChange(newValue) {
		dispatch('change', { value: newValue });
	}
</script>

<div class="mt-4 flex flex-col gap-4">
	<Card.Root>
		<CardContent>
			<RadioGroup.Root
				bind:value
				onValueChange={handleValueChange}
				class="grid grid-cols-1 gap-3 lg:grid-cols-2"
			>
				{#each iconThemes as theme (theme.value)}
					<div class="hover:bg-muted/50 flex items-center space-x-3 p-2">
						<RadioGroup.Item value={theme.value} id={theme.value} />
						<div class="flex flex-1 items-center space-x-3">
							<div class="h-4 w-4" style="background-color: {theme.color}"></div>
							<Label for={theme.value} class="flex-1 cursor-pointer">
								{theme.label}
							</Label>
						</div>
					</div>
				{/each}
			</RadioGroup.Root>
		</CardContent>
	</Card.Root>
</div>
