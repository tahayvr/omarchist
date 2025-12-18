<script>
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as ScrollArea from '$lib/components/ui/scroll-area/index.js';
	import ColorPickerWaybar from '../ColorPickerWaybar.svelte';

	let { module = null, style = {}, disabled = false, onStyleChange = () => {} } = $props();

	let open = $state(false);

	function parsePaddingMargin(value) {
		if (!value) return { top: '', right: '', bottom: '', left: '' };
		const parts = value.trim().replace(/px/g, '').split(/\s+/);
		if (parts.length === 1) {
			return { top: parts[0], right: parts[0], bottom: parts[0], left: parts[0] };
		} else if (parts.length === 2) {
			return { top: parts[0], right: parts[1], bottom: parts[0], left: parts[1] };
		} else if (parts.length === 4) {
			return { top: parts[0], right: parts[1], bottom: parts[2], left: parts[3] };
		}
		return { top: '', right: '', bottom: '', left: '' };
	}

	function buildPaddingMargin(values) {
		const { top, right, bottom, left } = values;
		if (!top && !right && !bottom && !left) return '';

		if (top === right && right === bottom && bottom === left && top) {
			return `${top}px`;
		}
		if (top === bottom && left === right && top && left) {
			return `${top}px ${left}px`;
		}
		const t = top || '0';
		const r = right || '0';
		const b = bottom || '0';
		const l = left || '0';
		return `${t}px ${r}px ${b}px ${l}px`;
	}

	let styleState = $state({
		color: '',
		background: '',
		paddingTop: '',
		paddingRight: '',
		paddingBottom: '',
		paddingLeft: '',
		marginTop: '',
		marginRight: '',
		marginBottom: '',
		marginLeft: '',
		fontSize: '',
		fontWeight: '',
		borderRadius: '',
		borderWidth: '',
		borderStyle: '',
		borderColor: '',
		minWidth: ''
	});

	$effect(() => {
		const currentStyle = {
			color: style.color,
			background: style.background,
			padding: style.padding,
			margin: style.margin,
			fontSize: style.fontSize,
			fontWeight: style.fontWeight,
			border: style.border,
			borderRadius: style.borderRadius,
			minWidth: style.minWidth
		};

		const padding = parsePaddingMargin(currentStyle.padding);
		const margin = parsePaddingMargin(currentStyle.margin);

		const borderParts = (currentStyle.border || '').split(/\s+/);
		const borderWidth = borderParts[0]?.replace('px', '') || '';
		const borderStyle = borderParts[1] || '';
		const borderColor = borderParts[2] || '';

		styleState = {
			color: currentStyle.color || '',
			background: currentStyle.background || '',
			paddingTop: padding.top,
			paddingRight: padding.right,
			paddingBottom: padding.bottom,
			paddingLeft: padding.left,
			marginTop: margin.top,
			marginRight: margin.right,
			marginBottom: margin.bottom,
			marginLeft: margin.left,
			fontSize: (currentStyle.fontSize || '').replace('px', ''),
			fontWeight: currentStyle.fontWeight || '',
			borderRadius: (currentStyle.borderRadius || '').replace('px', ''),
			borderWidth,
			borderStyle,
			borderColor,
			minWidth: (currentStyle.minWidth || '').replace('px', '')
		};
	});

	function emitStyle() {
		const cleanedStyle = {};

		if (styleState.color) cleanedStyle.color = styleState.color;
		if (styleState.background) cleanedStyle.background = styleState.background;

		const padding = buildPaddingMargin({
			top: styleState.paddingTop,
			right: styleState.paddingRight,
			bottom: styleState.paddingBottom,
			left: styleState.paddingLeft
		});
		if (padding) cleanedStyle.padding = padding;

		const margin = buildPaddingMargin({
			top: styleState.marginTop,
			right: styleState.marginRight,
			bottom: styleState.marginBottom,
			left: styleState.marginLeft
		});
		if (margin) cleanedStyle.margin = margin;

		if (styleState.fontSize) cleanedStyle.fontSize = `${styleState.fontSize}px`;
		if (styleState.fontWeight) cleanedStyle.fontWeight = styleState.fontWeight;

		if (styleState.borderWidth && styleState.borderStyle) {
			const border = `${styleState.borderWidth}px ${styleState.borderStyle}${styleState.borderColor ? ' ' + styleState.borderColor : ''}`;
			cleanedStyle.border = border.trim();
		}
		if (styleState.borderRadius) cleanedStyle.borderRadius = `${styleState.borderRadius}px`;

		if (styleState.minWidth) cleanedStyle.minWidth = `${styleState.minWidth}px`;

		onStyleChange(cleanedStyle);
	}

	const moduleTitle = $derived(module?.title ?? 'Module');
