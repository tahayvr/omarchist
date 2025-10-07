<script>
	import CopyIcon from '@lucide/svelte/icons/copy';
	import { Button } from '$lib/components/ui/button/index.js';
	import { toast } from 'svelte-sonner';
	import { writeText as writeTauriText } from '@tauri-apps/plugin-clipboard-manager';

	let { value = '' } = $props();
	let isCopying = $state(false);

	const isTauriRuntime = typeof window !== 'undefined' && !!window.__TAURI__;

	async function writeToClipboard(text) {
		const trimmed = text?.toString?.().trim();
		if (!trimmed) return false;

		if (isTauriRuntime) {
			try {
				await writeTauriText(trimmed);
				return true;
			} catch (err) {
				console.error('Tauri clipboard manager failed:', err);
			}
		}

		if (typeof navigator !== 'undefined' && navigator?.clipboard?.writeText) {
			try {
				await navigator.clipboard.writeText(trimmed);
				return true;
			} catch (err) {
				console.error('Browser clipboard API failed:', err);
			}
		}

		if (typeof document === 'undefined') return false;

		try {
			const textarea = document.createElement('textarea');
			textarea.value = trimmed;
			textarea.setAttribute('readonly', '');
			textarea.style.position = 'fixed';
			textarea.style.opacity = '0';
			document.body.appendChild(textarea);
			textarea.select();
			const ok = document.execCommand('copy');
			document.body.removeChild(textarea);
			return ok;
		} catch (err) {
			console.error('Legacy clipboard fallback failed:', err);
			return false;
		}
	}

	async function handleCopy(event) {
		event?.preventDefault?.();
		event?.stopPropagation?.();

		if (isCopying) return;

		const trimmedValue = value?.toString?.().trim();
		if (!trimmedValue) return;

		isCopying = true;

		try {
			const success = await writeToClipboard(trimmedValue);
			if (success) {
				toast.success('Copied to clipboard', { description: trimmedValue });
			} else {
				toast.error('Copy failed', { description: 'Clipboard is unavailable.' });
			}
		} catch (error) {
			toast.error('Copy failed', { description: error?.message || String(error) });
		} finally {
			isCopying = false;
		}
	}
</script>

<Button
	type="button"
	variant="ghost"
	size="icon"
	class="text-muted-foreground/50 hover:bg-primary dark:hover:bg-primary h-8 w-8 p-0"
	aria-label="Copy color value"
	onclick={handleCopy}
	disabled={isCopying || !value?.toString?.().trim()}
>
	<CopyIcon class="h-4 w-4" />
</Button>
