<script module>
	import { tv } from "tailwind-variants";

	const itemMediaVariants = tv({
		base: "flex shrink-0 items-center justify-center gap-2 group-has-[[data-slot=item-description]]/item:translate-y-0.5 group-has-[[data-slot=item-description]]/item:self-start [&_svg]:pointer-events-none",
		variants: {
			variant: {
				default: "bg-transparent",
				icon: "bg-muted size-8 rounded-sm border [&_svg:not([class*='size-'])]:size-4",
				image: "size-10 overflow-hidden rounded-sm [&_img]:size-full [&_img]:object-cover",
			},
		},
		defaultVariants: {
			variant: "default",
		},
	});
</script>

<script>
	import { cn } from "$lib/utils.js";
	let {
		class: className,
		children,
		variant = "default",
		...restProps
	} = $props();
</script>

<div
	data-slot="item-media"
	data-variant={variant}
	class={cn(itemMediaVariants({ variant }), className)}
	{...restProps}
>
	{@render children?.()}
</div>