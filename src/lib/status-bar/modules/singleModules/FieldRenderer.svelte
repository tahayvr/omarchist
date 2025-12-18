<script>
	import { Label } from '$lib/components/ui/label/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import * as Select from '$lib/components/ui/select/index.js';

	let { field, value = $bindable(), disabled = false, fieldKey } = $props();

	function getEnumLabel(val) {
		if (!field.enum) return val;
		const index = field.enum.indexOf(val);
		if (index >= 0 && field.enumLabels && field.enumLabels[index]) {
			return field.enumLabels[index];
		}
		return val;
	}

	const triggerContent = $derived.by(() => {
		if (!value || value === '__default') {
			return 'Select an option';
		}
		return getEnumLabel(value);
	});

	function handleNumberInput(event) {
		const raw = event.target.value;
		if (raw === '' || raw === null || raw === undefined) {
			value = '';
			return;
		}
		if (field.type === 'integer') {
			const parsed = parseInt(raw, 10);
			value = Number.isFinite(parsed) ? parsed : '';
		} else {
			const parsed = parseFloat(raw);
			value = Number.isFinite(parsed) ? parsed : '';
		}
	}

	function numberToInput(val) {
		if (val === '' || val === null || val === undefined) {
			return '';
		}
		return String(val);
	}
</script>

<div class="field-wrapper space-y-2">
	<div class="flex items-start justify-between gap-2">
		<div class="flex-1">
			<Label for={fieldKey} class="text-sm font-medium">
				{field.title || fieldKey}
			</Label>
			{#if field.description}
				<p class="text-muted-foreground mt-1 text-xs">
					{field.description}
				</p>
			{/if}
		</div>
	</div>

	<div class="field-control">
		{#if field.type === 'boolean'}
			<div class="flex items-center gap-2">
				<Switch id={fieldKey} bind:checked={value} {disabled} />
				<Label for={fieldKey} class="text-muted-foreground text-sm font-normal">
					{value ? 'Enabled' : 'Disabled'}
				</Label>
			</div>
		{:else if field.type === 'select' || (field.enum && Array.isArray(field.enum))}
			<Select.Root type="single" bind:value {disabled}>
				<Select.Trigger id={fieldKey} class="w-full">
					{triggerContent}
				</Select.Trigger>
				<Select.Content>
					{#each field.enum as enumValue, index (enumValue)}
						{@const label = field.enumLabels?.[index] || enumValue}
						<Select.Item value={enumValue} {label}>
							{label}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		{:else if field.type === 'integer' || field.type === 'number'}
			<Input
				id={fieldKey}
				type="number"
				value={numberToInput(value)}
				oninput={handleNumberInput}
				min={field.minimum}
				max={field.maximum}
				step={field.type === 'integer' ? 1 : 'any'}
				placeholder={field.placeholder || ''}
				{disabled}
				class="w-full"
			/>
		{:else if field.format === 'textarea'}
			<Textarea
				id={fieldKey}
				bind:value
				placeholder={field.placeholder || ''}
				{disabled}
				rows={4}
				class="w-full text-sm"
			/>
		{:else}
			<Input
				id={fieldKey}
				type="text"
				bind:value
				placeholder={field.placeholder || ''}
				{disabled}
				class="w-full"
			/>
		{/if}
	</div>
</div>

<style>
	.field-wrapper {
		padding: 0.75rem;
		border-radius: 0.375rem;
		background-color: hsl(var(--muted) / 0.3);
	}

	.field-wrapper:hover {
		background-color: hsl(var(--muted) / 0.5);
	}
</style>
