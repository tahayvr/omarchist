<script>
	import { createEventDispatcher } from 'svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as ScrollArea from '$lib/components/ui/scroll-area/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
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

	// Live preview state
	let currentTime = $state(new Date());
	let previewInterval = null;

	// Update time every second for live preview
	$effect(() => {
		if (previewInterval) {
			clearInterval(previewInterval);
		}
		previewInterval = setInterval(() => {
			currentTime = new Date();
		}, 1000);

		return () => {
			if (previewInterval) {
				clearInterval(previewInterval);
			}
		};
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
	function formatTimePreview(formatString) {
		if (!formatString || formatString.startsWith('__')) {
			return '';
		}

		try {
			// Simple format string parser for common patterns
			const cleaned = formatString.replace(/[{}]/g, '');

			// Handle locale prefix
			const hasLocale = cleaned.startsWith(':L');
			const pattern = hasLocale ? cleaned.substring(2) : cleaned.substring(1);

			// Common format codes
			const formats = {
				'%H': currentTime.getHours().toString().padStart(2, '0'),
				'%I': (currentTime.getHours() % 12 || 12).toString().padStart(2, '0'),
				'%M': currentTime.getMinutes().toString().padStart(2, '0'),
				'%S': currentTime.getSeconds().toString().padStart(2, '0'),
				'%p': currentTime.getHours() >= 12 ? 'PM' : 'AM',
				'%R': `${currentTime.getHours().toString().padStart(2, '0')}:${currentTime.getMinutes().toString().padStart(2, '0')}`,
				'%T': `${currentTime.getHours().toString().padStart(2, '0')}:${currentTime.getMinutes().toString().padStart(2, '0')}:${currentTime.getSeconds().toString().padStart(2, '0')}`,
				'%a': ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'][currentTime.getDay()],
				'%A': ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][
					currentTime.getDay()
				],
				'%b': ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'][
					currentTime.getMonth()
				],
				'%B': [
					'January',
					'February',
					'March',
					'April',
					'May',
					'June',
					'July',
					'August',
					'September',
					'October',
					'November',
					'December'
				][currentTime.getMonth()],
				'%d': currentTime.getDate().toString().padStart(2, '0'),
				'%m': (currentTime.getMonth() + 1).toString().padStart(2, '0'),
				'%Y': currentTime.getFullYear().toString(),
				'%y': currentTime.getFullYear().toString().substring(2),
				'%x': currentTime.toLocaleDateString(),
				'%c': currentTime.toLocaleString()
			};

			let result = pattern;
			for (const [code, value] of Object.entries(formats)) {
				result = result.replace(new RegExp(code, 'g'), value);
			}

			return result;
		} catch {
			return 'Invalid format';
		}
	}

	// Get current format for preview
	const currentFormat = $derived.by(() => {
		const format = fieldState.format;
		if (format === '__custom') {
			return fieldState['format-custom'] || '';
		}
		return format || '{:%H:%M}';
	});

	const currentFormatAlt = $derived.by(() => {
		const format = fieldState['format-alt'];
		if (format === '__custom') {
			return fieldState['format-alt-custom'] || '';
		}
		return format || '';
	});

	const formatPreview = $derived(formatTimePreview(currentFormat));
	const formatAltPreview = $derived(formatTimePreview(currentFormatAlt));
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
							<!-- Live Preview Card -->
							<Card.Root class="border-primary/20 bg-primary/5">
								<Card.Header class="pb-3">
									<Card.Title class="text-accent-foreground text-sm uppercase">
										Live Preview
									</Card.Title>
								</Card.Header>
								<Card.Content class="space-y-2">
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Primary Format:</span>
										<span class="font-mono text-sm font-semibold">{formatPreview}</span>
									</div>
									{#if formatAltPreview}
										<div class="flex items-center justify-between">
											<span class="text-muted-foreground text-xs uppercase">Alternate Format:</span>
											<span class="font-mono text-sm font-semibold">{formatAltPreview}</span>
										</div>
									{/if}
								</Card.Content>
							</Card.Root>

							<Separator class="my-4" />

							<!-- Time Format Section -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">Time Format</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'format' || field.key === 'format-custom' || field.key === 'format-alt' || field.key === 'format-alt-custom'}
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

							<!-- Timezone Section -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">Timezone</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key.includes('timezone') || field.key === 'locale' || field.key === 'locale-custom'}
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

							<!-- Display Options Section -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Display Options
								</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'interval' || field.key === 'max-length' || field.key === 'rotate' || field.key === 'tooltip' || field.key === 'tooltip-format' || field.key === 'tooltip-format-custom' || field.key === 'smooth-scrolling-threshold'}
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
							<!-- Module Actions Section -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Click Actions
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure what happens when you interact with the clock module in the bar.
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

							<!-- Scroll Actions Section -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Scroll Actions
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure what happens when you scroll on the clock module.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('on-scroll')}
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
						{:else if tabId === 'calendar'}
							<!-- Calendar Display Section -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Calendar Display
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure how the calendar appears in the tooltip popup.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('calendar.') && !field.key.includes('format') && !field.key.startsWith('actions.')}
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

							<!-- Calendar Styling Section -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Calendar Styling
								</h3>
								<p class="text-muted-foreground text-xs">
									Use Pango markup to style calendar elements. Common tags: &lt;b&gt; (bold),
									&lt;u&gt; (underline), &lt;span color="#hex"&gt; (color).
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.includes('calendar.format')}
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

							<!-- Calendar Popup Actions Section -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Calendar Popup Actions
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure interactions when the calendar tooltip is visible. These only work
									inside the popup.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('actions.')}
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
