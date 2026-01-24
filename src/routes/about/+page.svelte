<script>
	import { Button } from '$lib/components/ui/button/index.js';
	import GithubIcon from '@lucide/svelte/icons/github';
	import XIcon from '$lib/icons/XIcon.svelte';
	import * as Kbd from '$lib/components/ui/kbd/index.js';
</script>

<div class="flex h-full w-full flex-col items-center justify-center gap-8 px-6 py-12">
	<div class="flex flex-col items-center">
		<img src="/icon.png" alt="App Icon" class="mb-4 h-24 w-24" />
		<h2 class="text-4xl font-bold uppercase">Omarchist</h2>
		<span class="text-muted-foreground">v0.8.0</span>
	</div>
	<div class="flex items-center gap-2">
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
		<!-- <Button variant="outline" href="https://docs.omarchist.com" target="_blank">Docs</Button> -->
	</div>
	<div class="mt-12 max-w-md space-y-2 text-center">
		<h3 class="text-muted-foreground font-semibold uppercase">hotkeys</h3>
		<ul class="list-inside space-y-1 text-start text-sm leading-6">
			<li>
				<Kbd.Group>
					<Kbd.Root>Ctrl</Kbd.Root>
					<span>+</span>
					<Kbd.Root>S</Kbd.Root>
				</Kbd.Group>: Save
			</li>
			<li>
				<Kbd.Group>
					<Kbd.Root>Ctrl</Kbd.Root>
					<span>+</span>
					<Kbd.Root>B</Kbd.Root>
				</Kbd.Group>: Toggle Sidebar
			</li>
		</ul>
	</div>
</div>
