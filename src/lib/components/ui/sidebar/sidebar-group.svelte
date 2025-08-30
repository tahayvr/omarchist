<script>
	import { cn } from '$lib/utils.js';

	let { ref = $bindable(null), class: className, children, ...restProps } = $props();
</script>

<div
	bind:this={ref}
	data-slot="sidebar-group"
	data-sidebar="group"
	class={cn('relative flex w-full min-w-0 flex-col p-2', className)}
	{...restProps}
>
	{@render children?.()}
</div>
