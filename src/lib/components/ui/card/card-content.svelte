<script>
	import { cn } from '$lib/utils.js';

	let { ref = $bindable(null), class: className, children, ...restProps } = $props();
</script>

<div bind:this={ref} data-slot="card-content" class={cn('px-6', className)} {...restProps}>
	{@render children?.()}
</div>
