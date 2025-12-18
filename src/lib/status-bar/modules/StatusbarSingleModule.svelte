<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';
	import StatusbarModuleConfigDialog from './StatusbarModuleConfigDialog.svelte';
	import { isModuleConfigurable } from '$lib/utils/waybar/moduleRegistry.js';

	let {
		module = { id: 'module', title: 'Module', description: 'Module description goes here.' },
		position = $bindable('hidden'),
		fields = [],
		config = {},
		disabled = false,
		onChange = () => {},
		onConfigChange = () => {}
	} = $props();

	const showConfigButton = $derived(isModuleConfigurable(module.id));

	function handleValueChange(nextPosition) {
		position = nextPosition || 'hidden';
		onChange(position);
	}

	function handleDialogConfigChange(nextConfig) {
		if (!nextConfig || typeof nextConfig !== 'object') {
			return;
		}
		onConfigChange(nextConfig);
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
		<div class="flex flex-wrap items-center justify-between gap-4">
			<ToggleGroup.Root
				type="single"
				class="flex flex-wrap gap-2"
				aria-label="Module Position"
				bind:value={position}
				onValueChange={handleValueChange}
				size="lg"
				{disabled}
			>
				<ToggleGroup.Item value="left" class="min-w-[70px] flex-1 uppercase">Left</ToggleGroup.Item>
				<ToggleGroup.Item value="center" class="min-w-[70px] flex-1 uppercase"
					>Center</ToggleGroup.Item
				>
				<ToggleGroup.Item value="right" class="min-w-[70px] flex-1 uppercase"
					>Right</ToggleGroup.Item
				>
				<ToggleGroup.Item value="hidden" class="min-w-[70px] flex-1 uppercase"
					>Hidden</ToggleGroup.Item
				>
			</ToggleGroup.Root>

			{#if showConfigButton}
				<StatusbarModuleConfigDialog
					{module}
					{config}
					{disabled}
					{fields}
					onConfigChange={handleDialogConfigChange}
				/>
			{/if}
		</div>
	</Card.Content>
</Card.Root>
