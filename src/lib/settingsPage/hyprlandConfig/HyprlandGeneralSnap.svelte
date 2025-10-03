<script>
	import { Label } from '$lib/components/ui/label/index.js';
	import { Switch } from '$lib/components/ui/switch/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import Explainer from '$lib/components/Explainer.svelte';

	export let hyprlandGeneral;
</script>

<section class="space-y-6 uppercase">
	<p class="text-muted-foreground -mt-3 text-xs tracking-wide uppercase">
		Configure snapping behaviour for floating windows.
	</p>

	<div class="grid gap-4 md:grid-cols-2 md:gap-x-8 md:gap-y-4">
		<div class="flex items-center justify-between gap-4">
			<Label for="enabled" class="flex-1">
				Enable snapping for floating windows
				<Explainer
					explainerText="toggle snapping support for floating windows"
					docUrl="https://wiki.hypr.land/Configuring/General/#snap"
					docLabel="Hyprland docs"
				/>
			</Label>
			<Switch
				id="enabled"
				bind:checked={hyprlandGeneral.form.snap.enabled}
				disabled={hyprlandGeneral.isLoading}
			/>
		</div>

		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between gap-4">
				<Label for="window_gap" class="flex-1">
					Window gap
					<Explainer
						explainerText="minimum gap, in pixels, between windows before snap alignment engages"
					/>
				</Label>
				<Input
					id="window_gap"
					type="number"
					class="w-24"
					bind:value={hyprlandGeneral.form.snap.window_gap}
					disabled={hyprlandGeneral.isLoading}
					min="0"
				></Input>
			</div>
		</div>
		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between gap-4">
				<Label for="monitor_gap" class="flex-1">
					Monitor gap
					<Explainer
						explainerText="minimum gap, in pixels, between a window and monitor edges before snapping"
					/>
				</Label>
				<Input
					id="monitor_gap"
					type="number"
					class="w-24"
					bind:value={hyprlandGeneral.form.snap.monitor_gap}
					disabled={hyprlandGeneral.isLoading}
					min="0"
				></Input>
			</div>
		</div>
		<div class="flex items-center justify-between gap-4">
			<Label for="border_overlap" class="flex-1">
				Border overlap
				<Explainer
					explainerText="when enabled, snapped windows overlap by one border width instead of leaving a full gap"
				/>
			</Label>
			<Switch
				id="border_overlap"
				bind:checked={hyprlandGeneral.form.snap.border_overlap}
				disabled={hyprlandGeneral.isLoading}
			/>
		</div>

		<div class="flex items-center justify-between gap-4">
			<Label for="respect_gaps" class="flex-1">
				Respect gaps
				<Explainer
					explainerText="preserve configured gaps when snapping windows next to each other"
				/>
			</Label>
			<Switch
				id="respect_gaps"
				bind:checked={hyprlandGeneral.form.snap.respect_gaps}
				disabled={hyprlandGeneral.isLoading}
			/>
		</div>
	</div>
</section>
