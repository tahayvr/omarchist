<script>
	import { cn } from "$lib/utils.js";
	let { class: className, children, ...restProps } = $props();
</script>

<div
	role="list"
	data-slot="item-group"
	class={cn("group/item-group flex flex-col", className)}
	{...restProps}
>
	{@render children?.()}
</div>