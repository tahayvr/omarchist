<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { invoke } from '@tauri-apps/api/core';

	let { isLoading = false, isSaving = false, onReset = () => {} } = $props();

	function handleReset() {
		onReset?.();
	}

	async function restartApp(appName) {
		await invoke('execute_bash_command_async', {
			command: `omarchy-restart-app ${appName}`
		});
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Status Bar</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase">
			Changes auto-save to <span class="font-semibold">~/.config/waybar/config.jsonc</span> | Restart
			status bar to see changes.
		</Card.Description>
	</Card.Header>
	<Card.Content
		class="flex flex-col gap-3 uppercase md:flex-row md:items-center md:justify-between md:gap-4"
	>
		<div class="text-muted-foreground flex flex-col gap-1 text-xs tracking-wide md:text-sm"></div>
		<div class="flex items-center gap-4">
			<Button
				class="uppercase"
				variant="ghost"
				disabled={isLoading || isSaving}
				onclick={handleReset}
			>
				Reset
			</Button>
			<Button class="uppercase" variant="outline" onclick={() => restartApp('waybar')}
				>Restart Status Bar</Button
			>
		</div>
	</Card.Content>
</Card.Root>
