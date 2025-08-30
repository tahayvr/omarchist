<script>
	import { invoke } from '@tauri-apps/api/core';
	import { Button } from '$lib/components/ui/button/index.js';
	import { onMount } from 'svelte';

	let versionNum = $state('loading...');

	onMount(async () => {
		try {
			const version = await invoke('get_omarchy_version');
			versionNum = version;
		} catch (error) {
			console.error('Failed to get version:', error);
			versionNum = 'unknown';
		}
	});
</script>

<main class="bg-background flex h-screen flex-col items-center justify-center gap-8">
	<pre class="text-muted-foreground text-sm leading-none">
                 ▄▄▄
 ▄█████▄    ▄███████████▄    ▄███████   ▄███████   ▄███████   ▄█   █▄    ▄█   █▄
███   ███  ███   ███   ███  ███   ███  ███   ███  ███   ███  ███   ███  ███   ███
███   ███  ███   ███   ███  ███   ███  ███   ███  ███   █▀   ███   ███  ███   ███
███   ███  ███   ███   ███ ▄███▄▄▄███ ▄███▄▄▄██▀  ███       ▄███▄▄▄███▄ ███▄▄▄███
███   ███  ███   ███   ███ ▀███▀▀▀███ ▀███▀▀▀▀    ███      ▀▀███▀▀▀███  ▀▀▀▀▀▀███
███   ███  ███   ███   ███  ███   ███ ██████████  ███   █▄   ███   ███  ▄██   ███
███   ███  ███   ███   ███  ███   ███  ███   ███  ███   ███  ███   ███  ███   ███
 ▀█████▀    ▀█   ███   █▀   ███   █▀   ███   ███  ███████▀   ███   █▀    ▀█████▀
                                       ███   █▀
</pre>

	<p>Version {versionNum}</p>

	<div class="flex gap-4">
		<Button variant="outline" href="/themes">THEMES</Button>
		<!-- <Button variant="outline" href="/apps">APPS</Button> -->
		<Button variant="outline" href="/settings">SETTINGS</Button>
	</div>
</main>
