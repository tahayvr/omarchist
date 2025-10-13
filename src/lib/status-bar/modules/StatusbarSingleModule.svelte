<script>
	import { createEventDispatcher } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';

	let {
		module = { id: 'module', title: 'Module', description: 'Module description goes here.' },
		position = $bindable('hidden'),
		fields = [],
		config = {},
		disabled = false
	} = $props();

	const dispatch = createEventDispatcher();

	function handleValueChange(nextPosition) {
		position = nextPosition || 'hidden';
		dispatch('change', { moduleId: module.id, position });
	}

	function emitFieldChange(fieldKey, value) {
		dispatch('fieldChange', { moduleId: module.id, fieldKey, value });
	}

	function getTextValue(fieldKey) {
		const value = config?.[fieldKey];
		return typeof value === 'string' ? value : (value ?? '');
	}

	function getNumberValue(fieldKey) {
		const value = Number(config?.[fieldKey]);
		return Number.isFinite(value) ? value : '';
	}

	function getBooleanValue(fieldKey) {
		return Boolean(config?.[fieldKey]);
	}

	function handleTextChange(fieldKey, event) {
		emitFieldChange(fieldKey, event.target.value ?? '');
	}

	function handleNumberChange(field, event) {
		const raw = event.target.value;
		const parsed = Number(raw);
		if (!Number.isFinite(parsed)) {
			return;
		}
		let nextValue = parsed;
		if (typeof field.min === 'number') {
			nextValue = Math.max(field.min, nextValue);
		}
		if (typeof field.max === 'number') {
			nextValue = Math.min(field.max, nextValue);
		}
		emitFieldChange(field.key, nextValue);
	}

	function handleBooleanChange(fieldKey, checked) {
		emitFieldChange(fieldKey, Boolean(checked));
	}

	function toInputId(fieldKey) {
		return `${module.id}-${fieldKey}`.replace(/[^a-z0-9_-]/gi, '-');
	}
</script>

<Card.Root data-disabled={disabled ? '' : undefined} class={disabled ? 'opacity-75' : ''}>
	<Card.Header>
		<Card.Title class="text-accent-foreground/70 uppercase">{module.title}</Card.Title>
		<Card.Description class="text-muted-foreground text-xs tracking-wide uppercase">
			{module.description}
		</Card.Description>
	</Card.Header>
	<Card.Content>
		<ToggleGroup.Root
			type="single"
			aria-label="Module Position"
			bind:value={position}
			onValueChange={handleValueChange}
			size="lg"
			{disabled}
		>
			<ToggleGroup.Item value="left" class="uppercase">Left</ToggleGroup.Item>
			<ToggleGroup.Item value="center" class="uppercase">Center</ToggleGroup.Item>
			<ToggleGroup.Item value="right" class="uppercase">Right</ToggleGroup.Item>
			<ToggleGroup.Item value="hidden" class="uppercase">Hidden</ToggleGroup.Item>
		</ToggleGroup.Root>

		{#if fields.length}
			<div class="mt-4 space-y-3">
				{#each fields as field (field.key)}
					<div class="space-y-2">
						<div class="flex items-center justify-between gap-2">
							<Label for={toInputId(field.key)} class="text-[0.65rem] font-semibold uppercase">
								{field.label}
							</Label>
							{#if field.type !== 'boolean' && field.placeholder}
								<span class="text-muted-foreground text-[0.6rem] uppercase">
									{field.placeholder}
								</span>
							{/if}
						</div>
						{#if field.type === 'select'}
							<!-- Reserved for future select fields -->
						{:else if field.type === 'number'}
							<Input
								id={toInputId(field.key)}
								type="number"
								class="uppercase"
								value={getNumberValue(field.key)}
								min={field.min}
								max={field.max}
								step={field.step}
								placeholder={field.placeholder}
								{disabled}
								on:change={(event) => handleNumberChange(field, event)}
							/>
						{:else if field.type === 'boolean'}
							<div class="flex items-center justify-between">
								<Switch
									id={toInputId(field.key)}
									checked={getBooleanValue(field.key)}
									onCheckedChange={(checked) => handleBooleanChange(field.key, checked)}
									{disabled}
								/>
							</div>
						{:else}
							<Input
								id={toInputId(field.key)}
								type="text"
								class="uppercase"
								value={getTextValue(field.key)}
								placeholder={field.placeholder}
								{disabled}
								on:change={(event) => handleTextChange(field.key, event)}
							/>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</Card.Content>
</Card.Root>
