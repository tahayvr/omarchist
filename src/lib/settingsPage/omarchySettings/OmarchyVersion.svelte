<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { homeDir } from '@tauri-apps/api/path';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	let versionNum = $state('');
	let updateAvailable = $state(false);
	let checkingUpdate = $state(false);

	onMount(async () => {
		try {
			const version = await invoke('get_omarchy_version');
			versionNum = version;

			// Check for updates
			checkingUpdate = true;
			try {
				const hasUpdate = await invoke('check_omarchy_update');
				updateAvailable = hasUpdate;
			} catch (error) {
				console.error('Failed to check for updates:', error);
			} finally {
				checkingUpdate = false;
			}
		} catch (error) {
			console.error('Failed to get version:', error);
			versionNum = 'unknown';
		}
	});

	async function updateOmarchy() {
		try {
			const homePath = await homeDir();
			const scriptPath = `${homePath}/.local/share/omarchy/bin/omarchy-update`;
			await invoke('run_update_script', { scriptPath });
		} catch (error) {
			alert(`Failed to run update script: ${error}`);
		}
	}
</script>

<div>
	<Card.Root>
		<Card.Header>
			<Card.Title class="uppercase">Omarchy</Card.Title>
			<Card.Description class="text-xs tracking-wide uppercase"
				>Current Version: {versionNum}</Card.Description
			>
		</Card.Header>
		<Card.Content>
			{#if checkingUpdate}
				<p class="text-muted-foreground mb-4 text-sm">Checking for updates...</p>
			{/if}
			{#if updateAvailable}
				<Button onclick={updateOmarchy} variant="outline">Update Omarchy</Button>
			{:else}
				<Button onclick={updateOmarchy} variant="outline" class="uppercase" disabled
					>Omarchy is up to date</Button
				>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
