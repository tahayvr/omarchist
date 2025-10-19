<script>
	import { cn } from "$lib/utils.js";
	let { class: className, children, ...restProps } = $props();
</script>

<div
	data-slot="item-content"
	class={cn("flex flex-1 flex-col gap-1 [&+[data-slot=item-content]]:flex-none", className)}
	{...restProps}
>
	{@render children?.()}
</div>