<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { homeDir } from '@tauri-apps/api/path';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	let versionNum = $state('');
	let updateAvailable = $state(false);

	onMount(async () => {
		try {
			const version = await invoke('get_omarchy_version');
			versionNum = version;
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
			<Card.Description>Current Version: {versionNum}</Card.Description>
		</Card.Header>
		<Card.Content>
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
