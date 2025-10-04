<script>
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { open } from '@tauri-apps/plugin-dialog';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import ImageIcon from '@lucide/svelte/icons/image';
	import CardFooter from '$lib/components/ui/card/card-footer.svelte';

	let { themeName = '', backgrounds = $bindable([]) } = $props();
	let isLoading = $state(false);
	let isAdding = $state(false);
	let imageCache = $state(new Map());

	// Load backgrounds when component mounts or theme changes
	$effect(() => {
		if (themeName) {
			loadBackgrounds();
		}
	});

	async function loadBackgrounds() {
		if (!themeName) return;

		isLoading = true;
		try {
			const result = await invoke('get_theme_backgrounds', { themeName });
			backgrounds = result || [];
			// Clear image cache when reloading
			imageCache.clear();
		} catch (error) {
			console.error('Failed to load backgrounds:', error);
			backgrounds = [];
		} finally {
			isLoading = false;
		}
	}

	async function addBackgroundImages() {
		isAdding = true;
		try {
			// Open file dialog
			const selected = await open({
				multiple: true,
				filters: [
					{
						name: 'Images',
						extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp', 'gif']
					}
				]
			});

			if (selected && selected.length > 0) {
				// Copy files to theme backgrounds folder
				await invoke('add_theme_backgrounds', {
					themeName,
					sourcePaths: selected
				});

				// Reload backgrounds list
				await loadBackgrounds();
			}
		} catch (error) {
			console.error('Failed to add background images:', error);
			alert(`Failed to add background images: ${error}`);
		} finally {
			isAdding = false;
		}
	}

	async function removeBackground(filename) {
		try {
			await invoke('remove_theme_background', {
				themeName,
				filename
			});

			// Reload backgrounds list
			await loadBackgrounds();
		} catch (error) {
			console.error('Failed to remove background:', error);
			alert(`Failed to remove background: ${error}`);
		}
	}

	async function getImageData(filename) {
		// Check cache first
		if (imageCache.has(filename)) {
			return imageCache.get(filename);
		}

		try {
			const imageData = await invoke('get_background_image_data', {
				themeName,
				filename
			});

			// Cache the result
			imageCache.set(filename, imageData);
			return imageData;
		} catch (error) {
			console.error('Failed to load image data for', filename, ':', error);
			// Return placeholder SVG on error
			return `data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="100" height="60" viewBox="0 0 100 60"><rect width="100" height="60" fill="%23f3f4f6"/><text x="50" y="35" text-anchor="middle" font-family="Arial" font-size="10" fill="%236b7280">${filename}</text></svg>`;
		}
	}
</script>

<div class="mt-4 flex flex-col gap-4">
	<div class="flex items-center justify-between">
		<p class="text-muted-foreground text-xs">Background images for your desktop</p>
		<Button
			onclick={addBackgroundImages}
			disabled={isAdding || !themeName}
			variant="outline"
			size="sm"
		>
			Add Images
		</Button>
	</div>

	{#if isLoading}
		<div class="text-sm opacity-70">Loading backgrounds...</div>
	{:else if backgrounds.length === 0}
		<Card.Root>
			<Card.Content class="flex flex-col items-center justify-center py-8">
				<ImageIcon class="mb-4 h-12 w-12 opacity-50" />
				<p class="text-center text-sm opacity-70">
					No background images added yet.<br />
					Click "Add Images" to select image files from your computer.
				</p>
			</Card.Content>
		</Card.Root>
	{:else}
		<div class="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4">
			{#each backgrounds as filename (filename)}
				<Card.Root class="overflow-hidden pt-0">
					<Card.Content class="aspect-video p-0">
						<div class="group relative flex items-center justify-center bg-gray-100">
							{#await getImageData(filename)}
								<!-- Loading state -->
								<div class="flex h-full w-full items-center justify-center bg-gray-100">
									<ImageIcon class="h-8 w-8 opacity-50" />
								</div>
							{:then imageData}
								<!-- Image loaded -->
								<img src={imageData} alt={filename} class="h-full w-full object-cover" />
							{:catch}
								<!-- Error state -->
								<div class="flex h-full w-full items-center justify-center bg-gray-100">
									<ImageIcon class="h-8 w-8 opacity-50" />
								</div>
							{/await}

							<!-- Remove button overlay -->
							<div
								class="bg-opacity-50 absolute inset-0 flex items-center justify-center bg-black opacity-0 transition-opacity group-hover:opacity-100"
							>
								<Button onclick={() => removeBackground(filename)} variant="destructive" size="sm">
									<Trash2Icon class="h-4 w-4" />
								</Button>
							</div>
						</div>
					</Card.Content>
					<CardFooter class="truncate text-xs">
						{filename}
					</CardFooter>
				</Card.Root>
			{/each}
		</div>
	{/if}
</div>
