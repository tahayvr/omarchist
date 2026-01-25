<script>
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as ScrollArea from '$lib/components/ui/scroll-area/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import FieldRenderer from './FieldRenderer.svelte';
	import { hydrateFieldState, buildConfigFromFieldState } from '$lib/utils/waybar/schemaUtils.js';
	import { getModuleDefinition } from '$lib/utils/waybar/moduleRegistry.js';

	let { module = null, config = {}, disabled = false, onConfigChange = () => {} } = $props();

	const moduleDefinition = $derived.by(() => getModuleDefinition(module?.id));
	const schema = $derived.by(() => moduleDefinition?.schema);

	let fieldState = $state({});
	let lastConfigSignature = '';
	let lastEmittedSignature = '';
	let wasInitialized = false;

	let mockMemoryState = $state({
		percentage: 65,
		swapPercentage: 25,
		total: 16.0,
		swapTotal: 8.0,
		used: 10.4,
		swapUsed: 2.0,
		avail: 5.6,
		swapAvail: 6.0
	});

	const fieldsByTab = $derived.by(() => {
		if (!schema || !schema.properties) {
			return {};
		}

		const groups = {};

		if (schema.tabs) {
			for (const tab of schema.tabs) {
				groups[tab.id] = {
					label: tab.label,
					description: tab.description,
					fields: []
				};
			}
		}

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

		const initialConfig = buildConfigFromFieldState(fieldState, schema);
		lastEmittedSignature = JSON.stringify(initialConfig);

		wasInitialized = true;
	}

	hydrateFromConfig(true);

	$effect(() => {
		const signature = computeConfigSignature();
		if (signature !== lastConfigSignature) {
			hydrateFromConfig(false);
		}
	});

	let previousFieldValues = $state({});

	$effect(() => {
		if (!wasInitialized || !schema || !schema.properties) {
			return;
		}

		for (const [key, field] of Object.entries(schema.properties)) {
			if (field.type === 'select' || field.enum) {
				const currentValue = fieldState[key];
				const previousValue = previousFieldValues[key];

				if (currentValue === '__custom' && previousValue !== '__custom') {
					const customKey = `${key}-custom`;
					const customField = schema.properties[customKey];

					if (
						customField &&
						previousValue &&
						!previousValue.startsWith('__') &&
						previousValue !== '__custom'
					) {
						fieldState[customKey] = previousValue;
					}
				}

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
		onConfigChange(newConfig);
	});
	// Format preview helpers
	function formatMemoryPreview(formatString, state) {
		if (!formatString || formatString.startsWith('__')) {
			return '';
		}

		try {
			let result = formatString;

			// Get icon based on usage
			const icon = getMemoryIcon(state.percentage);

			// Handle formatted numbers (e.g., {used:0.1f})
			result = result.replace(/\{(\w+):([0-9.]+)f\}/g, (match, key, precision) => {
				const value = state[key];
				if (value !== undefined && value !== null) {
					return Number(value).toFixed(parseInt(precision) || 1);
				}
				return match;
			});

			// Replace simple placeholders
			const replacements = {
				'{percentage}': state.percentage.toString(),
				'{swapPercentage}': state.swapPercentage.toString(),
				'{total}': state.total.toFixed(1),
				'{swapTotal}': state.swapTotal.toFixed(1),
				'{used}': state.used.toFixed(1),
				'{swapUsed}': state.swapUsed.toFixed(1),
				'{avail}': state.avail.toFixed(1),
				'{swapAvail}': state.swapAvail.toFixed(1),
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

	function getMemoryIcon(percentage) {
		const icons = getIconsArray();
		if (icons.length === 0) {
			// Default icon when no custom icons are set
			return '';
		}

		// Map percentage (0-100) to icon index
		const index = Math.min(Math.floor((percentage / 100) * icons.length), icons.length - 1);
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
			return fieldState['format-custom'] || '{icon}';
		}
		if (format === '__default' || !format) {
			// Return the actual default format from config (icon only)
			return '{icon}';
		}
		return format;
	});

	const normalPreview = $derived(formatMemoryPreview(currentFormat, mockMemoryState));
	const warningPreview = $derived(
		formatMemoryPreview(currentFormat, { ...mockMemoryState, percentage: 80, used: 12.8 })
	);
	const criticalPreview = $derived(
		formatMemoryPreview(currentFormat, { ...mockMemoryState, percentage: 90, used: 14.4 })
	);

	// Format replacements reference
	const formatReplacements = [
		{ code: '{percentage}', desc: 'RAM usage percentage (0-100)' },
		{ code: '{swapPercentage}', desc: 'Swap usage percentage (0-100)' },
		{ code: '{total}', desc: 'Total RAM in GiB' },
		{ code: '{swapTotal}', desc: 'Total swap in GiB' },
		{ code: '{used}', desc: 'Used RAM in GiB' },
		{ code: '{swapUsed}', desc: 'Used swap in GiB' },
		{ code: '{avail}', desc: 'Available RAM in GiB' },
		{ code: '{swapAvail}', desc: 'Available swap in GiB' },
		{ code: '{icon}', desc: 'Memory icon (from format-icons or  default)' },
		{ code: '{used:0.1f}', desc: 'Formatted number (e.g., 10.4)' }
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
									Update Settings
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure how often memory usage is polled and displayed.
								</p>
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
										<span class="text-muted-foreground text-xs uppercase">Normal (65%):</span>
										<span class="font-nerd text-sm font-semibold">{normalPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Warning (80%):</span>
										<span class="font-nerd text-sm font-semibold">{warningPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Critical (90%):</span>
										<span class="font-nerd text-sm font-semibold">{criticalPreview || 'N/A'}</span>
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
										{#each formatReplacements as replacement (replacement.code)}
											<div class="flex items-start gap-2">
												<Badge variant="outline" class="font-nerd text-[0.65rem]">
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

							<!-- Format Configuration -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Display Format
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure how memory usage is displayed. Use {'{icon}'} for usage-based icons or
									{'{used:0.1f}'} for formatted numbers.
								</p>
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
							</div>
						{:else if tabId === 'states'}
							<!-- State Thresholds -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									State Thresholds
								</h3>
								<p class="text-muted-foreground text-xs">
									Define memory usage thresholds for warning and critical states. These activate CSS
									classes for styling.
								</p>
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
							</div>
						{:else if tabId === 'tooltip'}
							<!-- Tooltip Settings -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Tooltip Configuration
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure tooltip format to show detailed memory information on hover.
								</p>
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
							</div>
						{:else if tabId === 'actions'}
							<!-- Click Actions -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Click Actions
								</h3>
								<p class="text-muted-foreground text-xs">
									Commands to execute when clicking on the memory module. Common tools:
									gnome-system-monitor, htop, btop.
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
									Commands to execute when scrolling on the memory module.
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
