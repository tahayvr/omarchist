<script>
	/* eslint-disable svelte/prefer-writable-derived */
	import { createEventDispatcher } from 'svelte';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as HoverCard from '$lib/components/ui/hover-card/index.js';
	import ColorPicker from 'svelte-awesome-color-picker';

	// Props
	let { label = 'Color', color = '#000000', format = 'hex', description = null } = $props();

	const dispatch = createEventDispatcher();

	// Convert various formats to hex for the color picker
	function toHex(input, fmt) {
		if (fmt === 'rgba-comma') {
			if (typeof input === 'string') {
				const parts = input.split(',').map((p) => p.trim());
				if (parts.length >= 3) {
					const r = parseInt(parts[0]) || 0;
					const g = parseInt(parts[1]) || 0;
					const b = parseInt(parts[2]) || 0;
					const toHex = (n) => Math.max(0, Math.min(255, n)).toString(16).padStart(2, '0');
					return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
				}
			}
			return '#000000';
		}
		if (fmt === 'hex-no-hash') {
			return `#${String(input).replace(/^#/, '')}`;
		}
		return String(input) || '#000000';
	}

	// Convert hex back to the required format
	function fromHex(hexValue, fmt, rgbData) {
		if (fmt === 'hex-no-hash') {
			return hexValue.replace(/^#/, '');
		}
		if (fmt === 'rgba-comma' && rgbData) {
			const r = rgbData.r || 0;
			const g = rgbData.g || 0;
			const b = rgbData.b || 0;
			const a = rgbData.a !== undefined ? rgbData.a : 1;
			return `${r},${g},${b},${Number(a).toFixed(1)}`;
		}
		return hexValue;
	}

	// Internal state for the color picker
	let internalHex = $state(toHex(color, format));
	let internalRgb = $state({ r: 0, g: 0, b: 0, a: 1 });

	// Update internal state when color prop changes
	$effect(() => {
		internalHex = toHex(color, format);
	});

	// Watch for changes to internalHex from color picker interactions
	$effect(() => {
		if (internalHex && internalHex !== toHex(color, format)) {
			handleColorChange();
		}
	});

	// Handle color picker changes
	function handleColorChange() {
		const outputValue = fromHex(internalHex, format, internalRgb);
		dispatch('change', outputValue);
	}
</script>

<div class="flex items-center space-x-2">
	<Label>
		{#if description}
			<HoverCard.Root>
				<HoverCard.Trigger>
					{label}:
				</HoverCard.Trigger>
				<HoverCard.Content>
					{description}
				</HoverCard.Content>
			</HoverCard.Root>
		{:else}
			{label}:
		{/if}
	</Label>
	<div class="dark">
		<ColorPicker
			bind:hex={internalHex}
			bind:rgb={internalRgb}
			isAlpha={format === 'rgba-comma'}
			position="responsive"
			label=""
			on:input={handleColorChange}
			on:change={handleColorChange}
		/>
	</div>
</div>

<style>
	.dark {
		--input-size: 20px;
		--cp-bg-color: #121212;
		--cp-border-color: white;
		--cp-text-color: #eaeaea;
		--cp-input-color: #555;
		--cp-button-hover-color: #777;
	}
</style>
