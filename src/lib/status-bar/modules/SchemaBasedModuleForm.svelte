<script>
	import { createEventDispatcher } from 'svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as ScrollArea from '$lib/components/ui/scroll-area/index.js';
	import FieldRenderer from './FieldRenderer.svelte';
	import { hydrateFieldState, buildConfigFromFieldState } from '$lib/utils/waybar/schemaUtils.js';

	let { schema, config = {}, module, disabled = false } = $props();

	const dispatch = createEventDispatcher();

	// Internal field state for the form
	let fieldState = $state({});
	let lastConfigSignature = '';
	let lastEmittedSignature = '';
	let wasInitialized = false;

	// Get fields grouped by tab
	const fieldsByTab = $derived.by(() => {
		if (!schema || !schema.properties) {
			return {};
		}

		const groups = {};

		// Initialize tabs
		if (schema.tabs) {
			for (const tab of schema.tabs) {
				groups[tab.id] = {
					label: tab.label,
					description: tab.description,
					fields: []
				};
			}
		}

		// Group fields by tab
		for (const [key, field] of Object.entries(schema.properties)) {
			const tabId = field.tab || 'general';
			if (!groups[tabId]) {
				groups[tabId] = {
					label: tabId,
					description: '',
					fields: []
				};
			}
			groups[tabId].fields.push({ key, ...field });
		}

		return groups;
	});

	const tabIds = $derived(Object.keys(fieldsByTab));
	const defaultTab = $derived(tabIds.length > 0 ? tabIds[0] : 'general');

	function computeConfigSignature() {
		try {
			return JSON.stringify(config ?? {});
		} catch {
			return '';
		}
	}

	function hydrateFromConfig(force = false) {
		const signature = computeConfigSignature();
		if (!force && signature === lastConfigSignature) {
			return;
		}
		lastConfigSignature = signature;
		fieldState = hydrateFieldState(config, schema);
		wasInitialized = true;
	}

	// Initialize field state on mount
	hydrateFromConfig(true);

	// Watch for external config changes
	$effect(() => {
		const signature = computeConfigSignature();
		if (signature !== lastConfigSignature) {
			hydrateFromConfig(false);
		}
	});

	// Emit config changes when field state changes
	$effect(() => {
		if (!wasInitialized) {
			return;
		}

		const newConfig = buildConfigFromFieldState(fieldState, schema);
		const signature = JSON.stringify(newConfig);

		if (signature === lastEmittedSignature) {
			return;
		}

		lastEmittedSignature = signature;
		dispatch('configChange', { config: newConfig });
	});
</script>

{#if tabIds.length > 0}
	<Tabs.Root value={defaultTab} class="w-full">
		<Tabs.List class="grid w-full" style="grid-template-columns: repeat({tabIds.length}, 1fr);">
			{#each tabIds as tabId}
				{@const tab = fieldsByTab[tabId]}
				<Tabs.Trigger value={tabId} class="text-xs tracking-wide uppercase">
					{tab.label}
				</Tabs.Trigger>
			{/each}
		</Tabs.List>

		{#each tabIds as tabId}
			{@const tab = fieldsByTab[tabId]}
			<Tabs.Content value={tabId}>
				<ScrollArea.Root class="h-[400px] pr-4">
					<div class="space-y-4">
						{#if tab.description}
							<p class="text-muted-foreground text-sm">
								{tab.description}
							</p>
						{/if}

						{#each tab.fields as field}
							<FieldRenderer
								{field}
								bind:value={fieldState[field.key]}
								fieldKey={field.key}
								{disabled}
							/>
						{/each}
					</div>
				</ScrollArea.Root>
			</Tabs.Content>
		{/each}
	</Tabs.Root>
{:else}
	<div class="text-muted-foreground flex h-64 items-center justify-center">
		<p>No configuration fields available.</p>
	</div>
{/if}
