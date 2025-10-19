<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import * as Accordion from '$lib/components/ui/accordion/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
</script>

<Card.Root class="space-y-4">
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Screen Saver</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase"
			>Configure your screen saver settings</Card.Description
		>
	</Card.Header>
	<Card.Content class="space-y-6 uppercase">
		<div class="basic flex w-1/2 items-center justify-between gap-4">
			<Label for="enable_screensaver" class="flex-1">Enable Screen Saver</Label>
			<Switch id="enable_screensaver" />
		</div>
		<Accordion.Root type="single">
			<Accordion.Item>
				<Accordion.Trigger class="uppercase">Customize the screen saver text</Accordion.Trigger>
				<Accordion.Content class=" flex flex-col gap-4">
					<div class="flex flex-col gap-1.5">
						<Textarea />
						<p class="text-muted-foreground text-xs normal-case">
							This text will be displayed when the screensaver is active.
						</p>
					</div>
					<Button variant="outline" size="sm" class="self-end">Launch Screensaver</Button>
				</Accordion.Content>
			</Accordion.Item>
		</Accordion.Root>
	</Card.Content>
</Card.Root>
