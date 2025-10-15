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

	function emitFieldChange(fieldKey, value) {
		dispatch('fieldChange', { fieldKey, value });
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

	function getString(path) {
		const value = getNestedValue(config, path);
		return typeof value === 'string' ? value : '';
	}

	function getNumber(path) {
		const value = getNestedValue(config, path);
		const parsed = Number(value);
		return Number.isFinite(parsed) ? parsed : '';
	}

	function getBoolean(path, fallback = false) {
		const value = getNestedValue(config, path);
		if (typeof value === 'boolean') {
			return value;
		}
		return fallback;
	}

	function getTimezonesText() {
		const value = getNestedValue(config, 'timezones');
		if (Array.isArray(value)) {
			return value.join('\n');
		}
		return '';
	}

	function updateStringField(path, event) {
		const raw = event?.target?.value ?? '';
		emitFieldChange(path, raw.length ? raw : null);
	}

	function updateNumberField(path, event, { allowFloat = false } = {}) {
		const raw = event?.target?.value ?? '';
		if (raw === '') {
			emitFieldChange(path, null);
			return;
		}
		const parsed = allowFloat ? Number.parseFloat(raw) : Number.parseInt(raw, 10);
		if (Number.isFinite(parsed)) {
			emitFieldChange(path, parsed);
		} else {
			emitFieldChange(path, null);
		}
	}

	function updateSelectField(path, value) {
		if (!value || value === '__default') {
			emitFieldChange(path, null);
			return;
		}
		emitFieldChange(path, value);
	}

	function updateBooleanField(path, checked) {
		emitFieldChange(path, Boolean(checked));
	}

	function updateTimezones(event) {
		const raw = event?.target?.value ?? '';
		const values = raw
			.split(/\r?\n/)
			.map((entry) => entry.trim())
			.filter((entry) => entry.length > 0);
		emitFieldChange('timezones', values.length ? values : null);
	}

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

	function handleLegacyInput(field, event) {
		if (field.type === 'number') {
			updateNumberField(field.key, event, { allowFloat: false });
			return;
		}
		updateStringField(field.key, event);
	}

	function getCalendarModeValue() {
		const value = getString('calendar.mode');
		return value || '__default';
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
		const value = getString('calendar.weeks-pos');
		return value || '__default';
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
		const value = getString(`actions.${actionKey}`);
		return value || '__default';
	}

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
									value={getNumber('interval')}
									{disabled}
									on:change={(event) => updateNumberField('interval', event)}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-format">
									Format
								</Label>
								<Input
									id="clock-format"
									type="text"
									value={getString('format')}
									placeholder={'{:%H:%M}'}
									{disabled}
									on:change={(event) => updateStringField('format', event)}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-format-alt">
									Alternate Format
								</Label>
								<Input
									id="clock-format-alt"
									type="text"
									value={getString('format-alt')}
									placeholder={'{:%A, %B %d, %Y (%R)}'}
									{disabled}
									on:change={(event) => updateStringField('format-alt', event)}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-timezone">
									Primary Timezone
								</Label>
								<Input
									id="clock-timezone"
									type="text"
									value={getString('timezone')}
									placeholder="America/New_York"
									{disabled}
									on:change={(event) => updateStringField('timezone', event)}
								/>
							</div>
							<div class="space-y-2 md:col-span-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-timezones">
									Additional Timezones (one per line)
								</Label>
								<Textarea
									id="clock-timezones"
									rows={4}
									value={getTimezonesText()}
									placeholder="Etc/UTC\nAsia/Tokyo"
									{disabled}
									on:change={updateTimezones}
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
									value={getString('locale')}
									placeholder="en_US.UTF-8"
									{disabled}
									on:change={(event) => updateStringField('locale', event)}
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
									value={getNumber('max-length')}
									{disabled}
									on:change={(event) => updateNumberField('max-length', event)}
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
									value={getNumber('rotate')}
									{disabled}
									on:change={(event) => updateNumberField('rotate', event)}
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
									value={getNumber('smooth-scrolling-threshold')}
									{disabled}
									on:change={(event) =>
										updateNumberField('smooth-scrolling-threshold', event, { allowFloat: true })}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-on-click">
									On Click Command
								</Label>
								<Input
									id="clock-on-click"
									type="text"
									value={getString('on-click')}
									placeholder="command"
									{disabled}
									on:change={(event) => updateStringField('on-click', event)}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-on-click-middle">
									Middle Click Command
								</Label>
								<Input
									id="clock-on-click-middle"
									type="text"
									value={getString('on-click-middle')}
									placeholder="command"
									{disabled}
									on:change={(event) => updateStringField('on-click-middle', event)}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-on-click-right">
									Right Click Command
								</Label>
								<Input
									id="clock-on-click-right"
									type="text"
									value={getString('on-click-right')}
									placeholder="command"
									{disabled}
									on:change={(event) => updateStringField('on-click-right', event)}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-scroll-up">
									Scroll Up Command
								</Label>
								<Input
									id="clock-scroll-up"
									type="text"
									value={getString('on-scroll-up')}
									placeholder="command"
									{disabled}
									on:change={(event) => updateStringField('on-scroll-up', event)}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-scroll-down">
									Scroll Down Command
								</Label>
								<Input
									id="clock-scroll-down"
									type="text"
									value={getString('on-scroll-down')}
									placeholder="command"
									{disabled}
									on:change={(event) => updateStringField('on-scroll-down', event)}
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
								<Switch
									checked={getBoolean('tooltip', false)}
									onCheckedChange={(checked) => updateBooleanField('tooltip', checked)}
									{disabled}
								/>
							</div>
							<div class="space-y-2">
								<Label class="text-[0.65rem] font-semibold uppercase" for="clock-tooltip-format">
									Tooltip Format
								</Label>
								<Textarea
									id="clock-tooltip-format"
									rows={4}
									value={getString('tooltip-format')}
									placeholder={'{:%Y-%m-%d}'}
									{disabled}
									on:change={(event) => updateStringField('tooltip-format', event)}
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
										onValueChange={(value) => updateSelectField('calendar.mode', value)}
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
										value={getNumber('calendar.mode-mon-col')}
										{disabled}
										on:change={(event) => updateNumberField('calendar.mode-mon-col', event)}
									/>
								</div>
								<div class="space-y-2">
									<Label class="text-[0.65rem] font-semibold uppercase">Week Number Position</Label>
									<Select.Root
										type="single"
										value={getWeeksPositionValue()}
										onValueChange={(value) => updateSelectField('calendar.weeks-pos', value)}
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
										value={getNumber('calendar.on-scroll')}
										{disabled}
										on:change={(event) => updateNumberField('calendar.on-scroll', event)}
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
												value={getString(`calendar.format.${formatField.key}`)}
												{disabled}
												on:change={(event) =>
													updateStringField(`calendar.format.${formatField.key}`, event)}
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
											onValueChange={(value) =>
												updateSelectField(`actions.${actionField.key}`, value)}
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
									value={getNumber(field.key)}
									{disabled}
									on:change={(event) => handleLegacyInput(field, event)}
								/>
							{:else if field.type === 'boolean'}
								<Switch
									checked={getBoolean(field.key, false)}
									onCheckedChange={(checked) => updateBooleanField(field.key, checked)}
									{disabled}
								/>
							{:else}
								<Input
									id={`${module.id}-${field.key}-dialog`}
									type="text"
									value={getString(field.key)}
									placeholder={field.placeholder}
									{disabled}
									on:change={(event) => handleLegacyInput(field, event)}
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
