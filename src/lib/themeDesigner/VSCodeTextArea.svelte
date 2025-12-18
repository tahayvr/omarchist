<script>
	/* eslint-disable svelte/prefer-writable-derived */
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher();

	let { value = '', placeholder = 'Paste your VSCode theme configuration here...' } = $props();

	let textareaValue = $state(value);

	$effect(() => {
		textareaValue = value;
	});

	function handleInput(event) {
		textareaValue = event.target.value;
		dispatch('change', { value: textareaValue });
	}
</script>

<div class="mt-4 flex flex-col gap-4">
	<Textarea
		id="vscode-config"
		{placeholder}
		value={textareaValue}
		oninput={handleInput}
		class="min-h-[300px] text-sm"
		spellcheck="false"
	/>

	<div class="text-muted-foreground text-xs">
		<p><strong>Example:</strong></p>
		<pre class="bg-muted mt-1 rounded p-2 text-xs"><code
				>{`{
  "name": "Matte Black",
  "extension": "TahaYVR.matteblack"
}`}</code
			></pre>
	</div>
</div>