</script>

<Dialog.Root bind:open>
	<Dialog.Trigger asChild>
		<Button variant="ghost" size="sm" class="h-8 px-2 tracking-wide uppercase" {disabled}>
			<span class="ml-1">Style</span>
		</Button>
	</Dialog.Trigger>
	<Dialog.Content class="sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title class="text-sm font-semibold tracking-wide uppercase">
				{moduleTitle} Styling
			</Dialog.Title>
			<Dialog.Description class="text-muted-foreground text-xs uppercase">
				Customize the appearance of this module.
			</Dialog.Description>
		</Dialog.Header>

		<ScrollArea.Root class="h-[500px] pr-4">
			<div class="space-y-4">
				<!-- Colors -->
				<div class="space-y-3">
					<h3 class="text-accent-foreground text-sm font-semibold uppercase">Colors</h3>

					<div>
						<ColorPickerWaybar
							label="Text Color"
							color={styleState.color}
							onChange={(color) => {
								styleState.color = color;
								emitStyle();
							}}
						/>
					</div>

					<div>
						<ColorPickerWaybar
							label="Background"
							color={styleState.background}
							onChange={(color) => {
								styleState.background = color;
								emitStyle();
							}}
						/>
					</div>
				</div>

				<!-- Spacing -->
				<div class="space-y-3">
					<h3 class="text-accent-foreground text-sm font-semibold uppercase">Spacing</h3>

					<div class="space-y-2">
						<Label class="text-[0.65rem] font-semibold uppercase">Padding (px)</Label>
						<div class="grid grid-cols-4 gap-2">
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Top</Label>
								<Input
									type="number"
									placeholder="0"
									bind:value={styleState.paddingTop}
									oninput={emitStyle}
									min={0}
									{disabled}
								/>
							</div>
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Right</Label>
								<Input
									type="number"
									placeholder="0"
									bind:value={styleState.paddingRight}
									oninput={emitStyle}
									min={0}
									{disabled}
								/>
							</div>
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Bottom</Label>
								<Input
									type="number"
									placeholder="0"
									bind:value={styleState.paddingBottom}
									oninput={emitStyle}
									min={0}
									{disabled}
								/>
							</div>
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Left</Label>
								<Input
									type="number"
									placeholder="0"
									bind:value={styleState.paddingLeft}
									oninput={emitStyle}
									min={0}
									{disabled}
								/>
							</div>
						</div>
					</div>

					<div class="space-y-2">
						<Label class="text-[0.65rem] font-semibold uppercase">Margin (px)</Label>
						<div class="grid grid-cols-4 gap-2">
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Top</Label>
								<Input
									type="number"
									placeholder="0"
									bind:value={styleState.marginTop}
									oninput={emitStyle}
									min={0}
									{disabled}
								/>
							</div>
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Right</Label>
								<Input
									type="number"
									placeholder="0"
									bind:value={styleState.marginRight}
									oninput={emitStyle}
									min={0}
									{disabled}
								/>
							</div>
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Bottom</Label>
								<Input
									type="number"
									placeholder="0"
									bind:value={styleState.marginBottom}
									oninput={emitStyle}
									min={0}
									{disabled}
								/>
							</div>
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Left</Label>
								<Input
									type="number"
									placeholder="0"
									bind:value={styleState.marginLeft}
									oninput={emitStyle}
									min={0}
									{disabled}
								/>
							</div>
						</div>
					</div>

					<div class="space-y-2">
						<Label class="text-[0.65rem] font-semibold uppercase">Min Width (px)</Label>
						<Input
							type="number"
							placeholder="12"
							bind:value={styleState.minWidth}
							oninput={emitStyle}
							min={0}
							{disabled}
							class="w-32"
						/>
					</div>
				</div>

				<!-- Typography -->
				<div class="space-y-3">
					<h3 class="text-accent-foreground text-sm font-semibold uppercase">Typography</h3>

					<div class="space-y-2">
						<Label class="text-[0.65rem] font-semibold uppercase">Font Size (px)</Label>
						<Input
							type="number"
							placeholder="12"
							bind:value={styleState.fontSize}
							oninput={emitStyle}
							min={8}
							max={48}
							{disabled}
							class="w-32"
						/>
					</div>

					<div class="space-y-2">
						<Label class="text-[0.65rem] font-semibold uppercase">Font Weight</Label>
						<Select.Root
							type="single"
							bind:value={styleState.fontWeight}
							onValueChange={emitStyle}
							{disabled}
						>
							<Select.Trigger class="w-full">
								{styleState.fontWeight || 'Default'}
							</Select.Trigger>
							<Select.Content>
								<Select.Item value="" label="Default">Default</Select.Item>
								<Select.Item value="normal" label="Normal (400)">Normal (400)</Select.Item>
								<Select.Item value="500" label="Medium (500)">Medium (500)</Select.Item>
								<Select.Item value="600" label="Semi-Bold (600)">Semi-Bold (600)</Select.Item>
								<Select.Item value="bold" label="Bold (700)">Bold (700)</Select.Item>
								<Select.Item value="800" label="Extra-Bold (800)">Extra-Bold (800)</Select.Item>
							</Select.Content>
						</Select.Root>
					</div>
				</div>

				<!-- Border & Shape -->
				<div class="space-y-3">
					<h3 class="text-accent-foreground text-sm font-semibold uppercase">Border & Shape</h3>

					<div class="space-y-2">
						<Label class="text-[0.65rem] font-semibold uppercase">Border</Label>
						<div class="grid grid-cols-3 gap-2">
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Width (px)</Label>
								<Input
									type="number"
									placeholder="1"
									bind:value={styleState.borderWidth}
									oninput={emitStyle}
									min={0}
									{disabled}
								/>
							</div>
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Style</Label>
								<Select.Root
									type="single"
									bind:value={styleState.borderStyle}
									onValueChange={emitStyle}
									{disabled}
								>
									<Select.Trigger>
										{styleState.borderStyle || 'None'}
									</Select.Trigger>
									<Select.Content>
										<Select.Item value="" label="None">None</Select.Item>
										<Select.Item value="solid" label="Solid">Solid</Select.Item>
										<Select.Item value="dashed" label="Dashed">Dashed</Select.Item>
										<Select.Item value="dotted" label="Dotted">Dotted</Select.Item>
										<Select.Item value="double" label="Double">Double</Select.Item>
									</Select.Content>
								</Select.Root>
							</div>
							<div class="space-y-1">
								<Label class="text-muted-foreground text-[0.6rem]">Color</Label>
								<Input
									type="text"
									placeholder="#fff"
									bind:value={styleState.borderColor}
									oninput={emitStyle}
									{disabled}
								/>
							</div>
						</div>
					</div>

					<div class="space-y-2">
						<Label class="text-[0.65rem] font-semibold uppercase">Border Radius (px)</Label>
						<Input
							type="number"
							placeholder="0"
							bind:value={styleState.borderRadius}
							oninput={emitStyle}
							min={0}
							{disabled}
							class="w-32"
						/>
					</div>
				</div>
			</div>
		</ScrollArea.Root>

		<Dialog.Footer class="mt-4">
			<Dialog.Close
				class="border-border/70 text-accent-foreground hover:border-border focus-visible:ring-ring inline-flex items-center justify-center rounded border px-3 py-1 text-xs font-semibold tracking-wide uppercase transition focus-visible:ring-1 focus-visible:outline-none"
			>
				Close
			</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
