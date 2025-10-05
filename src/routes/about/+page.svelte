<script>
	import { Button } from '$lib/components/ui/button/index.js';
	import GithubIcon from '@lucide/svelte/icons/github';
	import TwitterIcon from '@lucide/svelte/icons/twitter';
	import XIcon from '$lib/icons/XIcon.svelte';
</script>

<div class="flex h-full w-full flex-col items-center justify-center gap-8 px-6 py-12">
	<div class="flex flex-col items-center">
		<img src="/icon.png" alt="App Icon" class="mb-4 h-24 w-24" />
		<h2 class="text-4xl font-bold uppercase">Omarchist</h2>
		<span class="text-muted-foreground">v0.6.0</span>
	</div>
	<div>
		<Button
			variant="outline"
			size="icon"
			href="https://github.com/tahayvr/omarchist"
			target="_blank"
			rel="noopener noreferrer"
		>
			<GithubIcon />
			<span class="sr-only">Source Code</span>
		</Button>
		<Button
			variant="outline"
			size="icon"
			href="https://x.com/tahayvr"
			target="_blank"
			rel="noopener noreferrer"
		>
			<XIcon />
			<span class="sr-only">Twitter</span>
		</Button>
	</div>
	<div class="mt-12 max-w-md space-y-2 text-center">
		<h3 class="text-muted-foreground font-semibold uppercase">hotkeys</h3>
		<ul class="list-inside list-disc space-y-1 text-start text-sm leading-6">
			<li><kbd>CTRL</kbd> + <kbd>S</kbd>: Save Theme/Settings</li>
			<li><kbd>CTRL</kbd> + <kbd>B</kbd>: Toggle Sidebar</li>
		</ul>
	</div>
</div>
