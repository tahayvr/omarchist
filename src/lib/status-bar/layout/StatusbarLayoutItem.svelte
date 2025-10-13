<script>
	import { createEventDispatcher } from 'svelte';
	import * as Item from '$lib/components/ui/item/index.js';

	let { modules = [], moduleLookup = new Map(), disabled = false } = $props();
	const hasModules = $derived(Array.isArray(modules) && modules.length > 0);
	const dispatch = createEventDispatcher();

	let draggingId = $state(null);
	let dragImageEl = null;

	function cleanupDragImage() {
		if (dragImageEl) {
			document.body.removeChild(dragImageEl);
			dragImageEl = null;
		}
	}

	function setDragImage(event, target) {
		cleanupDragImage();
		if (!target || !event.dataTransfer) {
			return;
		}
		const clone = target.cloneNode(true);
		const rect = target.getBoundingClientRect();
		clone.style.width = `${rect.width}px`;
		clone.style.height = `${rect.height}px`;
		clone.style.position = 'fixed';
		clone.style.top = '-9999px';
		clone.style.left = '-9999px';
		clone.style.pointerEvents = 'none';
		document.body.appendChild(clone);
		dragImageEl = clone;
		event.dataTransfer.setDragImage(clone, rect.width / 2, rect.height / 2);
	}

	function getLabel(moduleId) {
		return moduleLookup.get(moduleId)?.title ?? moduleId;
	}

	function handleDragStart(event, moduleId) {
		if (disabled) {
			event.preventDefault();
			return;
		}
		draggingId = moduleId;
		if (event.dataTransfer) {
			event.dataTransfer.effectAllowed = 'move';
			event.dataTransfer.setData('application/x-omarchist-waybar', moduleId);
			event.dataTransfer.setData('text/plain', moduleId);
			setDragImage(event, event.currentTarget);
		}
	}

	function handleDragOver(event) {
		event.preventDefault();
		if (disabled) {
			return;
		}
		if (event.dataTransfer) {
			event.dataTransfer.dropEffect = 'move';
		}
	}

	function handleDrop(event, index) {
		event.preventDefault();
		event.stopPropagation();
		if (disabled) {
			return;
		}
		const moduleId =
			event.dataTransfer?.getData('application/x-omarchist-waybar') ||
			event.dataTransfer?.getData('text/plain') ||
			draggingId;
		draggingId = null;
		if (!moduleId) {
			return;
		}
		const currentIndex = modules.indexOf(moduleId);
		if (currentIndex === -1) {
			return;
		}

		let targetIndex = index;
		if (currentIndex < targetIndex) {
			targetIndex -= 1;
		}
		if (targetIndex < 0) {
			targetIndex = 0;
		}
		if (targetIndex > modules.length) {
			targetIndex = modules.length;
		}
		if (currentIndex === targetIndex) {
			return;
		}

		const next = [...modules];
		next.splice(currentIndex, 1);
		next.splice(targetIndex, 0, moduleId);
		dispatch('reorder', { modules: next });
	}

	function handleDragEnd() {
		draggingId = null;
		cleanupDragImage();
	}
</script>

{#if hasModules}
	<div
		class="flex flex-wrap gap-2"
		role="list"
		ondragover={handleDragOver}
		ondrop={(event) => handleDrop(event, modules.length)}
	>
		{#each modules as moduleId, index (moduleId)}
			<div
				class={`border-muted-foreground/60 border transition-all ${
					draggingId === moduleId
						? 'ring-accent-foreground/70 bg-accent/10 cursor-grabbing opacity-70 ring-2'
						: 'hover:ring-accent-foreground/70 cursor-grab hover:ring-1'
				}`}
				draggable={!disabled}
				role="listitem"
				aria-grabbed={draggingId === moduleId}
				ondragstart={(event) => handleDragStart(event, moduleId)}
				ondragover={handleDragOver}
				ondrop={(event) => handleDrop(event, index)}
				ondragend={handleDragEnd}
			>
				<Item.Root class="bg-foreground text-background">
					<Item.Content class="px-3 py-1.5">
						<Item.Title class="text-xs font-semibold tracking-wide uppercase">
							{getLabel(moduleId)}
						</Item.Title>
					</Item.Content>
				</Item.Root>
			</div>
		{/each}
	</div>
{:else}
	<div
		class="border-muted-foreground/50 text-muted-foreground rounded border border-dashed px-3 py-2 text-xs tracking-wide uppercase"
	>
		No modules assigned
	</div>
{/if}
