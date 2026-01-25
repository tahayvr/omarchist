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

	let mockNetworkState = $state({
		type: 'wifi', // wifi, ethernet, linked, disconnected, disabled
		ifname: 'wlan0',
		essid: 'MyNetwork',
		signalStrength: 75,
		signaldBm: -55,
		frequency: 5.2,
		ipaddr: '192.168.1.100',
		cidr: 24,
		gwaddr: '192.168.1.1',
		bandwidthUpBytes: '125 KB/s',
		bandwidthDownBytes: '1.2 MB/s',
		bandwidthUpBits: '1 Mbps',
		bandwidthDownBits: '9.6 Mbps'
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
	function formatNetworkPreview(formatString, state) {
		if (!formatString || formatString.startsWith('__')) {
			return '';
		}

		try {
			let result = formatString;

			// Replace format placeholders
			const replacements = {
				'{ifname}': state.ifname,
				'{essid}': state.essid,
				'{signalStrength}': state.signalStrength.toString(),
				'{signaldBm}': state.signaldBm.toString(),
				'{frequency}': state.frequency.toString(),
				'{ipaddr}': state.ipaddr,
				'{cidr}': state.cidr.toString(),
				'{gwaddr}': state.gwaddr,
				'{netmask}': '255.255.255.0',
				'{bandwidthUpBytes}': state.bandwidthUpBytes,
				'{bandwidthDownBytes}': state.bandwidthDownBytes,
				'{bandwidthUpBits}': state.bandwidthUpBits,
				'{bandwidthDownBits}': state.bandwidthDownBits,
				'{icon}': getSignalIcon(state.signalStrength)
			};

			for (const [placeholder, value] of Object.entries(replacements)) {
				result = result.replace(new RegExp(placeholder.replace(/[{}]/g, '\\$&'), 'g'), value);
			}

			return result;
		} catch {
			return 'Invalid format';
		}
	}

	function getSignalIcon(strength) {
		const icons = getIconsArray();
		if (icons.length === 0) {
			return '󰤨'; // Default strong signal icon
		}

		// Map strength (0-100) to icon index
		const index = Math.min(Math.floor((strength / 100) * icons.length), icons.length - 1);
		return icons[index] || '󰤨';
	}

	function getIconsArray() {
		const iconsField = fieldState['format-icons'];
		if (!iconsField) {
			return ['󰤯', '󰤟', '󰤢', '󰤥', '󰤨'];
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

		return ['󰤯', '󰤟', '󰤢', '󰤥', '󰤨'];
	}

	// Get current formats for preview
	const currentFormat = $derived.by(() => {
		const format = fieldState.format;
		if (format === '__custom') {
			return fieldState['format-custom'] || '{ifname}';
		}
		if (format === '__default') {
			return '{ifname}';
		}
		return format || '{ifname}';
	});

	const currentWifiFormat = $derived.by(() => {
		const format = fieldState['format-wifi'];
		if (format === '__custom') {
			return fieldState['format-wifi-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentEthernetFormat = $derived.by(() => {
		const format = fieldState['format-ethernet'];
		if (format === '__custom') {
			return fieldState['format-ethernet-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const currentDisconnectedFormat = $derived.by(() => {
		const format = fieldState['format-disconnected'];
		if (format === '__custom') {
			return fieldState['format-disconnected-custom'] || '';
		}
		if (format === '__default') {
			return currentFormat;
		}
		return format || '';
	});

	const wifiPreview = $derived(formatNetworkPreview(currentWifiFormat, mockNetworkState));
	const ethernetPreview = $derived(
		formatNetworkPreview(currentEthernetFormat, {
			...mockNetworkState,
			ifname: 'eth0'
		})
	);
	const disconnectedPreview = $derived(
		formatNetworkPreview(currentDisconnectedFormat, mockNetworkState)
	);

	// Format replacements reference
	const formatReplacements = [
		{ code: '{ifname}', desc: 'Interface name (e.g., wlan0, eth0)' },
		{ code: '{essid}', desc: 'WiFi network name (SSID)' },
		{ code: '{signalStrength}', desc: 'WiFi signal strength (0-100%)' },
		{ code: '{signaldBm}', desc: 'WiFi signal strength in dBm' },
		{ code: '{frequency}', desc: 'WiFi frequency in GHz' },
		{ code: '{ipaddr}', desc: 'IP address' },
		{ code: '{cidr}', desc: 'CIDR notation (e.g., 24)' },
		{ code: '{gwaddr}', desc: 'Gateway address' },
		{ code: '{icon}', desc: 'Signal strength icon' },
		{ code: '{bandwidthUpBytes}', desc: 'Upload speed (bytes/s)' },
		{ code: '{bandwidthDownBytes}', desc: 'Download speed (bytes/s)' },
		{ code: '{bandwidthUpBits}', desc: 'Upload speed (bits/s)' },
		{ code: '{bandwidthDownBits}', desc: 'Download speed (bits/s)' }
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
									Interface Selection
								</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'interface' || field.key === 'family'}
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

							<!-- Update Settings -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Update & Display
								</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'interval' || field.key === 'max-length' || field.key === 'rotate'}
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
										<span class="text-muted-foreground text-xs uppercase">WiFi:</span>
										<span class="text-sm font-semibold">{wifiPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Ethernet:</span>
										<span class="text-sm font-semibold">{ethernetPreview || 'N/A'}</span>
									</div>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Disconnected:</span>
										<span class="text-sm font-semibold">
											{disconnectedPreview || '(hidden)'}
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
									<div class="grid grid-cols-1 gap-1 text-xs md:grid-cols-2">
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

							<!-- Default Format -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Default Format
								</h3>
								<p class="text-muted-foreground text-xs">
									Fallback format used when connection-specific formats are not set.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'format' || field.key === 'format-custom' || field.key === 'format-icons'}
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

							<!-- Connection-Specific Formats -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Connection-Specific Formats
								</h3>
								<p class="text-muted-foreground text-xs">
									Override the default format for specific connection states.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key.startsWith('format-') && field.key !== 'format' && field.key !== 'format-custom' && field.key !== 'format-icons' && field.key !== 'format-alt'}
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

							<!-- Alternate Format -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Alternate Format
								</h3>
								<p class="text-muted-foreground text-xs">
									Format toggled when clicking on the module.
								</p>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'format-alt'}
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

							<!-- Connection-Specific Tooltips -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Connection-Specific Tooltips
								</h3>
								<p class="text-muted-foreground text-xs">
									Override the default tooltip for specific connection states.
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
									Commands to execute when clicking on the network module.
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
									Commands to execute when scrolling on the network module.
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
