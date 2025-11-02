<script>
	import { createEventDispatcher } from 'svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as ScrollArea from '$lib/components/ui/scroll-area/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import FieldRenderer from './FieldRenderer.svelte';
	import { hydrateFieldState, buildConfigFromFieldState } from '$lib/utils/waybar/schemaUtils.js';
	import { getModuleDefinition } from '$lib/utils/waybar/moduleRegistry.js';

	let { module = null, config = {}, disabled = false } = $props();

	const moduleDefinition = getModuleDefinition(module?.id);
	const schema = moduleDefinition?.schema;

	const dispatch = createEventDispatcher();

	// Internal field state for the form
	let fieldState = $state({});
	let lastConfigSignature = '';
	let lastEmittedSignature = '';
	let wasInitialized = false;

	// Mock Bluetooth state for preview
	let mockBluetoothState = $state({
		status: 'connected',
		num_connections: 2,
		controller_alias: 'MyController',
		controller_address: '00:11:22:33:44:55',
		controller_address_type: 'public',
		device_alias: 'AirPods Pro',
		device_address: 'AA:BB:CC:DD:EE:FF',
		device_address_type: 'public',
		device_battery_percentage: 85
	});

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

		// Initialize the emitted signature to prevent spurious change events
		const initialConfig = buildConfigFromFieldState(fieldState, schema);
		lastEmittedSignature = JSON.stringify(initialConfig);

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

	// Track previous values to handle switching to __custom
	let previousFieldValues = $state({});

	// Handle switching to __custom - populate custom field with previous value
	$effect(() => {
		if (!wasInitialized || !schema || !schema.properties) {
			return;
		}

		for (const [key, field] of Object.entries(schema.properties)) {
			// Check if this is a select field
			if (field.type === 'select' || field.enum) {
				const currentValue = fieldState[key];
				const previousValue = previousFieldValues[key];

				// If just switched to __custom
				if (currentValue === '__custom' && previousValue !== '__custom') {
					const customKey = `${key}-custom`;
					const customField = schema.properties[customKey];

					// If custom field exists and previous value was a real value (not a sentinel)
					if (
						customField &&
						previousValue &&
						!previousValue.startsWith('__') &&
						previousValue !== '__custom'
					) {
						// Populate custom field with the previous selected value
						fieldState[customKey] = previousValue;
					}
				}

				// Update previous value tracker
				previousFieldValues[key] = currentValue;
			}
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

	// Format preview helpers
	function formatBluetoothPreview(formatString, state) {
		if (!formatString || formatString.startsWith('__')) {
			return '';
		}

		try {
			let result = formatString;

			// Replace format placeholders
			const replacements = {
				'{status}': state.status,
				'{num_connections}': state.num_connections.toString(),
				'{controller_alias}': state.controller_alias,
				'{controller_address}': state.controller_address,
				'{controller_address_type}': state.controller_address_type,
				'{device_alias}': state.device_alias,
				'{device_address}': state.device_address,
				'{device_address_type}': state.device_address_type,
				'{device_battery_percentage}': state.device_battery_percentage.toString(),
				'{device_enumerate}': `${state.device_alias}\t${state.device_address}\nHeadphones\tBB:CC:DD:EE:FF:00`
			};

			for (const [placeholder, value] of Object.entries(replacements)) {
				result = result.replace(new RegExp(placeholder.replace(/[{}]/g, '\\$&'), 'g'), value);
			}

			return result;
		} catch {
			return 'Invalid format';
		}
	}

	// Get current formats for preview
	const currentFormat = $derived.by(() => {
		const format = fieldState.format;
		if (format === '__custom') {
			return fieldState['format-custom'] || ' {status}';
		}
		if (format === '__default') {
			return ' {status}';
		}
		return format || ' {status}';
	});

	const currentDisabledFormat = $derived.by(() => {
		const format = fieldState['format-disabled'];
		if (format === '__custom') {
			return fieldState['format-disabled-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentOffFormat = $derived.by(() => {
		const format = fieldState['format-off'];
		if (format === '__custom') {
			return fieldState['format-off-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentOnFormat = $derived.by(() => {
		const format = fieldState['format-on'];
		if (format === '__custom') {
			return fieldState['format-on-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentConnectedFormat = $derived.by(() => {
		const format = fieldState['format-connected'];
		if (format === '__custom') {
			return fieldState['format-connected-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentConnectedBatteryFormat = $derived.by(() => {
		const format = fieldState['format-connected-battery'];
		if (format === '__custom') {
			return fieldState['format-connected-battery-custom'] || '';
		}
		if (format === '__default') {
			return currentConnectedFormat;
		}
		return format || '';
	});

	const disabledPreview = $derived(
		formatBluetoothPreview(currentDisabledFormat, mockBluetoothState)
	);
	const offPreview = $derived(formatBluetoothPreview(currentOffFormat, mockBluetoothState));
	const onPreview = $derived(formatBluetoothPreview(currentOnFormat, mockBluetoothState));
	const connectedPreview = $derived(
		formatBluetoothPreview(currentConnectedFormat, mockBluetoothState)
	);
	const connectedBatteryPreview = $derived(
		formatBluetoothPreview(currentConnectedBatteryFormat, mockBluetoothState)
	);

	// Format replacements reference
	const formatReplacements = [
		{ code: '{status}', desc: 'Bluetooth status' },
		{ code: '{num_connections}', desc: 'Number of connected devices' },
		{ code: '{controller_alias}', desc: 'Controller alias/name' },
		{ code: '{controller_address}', desc: 'Controller MAC address' },
		{ code: '{device_alias}', desc: 'Connected device alias/name' },
		{ code: '{device_address}', desc: 'Connected device MAC address' },
		{ code: '{device_battery_percentage}', desc: 'Device battery % (experimental)' },
		{ code: '{device_enumerate}', desc: 'List of connected devices (tooltip only)' }
	];
</script>

{#if tabIds.length > 0}
	<Tabs.Root value={defaultTab} class="w-full">
		<Tabs.List class="grid w-full" style="grid-template-columns: repeat({tabIds.length}, 1fr);">
			{#each tabIds as tabId (tabId)}
				{@const tab = fieldsByTab[tabId]}
				<Tabs.Trigger value={tabId} class="text-xs tracking-wide uppercase">
					{tab.label}
				</Tabs.Trigger>
			{/each}
		</Tabs.List>

		{#each tabIds as tabId (tabId)}
			{@const tab = fieldsByTab[tabId]}
			<Tabs.Content value={tabId}>
				<ScrollArea.Root class="h-[400px] pr-4">
					<div class="space-y-4">
						{#if tab.description}
							<p class="text-muted-foreground text-sm">
								{tab.description}
							</p>
						{/if}

						{#if tabId === 'general'}
							<!-- General Settings -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Controller Selection
								</h3>
								<p class="text-muted-foreground text-xs">
									Specify which Bluetooth controller to monitor and device display preferences.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'controller' || field.key === 'format-device-preference'}
										{@const isVisible =
											!field.visibleWhen ||
											fieldState[field.visibleWhen.field] === field.visibleWhen.value}
										{#if isVisible}
											<FieldRenderer
												{field}
												bind:value={fieldState[field.key]}
												fieldKey={field.key}
												{disabled}
											/>
										{/if}
									{/if}
								{/each}
							</div>

							<Separator class="my-4" />

							<!-- Display Settings -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Display Settings
								</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key !== 'controller' && field.key !== 'format-device-preference'}
										{@const isVisible =
											!field.visibleWhen ||
											fieldState[field.visibleWhen.field] === field.visibleWhen.value}
										{#if isVisible}
											<FieldRenderer
												{field}
												bind:value={fieldState[field.key]}
												fieldKey={field.key}
												{disabled}
											/>
										{/if}
									{/if}
								{/each}
							</div>
						{:else if tabId === 'formats'}
							<!-- Live Preview Card -->
							<Card.Root class="border-primary/20 bg-primary/5">
								<Card.Header class="pb-3">
									<Card.Title class="text-accent-foreground text-sm uppercase">
										Live Preview
									</Card.Title>
								</Card.Header>
								<Card.Content class="space-y-2">
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Disabled:</span>
										<span class="font-mono text-sm font-semibold">
											{disabledPreview || '(hidden)'}
										</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Off:</span>
										<span class="font-mono text-sm font-semibold">{offPreview || '(hidden)'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">On:</span>
										<span class="font-mono text-sm font-semibold">{onPreview || '(hidden)'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Connected:</span>
										<span class="font-mono text-sm font-semibold">
											{connectedPreview || 'N/A'}
										</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">With Battery:</span>
										<span class="font-mono text-sm font-semibold">
											{connectedBatteryPreview || 'N/A'}
										</span>
									</div>
								</Card.Content>
							</Card.Root>

							<Separator class="my-4" />

							<!-- Format Replacements Reference -->
							<Card.Root class="border-muted/50">
								<Card.Header class="pb-3">
									<Card.Title class="text-accent-foreground text-xs uppercase">
										Format Codes Reference
									</Card.Title>
								</Card.Header>
								<Card.Content>
									<div class="grid grid-cols-1 gap-1 text-xs">
										{#each formatReplacements as replacement}
											<div class="flex items-start gap-2">
												<Badge variant="outline" class="font-mono text-[0.65rem]">
													{replacement.code}
												</Badge>
												<span class="text-muted-foreground text-[0.65rem]">
													{replacement.desc}
												</span>
											</div>
										{/each}
									</div>
								</Card.Content>
							</Card.Root>

							<Separator class="my-4" />

							<!-- Default Format -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Default Format
								</h3>
								<p class="text-muted-foreground text-xs">
									Fallback format used when state-specific formats are not set.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'format' || field.key === 'format-custom'}
										{@const isVisible =
											!field.visibleWhen ||
											fieldState[field.visibleWhen.field] === field.visibleWhen.value}
										{#if isVisible}
											<FieldRenderer
												{field}
												bind:value={fieldState[field.key]}
												fieldKey={field.key}
												{disabled}
											/>
										{/if}
									{/if}
								{/each}
							</div>

							<Separator class="my-4" />

							<!-- State-Specific Formats -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									State-Specific Formats
								</h3>
								<p class="text-muted-foreground text-xs">
									Override the default format for specific Bluetooth states. Empty formats hide the
									module.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('format-') && field.key !== 'format' && field.key !== 'format-custom' && field.key !== 'format-device-preference'}
										{@const isVisible =
											!field.visibleWhen ||
											fieldState[field.visibleWhen.field] === field.visibleWhen.value}
										{#if isVisible}
											<FieldRenderer
												{field}
												bind:value={fieldState[field.key]}
												fieldKey={field.key}
												{disabled}
											/>
										{/if}
									{/if}
								{/each}
							</div>
						{:else if tabId === 'tooltip'}
							<!-- Tooltip Settings -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Tooltip Display
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure tooltip visibility and default format.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'tooltip' || field.key === 'tooltip-format' || field.key === 'tooltip-format-custom'}
										{@const isVisible =
											!field.visibleWhen ||
											fieldState[field.visibleWhen.field] === field.visibleWhen.value}
										{#if isVisible}
											<FieldRenderer
												{field}
												bind:value={fieldState[field.key]}
												fieldKey={field.key}
												{disabled}
											/>
										{/if}
									{/if}
								{/each}
							</div>

							<Separator class="my-4" />

							<!-- State-Specific Tooltips -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									State-Specific Tooltips
								</h3>
								<p class="text-muted-foreground text-xs">
									Override the default tooltip for specific Bluetooth states.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('tooltip-format-') && field.key !== 'tooltip-format' && field.key !== 'tooltip-format-custom' && !field.key.includes('enumerate')}
										{@const isVisible =
											!field.visibleWhen ||
											fieldState[field.visibleWhen.field] === field.visibleWhen.value}
										{#if isVisible}
											<FieldRenderer
												{field}
												bind:value={fieldState[field.key]}
												fieldKey={field.key}
												{disabled}
											/>
										{/if}
									{/if}
								{/each}
							</div>

							<Separator class="my-4" />

							<!-- Device List Formatting -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Device List Formatting
								</h3>
								<p class="text-muted-foreground text-xs">
									Format how each device appears in the {'{device_enumerate}'} list within tooltips.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.includes('enumerate')}
										{@const isVisible =
											!field.visibleWhen ||
											fieldState[field.visibleWhen.field] === field.visibleWhen.value}
										{#if isVisible}
											<FieldRenderer
												{field}
												bind:value={fieldState[field.key]}
												fieldKey={field.key}
												{disabled}
											/>
										{/if}
									{/if}
								{/each}
							</div>
						{:else if tabId === 'actions'}
							<!-- Click Actions -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Click Actions
								</h3>
								<p class="text-muted-foreground text-xs">
									Commands to execute when clicking on the Bluetooth module. Common tools:
									blueberry, blueman-manager.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('on-click')}
										{@const isVisible =
											!field.visibleWhen ||
											fieldState[field.visibleWhen.field] === field.visibleWhen.value}
										{#if isVisible}
											<FieldRenderer
												{field}
												bind:value={fieldState[field.key]}
												fieldKey={field.key}
												{disabled}
											/>
										{/if}
									{/if}
								{/each}
							</div>

							<Separator class="my-4" />

							<!-- Scroll Actions -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Scroll Actions
								</h3>
								<p class="text-muted-foreground text-xs">
									Commands to execute when scrolling on the Bluetooth module.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('on-scroll') || field.key === 'smooth-scrolling-threshold'}
										{@const isVisible =
											!field.visibleWhen ||
											fieldState[field.visibleWhen.field] === field.visibleWhen.value}
										{#if isVisible}
											<FieldRenderer
												{field}
												bind:value={fieldState[field.key]}
												fieldKey={field.key}
												{disabled}
											/>
										{/if}
									{/if}
								{/each}
							</div>
						{:else}
							<!-- Default rendering for other tabs -->
							{#each tab.fields as field (field.key)}
								{@const isVisible =
									!field.visibleWhen ||
									fieldState[field.visibleWhen.field] === field.visibleWhen.value}
								{#if isVisible}
									<FieldRenderer
										{field}
										bind:value={fieldState[field.key]}
										fieldKey={field.key}
										{disabled}
									/>
								{/if}
							{/each}
						{/if}
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
