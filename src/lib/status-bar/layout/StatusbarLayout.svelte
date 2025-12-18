<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import StatusbarLayoutItem from './StatusbarLayoutItem.svelte';
	import { KNOWN_MODULES } from '$lib/utils/waybar/waybarConfigUtils.js';

	let {
		layout = { left: [], center: [], right: [] },
		modules = KNOWN_MODULES,
		disabled = false,
		onReorder = () => {}
	} = $props();

	const moduleLookup = $derived(new Map(modules.map((entry) => [entry.id, entry])));

	const sections = [
		{ key: 'left', title: 'Left Panel', description: 'Typically workspace and window info.' },
		{ key: 'center', title: 'Center Panel', description: 'Commonly used for time or status.' },
		{
			key: 'right',
			title: 'Right Panel',
			description: 'Good spot for battery, network, or tray modules.'
		}
	];

	function handleReorder(sectionKey, event) {
		if (disabled) {
			return;
		}

		const modules = event?.modules;
		if (!Array.isArray(modules)) {
			return;
		}
		onReorder({ section: sectionKey, modules });
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Layout</Card.Title>
	</Card.Header>
	<Card.Content class="flex flex-col gap-4">
		{#each sections as section (section.key)}
			<Card.Root class="h-full w-full">
				<Card.Header>
					<Card.Title class="text-accent-foreground/70 uppercase">{section.title}</Card.Title>
				</Card.Header>
				<Card.Content>
					<StatusbarLayoutItem
						modules={layout[section.key] ?? []}
						{moduleLookup}
						{disabled}
						onReorder={(event) => handleReorder(section.key, event)}
					/>
				</Card.Content>
			</Card.Root>
		{/each}
	</Card.Content>
</Card.Root>
