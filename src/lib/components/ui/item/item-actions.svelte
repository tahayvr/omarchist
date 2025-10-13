<script>
	import { cn } from "$lib/utils.js";
	let { class: className, children, ...restProps } = $props();
</script>

<div data-slot="item-actions" class={cn("flex items-center gap-2", className)} {...restProps}>
	{@render children?.()}
</div>