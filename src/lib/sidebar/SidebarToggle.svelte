<script>
	import { useSidebar } from '$lib/components/ui/sidebar/index.js';
	import OpenIcon from '@lucide/svelte/icons/arrow-right-to-line';
	import CloseIcon from '@lucide/svelte/icons/arrow-left-to-line';
	import { Button } from '$lib/components/ui/button/index.js';

	const sidebar = useSidebar();
</script>

<Button size="icon" variant="ghost" onclick={() => sidebar.toggle()}>
	{#if sidebar.state === 'expanded'}
		<CloseIcon />
	{:else}
		<OpenIcon />
	{/if}
</Button>
