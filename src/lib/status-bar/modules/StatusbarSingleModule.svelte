<script>
	import { createEventDispatcher } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';
	import StatusbarModuleDialog from './StatusbarModuleDialog.svelte';
	import StatusbarModuleStyleDialog from './StatusbarModuleStyleDialog.svelte';
	import { isModuleConfigurable } from '$lib/utils/waybar/moduleRegistry.js';

	let {
		module = { id: 'module', title: 'Module', description: 'Module description goes here.' },
		position = $bindable('hidden'),
		fields = [],
		config = {},
		style = {},
		disabled = false
	} = $props();

	const dispatch = createEventDispatcher();

	const showConfigButton = $derived(isModuleConfigurable(module.id));

	function handleValueChange(nextPosition) {
		position = nextPosition || 'hidden';
		dispatch('change', { moduleId: module.id, position });
	}

	function handleDialogConfigChange(event) {
		const { config: nextConfig } = event.detail ?? {};
		if (!nextConfig || typeof nextConfig !== 'object') {
			return;
		}
		dispatch('configChange', { moduleId: module.id, config: nextConfig });
	}

	function handleStyleChange(event) {
		const { style: nextStyle } = event.detail ?? {};
		if (!nextStyle || typeof nextStyle !== 'object') {
			return;
		}
		dispatch('styleChange', { moduleId: module.id, style: nextStyle });
	}
</script>

<Card.Root data-disabled={disabled ? '' : undefined} class={disabled ? 'opacity-75' : ''}>
	<Card.Header>
		<Card.Title class="text-accent-foreground/70 uppercase">{module.title}</Card.Title>
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

		<div class="mt-4 flex items-center justify-end gap-2">
			<StatusbarModuleStyleDialog {module} {style} {disabled} on:styleChange={handleStyleChange} />
			{#if showConfigButton}
				<StatusbarModuleDialog
					{module}
					{config}
					{disabled}
					{fields}
					on:configChange={handleDialogConfigChange}
				/>
			{/if}
		</div>
	</Card.Content>
</Card.Root>
