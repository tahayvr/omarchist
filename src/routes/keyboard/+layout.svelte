<script>
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/sidebar/AppSidebar.svelte';
	let { children } = $props();
</script>

<Sidebar.Provider open={false}>
	<AppSidebar />
	<main class="w-full">
		{@render children()}
	</main>
</Sidebar.Provider>
