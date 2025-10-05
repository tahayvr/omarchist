<script>
	import { createEventDispatcher } from 'svelte';
	import ColorPickerField from './ColorPickerField.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as HoverCard from '$lib/components/ui/hover-card/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import CardContent from '$lib/components/ui/card/card-content.svelte';

	let { schema = {}, data = {} } = $props();

	const dispatch = createEventDispatcher();

	function handleFieldChange(fieldPath, value) {
		dispatch('field-change', {
			field: fieldPath,
			value: value
		});
	}

	// Create form fields from schema
	function getFieldValue(fieldPath) {
		const parts = fieldPath.split('.');
		let value = data;
		for (const part of parts) {
			value = value?.[part];
		}
		return value;
	}

	// Return entries with "x-order" array if present
	function orderedEntries(schemaObj) {
		const props = schemaObj?.properties || {};
		const order = schemaObj?.['x-order'];
		if (Array.isArray(order) && order.length) {
			const result = [];
			for (const key of order) {
				if (key in props) result.push([key, props[key]]);
			}
			for (const [k, v] of Object.entries(props)) {
				if (!order.includes(k)) result.push([k, v]);
			}
			return result;
		}
		return Object.entries(props);
	}

	// Determine if a schema string property should be rendered with the ColorPickerField
	function isColorish(prop) {
		if (!prop || prop.type !== 'string') return false;
		if (prop.format === 'color') return true;
		const fmt = prop.output_format;
		return fmt === 'hex' || fmt === 'hex-no-hash' || fmt === 'hex-alpha' || fmt === 'rgba-comma';
	}

	// Determine the color picker format based on schema property
	function getColorFormat(prop) {
		if (prop.output_format) {
			return prop.output_format;
		}
		if (prop.format === 'color') {
			return 'hex';
		}
		return 'hex';
	}
</script>

