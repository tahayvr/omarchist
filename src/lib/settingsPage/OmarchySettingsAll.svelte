<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { homeDir } from '@tauri-apps/api/path';
	import { invoke } from '@tauri-apps/api/core';

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
			<Card.Title class="uppercase">Omarchy Settings</Card.Title>
		</Card.Header>
		<Card.Content>
			<!-- <p class="text-sm">Click the button below to update Omarchy to the latest version.</p> -->
		</Card.Content>
		<Card.Footer>
			<Button onclick={updateOmarchy} variant="outline">Update Omarchy</Button>
		</Card.Footer>
	</Card.Root>
</div>
