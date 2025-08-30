<script>
	import { cn } from '$lib/utils.js';
	let { ref = $bindable(null), children, class: className, ...restProps } = $props();
</script>

<li
	bind:this={ref}
	data-slot="sidebar-menu-sub-item"
	data-sidebar="menu-sub-item"
	class={cn('group/menu-sub-item relative', className)}
	{...restProps}
>
	{@render children?.()}
</li>
