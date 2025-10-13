<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import MoreIcon from '@lucide/svelte/icons/ellipsis-vertical';
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Status Bar</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase"
			>Configure your waybar</Card.Description
		>
	</Card.Header>
	<Card.Content class="flex items-center justify-between uppercase">
		<div class="flex items-center gap-2 uppercase">
			<Select.Root type="single" value="daily" aria-label="Waybar Profile">
				<Select.Trigger class="w-[180px]"></Select.Trigger>
				<Select.Content>
					<Select.Item value="daily">Daily Driver</Select.Item>
					<Select.Item value="fun">Fun</Select.Item>
					<Select.Item value="work">Work</Select.Item>
					<Select.Item value="vertical">Vertical</Select.Item>
				</Select.Content>
			</Select.Root>
			<Button variant="outline" class="ml-4 uppercase">New Configuration</Button>
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="ghost" size="icon" class="h-8 w-8 p-0 ">
							<MoreIcon class="h-4 w-4" />
							<span class="sr-only">Open options</span>
						</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end" class="uppercase">
					<DropdownMenu.Group>
						<DropdownMenu.Item>Share configuration</DropdownMenu.Item>
						<DropdownMenu.Separator />
						<DropdownMenu.Item class="bg-red-500/15 text-red-500 hover:text-red-500"
							>Delete configuration</DropdownMenu.Item
						>
					</DropdownMenu.Group>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		</div>
		<Button variant="outline">Update Status Bar</Button>
	</Card.Content>
</Card.Root>
