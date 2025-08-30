<script>
	import { cn } from '$lib/utils.js';
	let { ref = $bindable(null), class: className, children, ...restProps } = $props();
</script>

<li
	bind:this={ref}
	data-slot="sidebar-menu-item"
	data-sidebar="menu-item"
	class={cn('group/menu-item relative', className)}
	{...restProps}
>
	{@render children?.()}
</li>
