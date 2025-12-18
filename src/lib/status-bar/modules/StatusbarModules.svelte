<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import StatusbarSingleModule from './StatusbarSingleModule.svelte';
	import { KNOWN_MODULES } from '$lib/utils/waybar/waybarConfigUtils.js';

	let {
		modules = KNOWN_MODULES,
		getRegion = () => 'hidden',
		getFields = () => [],
		getConfig = () => ({}),
		onRegionChange = () => {},
		onFieldChange = () => {},
		onConfigChange = () => {},
		disabled = false
	} = $props();

	function handleChange(moduleId, position) {
		if (!moduleId || !position) {
			return;
		}
		onRegionChange?.(moduleId, position);
	}

	function handleFieldChange(moduleId, fieldKey, value) {
		if (!moduleId || !fieldKey) {
			return;
		}
		onFieldChange?.(moduleId, fieldKey, value);
	}

	function handleConfigChange(moduleId, moduleConfig) {
		if (!moduleId || !moduleConfig) {
			return;
		}
		onConfigChange?.(moduleId, moduleConfig);
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Modules</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase">
			Choose where each module appears & Configure module settings.
		</Card.Description>
	</Card.Header>
	<Card.Content class="grid grid-cols-1 gap-4 lg:grid-cols-2 2xl:grid-cols-3">
		{#each modules as module (module.id)}
			<StatusbarSingleModule
				{module}
				position={getRegion(module.id)}
				fields={getFields(module.id)}
				config={getConfig(module.id)}
				{disabled}
				onChange={(pos) => handleChange(module.id, pos)}
				onFieldChange={(key, val) => handleFieldChange(module.id, key, val)}
				onConfigChange={(cfg) => handleConfigChange(module.id, cfg)}
			/>
		{/each}
	</Card.Content>
</Card.Root>
