<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import StatusbarLayoutItem from './StatusbarLayoutItem.svelte';
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Layout</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase"
			>Configure the layout of your waybar. You can drag modules to change their order inside the
			panel.
		</Card.Description>
	</Card.Header>
	<Card.Content class="flex flex-col items-center justify-between gap-4">
		<Card.Root class="h-full  w-full">
			<Card.Header>
				<Card.Title class="text-accent-foreground uppercase">Left Panel</Card.Title>
			</Card.Header>
			<Card.Content class="flex items-center justify-between gap-6 p-6">
				<StatusbarLayoutItem />
			</Card.Content>
		</Card.Root>

		<Card.Root class="h-full w-full">
			<Card.Header>
				<Card.Title class="text-accent-foreground uppercase">Center Panel</Card.Title>
			</Card.Header>
			<Card.Content class="flex items-center justify-between gap-6 p-6"
				><StatusbarLayoutItem />
			</Card.Content>
		</Card.Root>

		<Card.Root class="h-full w-full">
			<Card.Header>
				<Card.Title class="text-accent-foreground uppercase">Right Panel</Card.Title>
			</Card.Header>
			<Card.Content class="flex items-center justify-between gap-6 p-6">
				<StatusbarLayoutItem />
			</Card.Content>
		</Card.Root>
	</Card.Content>
</Card.Root>
