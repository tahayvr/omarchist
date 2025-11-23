<script>
	import { createEventDispatcher } from 'svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import SchemaBasedModuleForm from './SchemaBasedModuleForm.svelte';
	import { getModuleDefinition } from '$lib/utils/waybar/moduleRegistry.js';
	import GearIcon from '@lucide/svelte/icons/settings';

	let { module = null, config = {}, disabled = false } = $props();

	const dispatch = createEventDispatcher();
	let open = $state(false);
	let lastEmittedSignature = '';

	const moduleDefinition = $derived(getModuleDefinition(module?.id));
	const hasSchema = $derived(moduleDefinition && moduleDefinition.schema);
	const CustomComponent = $derived(moduleDefinition?.component);

	const moduleTitle = $derived(module?.title ?? 'Waybar Module');
	const moduleDescription = $derived(module?.description ?? '');

	function handleConfigChange(event) {
		const newConfig = event.detail?.config;
		if (!newConfig || typeof newConfig !== 'object') {
			return;
		}

		const signature = JSON.stringify(newConfig);
		if (signature === lastEmittedSignature) {
			return;
		}

		lastEmittedSignature = signature;
		dispatch('configChange', { config: newConfig });
	}

	// Initialize/reset signature when dialog opens/closes
	$effect(() => {
		if (open) {
			// When opening, set the signature to prevent initial spurious events
			lastEmittedSignature = JSON.stringify(config ?? {});
		} else {
			// When closing, reset to allow re-opening with same config
			lastEmittedSignature = JSON.stringify(config ?? {});
		}
	});
</script>

<Dialog.Root bind:open>
	<Dialog.Trigger asChild>
		<Button variant="secondary" size="icon" class="tracking-wide uppercase" {disabled}>
			<GearIcon />
		</Button>
	</Dialog.Trigger>
	<Dialog.Content class="w-[90vw] !max-w-4xl">
		<Dialog.Header>
			<Dialog.Title class="text-sm font-semibold tracking-wide uppercase">
				{moduleTitle}
			</Dialog.Title>
			{#if moduleDescription}
				<Dialog.Description class="text-muted-foreground text-xs uppercase">
					{moduleDescription}
				</Dialog.Description>
			{/if}
		</Dialog.Header>

		<div class="mt-4">
			{#if CustomComponent}
				<!-- Custom component for modules that need special UI -->
				<CustomComponent {module} {config} {disabled} on:configChange={handleConfigChange} />
			{:else if hasSchema}
				<!-- Schema-based form for modules with schema definitions -->
				<SchemaBasedModuleForm
					schema={moduleDefinition.schema}
					{config}
					{disabled}
					on:configChange={handleConfigChange}
				/>
			{:else}
				<!-- Fallback message for modules without configuration -->
				<p class="text-muted-foreground text-xs uppercase">
					No additional options are available for this module yet.
				</p>
			{/if}
		</div>

		<Dialog.Footer class="mt-4">
			<Dialog.Close
				class="border-border/70 text-accent-foreground hover:border-border focus-visible:ring-ring inline-flex items-center justify-center rounded border px-3 py-1 text-xs font-semibold tracking-wide uppercase transition focus-visible:ring-1 focus-visible:outline-none"
			>
				Close
			</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
