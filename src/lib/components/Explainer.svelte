<script>
	import * as HoverCard from '$lib/components/ui/hover-card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import ManualIcon from '@lucide/svelte/icons/square-m';

	let { explainerText = '', docUrl = '', docLabel = 'Read more' } = $props();

	const hasDocLink = $derived(Boolean(docUrl && String(docUrl).trim().length > 0));
</script>

<HoverCard.Root>
	<HoverCard.Trigger>
		<Button
			variant="ghost"
			size="icon"
			class="text-muted-foreground/50 hover:bg-primary dark:hover:bg-primary h-6 w-6 p-0"
			aria-label="Explain setting"
		>
			<ManualIcon />
		</Button>
	</HoverCard.Trigger>
	<HoverCard.Content class="space-y-2 text-sm leading-relaxed">
		<p class="text-muted-foreground text-xs tracking-wide uppercase">{explainerText}</p>
		{#if hasDocLink}
			<a
				href={docUrl}
				target="_blank"
				rel="noreferrer noopener"
				class="text-muted-foreground text-xs font-semibold tracking-wide uppercase hover:underline"
			>
				{docLabel}
			</a>
		{/if}
	</HoverCard.Content>
</HoverCard.Root>
