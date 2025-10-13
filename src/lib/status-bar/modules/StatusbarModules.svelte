<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import StatusbarSingleModule from './StatusbarSingleModule.svelte';
	import { KNOWN_MODULES } from '$lib/utils/waybarConfigUtils.js';

	let {
		modules = KNOWN_MODULES,
		getRegion = () => 'hidden',
		getFields = () => [],
		getConfig = () => ({}),
		onRegionChange = () => {},
		onFieldChange = () => {},
		disabled = false
	} = $props();

	function handleChange(event) {
		const { moduleId, position } = event.detail ?? {};
		if (!moduleId || !position) {
			return;
		}
		onRegionChange?.(moduleId, position);
	}

	function handleFieldChange(event) {
		const { moduleId, fieldKey, value } = event.detail ?? {};
		if (!moduleId || !fieldKey) {
			return;
		}
		onFieldChange?.(moduleId, fieldKey, value);
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Modules</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase">
			Choose where each module appears. Hidden modules remain available for later.
		</Card.Description>
	</Card.Header>
	<Card.Content class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
		{#each modules as module (module.id)}
			<StatusbarSingleModule
				{module}
				position={getRegion(module.id)}
				fields={getFields(module.id)}
				config={getConfig(module.id)}
				{disabled}
				on:change={handleChange}
				on:fieldChange={handleFieldChange}
			/>
		{/each}
	</Card.Content>
</Card.Root>
