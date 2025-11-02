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

	// Mock audio state for preview
	let mockAudioState = $state({
		volume: 65,
		desc: 'Built-in Audio Analog Stereo',
		isMuted: false,
		isSourceMuted: false,
		isBluetooth: false
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
	function formatAudioPreview(formatString, state) {
		if (!formatString || formatString.startsWith('__')) {
			return '';
		}

		try {
			let result = formatString;

			// Get icon based on volume
			const icon = getVolumeIcon(state.volume);

			// Get source format
			const sourceFormat = getSourceFormat();

			// Replace format placeholders
			const replacements = {
				'{volume}': state.volume.toString(),
				'{icon}': icon,
				'{desc}': state.desc,
				'{format_source}': sourceFormat
			};

			for (const [placeholder, value] of Object.entries(replacements)) {
				result = result.replace(new RegExp(placeholder.replace(/[{}]/g, '\\$&'), 'g'), value);
			}

			return result;
		} catch {
			return 'Invalid format';
		}
	}

	function getVolumeIcon(volume) {
		const icons = getIconsArray();
		if (icons.length === 0) {
			// Default icons based on volume
			if (volume === 0) return '';
			if (volume < 33) return '';
			if (volume < 66) return '';
			return '';
		}

		// Map volume (0-100) to icon index
		const index = Math.min(Math.floor((volume / 100) * icons.length), icons.length - 1);
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

	function getSourceFormat() {
		const format = fieldState['format-source'];
		if (format === '__custom') {
			return fieldState['format-source-custom'] || '';
		}
		if (format === '__default') {
			return '{volume}%';
		}
		return format || '';
	}

	// Get current formats for preview
	const currentFormat = $derived.by(() => {
		const format = fieldState.format;
		if (format === '__custom') {
			return fieldState['format-custom'] || '{volume}%';
		}
		if (format === '__default') {
			return '{volume}%';
		}
		return format || '{volume}%';
	});

	const currentMutedFormat = $derived.by(() => {
		const format = fieldState['format-muted'];
		if (format === '__custom') {
			return fieldState['format-muted-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentBluetoothFormat = $derived.by(() => {
		const format = fieldState['format-bluetooth'];
		if (format === '__custom') {
			return fieldState['format-bluetooth-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const normalPreview = $derived(formatAudioPreview(currentFormat, mockAudioState));
	const mutedPreview = $derived(
		formatAudioPreview(currentMutedFormat, { ...mockAudioState, isMuted: true })
	);
	const bluetoothPreview = $derived(
		formatAudioPreview(currentBluetoothFormat, {
			...mockAudioState,
			isBluetooth: true,
			desc: 'AirPods Pro'
		})
	);

	// Format replacements reference
	const formatReplacements = [
		{ code: '{volume}', desc: 'Volume percentage (0-100+)' },
		{ code: '{icon}', desc: 'Volume icon (from format-icons)' },
		{ code: '{desc}', desc: 'Device description/name' },
		{ code: '{format_source}', desc: 'Source format (microphone)' }
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
							<!-- Volume Control -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Volume Control
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure scroll behavior and maximum volume limits.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'scroll-step' || field.key === 'max-volume' || field.key === 'reverse-scrolling' || field.key === 'reverse-mouse-scrolling'}
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

							<!-- Device Settings -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Device Settings
								</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'ignored-sinks'}
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
									{#if field.key === 'max-length' || field.key === 'rotate' || field.key === 'tooltip'}
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
										<span class="text-muted-foreground text-xs uppercase">Normal (65%):</span>
										<span class="font-mono text-sm font-semibold">{normalPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Muted:</span>
										<span class="font-mono text-sm font-semibold">{mutedPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Bluetooth:</span>
										<span class="font-mono text-sm font-semibold">{bluetoothPreview || 'N/A'}</span>
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

							<!-- Output Formats -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Output Formats
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure how volume information is displayed for different output states.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'format' || field.key === 'format-custom' || field.key === 'format-bluetooth' || field.key === 'format-bluetooth-custom' || field.key === 'format-muted' || field.key === 'format-muted-custom' || field.key === 'format-icons'}
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

							<!-- Input Source Formats -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Input Source Formats
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure how microphone/input source information is displayed.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('format-source')}
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
									Volume Thresholds
								</h3>
								<p class="text-muted-foreground text-xs">
									Define volume thresholds for warning and critical states (useful for hearing
									protection).
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
									Override the default format when volume enters warning or critical states.
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
									Tooltip Configuration
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure tooltip format to show device information on hover.
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
									Commands to execute when clicking on the audio module. Common tools: pavucontrol,
									pactl, pamixer.
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
									⚠️ Setting scroll commands replaces the default volume control behavior. Leave
									empty to use built-in volume adjustment.
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
