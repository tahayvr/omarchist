<script>
	import { Button } from '$lib/components/ui/button/index.js';
	import GithubIcon from '@lucide/svelte/icons/github';
	import TwitterIcon from '@lucide/svelte/icons/twitter';
</script>

<div class="flex h-full w-full flex-col items-center justify-center gap-8 px-6 py-12">
	<div class="flex flex-col items-center">
		<h2 class="text-4xl font-bold uppercase">Omarchist</h2>
		<span>v0.3.0</span>
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
			<TwitterIcon />
			<span class="sr-only">Twitter</span>
		</Button>
	</div>
	<div class="mt-12 max-w-md space-y-2 text-center">
		<h3 class="text-muted-foreground font-semibold uppercase">hotkeys</h3>
		<ul class="text-sm">
			<li><kbd>CTRL</kbd> + <kbd>S</kbd>: Update Theme</li>
		</ul>
	</div>
</div>
