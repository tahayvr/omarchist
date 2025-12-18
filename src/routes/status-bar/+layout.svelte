<script>
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/sidebar/AppSidebar.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	let { children } = $props();

	async function restartApp(appName) {
		await invoke('execute_bash_command_async', {
			command: `omarchy-restart-app ${appName}`
		});
	}

	function handleKeydown(event) {
		if (event.ctrlKey && event.key === 's') {
			event.preventDefault();
			restartApp('waybar');
		}
	}

	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => {
			window.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

<Sidebar.Provider open={false}>
	<AppSidebar />
	<main class="w-full">
		{@render children()}
	</main>
</Sidebar.Provider>
