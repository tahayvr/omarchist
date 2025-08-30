<script>
	import { Tabs as TabsPrimitive } from 'bits-ui';
	import { cn } from '$lib/utils.js';

	let { ref = $bindable(null), class: className, ...restProps } = $props();
</script>

<TabsPrimitive.List
	bind:ref
	data-slot="tabs-list"
	class={cn(
		'text-foreground inline-flex h-9 w-fit items-center justify-center bg-transparent p-[3px]',
		className
	)}
	{...restProps}
/>
