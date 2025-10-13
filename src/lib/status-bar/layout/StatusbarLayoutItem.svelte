<script>
	import { createEventDispatcher } from 'svelte';
	import {
		dndzone,
		SHADOW_ITEM_MARKER_PROPERTY_NAME,
		SHADOW_PLACEHOLDER_ITEM_ID
	} from 'svelte-dnd-action';
	import * as Item from '$lib/components/ui/item/index.js';

	let { modules = [], moduleLookup = new Map(), disabled = false } = $props();
	const dispatch = createEventDispatcher();

	let orderedItems = $state([]);

	$effect(() => {
		orderedItems = Array.isArray(modules)
			? modules.map((moduleId) => ({ id: moduleId, moduleId }))
			: [];
	});

	const hasModules = $derived(Array.isArray(modules) && modules.length > 0);

	function getLabel(moduleId) {
		return moduleLookup.get(moduleId)?.title ?? moduleId;
	}

	function mapItemsToModules(items) {
		return items
			.filter((item) => {
				if (!item) {
					return false;
				}
				if (item.id === SHADOW_PLACEHOLDER_ITEM_ID) {
					return false;
				}
				if (item[SHADOW_ITEM_MARKER_PROPERTY_NAME]) {
					return false;
				}
				return true;
			})
			.map((item) => item.moduleId ?? item.id)
			.filter((id) => typeof id === 'string' || typeof id === 'number');
	}

	function maybeDispatchReorder(nextOrder) {
		if (!Array.isArray(nextOrder) || nextOrder.length !== modules.length) {
			return;
		}
		const unchanged = nextOrder.every((moduleId, index) => modules[index] === moduleId);
		if (!unchanged) {
			dispatch('reorder', { modules: nextOrder });
		}
	}

	function handleDndConsider(event) {
		if (disabled) {
			return;
		}
		const items = event.detail?.items;
		if (!Array.isArray(items)) {
			return;
		}
		orderedItems = items.map((item) => ({ ...item }));
	}

	function handleDndFinalize(event) {
		const items = event.detail?.items;
		if (!Array.isArray(items)) {
			return;
		}
		orderedItems = items.map((item) => ({ ...item }));
		const nextOrder = mapItemsToModules(items);
		if (!disabled) {
			maybeDispatchReorder(nextOrder);
		}
	}
</script>

{#if hasModules}
	<div
		class="flex flex-wrap gap-2"
		role="list"
		use:dndzone={{
			items: orderedItems,
			flipDurationMs: 150,
			dragDisabled: disabled,
			dropFromOthersDisabled: true
		}}
		onconsider={handleDndConsider}
		onfinalize={handleDndFinalize}
	>
		{#each orderedItems as item (item.id)}
			{#if item.id === SHADOW_PLACEHOLDER_ITEM_ID || item[SHADOW_ITEM_MARKER_PROPERTY_NAME]}
				<div
					class="border-muted-foreground/40 bg-accent/10 text-muted-foreground/80 flex min-w-[3rem] items-center justify-center border border-dashed px-3 py-1.5 text-xs uppercase"
					aria-hidden="true"
				>
					Drop here
				</div>
			{:else}
				<div
					class="border-muted-foreground/60 hover:ring-accent-foreground/70 border transition-all hover:ring-1"
					role="listitem"
					aria-label={getLabel(item.moduleId ?? item.id)}
					data-module-id={item.moduleId ?? item.id}
				>
					<Item.Root class="bg-foreground text-background">
						<Item.Content class="px-3 py-1.5">
							<Item.Title class="text-xs font-semibold tracking-wide uppercase">
								{getLabel(item.moduleId ?? item.id)}
							</Item.Title>
						</Item.Content>
					</Item.Root>
				</div>
			{/if}
		{/each}
	</div>
{:else}
	<div
		class="border-muted-foreground/50 text-muted-foreground rounded border border-dashed px-3 py-2 text-xs tracking-wide uppercase"
	>
		No modules assigned
	</div>
{/if}
