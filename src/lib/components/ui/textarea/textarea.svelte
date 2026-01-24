<script>
	import { cn } from '$lib/utils.js';
	let { ref = $bindable(null), value = $bindable(), class: className, ...restProps } = $props();
</script>

<textarea
	bind:this={ref}
	data-slot="textarea"
	class={cn(
		'border-input placeholder:text-muted-foreground/30 focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 flex field-sizing-content min-h-16 w-full border bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 md:text-sm',
		className
	)}
	bind:value
	{...restProps}
></textarea>
