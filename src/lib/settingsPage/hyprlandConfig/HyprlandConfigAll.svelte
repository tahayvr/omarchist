<script>
	import HyprlandAnimation from './HyprlandAnimation.svelte';
	import HyprlandDecoration from './HyprlandDecoration.svelte';
	import HyprlandGeneral from './HyprlandGeneral.svelte';
</script>

<div class="flex flex-col gap-4 xl:grid xl:grid-cols-2">
	<div class="grid-span-full">
		<HyprlandGeneral />
	</div>
	<div class="grid-span-full">
		<HyprlandDecoration />
	</div>
	<div class="grid-span-full">
		<HyprlandAnimation />
	</div>
</div>
