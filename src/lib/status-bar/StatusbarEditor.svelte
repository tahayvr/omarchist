<script>
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import * as Card from '$lib/components/ui/card/index.js';
	import StatusbarHeader from './StatusbarHeader.svelte';
	import StatusbarLayout from './layout/StatusbarLayout.svelte';
	import StatusbarModules from './modules/StatusbarModules.svelte';
	import {
		KNOWN_MODULES,
		initializeWaybarConfigState,
		loadWaybarConfig,
		saveWaybarConfig,
		resetWaybarConfigToDefaults,
		getModuleRegion,
		setModuleRegion
	} from '$lib/utils/waybarConfigUtils.js';

	const config = $state(initializeWaybarConfigState());
	const moduleDefinitions = KNOWN_MODULES;

	onMount(async () => {
		await loadWaybarConfig(config);
	});

	$effect(() => {
		if (config.error) {
			toast.error('Waybar configuration error', {
				description: config.error
			});
			config.error = null;
		}
	});

	$effect(() => {
		if (config.success) {
			toast.success(config.success);
			config.success = null;
		}
	});

	function getRegion(moduleId) {
		return getModuleRegion(config, moduleId);
	}

	function handleRegionChange(moduleId, region) {
		setModuleRegion(config, moduleId, region);
	}

	async function handleSave() {
		await saveWaybarConfig(config);
	}

	function handleReset() {
		resetWaybarConfigToDefaults(config);
	}

	const isBusy = $derived(config.isLoading || config.isSaving);
	const isValid = $derived(config.validation?.isValid ?? true);
</script>

<div class="space-y-6">
	<StatusbarHeader
		isLoading={config.isLoading}
		isSaving={config.isSaving}
		dirty={config.dirty}
		{isValid}
		onSave={handleSave}
		onReset={handleReset}
	/>

	<div class="flex w-full flex-col gap-6 xl:flex-row">
		<div class="flex w-full flex-col gap-6 xl:w-1/2">
			<StatusbarLayout layout={config.layout} modules={moduleDefinitions} />
		</div>
		<div class="flex w-full flex-col gap-6 xl:w-1/2">
			<StatusbarModules
				modules={moduleDefinitions}
				{getRegion}
				onRegionChange={handleRegionChange}
				disabled={isBusy}
			/>
		</div>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title class="text-accent-foreground uppercase">Generated JSON Preview</Card.Title>
			<Card.Description class="text-muted-foreground text-xs tracking-wide uppercase">
				This is what will be written to <code>config.jsonc</code> when you save.
			</Card.Description>
		</Card.Header>
		<Card.Content>
			<pre class="bg-muted max-h-72 overflow-auto rounded px-4 py-3 text-xs">
{config.raw ?? '// Save to populate preview'}
			</pre>
		</Card.Content>
	</Card.Root>
</div>
