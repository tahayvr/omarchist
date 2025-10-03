<script>
	import * as Accordion from '$lib/components/ui/accordion/index.js';
	import HyprlandConfigAll from '$lib/settingsPage/hyprlandConfig/HyprlandConfigAll.svelte';
	import AppSettingsAll from '$lib/settingsPage/omarchistSettings/AppSettingsAll.svelte';
	import OmarchySettingsAll from '$lib/settingsPage/omarchySettings/OmarchySettingsAll.svelte';
</script>

<div class="px-6 py-12">
	<Accordion.Root type="single" value="item-1">
		<Accordion.Item value="item-1">
			<Accordion.Trigger class="uppercase">Omarchist</Accordion.Trigger>
			<Accordion.Content><AppSettingsAll /></Accordion.Content>
		</Accordion.Item>
		<Accordion.Item value="item-2">
			<Accordion.Trigger class="uppercase">Hyprland</Accordion.Trigger>
			<Accordion.Content><HyprlandConfigAll /></Accordion.Content>
		</Accordion.Item>
		<Accordion.Item value="item-3">
			<Accordion.Trigger class="uppercase">Omarchy</Accordion.Trigger>
			<Accordion.Content><OmarchySettingsAll /></Accordion.Content>
		</Accordion.Item>
	</Accordion.Root>
</div>
