<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';

	let { module_name = 'Module Name', module_description = 'Module description goes here.' } =
		$props();
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">{module_name}</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase">
			{module_description}
		</Card.Description>
	</Card.Header>
	<Card.Content>
		<ToggleGroup.Root type="single" aria-label="Module Position" value="hidden" size="lg">
			<ToggleGroup.Item value="left" class="uppercase">Left</ToggleGroup.Item>
			<ToggleGroup.Item value="center" class="uppercase">Center</ToggleGroup.Item>
			<ToggleGroup.Item value="right" class="uppercase">Right</ToggleGroup.Item>
			<ToggleGroup.Item value="hidden" class="uppercase">Hidden</ToggleGroup.Item>
		</ToggleGroup.Root>
	</Card.Content>
</Card.Root>
