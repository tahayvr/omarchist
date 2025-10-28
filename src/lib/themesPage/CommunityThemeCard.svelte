<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { Command } from '@tauri-apps/plugin-shell';

	const {
		title = '',
		author = '',
		imageUrl = '',
		detailUrl = '',
		githubUrl = '',
		installCommand = '',
		installUrl = ''
	} = $props();

	let isApplying = $state(false);

	const canApply = $derived(
		Boolean((installCommand && installCommand.trim()) || (installUrl && installUrl.trim()))
	);

	function resolveInstallInvocation() {
		const cleanedCommand = sanitizeInstallCommand(installCommand);
		if (cleanedCommand.startsWith('omarchy-theme-install')) {
			const tokens = cleanedCommand.split(/\s+/).filter(Boolean);
			if (tokens.length >= 2) {
				const [command, ...args] = tokens;
				return { command, args };
			}
		}

		if (installUrl && installUrl.trim()) {
			return { command: 'omarchy-theme-install', args: [installUrl.trim()] };
		}

		return null;
	}

	function sanitizeInstallCommand(raw) {
		if (!raw) return '';
		let line = raw.trim();
		line = line.replace(/^`+|`+$/g, '').trim();
		line = line.replace(/^\$+\s*/, '').trim();
		line = line.replace(/^[-*]\s+/, '').trim();
		return line;
	}

	async function handleOpen() {
		const target = githubUrl?.trim() || detailUrl?.trim();
		if (!target) {
			alert('No project link is available for this theme yet.');
			return;
		}

		try {
			await openUrl(target);
		} catch (err) {
			console.error('Failed to open community theme URL:', err);
			alert('Unable to open theme URL. Please try again later.');
		}
	}

	async function handleApply() {
		if (isApplying) return;

		const invocation = resolveInstallInvocation();
		if (!invocation) {
			alert('Installation details are not available for this theme yet.');
			return;
		}

		try {
			isApplying = true;
			await Command.create(invocation.command, invocation.args).execute();
			window.dispatchEvent(
				new CustomEvent('themes:changed', { detail: { action: 'installed', theme: title } })
			);
		} catch (err) {
			console.error('Failed to install community theme:', err);
			alert('Unable to install theme. Check the console for details.');
		} finally {
			isApplying = false;
		}
	}
</script>

<div class="flex w-full items-center justify-center">
	<Card.Root class="h-full w-full">
		<Card.Header class="flex flex-row items-start justify-between gap-4">
			<div class="flex flex-col gap-1 uppercase">
				<Card.Title>{title}</Card.Title>
				{#if author}
					<Card.Description class="text-xs normal-case tracking-wide text-muted-foreground">
						by {author}
					</Card.Description>
				{/if}
			</div>
			<Badge variant="primary" class="text-muted-foreground">Community</Badge>
		</Card.Header>
		<Card.Content>
			{#if imageUrl}
				<img src={imageUrl} alt={title} class="aspect-video w-full rounded-md object-cover" />
			{:else}
				<div class="flex aspect-video w-full items-center justify-center rounded-md bg-muted text-xs uppercase tracking-widest text-muted-foreground">
					Preview unavailable
				</div>
			{/if}
		</Card.Content>
		<Card.Footer class="flex items-center justify-between uppercase">
			<Button
				variant="ghost"
				size="sm"
				class="uppercase"
				onclick={handleApply}
				disabled={!canApply || isApplying}
			>
				{#if isApplying}
					Installing...
				{:else}
					Apply Theme
				{/if}
			</Button>
			<Button
				variant="ghost"
				size="sm"
				class="uppercase"
				onclick={handleOpen}
				disabled={!githubUrl && !detailUrl}
			>
				View on GitHub
			</Button>
		</Card.Footer>
	</Card.Root>
</div>
