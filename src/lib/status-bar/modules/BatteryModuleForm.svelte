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

	// Mock battery state for preview
	let mockBatteryState = $state({
		capacity: 75,
		power: 15.5,
		time: '2 h 30 min',
		timeTo: '2 h 30 min remaining',
		cycles: 245,
		health: 92
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
	function formatBatteryPreview(formatString, state) {
		if (!formatString || formatString.startsWith('__')) {
			return '';
		}

		try {
			let result = formatString;

			// Get icon based on capacity
			const icon = getBatteryIcon(state.capacity);

			// Replace format placeholders
			const replacements = {
				'{capacity}': state.capacity.toString(),
				'{power}': state.power.toString(),
				'{time}': state.time,
				'{timeTo}': state.timeTo,
				'{cycles}': state.cycles.toString(),
				'{health}': state.health.toString(),
				'{icon}': icon
			};

			for (const [placeholder, value] of Object.entries(replacements)) {
				result = result.replace(new RegExp(placeholder.replace(/[{}]/g, '\\$&'), 'g'), value);
			}

			return result;
		} catch {
			return 'Invalid format';
		}
	}

	function getBatteryIcon(capacity) {
		const icons = getIconsArray();
		if (icons.length === 0) {
			// Default icons
			if (capacity >= 90) return '';
			if (capacity >= 70) return '';
			if (capacity >= 50) return '';
			if (capacity >= 30) return '';
			if (capacity >= 10) return '';
			return '';
		}

		// Map capacity (0-100) to icon index
		const index = Math.min(Math.floor((capacity / 100) * icons.length), icons.length - 1);
		return icons[index] || '';
	}

	function getIconsArray() {
		const iconsField = fieldState['format-icons'];
		if (!iconsField) {
			return [];
		}

		if (typeof iconsField === 'string') {
			return iconsField
				.split('\n')
				.map((line) => line.trim())
				.filter((line) => line.length > 0);
		}

		if (Array.isArray(iconsField)) {
			return iconsField;
		}

		return [];
	}

	// Get current formats for preview
	const currentFormat = $derived.by(() => {
		const format = fieldState.format;
		if (format === '__custom') {
			return fieldState['format-custom'] || '{capacity}%';
		}
		if (format === '__default') {
			return '{capacity}%';
		}
		return format || '{capacity}%';
	});

	const currentChargingFormat = $derived.by(() => {
		const format = fieldState['format-charging'];
		if (format === '__custom') {
			return fieldState['format-charging-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentDischargingFormat = $derived.by(() => {
		const format = fieldState['format-discharging'];
		if (format === '__custom') {
			return fieldState['format-discharging-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentFullFormat = $derived.by(() => {
		const format = fieldState['format-full'];
		if (format === '__custom') {
			return fieldState['format-full-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentWarningFormat = $derived.by(() => {
		const format = fieldState['format-warning'];
		if (format === '__custom') {
			return fieldState['format-warning-custom'] || '';
		}
		if (format === '__default') {
			return currentDischargingFormat || currentFormat;
		}
		return format || '';
	});

	const currentCriticalFormat = $derived.by(() => {
		const format = fieldState['format-critical'];
		if (format === '__custom') {
			return fieldState['format-critical-custom'] || '';
		}
		if (format === '__default') {
			return currentDischargingFormat || currentFormat;
		}
		return format || '';
	});

	const chargingPreview = $derived(formatBatteryPreview(currentChargingFormat, mockBatteryState));
	const dischargingPreview = $derived(
		formatBatteryPreview(currentDischargingFormat, mockBatteryState)
	);
	const fullPreview = $derived(
		formatBatteryPreview(currentFullFormat, { ...mockBatteryState, capacity: 100 })
	);
	const warningPreview = $derived(
		formatBatteryPreview(currentWarningFormat, { ...mockBatteryState, capacity: 25 })
	);
	const criticalPreview = $derived(
		formatBatteryPreview(currentCriticalFormat, { ...mockBatteryState, capacity: 10 })
	);

	// Format replacements reference
	const formatReplacements = [
		{ code: '{capacity}', desc: 'Battery capacity percentage (0-100)' },
		{ code: '{power}', desc: 'Power draw in watts' },
		{ code: '{time}', desc: 'Time estimate (formatted)' },
		{ code: '{timeTo}', desc: 'Time to full/empty or status text' },
		{ code: '{icon}', desc: 'Battery icon based on capacity' },
		{ code: '{cycles}', desc: 'Charge cycles (Linux only)' },
		{ code: '{health}', desc: 'Battery health percentage (Linux only)' }
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
							<!-- Battery Selection -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Battery Selection
								</h3>
								<p class="text-muted-foreground text-xs">
									Specify which battery and adapter to monitor. Leave empty for auto-detection.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'bat' || field.key === 'adapter'}
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

							<!-- Polling & Capacity -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Polling & Capacity
								</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'interval' || field.key === 'design-capacity' || field.key === 'full-at' || field.key === 'weighted-average'}
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
									{#if field.key === 'bat-compatibility' || field.key === 'max-length' || field.key === 'rotate'}
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
										<span class="text-muted-foreground text-xs uppercase">Charging:</span>
										<span class="font-mono text-sm font-semibold">{chargingPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Discharging:</span>
										<span class="font-mono text-sm font-semibold">
											{dischargingPreview || 'N/A'}
										</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Full:</span>
										<span class="font-mono text-sm font-semibold">{fullPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Warning (25%):</span>
										<span class="font-mono text-sm font-semibold">{warningPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Critical (10%):</span>
										<span class="font-mono text-sm font-semibold">{criticalPreview || 'N/A'}</span>
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
									Fallback format when status/state-specific formats are not set.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'format' || field.key === 'format-custom' || field.key === 'format-time' || field.key === 'format-time-custom' || field.key === 'format-icons'}
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

							<!-- Status-Based Formats -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Status-Based Formats
								</h3>
								<p class="text-muted-foreground text-xs">
									Override the default format for specific battery statuses (charging, discharging,
									full, etc.).
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('format-') && !field.key.includes('time') && !field.key.includes('icons') && field.key !== 'format' && field.key !== 'format-custom' && !field.key.includes('warning') && !field.key.includes('critical')}
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
						{:else if tabId === 'states'}
							<!-- State Thresholds -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									State Thresholds
								</h3>
								<p class="text-muted-foreground text-xs">
									Define battery percentage thresholds for warning and critical states. These
									activate CSS classes and can have custom formats.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('states.')}
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

							<!-- State-Based Formats -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									State-Based Formats
								</h3>
								<p class="text-muted-foreground text-xs">
									Override formats when battery enters warning or critical states. These take
									precedence over status-based formats.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.includes('warning') || field.key.includes('critical')}
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

							<!-- Status-Specific Tooltips -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Status-Specific Tooltips
								</h3>
								<p class="text-muted-foreground text-xs">
									Override the default tooltip for specific battery statuses.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('tooltip-format-') && field.key !== 'tooltip-format' && field.key !== 'tooltip-format-custom'}
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
									Commands to execute when clicking on the battery module. Common tools:
									gnome-power-statistics, xfce4-power-manager-settings.
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
									Commands to execute when scrolling on the battery module.
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
