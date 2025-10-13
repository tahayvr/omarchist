<script>
	import { createEventDispatcher } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';

	let {
		module = { id: 'module', title: 'Module', description: 'Module description goes here.' },
		position = $bindable('hidden'),
		disabled = false
	} = $props();

	const dispatch = createEventDispatcher();

	function handleValueChange(nextPosition) {
		position = nextPosition || 'hidden';
		dispatch('change', { moduleId: module.id, position });
	}
</script>

<Card.Root data-disabled={disabled ? '' : undefined} class={disabled ? 'opacity-75' : ''}>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">{module.title}</Card.Title>
		<Card.Description class="text-muted-foreground text-xs tracking-wide uppercase">
			{module.description}
		</Card.Description>
	</Card.Header>
	<Card.Content>
		<ToggleGroup.Root
			type="single"
			aria-label="Module Position"
			bind:value={position}
			onValueChange={handleValueChange}
			size="lg"
			{disabled}
		>
			<ToggleGroup.Item value="left" class="uppercase">Left</ToggleGroup.Item>
			<ToggleGroup.Item value="center" class="uppercase">Center</ToggleGroup.Item>
			<ToggleGroup.Item value="right" class="uppercase">Right</ToggleGroup.Item>
			<ToggleGroup.Item value="hidden" class="uppercase">Hidden</ToggleGroup.Item>
		</ToggleGroup.Root>
	</Card.Content>
</Card.Root>
