<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import ColorPickerField from '$lib/themeDesigner/ColorPickerField.svelte';
	import * as HoverCard from '$lib/components/ui/hover-card/index.js';

	let layoutValue = $state('');
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="uppercase">General</Card.Title>
	</Card.Header>
	<Card.Content class="space-y-6 uppercase">
		<div class="flex items-center space-x-4">
			<Switch id="no_border_on_floating" />
			<Label for="no_border_on_floating">No border on floating windows</Label>
		</div>
		<div class="flex items-center space-x-4">
			<Label for="no_border_on_floating">Layout</Label>
			<Select.Root type="single" name="layout" bind:value={layoutValue}>
				<Select.Trigger class="w-[180px]">{layoutValue}</Select.Trigger>
				<Select.Content>
					<Select.Item value="master">MASTER</Select.Item>
					<Select.Item value="dwindle">DWINLDE</Select.Item>
				</Select.Content>
			</Select.Root>
		</div>
		<div class="flex items-center space-x-4">
			<Switch id="no_focus_fallback" />
			<Label for="no_focus_fallback">No focus fallback</Label>
		</div>
		<div class="flex items-center space-x-4">
			<Switch id="resize_on_border" />
			<Label for="resize_on_border">Resize on border</Label>
		</div>
		<div class="flex items-center space-x-4">
			<Label for="extend_border_grab_area">Extend border grab area</Label>
			<Input id="extend_border_grab_area" type="number" class="w-32" placeholder="15" />
		</div>
		<div class="flex items-center space-x-4">
			<Switch id="hover_icon_on_border" checked />
			<Label for="hover_icon_on_border">Hover icon on border</Label>
		</div>
		<div class="flex items-center space-x-4">
			<Switch id="allow_tearing" />
			<Label for="allow_tearing">Allow tearing</Label>
		</div>
		<div class="flex items-center space-x-4">
			<Label for="resize_corner">Resize corner</Label>
			<Input id="resize_corner" type="number" class="w-32" placeholder="0" />
		</div>
	</Card.Content>
</Card.Root>
