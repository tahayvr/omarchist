<script>
	import * as Item from '$lib/components/ui/item/index.js';

	let { modules = [], moduleLookup = new Map() } = $props();
	const hasModules = $derived(Array.isArray(modules) && modules.length > 0);

	function getLabel(moduleId) {
		return moduleLookup.get(moduleId)?.title ?? moduleId;
	}
</script>

{#if hasModules}
	<div class="flex flex-wrap gap-2">
		{#each modules as moduleId (moduleId)}
			<Item.Root class="border-muted-foreground/60 border">
				<Item.Content class="px-3 py-1.5">
					<Item.Title class="text-xs font-semibold tracking-wide uppercase">
						{getLabel(moduleId)}
					</Item.Title>
				</Item.Content>
			</Item.Root>
		{/each}
	</div>
{:else}
	<div
		class="border-muted-foreground/50 text-muted-foreground rounded border border-dashed px-3 py-2 text-xs tracking-wide uppercase"
	>
		No modules assigned
	</div>
{/if}
