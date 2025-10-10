<script>
	import { cn } from "$lib/utils.js";
	let { class: className, children, ...restProps } = $props();
</script>

<kbd data-slot="kbd-group" class={cn("inline-flex items-center gap-1", className)} {...restProps}>
	{@render children?.()}
</kbd>