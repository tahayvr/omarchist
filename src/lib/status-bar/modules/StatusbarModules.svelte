<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import StatusbarSingleModule from './StatusbarSingleModule.svelte';
	import { KNOWN_MODULES } from '$lib/utils/waybarConfigUtils.js';

	let {
		modules = KNOWN_MODULES,
		getRegion = () => 'hidden',
		getFields = () => [],
		getConfig = () => ({}),
		getStyle = () => ({}),
		onRegionChange = () => {},
		onFieldChange = () => {},
		onConfigChange = () => {},
		onStyleChange = () => {},
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

	function handleConfigChange(event) {
		const { moduleId, config: moduleConfig } = event.detail ?? {};
		if (!moduleId || !moduleConfig) {
			return;
		}
		onConfigChange?.(moduleId, moduleConfig);
	}

	function handleStyleChange(event) {
		const { moduleId, style: moduleStyle } = event.detail ?? {};
		if (!moduleId || !moduleStyle) {
			return;
		}
		onStyleChange?.(moduleId, moduleStyle);
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Modules</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase">
			Choose where each module appears. Configure each module's settings.
		</Card.Description>
	</Card.Header>
	<Card.Content class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
		{#each modules as module (module.id)}
			<StatusbarSingleModule
				{module}
				position={getRegion(module.id)}
				fields={getFields(module.id)}
				config={getConfig(module.id)}
				style={getStyle(module.id)}
				{disabled}
				on:change={handleChange}
				on:fieldChange={handleFieldChange}
				on:configChange={handleConfigChange}
				on:styleChange={handleStyleChange}
			/>
		{/each}
	</Card.Content>
</Card.Root>
