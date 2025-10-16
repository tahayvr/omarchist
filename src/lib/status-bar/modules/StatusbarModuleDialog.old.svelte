<script>
	import { createEventDispatcher } from 'svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as ScrollArea from '$lib/components/ui/scroll-area/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Button } from '$lib/components/ui/button/index.js';

	let { module = null, config = {}, disabled = false, fields = [] } = $props();

	const dispatch = createEventDispatcher();
	let open = $state(false);
	let activeTab = $state('general');
	let fieldState = $state({});
	let lastConfigSignature = '';
	let lastEmittedSignature = '';
	let wasOpen = false;

	const legacyFieldDefinitions = $derived(
		fields.length
			? fields.map((field) => ({
					key: field.key,
					label: field.label,
					type: field.type,
					placeholder: field.placeholder,
					min: field.min,
					max: field.max,
					step: field.step
				}))
			: null
	);

	const hasClockDialog = module?.id === 'clock';
	const moduleTitle = module?.title ?? 'Waybar Module';
	const moduleDescription = module?.description ?? '';

	const CLOCK_NUMBER_FIELDS = [
		['interval', { allowFloat: false }],
		['max-length', { allowFloat: false }],
		['rotate', { allowFloat: false }],
		['smooth-scrolling-threshold', { allowFloat: true }],
		['calendar.mode-mon-col', { allowFloat: false }],
		['calendar.on-scroll', { allowFloat: false }]
	];

	const CLOCK_STRING_FIELDS = [
		'format',
		'format-alt',
		'timezone',
		'locale',
		'on-click',
		'on-click-middle',
		'on-click-right',
		'on-scroll-up',
		'on-scroll-down',
		'tooltip-format'
	];

	const CLOCK_CALENDAR_FORMAT_FIELDS = ['months', 'days', 'weeks', 'weekdays', 'today'];

	const actionOptions = [
		{ label: 'Default', value: '__default' },
		{ label: 'Switch Calendar Mode', value: 'mode' },
		{ label: 'Time Zone Next', value: 'tz_up' },
		{ label: 'Time Zone Previous', value: 'tz_down' },
		{ label: 'Calendar Forward', value: 'shift_up' },
		{ label: 'Calendar Back', value: 'shift_down' },
		{ label: 'Calendar Reset', value: 'shift_reset' }
	];

	const actionEventFields = [
		{ key: 'on-click-right', label: 'Right Click' },
		{ key: 'on-click-middle', label: 'Middle Click' },
		{ key: 'on-scroll-up', label: 'Scroll Up' },
		{ key: 'on-scroll-down', label: 'Scroll Down' },
		{ key: 'on-click-forward', label: 'Mouse Forward Button' },
		{ key: 'on-click-backward', label: 'Mouse Back Button' }
	];

	function getActionLabel(value) {
		const option = actionOptions.find((entry) => entry.value === value);
		return option ? option.label : 'Default';
	}

	function getNestedValue(target, path) {
		if (!target || typeof path !== 'string') {
			return undefined;
		}
		if (!path.includes('.')) {
			return target[path];
		}
		const segments = path.split('.');
		let cursor = target;
		for (const segment of segments) {
			if (!cursor || typeof cursor !== 'object') {
				return undefined;
			}
			cursor = cursor[segment];
		}
		return cursor;
	}

	function readString(path) {
		const value = getNestedValue(config, path);
		return typeof value === 'string' ? value : '';
	}

	function readNumber(path) {
		const value = getNestedValue(config, path);
		const parsed = Number(value);
		return Number.isFinite(parsed) ? parsed : '';
	}

	function numberToInput(value) {
		if (value === '' || value === null || value === undefined) {
			return '';
		}
		return String(value);
	}

	function readBoolean(path, fallback = false) {
		const value = getNestedValue(config, path);
		if (typeof value === 'boolean') {
			return value;
		}
		return fallback;
	}

	function readTimezonesText() {
		const value = getNestedValue(config, 'timezones');
		if (Array.isArray(value)) {
			return value.join('\n');
		}
		return '';
	}

	function readCalendarModeFromConfig() {
		const value = readString('calendar.mode');
		return value || '__default';
	}

	function readWeeksPositionFromConfig() {
		const value = readString('calendar.weeks-pos');
		return value || '__default';
	}

	function readActionValueFromConfig(actionKey) {
		const value = readString(`actions.${actionKey}`);
		return value || '__default';
	}

	function computeConfigSignature() {
		try {
			return JSON.stringify(config ?? {});
		} catch (error) {
			console.warn('Unable to compute module config signature', error);
			return '';
		}
	}

	function cloneConfigSource() {
		const source = config && typeof config === 'object' ? config : {};
		if (typeof structuredClone === 'function') {
			try {
				return structuredClone(source);
			} catch {
				/* no-op */
			}
		}
		try {
			return JSON.parse(JSON.stringify(source));
		} catch {
			if (Array.isArray(source)) {
				return [...source];
			}
			if (source && typeof source === 'object') {
				return { ...source };
			}
			return source;
		}
	}

	function pruneEmptyBranches(target, segments) {
		let cursor = target;
		const stack = [];
		for (const segment of segments) {
			if (!cursor || typeof cursor !== 'object' || Array.isArray(cursor)) {
				return;
			}
			stack.push({ parent: cursor, key: segment });
			cursor = cursor[segment];
		}
		for (let index = stack.length - 1; index >= 0; index -= 1) {
			const { parent, key } = stack[index];
			const value = parent[key];
			if (
				value &&
				typeof value === 'object' &&
				!Array.isArray(value) &&
				Object.keys(value).length === 0
			) {
				delete parent[key];
				continue;
			}
			break;
		}
	}

	function removeFieldValue(target, path) {
		if (!target || typeof target !== 'object' || !path) {
			return;
		}
		if (!path.includes('.')) {
			delete target[path];
			return;
		}
		const segments = path.split('.');
		let cursor = target;
		for (let index = 0; index < segments.length - 1; index += 1) {
			const segment = segments[index];
			if (!cursor || typeof cursor !== 'object') {
				return;
			}
			cursor = cursor[segment];
		}
		if (!cursor || typeof cursor !== 'object') {
			return;
		}
		delete cursor[segments[segments.length - 1]];
		pruneEmptyBranches(target, segments.slice(0, -1));
	}

	function setFieldValue(target, path, value) {
		if (!path || typeof path !== 'string') {
			return;
		}
		if (value === null || value === undefined) {
			removeFieldValue(target, path);
			return;
		}
		if (!path.includes('.')) {
			target[path] = value;
			return;
		}
		const segments = path.split('.');
		let cursor = target;
		for (let index = 0; index < segments.length - 1; index += 1) {
			const segment = segments[index];
			const current = cursor[segment];
			if (!current || typeof current !== 'object' || Array.isArray(current)) {
				cursor[segment] = {};
			}
			cursor = cursor[segment];
		}
		cursor[segments[segments.length - 1]] = value;
	}

	function parseNumber(raw, { allowFloat = false } = {}) {
		if (raw === null || raw === undefined || raw === '') {
			return null;
		}
		const parsed = allowFloat ? Number.parseFloat(raw) : Number.parseInt(raw, 10);
		return Number.isFinite(parsed) ? parsed : null;
	}

	function setNumberField(target, path, options = {}, source = fieldState) {
		const parsed = parseNumber(source[path], options);
		setFieldValue(target, path, parsed);
	}

	function setStringField(target, path, source = fieldState) {
		const value = source[path];
		if (typeof value === 'string' && value.length > 0) {
			setFieldValue(target, path, value);
		} else {
			setFieldValue(target, path, null);
		}
	}

	function setSelectField(target, path, source = fieldState) {
		const value = source[path];
		if (typeof value === 'string' && value.length && value !== '__default') {
			setFieldValue(target, path, value);
		} else {
			setFieldValue(target, path, null);
		}
	}

	function setBooleanField(target, path, source = fieldState) {
		const value = source[path];
		if (typeof value === 'boolean') {
			setFieldValue(target, path, value);
		} else {
			setFieldValue(target, path, readBoolean(path, false));
		}
	}

	function setTimezonesField(target, source = fieldState) {
		const raw = source.timezonesText ?? '';
		const values = raw
			.split(/\r?\n/)
			.map((entry) => entry.trim())
			.filter((entry) => entry.length > 0);
		setFieldValue(target, 'timezones', values.length ? values : null);
	}

	function buildModuleConfigFromFieldState(source = fieldState) {
		const base = cloneConfigSource();
		if (hasClockDialog) {
			for (const [path, options] of CLOCK_NUMBER_FIELDS) {
				setNumberField(base, path, options, source);
			}
			for (const path of CLOCK_STRING_FIELDS) {
				setStringField(base, path, source);
			}
			for (const key of CLOCK_CALENDAR_FORMAT_FIELDS) {
				setStringField(base, `calendar.format.${key}`, source);
			}
			setBooleanField(base, 'tooltip', source);
			setTimezonesField(base, source);
			setSelectField(base, 'calendar.mode', source);
			setSelectField(base, 'calendar.weeks-pos', source);
			for (const actionField of actionEventFields) {
				setSelectField(base, `actions.${actionField.key}`, source);
			}
		} else if (legacyFieldDefinitions) {
			for (const field of legacyFieldDefinitions) {
				if (field.type === 'number') {
					const allowFloat = typeof field.step === 'number' ? !Number.isInteger(field.step) : false;
					setNumberField(base, field.key, { allowFloat }, source);
				} else if (field.type === 'boolean') {
					setBooleanField(base, field.key, source);
				} else {
					setStringField(base, field.key, source);
				}
			}
		}
		return base;
	}

	function getCalendarModeValue() {
		const value = fieldState['calendar.mode'];
		return typeof value === 'string' && value.length ? value : '__default';
	}

	function getCalendarModeLabel() {
		const value = getCalendarModeValue();
		if (value === 'year') {
			return 'Year';
		}
		if (value === 'month') {
			return 'Month';
		}
		return 'Default (Month)';
	}

	function getWeeksPositionValue() {
		const value = fieldState['calendar.weeks-pos'];
		return typeof value === 'string' && value.length ? value : '__default';
	}

	function getWeeksPositionLabel() {
		const value = getWeeksPositionValue();
		if (value === 'left') {
			return 'Left';
		}
		if (value === 'right') {
			return 'Right';
		}
		return 'Hidden';
	}

	function getActionValue(actionKey) {
		const value = fieldState[`actions.${actionKey}`];
		return typeof value === 'string' && value.length ? value : '__default';
	}

	function hydrateFieldStateFromConfig(force = false) {
		const signature = computeConfigSignature();
		if (!force && signature === lastConfigSignature) {
			return;
		}
		const next = {};
		if (hasClockDialog) {
			next.interval = numberToInput(readNumber('interval'));
			next.format = readString('format');
			next['format-alt'] = readString('format-alt');
			next.timezone = readString('timezone');
			next.timezonesText = readTimezonesText();
			next.locale = readString('locale');
			next['max-length'] = numberToInput(readNumber('max-length'));
			next.rotate = numberToInput(readNumber('rotate'));
			next['smooth-scrolling-threshold'] = numberToInput(readNumber('smooth-scrolling-threshold'));
			next['on-click'] = readString('on-click');
			next['on-click-middle'] = readString('on-click-middle');
			next['on-click-right'] = readString('on-click-right');
			next['on-scroll-up'] = readString('on-scroll-up');
			next['on-scroll-down'] = readString('on-scroll-down');
			next.tooltip = readBoolean('tooltip', false);
			next['tooltip-format'] = readString('tooltip-format');
			next['calendar.mode'] = readCalendarModeFromConfig();
			next['calendar.mode-mon-col'] = numberToInput(readNumber('calendar.mode-mon-col'));
			next['calendar.weeks-pos'] = readWeeksPositionFromConfig();
			next['calendar.on-scroll'] = numberToInput(readNumber('calendar.on-scroll'));
			for (const key of CLOCK_CALENDAR_FORMAT_FIELDS) {
				next[`calendar.format.${key}`] = readString(`calendar.format.${key}`);
			}
			for (const actionField of actionEventFields) {
				next[`actions.${actionField.key}`] = readActionValueFromConfig(actionField.key);
			}
		} else if (legacyFieldDefinitions) {
			for (const field of legacyFieldDefinitions) {
				if (field.type === 'number') {
					next[field.key] = numberToInput(readNumber(field.key));
				} else if (field.type === 'boolean') {
					next[field.key] = readBoolean(field.key, false);
				} else {
					next[field.key] = readString(field.key);
				}
			}
		}
		fieldState = next;
		lastConfigSignature = signature;
		lastEmittedSignature = JSON.stringify(buildModuleConfigFromFieldState(next));
	}

	hydrateFieldStateFromConfig(true);

	$effect(() => {
		const signature = computeConfigSignature();
		if (!wasOpen && open) {
			hydrateFieldStateFromConfig(true);
		} else if (!open && signature !== lastConfigSignature) {
			hydrateFieldStateFromConfig(true);
		}
		wasOpen = open;
	});

	$effect(() => {
		if (!open) {
			lastEmittedSignature = JSON.stringify(buildModuleConfigFromFieldState());
			return;
		}
		const payload = buildModuleConfigFromFieldState();
		const signature = JSON.stringify(payload);
		if (signature === lastEmittedSignature) {
			return;
		}
		lastEmittedSignature = signature;
		dispatch('configChange', { config: payload });
	});
