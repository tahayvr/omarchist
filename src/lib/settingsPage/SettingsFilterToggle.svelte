<script>
	import { createEventDispatcher } from 'svelte';
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';
	import { cn } from '$lib/utils.js';

	let {
		value = $bindable('basic'),
		class: className,
		variant = 'outline',
		toggleClass = 'px-4 py-2',
		...restProps
	} = $props();

	const dispatch = createEventDispatcher();

	function handleValueChange(nextValue) {
		value = nextValue || 'basic';
		dispatch('change', { value });
	}
</script>

<ToggleGroup.Root
	{variant}
	type="single"
	bind:value
	class={cn('text-xs font-semibold tracking-wide uppercase', className)}
	onValueChange={handleValueChange}
	{...restProps}
>
	<ToggleGroup.Item value="basic" class={toggleClass}>Basic</ToggleGroup.Item>
	<ToggleGroup.Item value="all" class={toggleClass}>All</ToggleGroup.Item>
</ToggleGroup.Root>
