<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';

	let {
		isLoading = false,
		isSaving = false,
		dirty = false,
		isValid = true,
		onSave = () => {},
		onReset = () => {}
	} = $props();

	const busyLabel = $derived(isSaving ? 'Saving…' : isLoading ? 'Loading…' : '');

	function handleSave() {
		onSave?.();
	}

	function handleReset() {
		onReset?.();
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Waybar Configuration</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase">
			Edit the active Waybar layout and save to <span class="font-semibold"
				>~/.config/waybar/config.jsonc</span
			>.
		</Card.Description>
	</Card.Header>
	<Card.Content
		class="flex flex-col gap-3 uppercase md:flex-row md:items-center md:justify-between md:gap-4"
	>
		<div class="text-muted-foreground flex flex-col gap-1 text-xs tracking-wide md:text-sm">
			{#if busyLabel}
				<span>{busyLabel}</span>
			{:else}
				<span>{dirty ? 'Unsaved changes' : 'All changes saved'}</span>
			{/if}
			{#if !isValid}
				<span class="text-destructive">Validation required before saving.</span>
			{/if}
		</div>
		<div class="flex items-center gap-2">
			<Button variant="ghost" size="sm" disabled={isLoading || isSaving} onClick={handleReset}>
				Reset to defaults
			</Button>
			<Button
				variant="outline"
				disabled={isLoading || isSaving || !dirty || !isValid}
				onClick={handleSave}
			>
				Save configuration
			</Button>
		</div>
	</Card.Content>
</Card.Root>