<div class="mt-4 flex flex-col gap-4 uppercase">
	{#if schema?.properties}
		{#each orderedEntries(schema) as [key, property] (key)}
			{@const fieldPath = key}
			{@const value = getFieldValue(fieldPath)}
			{#if isColorish(property)}
				<ColorPickerField
					label={property.title || key}
					color={value ?? property.default ?? '#1e1e1e'}
					format={getColorFormat(property)}
					description={property.description}
					on:change={(e) => handleFieldChange(fieldPath, e.detail)}
				/>
			{:else if property.type === 'string'}
				<Label>
					{#if property.description}
						<HoverCard.Root>
							<HoverCard.Trigger>
								{property.title || key}:
							</HoverCard.Trigger>
							<HoverCard.Content>
								{property.description}
							</HoverCard.Content>
						</HoverCard.Root>
					{:else}
						{property.title || key}:
					{/if}
					<Input
						type="text"
						value={value ?? property.default ?? ''}
						onchange={(e) => handleFieldChange(fieldPath, e.target.value)}
					/>
				</Label>
			{:else if property.type === 'number'}
				<Label>
					{#if property.description}
						<HoverCard.Root>
							<HoverCard.Trigger>
								{property.title || key}:
							</HoverCard.Trigger>
							<HoverCard.Content>
								{property.description}
							</HoverCard.Content>
						</HoverCard.Root>
					{:else}
						{property.title || key}:
					{/if}
					<Input
						type="number"
						value={value ?? property.default ?? property.minimum ?? 0}
						min={property.minimum}
						max={property.maximum}
						onchange={(e) => handleFieldChange(fieldPath, parseFloat(e.target.value))}
					/>
				</Label>
			{:else if property.type === 'boolean'}
				<Label>
					<Input
						type="checkbox"
						checked={value !== undefined ? value : (property.default ?? true)}
						onchange={(e) => handleFieldChange(fieldPath, e.target.checked)}
					/>
					{#if property.description}
						<HoverCard.Root>
							<HoverCard.Trigger>
								{property.title || key}
							</HoverCard.Trigger>
							<HoverCard.Content>
								{property.description}
							</HoverCard.Content>
						</HoverCard.Root>
					{:else}
						{property.title || key}
					{/if}
				</Label>
			{:else if property.type === 'object' && property.properties}
				<div class="flex flex-col gap-2">
					<h4 id="heading" class="text-muted-foreground font-bold">{property.title || key}:</h4>
					<div class="grid h-full grid-cols-1 gap-2 lg:grid-cols-2">
						{#each orderedEntries(property) as [nestedKey, nestedProperty] (nestedKey)}
							{@const nestedFieldPath = `${fieldPath}.${nestedKey}`}
							{@const nestedValue = getFieldValue(nestedFieldPath)}
							<div>
								{#if isColorish(nestedProperty)}
									<ColorPickerField
										label={nestedProperty.title || nestedKey}
										color={nestedValue ?? nestedProperty.default ?? '#1e1e1e'}
										format={getColorFormat(nestedProperty)}
										description={nestedProperty.description}
										on:change={(e) => handleFieldChange(nestedFieldPath, e.detail)}
									/>
								{:else if nestedProperty.type === 'string'}
									<div class="mb-2 flex items-center">
										<Label>
											{#if nestedProperty.description}
												<HoverCard.Root>
													<HoverCard.Trigger>
														{nestedProperty.title || nestedKey}:
													</HoverCard.Trigger>
													<HoverCard.Content>
														{nestedProperty.description}
													</HoverCard.Content>
												</HoverCard.Root>
											{:else}
												{nestedProperty.title || nestedKey}:
											{/if}
											<Input
												type="text"
												value={nestedValue ?? nestedProperty.default ?? ''}
												onchange={(e) => handleFieldChange(nestedFieldPath, e.target.value)}
											/>
										</Label>
									</div>
								{:else if nestedProperty.type === 'number'}
									<div class="mb-2 flex items-center">
										<Label>
											{#if nestedProperty.description}
												<HoverCard.Root>
													<HoverCard.Trigger>
														{nestedProperty.title || nestedKey}:
													</HoverCard.Trigger>
													<HoverCard.Content>
														{nestedProperty.description}
													</HoverCard.Content>
												</HoverCard.Root>
											{:else}
												{nestedProperty.title || nestedKey}:
											{/if}
											<Input
												type="number"
												class="max-w-xs"
												value={nestedValue ?? nestedProperty.default ?? nestedProperty.minimum ?? 0}
												min={nestedProperty.minimum}
												max={nestedProperty.maximum}
												onchange={(e) =>
													handleFieldChange(nestedFieldPath, parseFloat(e.target.value))}
											/>
										</Label>
									</div>
								{:else if nestedProperty.type === 'boolean'}
									<Label>
										<input
											type="checkbox"
											checked={nestedValue !== undefined
												? nestedValue
												: (nestedProperty.default ?? true)}
											onchange={(e) => handleFieldChange(nestedFieldPath, e.target.checked)}
										/>
										{#if nestedProperty.description}
											<HoverCard.Root>
												<HoverCard.Trigger>
													{nestedProperty.title || nestedKey}
												</HoverCard.Trigger>
												<HoverCard.Content>
													{nestedProperty.description}
												</HoverCard.Content>
											</HoverCard.Root>
										{:else}
											{nestedProperty.title || nestedKey}
										{/if}
									</Label>
								{:else if nestedProperty.type === 'object' && nestedProperty.properties}
									<div class="h-full">
										<Card.Root class="flex h-full flex-col">
											<Card.Header>
												<Card.Title>
													{nestedProperty.title || nestedKey}
												</Card.Title>
											</Card.Header>
											<CardContent class="flex-1">
												<div id="subsection" class="grid grid-cols-2 gap-4">
													{#each orderedEntries(nestedProperty) as [deepKey, deepProperty] (deepKey)}
														{@const deepFieldPath = `${nestedFieldPath}.${deepKey}`}
														{@const deepValue = getFieldValue(deepFieldPath)}
														<div>
															{#if isColorish(deepProperty)}
																<ColorPickerField
																	label={deepProperty.title || deepKey}
																	color={deepValue ?? deepProperty.default ?? '#1e1e1e'}
																	format={getColorFormat(deepProperty)}
																	description={deepProperty.description}
																	on:change={(e) => handleFieldChange(deepFieldPath, e.detail)}
																/>
															{:else if deepProperty.type === 'string'}
																<div class="flex items-center">
																	<Label>
																		{#if deepProperty.description}
																			<HoverCard.Root>
																				<HoverCard.Trigger>
																					{deepProperty.title || deepKey}:
																				</HoverCard.Trigger>
																				<HoverCard.Content>
																					{deepProperty.description}
																				</HoverCard.Content>
																			</HoverCard.Root>
																		{:else}
																			{deepProperty.title || deepKey}:
																		{/if}
																		<Input
																			type="text"
																			value={deepValue ?? deepProperty.default ?? ''}
																			onchange={(e) =>
																				handleFieldChange(deepFieldPath, e.target.value)}
																		/>
																	</Label>
																</div>
															{:else if deepProperty.type === 'number'}
																<div class="mb-2 flex items-center">
																	<Label>
																		{#if deepProperty.description}
																			<HoverCard.Root>
																				<HoverCard.Trigger>
																					{deepProperty.title || deepKey}:
																				</HoverCard.Trigger>
																				<HoverCard.Content>
																					{deepProperty.description}
																				</HoverCard.Content>
																			</HoverCard.Root>
																		{:else}
																			{deepProperty.title || deepKey}:
																		{/if}
																		<Input
																			type="number"
																			class="max-w-xs"
																			value={deepValue ??
																				deepProperty.default ??
																				deepProperty.minimum ??
																				0}
																			min={deepProperty.minimum}
																			max={deepProperty.maximum}
																			onchange={(e) =>
																				handleFieldChange(
																					deepFieldPath,
																					parseFloat(e.target.value)
																				)}
																		/>
																	</Label>
																</div>
															{:else if deepProperty.type === 'boolean'}
																<Label>
																	<input
																		type="checkbox"
																		checked={deepValue !== undefined
																			? deepValue
																			: (deepProperty.default ?? true)}
																		onchange={(e) =>
																			handleFieldChange(deepFieldPath, e.target.checked)}
																	/>
																	{#if deepProperty.description}
																		<HoverCard.Root>
																			<HoverCard.Trigger>
																				{deepProperty.title || deepKey}
																			</HoverCard.Trigger>
																			<HoverCard.Content>
																				{deepProperty.description}
																			</HoverCard.Content>
																		</HoverCard.Root>
																	{:else}
																		{deepProperty.title || deepKey}
																	{/if}
																</Label>
															{/if}
														</div>
													{/each}
												</div>
											</CardContent>
										</Card.Root>
									</div>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			{/if}
		{/each}
	{/if}
</div>
