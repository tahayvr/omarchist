<script>
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';

	let { colors = null, loading = false } = $props();

	// Fallback colors when no color data is available
	const fallbackColors = {
		primary: {
			background: '#1a1a1a',
			foreground: '#ffffff'
		},
		terminal: {
			red: '#ff5555',
			green: '#50fa7b',
			yellow: '#f1fa8c',
			blue: '#8be9fd',
			magenta: '#ff79c6',
			cyan: '#8be9fd'
		}
	};

	// Use provided colors or fallback
	const displayColors = $derived(colors || fallbackColors);
</script>

{#if loading}
	<!-- Loading state with skeleton -->
	<div class="flex aspect-video flex-col">
		<div class="flex h-1/2 w-full">
			<Skeleton class="h-full w-1/2 rounded-none" />
			<Skeleton class="h-full w-1/2 rounded-none" />
		</div>
		<div class="flex h-1/2 w-full">
			<Skeleton class="h-full w-1/6 rounded-none" />
			<Skeleton class="h-full w-1/6 rounded-none" />
			<Skeleton class="h-full w-1/6 rounded-none" />
			<Skeleton class="h-full w-1/6 rounded-none" />
			<Skeleton class="h-full w-1/6 rounded-none" />
			<Skeleton class="h-full w-1/6 rounded-none" />
		</div>
	</div>
{:else}
	<!-- Color palette display -->
	<div class="flex aspect-video flex-col">
		<!-- Primary colors row (background and foreground) -->
		<div class="flex h-1/2 w-full">
			<div
				class="w-1/2"
				style="background-color: {displayColors.primary.background}"
				title="Background: {displayColors.primary.background}"
			></div>
			<div
				class="w-1/2"
				style="background-color: {displayColors.primary.foreground}"
				title="Foreground: {displayColors.primary.foreground}"
			></div>
		</div>
		<!-- Terminal colors row (red, green, yellow, blue, magenta, cyan) -->
		<div class="flex h-1/2 w-full">
			<div
				class="w-1/6"
				style="background-color: {displayColors.terminal.red}"
				title="Red: {displayColors.terminal.red}"
			></div>
			<div
				class="w-1/6"
				style="background-color: {displayColors.terminal.green}"
				title="Green: {displayColors.terminal.green}"
			></div>
			<div
				class="w-1/6"
				style="background-color: {displayColors.terminal.yellow}"
				title="Yellow: {displayColors.terminal.yellow}"
			></div>
			<div
				class="w-1/6"
				style="background-color: {displayColors.terminal.blue}"
				title="Blue: {displayColors.terminal.blue}"
			></div>
			<div
				class="w-1/6"
				style="background-color: {displayColors.terminal.magenta}"
				title="Magenta: {displayColors.terminal.magenta}"
			></div>
			<div
				class="w-1/6"
				style="background-color: {displayColors.terminal.cyan}"
				title="Cyan: {displayColors.terminal.cyan}"
			></div>
		</div>
	</div>

	<!-- Fallback indicator when using default colors -->
	{#if !colors}
		<div class="text-muted-foreground mt-1 text-center text-xs opacity-60">Default colors</div>
	{/if}
{/if}
