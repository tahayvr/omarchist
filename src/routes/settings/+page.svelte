<script>
	import AccordionTrigger from '$lib/components/ui/accordion/accordion-trigger.svelte';
	import * as Accordion from '$lib/components/ui/accordion/index.js';
	import HyprlandConfigAll from '$lib/settingsPage/hyprlandConfig/HyprlandConfigAll.svelte';
	import AppSettingsAll from '$lib/settingsPage/omarchistSettings/AppSettingsAll.svelte';
</script>

<div class="px-6 py-12">
	<Accordion.Root type="single" value="item-1">
		<Accordion.Item value="item-1">
			<Accordion.Trigger>Omarchist Settings</Accordion.Trigger>
			<Accordion.Content><AppSettingsAll /></Accordion.Content>
		</Accordion.Item>
		<Accordion.Item value="item-2">
			<Accordion.Trigger>Hyprland Configuration</Accordion.Trigger>
			<Accordion.Content><HyprlandConfigAll /></Accordion.Content>
		</Accordion.Item>
		<Accordion.Item value="item-3">
			<Accordion.Trigger>Omarchy Settings</Accordion.Trigger>
			<Accordion.Content>This is where Omarchy settings will go.</Accordion.Content>
		</Accordion.Item>
	</Accordion.Root>
</div>
