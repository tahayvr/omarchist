<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import StatusbarSingleModule from './StatusbarSingleModule.svelte';
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Modules</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase"
			>Choose your modules and their positions. changing the possition from hidden to a panel will
			add the module to its corresponding layout panel above.
		</Card.Description>
	</Card.Header>
	<Card.Content class="grid grid-cols-2 gap-4 md:grid-cols-3">
		<StatusbarSingleModule />
		<StatusbarSingleModule />
		<StatusbarSingleModule />
		<StatusbarSingleModule />
		<StatusbarSingleModule />
		<StatusbarSingleModule />
	</Card.Content>
</Card.Root>