</script>

<Dialog.Root bind:open>
	<Dialog.Trigger asChild>
		<Button variant="outline" size="sm" class="tracking-wide uppercase" {disabled}>
			Configure
		</Button>
	</Dialog.Trigger>
	<Dialog.Content class="sm:max-w-3xl">
		<Dialog.Header>
			<Dialog.Title class="text-sm font-semibold tracking-wide uppercase">
				{moduleTitle}
			</Dialog.Title>
			{#if moduleDescription}
				<Dialog.Description class="text-muted-foreground text-xs uppercase">
					{moduleDescription}
				</Dialog.Description>
			{/if}
		</Dialog.Header>

		{#if hasClockDialog}
			<Tabs.Root value={activeTab} onValueChange={(value) => (activeTab = value)}>
				<Tabs.List
					class="border-border text-muted-foreground mb-4 flex gap-2 border-b pb-2 text-[0.65rem] font-semibold uppercase"
				>
					<Tabs.Trigger value="general">General</Tabs.Trigger>
					<Tabs.Trigger value="tooltip">Tooltip</Tabs.Trigger>
					<Tabs.Trigger value="calendar">Calendar</Tabs.Trigger>
					<Tabs.Trigger value="actions">Actions</Tabs.Trigger>
				</Tabs.List>

				<Tabs.Content value="general" class="space-y-4">
					<ScrollArea.Root class="max-h-[60vh] pr-2">
						<div class="grid gap-4 md:grid-cols-2">
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-interval">
									Interval (Seconds)
								</Label>
								<Input
									id="clock-interval"
									type="number"
									min={1}
									step={1}
									bind:value={fieldState.interval}
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-format">
									Format
								</Label>
								<Input
									id="clock-format"
									type="text"
									bind:value={fieldState.format}
									placeholder={'{:%H:%M}'}
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-format-alt">
									Alternate Format
								</Label>
								<Input
									id="clock-format-alt"
									type="text"
									bind:value={fieldState['format-alt']}
									placeholder={'{:%A, %B %d, %Y (%R)}'}
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-timezone">
									Primary Timezone
								</Label>
								<Input
									id="clock-timezone"
									type="text"
									bind:value={fieldState.timezone}
									placeholder="America/New_York"
									{disabled}
								/>
							</div>
							<div class="space-y-2 md:col-span-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-timezones">
									Additional Timezones (one per line)
								</Label>
								<Textarea
									id="clock-timezones"
									rows={4}
									bind:value={fieldState.timezonesText}
									placeholder="Etc/UTC\nAsia/Tokyo"
									{disabled}
								/>
								<p class="text-muted-foreground text-[0.6rem] uppercase">
									Leave blank to inherit the primary timezone.
								</p>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-locale">
									Locale
								</Label>
								<Input
									id="clock-locale"
									type="text"
									bind:value={fieldState.locale}
									placeholder="en_US.UTF-8"
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-max-length">
									Max Length
								</Label>
								<Input
									id="clock-max-length"
									type="number"
									min={1}
									step={1}
									bind:value={fieldState['max-length']}
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-rotate">
									Rotate (Degrees)
								</Label>
								<Input
									id="clock-rotate"
									type="number"
									step={1}
									bind:value={fieldState.rotate}
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-smooth-threshold">
									Smooth Scroll Threshold
								</Label>
								<Input
									id="clock-smooth-threshold"
									type="number"
									step="0.1"
									bind:value={fieldState['smooth-scrolling-threshold']}
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-on-click">
									On Click Command
								</Label>
								<Input
									id="clock-on-click"
									type="text"
									bind:value={fieldState['on-click']}
									placeholder="command"
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-on-click-middle">
									Middle Click Command
								</Label>
								<Input
									id="clock-on-click-middle"
									type="text"
									bind:value={fieldState['on-click-middle']}
									placeholder="command"
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-on-click-right">
									Right Click Command
								</Label>
								<Input
									id="clock-on-click-right"
									type="text"
									bind:value={fieldState['on-click-right']}
									placeholder="command"
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-scroll-up">
									Scroll Up Command
								</Label>
								<Input
									id="clock-scroll-up"
									type="text"
									bind:value={fieldState['on-scroll-up']}
									placeholder="command"
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-scroll-down">
									Scroll Down Command
								</Label>
								<Input
									id="clock-scroll-down"
									type="text"
									bind:value={fieldState['on-scroll-down']}
									placeholder="command"
									{disabled}
								/>
							</div>
						</div>
					</ScrollArea.Root>
				</Tabs.Content>

				<Tabs.Content value="tooltip" class="space-y-4">
					<ScrollArea.Root class="max-h-[60vh] pr-2">
						<div class="space-y-4">
							<div
								class="border-border/60 flex items-center justify-between gap-4 rounded border px-3 py-2"
							>
								<div>
									<p class="text-[0.65rem] font-semibold uppercase">Tooltip</p>
									<p class="text-muted-foreground text-[0.6rem] uppercase">
										Toggle hover tooltip visibility.
									</p>
								</div>
								<Switch bind:checked={fieldState.tooltip} {disabled} />
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-tooltip-format">
									Tooltip Format
								</Label>
								<Textarea
									id="clock-tooltip-format"
									rows={4}
									bind:value={fieldState['tooltip-format']}
									placeholder={'{:%Y-%m-%d}'}
									{disabled}
								/>
								<p class="text-muted-foreground text-[0.6rem] uppercase">
									Supports calendar placeholders such as {'{calendar}'} and {'{tz_list}'}.
								</p>
							</div>
						</div>
					</ScrollArea.Root>
				</Tabs.Content>

				<Tabs.Content value="calendar" class="space-y-4">
					<ScrollArea.Root class="max-h-[60vh] pr-2">
						<div class="space-y-6">
							<div class="grid gap-4 md:grid-cols-2">
								<div class="space-y-2">
									<Label class="text-[0.65rem] font-semibold uppercase">Mode</Label>
									<Select.Root
										type="single"
										value={getCalendarModeValue()}
										onValueChange={(value) => (fieldState['calendar.mode'] = value)}
										{disabled}
									>
										<Select.Trigger class="uppercase">
											{getCalendarModeLabel()}
										</Select.Trigger>
										<Select.Content>
											<Select.Item value="__default">Default (Month)</Select.Item>
											<Select.Item value="month">Month</Select.Item>
											<Select.Item value="year">Year</Select.Item>
										</Select.Content>
									</Select.Root>
								</div>
								<div class="space-y-2">
									<Label
										class="text-[0.65rem] font-semibold uppercase"
										for="clock-calendar-mode-col"
									>
										Months Per Row
									</Label>
									<Input
										id="clock-calendar-mode-col"
										type="number"
										min={1}
										step={1}
										bind:value={fieldState['calendar.mode-mon-col']}
										{disabled}
									/>
								</div>
								<div class="space-y-2">
									<Label class="text-[0.65rem] font-semibold uppercase">Week Number Position</Label>
									<Select.Root
										type="single"
										value={getWeeksPositionValue()}
										onValueChange={(value) => (fieldState['calendar.weeks-pos'] = value)}
										{disabled}
									>
										<Select.Trigger class="uppercase">
											{getWeeksPositionLabel()}
										</Select.Trigger>
										<Select.Content>
											<Select.Item value="__default">Hidden</Select.Item>
											<Select.Item value="left">Left</Select.Item>
											<Select.Item value="right">Right</Select.Item>
										</Select.Content>
									</Select.Root>
								</div>
								<div class="space-y-2">
									<Label
										class="text-[0.65rem] font-semibold uppercase"
										for="clock-calendar-on-scroll"
									>
										Scroll Increment
									</Label>
									<Input
										id="clock-calendar-on-scroll"
										type="number"
										step={1}
										bind:value={fieldState['calendar.on-scroll']}
										{disabled}
									/>
								</div>
							</div>

							<div class="space-y-4">
								<h3
									class="text-accent-foreground text-[0.65rem] font-semibold tracking-wide uppercase"
								>
									Calendar Formatting
								</h3>
								<div class="grid gap-4 md:grid-cols-2">
									{#each [{ key: 'months', label: 'Months' }, { key: 'days', label: 'Days' }, { key: 'weeks', label: 'Weeks' }, { key: 'weekdays', label: 'Weekdays' }, { key: 'today', label: 'Today Highlight' }] as formatField (formatField.key)}
										<div class="space-y-2">
											<Label
												class="text-[0.65rem] font-semibold uppercase"
												for={`clock-calendar-format-${formatField.key}`}
											>
												{formatField.label}
											</Label>
											<Input
												id={`clock-calendar-format-${formatField.key}`}
												type="text"
												bind:value={fieldState[`calendar.format.${formatField.key}`]}
												{disabled}
											/>
										</div>
									{/each}
								</div>
							</div>
						</div>
					</ScrollArea.Root>
				</Tabs.Content>

				<Tabs.Content value="actions" class="space-y-4">
					<ScrollArea.Root class="max-h-[60vh] pr-2">
						<div class="space-y-4">
							<p class="text-muted-foreground text-[0.6rem] uppercase">
								Assign preset calendar or timezone actions to input events.
							</p>
							<div class="grid gap-4 md:grid-cols-2">
								{#each actionEventFields as actionField (actionField.key)}
									<div class="space-y-2">
										<Label class="text-[0.65rem] font-semibold uppercase">{actionField.label}</Label
										>
										<Select.Root
											type="single"
											value={getActionValue(actionField.key)}
											onValueChange={(value) => (fieldState[`actions.${actionField.key}`] = value)}
											{disabled}
										>
											<Select.Trigger class="uppercase">
												{getActionLabel(getActionValue(actionField.key))}
											</Select.Trigger>
											<Select.Content>
												{#each actionOptions as option (option.value)}
													<Select.Item value={option.value}>
														{option.label}
													</Select.Item>
												{/each}
											</Select.Content>
										</Select.Root>
									</div>
								{/each}
							</div>
						</div>
					</ScrollArea.Root>
				</Tabs.Content>
			</Tabs.Root>
		{:else if legacyFieldDefinitions}
			<ScrollArea.Root class="max-h-[60vh] pr-2">
				<div class="space-y-3">
					{#each legacyFieldDefinitions as field (field.key)}
						<div class="space-y-2">
							<Label
								class="text-[0.65rem] font-semibold uppercase"
								for={`${module.id}-${field.key}-dialog`}
							>
								{field.label}
							</Label>
							{#if field.type === 'number'}
								<Input
									id={`${module.id}-${field.key}-dialog`}
									type="number"
									min={field.min}
									max={field.max}
									step={field.step}
									bind:value={fieldState[field.key]}
									{disabled}
								/>
							{:else if field.type === 'boolean'}
								<Switch bind:checked={fieldState[field.key]} {disabled} />
							{:else}
								<Input
									id={`${module.id}-${field.key}-dialog`}
									type="text"
									bind:value={fieldState[field.key]}
									placeholder={field.placeholder}
									{disabled}
								/>
							{/if}
						</div>
					{/each}
				</div>
			</ScrollArea.Root>
		{:else}
			<p class="text-muted-foreground text-xs uppercase">
				No additional options are available for this module yet.
			</p>
		{/if}

		<Dialog.Footer class="mt-4">
			<Dialog.Close
				class="border-border/70 text-accent-foreground hover:border-border focus-visible:ring-ring inline-flex items-center justify-center rounded border px-3 py-1 text-xs font-semibold tracking-wide uppercase transition focus-visible:ring-1 focus-visible:outline-none"
			>
				Close
			</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
