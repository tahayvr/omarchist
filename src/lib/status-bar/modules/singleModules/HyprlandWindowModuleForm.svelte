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

	let mockWindowState = $state({
		title: 'Omarchist App - Visual Studio Code',
		class: 'Code',
		initialClass: 'code',
		initialTitle: 'Visual Studio Code'
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
	function formatWindowPreview(formatString, state) {
		if (!formatString || formatString.startsWith('__')) {
			return '';
		}

		try {
			let result = formatString;

			// Apply rewrite rules first
			result = applyRewriteRules(result, state);

			// Replace format placeholders
			const replacements = {
				'{title}': state.title,
				'{class}': state.class,
				'{initialClass}': state.initialClass,
				'{initialTitle}': state.initialTitle
			};

			for (const [placeholder, value] of Object.entries(replacements)) {
				result = result.replace(new RegExp(placeholder.replace(/[{}]/g, '\\$&'), 'g'), value);
			}

			// Apply max-length truncation
			const maxLength = fieldState['max-length'];
			if (maxLength && result.length > maxLength) {
				result = result.substring(0, maxLength) + '...';
			}

			return result;
		} catch {
			return 'Invalid format';
		}
	}

	function applyRewriteRules(text, state) {
		const rulesField = fieldState['rewrite-rules'];
		if (!rulesField) {
			return text;
		}

		let rules = [];
		if (typeof rulesField === 'string') {
			// Parse textarea format: "pattern": "replacement"
			const lines = rulesField.split('\n').filter((line) => line.trim());
			for (const line of lines) {
				const match = line.match(/^"(.*)"\s*:\s*"(.*)"$/);
				if (match) {
					rules.push({ pattern: match[1], replacement: match[2] });
				}
			}
		} else if (Array.isArray(rulesField)) {
			rules = rulesField;
		}

		// Apply rules to title
		let result = state.title;
		for (const rule of rules) {
			try {
				const regex = new RegExp(rule.pattern);
				if (regex.test(result)) {
					result = result.replace(regex, rule.replacement);
					break; // Apply only first matching rule
				}
			} catch {
				// Invalid regex, skip
			}
		}

		return result;
	}

	// Get current format for preview
	const currentFormat = $derived.by(() => {
		const format = fieldState.format;
		if (format === '__custom') {
			return fieldState['format-custom'] || '{title}';
		}
		if (format === '__default') {
			return '{title}';
		}
		return format || '{title}';
	});

	const windowPreview = $derived(formatWindowPreview(currentFormat, mockWindowState));

	// Format replacements reference
	const formatReplacements = [
		{ code: '{title}', desc: 'Current window title' },
		{ code: '{class}', desc: 'Current window class' },
		{ code: '{initialClass}', desc: 'Initial window class (at creation)' },
		{ code: '{initialTitle}', desc: 'Initial window title (at creation)' }
	];

	// Example rewrite rules
	const exampleRules = [
		{ pattern: '(.*) — Mozilla Firefox', replacement: '🌎 $1', desc: 'Firefox tabs' },
		{ pattern: '(.*) - Visual Studio Code', replacement: '󰨞 $1', desc: 'VS Code files' },
		{ pattern: '(.*) - fish', replacement: '> [$1]', desc: 'Terminal' },
		{ pattern: '(.*) - YouTube', replacement: ' $1', desc: 'YouTube videos' }
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
							<!-- Live Preview Card -->
							<Card.Root class="border-primary/20 bg-primary/5">
								<Card.Header class="pb-3">
									<Card.Title class="text-accent-foreground text-sm uppercase">
										Live Preview
									</Card.Title>
								</Card.Header>
								<Card.Content>
									<div class="flex items-center justify-between">
										<span class="text-muted-foreground text-xs uppercase">Window:</span>
										<span class="text-sm font-semibold">{windowPreview || 'N/A'}</span>
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

							<!-- Display Format -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Display Format
								</h3>
								<p class="text-muted-foreground text-xs">
									Configure how window information is displayed in the bar.
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

							<!-- Display Settings -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Display Settings
								</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'max-length' || field.key === 'separate-outputs' || field.key === 'icon' || field.key === 'icon-size'}
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
						{:else if tabId === 'rewrite'}
							<!-- Rewrite Rules Examples -->
							<Card.Root class="border-muted/50">
								<Card.Header class="pb-3">
									<Card.Title class="text-accent-foreground text-xs uppercase">
										Example Rewrite Rules
									</Card.Title>
								</Card.Header>
								<Card.Content>
									<div class="space-y-2 text-xs">
										{#each exampleRules as example (example.desc)}
											<div class="space-y-1">
												<div class="flex items-start gap-2">
													<Badge variant="outline" class="text-[0.65rem]">
														{example.desc}
													</Badge>
												</div>
												<code class="text-muted-foreground block text-[0.65rem]">
													"{example.pattern}": "{example.replacement}"
												</code>
											</div>
										{/each}
									</div>
								</Card.Content>
							</Card.Root>

							<Separator class="my-4" />

							<!-- Rewrite Rules Configuration -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Rewrite Rules
								</h3>
								<p class="text-muted-foreground text-xs">
									Transform window titles using regex patterns. Enter one rule per line in the
									format:
									<code class="text-[0.65rem]">"pattern": "replacement"</code>. Use $1, $2, etc. for
									capture groups.
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
									Commands to execute when clicking on the window module.
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

							<Separator class="my-4" />

							<!-- Other Settings -->
							<div class="space-y-4">
								<h3 class="text-accent-foreground text-sm font-semibold uppercase">
									Other Settings
								</h3>
								{#each tab.fields as field (field.key)}
									{#if field.key === 'rotate' || field.key === 'tooltip'}
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
