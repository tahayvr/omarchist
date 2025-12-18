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

	const moduleDefinition = getModuleDefinition(module?.id);
	const schema = moduleDefinition?.schema;

	let fieldState = $state({});
	let lastConfigSignature = '';
	let lastEmittedSignature = '';
	let wasInitialized = false;

	let mockCpuState = $state({
		usage: 45,
		load: 1.25,
		avg_frequency: 2.8,
		max_frequency: 3.5,
		min_frequency: 2.1,
		cores: [35, 52, 41, 38, 48, 44, 39, 50]
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
	function formatCpuPreview(formatString, state) {
		if (!formatString || formatString.startsWith('__')) {
			return '';
		}

		try {
			let result = formatString;

			// Get icon based on usage
			const icon = getCpuIcon(state.usage);

			// Build replacements object
			const replacements = {
				'{usage}': state.usage.toString(),
				'{load}': state.load.toFixed(2),
				'{avg_frequency}': state.avg_frequency.toFixed(1),
				'{max_frequency}': state.max_frequency.toFixed(1),
				'{min_frequency}': state.min_frequency.toFixed(1),
				'{icon}': icon
			};

			// Add per-core replacements
			state.cores.forEach((coreUsage, index) => {
				replacements[`{usage${index}}`] = coreUsage.toString();
				replacements[`{icon${index}}`] = getCpuIcon(coreUsage);
			});

			// Replace all placeholders
			for (const [placeholder, value] of Object.entries(replacements)) {
				result = result.replace(new RegExp(placeholder.replace(/[{}]/g, '\\$&'), 'g'), value);
			}

			return result;
		} catch {
			return 'Invalid format';
		}
	}

	function getCpuIcon(usage) {
		const icons = getIconsArray();
		if (icons.length === 0) {
			// Default icon
			return '󰍛';
		}

		// Map usage (0-100) to icon index
		const index = Math.min(Math.floor((usage / 100) * icons.length), icons.length - 1);
		return icons[index] || '󰍛';
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
			return fieldState['format-custom'] || '{usage}%';
		}
		if (format === '__default') {
			return '{usage}%';
		}
		return format || '{usage}%';
	});

	const currentWarningFormat = $derived.by(() => {
		const format = fieldState['format-warning'];
		if (format === '__custom') {
			return fieldState['format-warning-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentCriticalFormat = $derived.by(() => {
		const format = fieldState['format-critical'];
		if (format === '__custom') {
			return fieldState['format-critical-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const normalPreview = $derived(formatCpuPreview(currentFormat, mockCpuState));
	const warningPreview = $derived(
		formatCpuPreview(currentWarningFormat, { ...mockCpuState, usage: 75 })
	);
	const criticalPreview = $derived(
		formatCpuPreview(currentCriticalFormat, { ...mockCpuState, usage: 95 })
	);

	// Format replacements reference
	const formatReplacements = [
		{ code: '{usage}', desc: 'Overall CPU usage percentage (0-100)' },
		{ code: '{load}', desc: 'CPU load average' },
		{ code: '{icon}', desc: 'Usage icon (from format-icons)' },
		{ code: '{usageN}', desc: 'Nth core usage (e.g., {usage0}, {usage1})' },
		{ code: '{iconN}', desc: 'Nth core icon (e.g., {icon0}, {icon1})' },
		{ code: '{avg_frequency}', desc: 'Average frequency across all cores (GHz)' },
		{ code: '{max_frequency}', desc: 'Highest core frequency (GHz)' },
		{ code: '{min_frequency}', desc: 'Lowest core frequency (GHz)' }
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
									Configure how often CPU usage is polled and displayed.
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
										<span class="text-muted-foreground text-xs uppercase">Normal (45%):</span>
										<span class="text-sm font-semibold">{normalPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Warning (75%):</span>
										<span class="text-sm font-semibold">{warningPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Critical (95%):</span>
										<span class="text-sm font-semibold">{criticalPreview || 'N/A'}</span>
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
												<Badge variant="outline" class="text-[0.65rem]">
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
									Configure how CPU usage is displayed. Use {'{icon}'} for usage-based icons or
									{'{iconN}'} for per-core icons.
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
									Define CPU usage thresholds for warning and critical states. These activate CSS
									classes and can have custom formats.
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
									Override the default format when CPU usage enters warning or critical states.
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
						{:else if tabId === 'actions'}
							<!-- Click Actions -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Click Actions
								</h3>
								<p class="text-muted-foreground text-xs">
									Commands to execute when clicking on the CPU module. Common tools:
									gnome-system-monitor, htop, btop, top.
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
									Commands to execute when scrolling on the CPU module.
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
