<script>
	import { Accordion as AccordionPrimitive } from "bits-ui";

	let {
		ref = $bindable(null),
		value = $bindable(),
		...restProps
	} = $props();
</script>

<AccordionPrimitive.Root
	bind:ref
	bind:value={value}
	data-slot="accordion"
	{...restProps}
/>