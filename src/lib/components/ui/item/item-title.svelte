<script>
	import { cn } from "$lib/utils.js";
	let { class: className, children, ...restProps } = $props();
</script>

<div
	data-slot="item-title"
	class={cn("flex w-fit items-center gap-2 text-sm font-medium leading-snug", className)}
	{...restProps}
>
	{@render children?.()}
</div>