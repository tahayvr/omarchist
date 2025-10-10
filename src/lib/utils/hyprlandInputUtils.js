import { invoke } from '@tauri-apps/api/core';

const DEFAULT_TOUCHPAD = Object.freeze({
	disable_while_typing: true,
	natural_scroll: false,
	scroll_factor: 1,
	middle_button_emulation: false,
	tap_button_map: '',
	clickfinger_behavior: false,
	tap_to_click: true,
	drag_lock: 0,
	tap_and_drag: true,
	flip_x: false,
	flip_y: false,
	drag_3fg: 0
});

const DEFAULT_FORM = Object.freeze({
	kb_model: '',
	kb_layout: 'us',
	kb_variant: '',
	kb_options: '',
	kb_rules: '',
	kb_file: '',
	numlock_by_default: false,
	resolve_binds_by_sym: false,
	repeat_rate: 25,
	repeat_delay: 600,

	// Mouse
	sensitivity: 0,
	accel_profile: '',
	force_no_accel: false,
	left_handed: false,

	// Scroll
	scroll_points: '',
	scroll_method: '',
	scroll_button: 0,
	scroll_button_lock: false,
	scroll_factor: 1,
	natural_scroll: false,

	// Focus
	follow_mouse: 1,
	follow_mouse_threshold: 0,
	focus_on_close: 0,
	mouse_refocus: true,
	float_switch_override_focus: 1,
	special_fallthrough: false,

	// Misc
	off_window_axis_events: 1,
	emulate_discrete_scroll: 1,

	// Touchpad
	touchpad: DEFAULT_TOUCHPAD
});

function normalizeString(value, fallback = '') {
	if (typeof value === 'string') {
		return value.trim();
	}

	if (value === null || value === undefined) {
		return fallback;
	}

	return String(value).trim();
}

function normalizeOptions(value) {
	if (typeof value !== 'string') {
		return '';
	}

	return value
		.split(',')
		.map((entry) => entry.trim())
		.filter(Boolean)
		.join(',');
}

function normalizeBoolean(value, fallback = false) {
	if (typeof value === 'boolean') {
		return value;
	}

	if (typeof value === 'number') {
		return value !== 0;
	}

	if (typeof value === 'string') {
		const normalized = value.trim().toLowerCase();
		if (['true', '1', 'yes', 'on'].includes(normalized)) {
			return true;
		}
		if (['false', '0', 'no', 'off'].includes(normalized)) {
			return false;
		}
	}

	return fallback;
}

function normalizeInteger(value, fallback = 0, options = {}) {
	const parsed = Number(value);
	if (!Number.isFinite(parsed)) {
		return fallback;
	}

	const integer = Math.trunc(parsed);
	const { min = Number.NEGATIVE_INFINITY, max = Number.POSITIVE_INFINITY } = options;

	if (integer < min) {
		return min;
	}

	if (integer > max) {
		return max;
	}

	return integer;
}

function normalizeFloat(value, fallback = 0, options = {}) {
	const parsed = Number(value);
	if (!Number.isFinite(parsed)) {
		return fallback;
	}

	const { min = Number.NEGATIVE_INFINITY, max = Number.POSITIVE_INFINITY, precision = 3 } = options;
	let clamped = parsed;

	if (clamped < min) {
		clamped = min;
	}

	if (clamped > max) {
		clamped = max;
	}

	const factor = 10 ** precision;
	return Math.round(clamped * factor) / factor;
}

function cloneTouchpad(source = DEFAULT_TOUCHPAD) {
	return {
		disable_while_typing: normalizeBoolean(
			source.disable_while_typing ?? DEFAULT_TOUCHPAD.disable_while_typing,
			DEFAULT_TOUCHPAD.disable_while_typing
		),
		natural_scroll: normalizeBoolean(
			source.natural_scroll ?? DEFAULT_TOUCHPAD.natural_scroll,
			DEFAULT_TOUCHPAD.natural_scroll
		),
		scroll_factor: normalizeFloat(
			source.scroll_factor ?? DEFAULT_TOUCHPAD.scroll_factor,
			DEFAULT_TOUCHPAD.scroll_factor,
			{ min: 0, precision: 3 }
		),
		middle_button_emulation: normalizeBoolean(
			source.middle_button_emulation ?? DEFAULT_TOUCHPAD.middle_button_emulation,
			DEFAULT_TOUCHPAD.middle_button_emulation
		),
		tap_button_map: normalizeString(
			source.tap_button_map ?? DEFAULT_TOUCHPAD.tap_button_map,
			DEFAULT_TOUCHPAD.tap_button_map
		),
		clickfinger_behavior: normalizeBoolean(
			source.clickfinger_behavior ?? DEFAULT_TOUCHPAD.clickfinger_behavior,
			DEFAULT_TOUCHPAD.clickfinger_behavior
		),
		tap_to_click: normalizeBoolean(
			source.tap_to_click ?? DEFAULT_TOUCHPAD.tap_to_click,
			DEFAULT_TOUCHPAD.tap_to_click
		),
		drag_lock: normalizeInteger(
			source.drag_lock ?? DEFAULT_TOUCHPAD.drag_lock,
			DEFAULT_TOUCHPAD.drag_lock,
			{ min: 0, max: 2 }
		),
		tap_and_drag: normalizeBoolean(
			source.tap_and_drag ?? DEFAULT_TOUCHPAD.tap_and_drag,
			DEFAULT_TOUCHPAD.tap_and_drag
		),
		flip_x: normalizeBoolean(source.flip_x ?? DEFAULT_TOUCHPAD.flip_x, DEFAULT_TOUCHPAD.flip_x),
		flip_y: normalizeBoolean(source.flip_y ?? DEFAULT_TOUCHPAD.flip_y, DEFAULT_TOUCHPAD.flip_y),
		drag_3fg: normalizeInteger(
			source.drag_3fg ?? DEFAULT_TOUCHPAD.drag_3fg,
			DEFAULT_TOUCHPAD.drag_3fg,
			{ min: 0, max: 2 }
		)
	};
}

function cloneForm(source = DEFAULT_FORM) {
	return {
		kb_model: normalizeString(source.kb_model ?? DEFAULT_FORM.kb_model, ''),
		kb_layout: normalizeString(source.kb_layout ?? DEFAULT_FORM.kb_layout, 'us') || 'us',
		kb_variant: normalizeString(source.kb_variant ?? DEFAULT_FORM.kb_variant, ''),
		kb_options: normalizeOptions(source.kb_options ?? DEFAULT_FORM.kb_options),
		kb_rules: normalizeString(source.kb_rules ?? DEFAULT_FORM.kb_rules, ''),
		kb_file: normalizeString(source.kb_file ?? DEFAULT_FORM.kb_file, ''),
		numlock_by_default: normalizeBoolean(
			source.numlock_by_default ?? DEFAULT_FORM.numlock_by_default,
			DEFAULT_FORM.numlock_by_default
		),
		resolve_binds_by_sym: normalizeBoolean(
			source.resolve_binds_by_sym ?? DEFAULT_FORM.resolve_binds_by_sym,
			DEFAULT_FORM.resolve_binds_by_sym
		),
		repeat_rate: normalizeInteger(
			source.repeat_rate ?? DEFAULT_FORM.repeat_rate,
			DEFAULT_FORM.repeat_rate,
			{ min: 1, max: 100 }
		),
		repeat_delay: normalizeInteger(
			source.repeat_delay ?? DEFAULT_FORM.repeat_delay,
			DEFAULT_FORM.repeat_delay,
			{ min: 100, max: 10000 }
		),

		sensitivity: normalizeFloat(
			source.sensitivity ?? DEFAULT_FORM.sensitivity,
			DEFAULT_FORM.sensitivity,
			{ min: -1, max: 1, precision: 3 }
		),
		accel_profile: normalizeString(source.accel_profile ?? DEFAULT_FORM.accel_profile, ''),
		force_no_accel: normalizeBoolean(
			source.force_no_accel ?? DEFAULT_FORM.force_no_accel,
			DEFAULT_FORM.force_no_accel
		),
		left_handed: normalizeBoolean(
			source.left_handed ?? DEFAULT_FORM.left_handed,
			DEFAULT_FORM.left_handed
		),

		scroll_points: normalizeString(source.scroll_points ?? DEFAULT_FORM.scroll_points, ''),
		scroll_method: normalizeString(source.scroll_method ?? DEFAULT_FORM.scroll_method, ''),
		scroll_button: normalizeInteger(
			source.scroll_button ?? DEFAULT_FORM.scroll_button,
			DEFAULT_FORM.scroll_button
		),
		scroll_button_lock: normalizeBoolean(
			source.scroll_button_lock ?? DEFAULT_FORM.scroll_button_lock,
			DEFAULT_FORM.scroll_button_lock
		),
		scroll_factor: normalizeFloat(
			source.scroll_factor ?? DEFAULT_FORM.scroll_factor,
			DEFAULT_FORM.scroll_factor,
			{ min: 0, precision: 3 }
		),
		natural_scroll: normalizeBoolean(
			source.natural_scroll ?? DEFAULT_FORM.natural_scroll,
			DEFAULT_FORM.natural_scroll
		),

		follow_mouse: normalizeInteger(
			source.follow_mouse ?? DEFAULT_FORM.follow_mouse,
			DEFAULT_FORM.follow_mouse,
			{ min: 0, max: 3 }
		),
		follow_mouse_threshold: normalizeFloat(
			source.follow_mouse_threshold ?? DEFAULT_FORM.follow_mouse_threshold,
			DEFAULT_FORM.follow_mouse_threshold,
			{ min: 0, precision: 3 }
		),
		focus_on_close: normalizeInteger(
			source.focus_on_close ?? DEFAULT_FORM.focus_on_close,
			DEFAULT_FORM.focus_on_close,
			{ min: 0, max: 1 }
		),
		mouse_refocus: normalizeBoolean(
			source.mouse_refocus ?? DEFAULT_FORM.mouse_refocus,
			DEFAULT_FORM.mouse_refocus
		),
		float_switch_override_focus: normalizeInteger(
			source.float_switch_override_focus ?? DEFAULT_FORM.float_switch_override_focus,
			DEFAULT_FORM.float_switch_override_focus,
			{ min: 0, max: 2 }
		),
		special_fallthrough: normalizeBoolean(
			source.special_fallthrough ?? DEFAULT_FORM.special_fallthrough,
			DEFAULT_FORM.special_fallthrough
		),

		off_window_axis_events: normalizeInteger(
			source.off_window_axis_events ?? DEFAULT_FORM.off_window_axis_events,
			DEFAULT_FORM.off_window_axis_events,
			{ min: 0, max: 3 }
		),
		emulate_discrete_scroll: normalizeInteger(
			source.emulate_discrete_scroll ?? DEFAULT_FORM.emulate_discrete_scroll,
			DEFAULT_FORM.emulate_discrete_scroll,
			{ min: 0, max: 2 }
		),
		touchpad: cloneTouchpad(source.touchpad ?? DEFAULT_TOUCHPAD)
	};
}

function buildTouchpadPayload(formTouchpad) {
	const source = formTouchpad && typeof formTouchpad === 'object' ? formTouchpad : {};
	return {
		disable_while_typing: normalizeBoolean(
			source.disable_while_typing ?? DEFAULT_TOUCHPAD.disable_while_typing,
			DEFAULT_TOUCHPAD.disable_while_typing
		),
		natural_scroll: normalizeBoolean(
			source.natural_scroll ?? DEFAULT_TOUCHPAD.natural_scroll,
			DEFAULT_TOUCHPAD.natural_scroll
		),
		scroll_factor: normalizeFloat(
			source.scroll_factor ?? DEFAULT_TOUCHPAD.scroll_factor,
			DEFAULT_TOUCHPAD.scroll_factor,
			{ min: 0, precision: 3 }
		),
		middle_button_emulation: normalizeBoolean(
			source.middle_button_emulation ?? DEFAULT_TOUCHPAD.middle_button_emulation,
			DEFAULT_TOUCHPAD.middle_button_emulation
		),
		tap_button_map: normalizeString(
			source.tap_button_map ?? DEFAULT_TOUCHPAD.tap_button_map,
			DEFAULT_TOUCHPAD.tap_button_map
		),
		clickfinger_behavior: normalizeBoolean(
			source.clickfinger_behavior ?? DEFAULT_TOUCHPAD.clickfinger_behavior,
			DEFAULT_TOUCHPAD.clickfinger_behavior
		),
		tap_to_click: normalizeBoolean(
			source.tap_to_click ?? DEFAULT_TOUCHPAD.tap_to_click,
			DEFAULT_TOUCHPAD.tap_to_click
		),
		drag_lock: normalizeInteger(
			source.drag_lock ?? DEFAULT_TOUCHPAD.drag_lock,
			DEFAULT_TOUCHPAD.drag_lock,
			{ min: 0, max: 2 }
		),
		tap_and_drag: normalizeBoolean(
			source.tap_and_drag ?? DEFAULT_TOUCHPAD.tap_and_drag,
			DEFAULT_TOUCHPAD.tap_and_drag
		),
		flip_x: normalizeBoolean(source.flip_x ?? DEFAULT_TOUCHPAD.flip_x, DEFAULT_TOUCHPAD.flip_x),
		flip_y: normalizeBoolean(source.flip_y ?? DEFAULT_TOUCHPAD.flip_y, DEFAULT_TOUCHPAD.flip_y),
		drag_3fg: normalizeInteger(
			source.drag_3fg ?? DEFAULT_TOUCHPAD.drag_3fg,
			DEFAULT_TOUCHPAD.drag_3fg,
			{ min: 0, max: 2 }
		)
	};
}

function parseOptionList(options) {
	if (!options) {
		return [];
	}

	return options
		.split(',')
		.map((entry) => entry.trim())
		.filter(Boolean);
}

function compareCatalogEntries(a, b) {
	const labelA = normalizeString(a?.description ?? a?.name ?? '');
	const labelB = normalizeString(b?.description ?? b?.name ?? '');
	const labelComparison = labelA.localeCompare(labelB, undefined, {
		numeric: true,
		sensitivity: 'base'
	});

	if (labelComparison !== 0) {
		return labelComparison;
	}

	const nameA = normalizeString(a?.name ?? '');
	const nameB = normalizeString(b?.name ?? '');
	return nameA.localeCompare(nameB, undefined, { numeric: true, sensitivity: 'base' });
}

export function prepareCatalog(rawCatalog) {
	const modelMap = new Map();
	if (Array.isArray(rawCatalog?.models)) {
		for (const entry of rawCatalog.models) {
			const name = normalizeString(entry?.name ?? '');
			if (!name) continue;
			const rawDescription = normalizeString(entry?.description ?? '');
			const description = rawDescription || normalizeString(entry?.name ?? '');
			if (!modelMap.has(name)) {
				modelMap.set(name, { name, description });
			}
		}
	}

	const layoutMap = new Map();
	if (Array.isArray(rawCatalog?.layouts)) {
		for (const entry of rawCatalog.layouts) {
			const name = normalizeString(entry?.name ?? '');
			if (!name) continue;
			const rawDescription = normalizeString(entry?.description ?? '');
			const description = rawDescription || normalizeString(entry?.name ?? '');
			const variantMap = new Map();

			if (Array.isArray(entry?.variants)) {
				for (const variantEntry of entry.variants) {
					const variantName = normalizeString(variantEntry?.name ?? '');
					if (!variantName) continue;
					const rawVariantDescription = normalizeString(variantEntry?.description ?? '');
					const variantDescription =
						rawVariantDescription || normalizeString(variantEntry?.name ?? '');
					if (!variantMap.has(variantName)) {
						variantMap.set(variantName, {
							name: variantName,
							description: variantDescription
						});
					}
				}
			}

			const existing = layoutMap.get(name);
			if (existing) {
				if (!existing.description && description) {
					existing.description = description;
				}
				for (const [variantName, variantValue] of variantMap) {
					if (!existing.variantMap.has(variantName)) {
						existing.variantMap.set(variantName, variantValue);
					}
				}
			} else {
				layoutMap.set(name, {
					name,
					description,
					variantMap
				});
			}
		}
	}

	const optionGroupMap = new Map();
	if (Array.isArray(rawCatalog?.option_groups)) {
		for (const groupEntry of rawCatalog.option_groups) {
			const groupName = normalizeString(groupEntry?.name ?? '');
			if (!groupName) continue;
			const rawGroupDescription = normalizeString(groupEntry?.description ?? '');
			const groupDescription = rawGroupDescription || normalizeString(groupEntry?.name ?? '');
			const optionMap = new Map();

			if (Array.isArray(groupEntry?.options)) {
				for (const optionEntry of groupEntry.options) {
					const optionName = normalizeString(optionEntry?.name ?? '');
					if (!optionName) continue;
					const rawOptionDescription = normalizeString(optionEntry?.description ?? '');
					const optionDescription =
						rawOptionDescription || normalizeString(optionEntry?.name ?? '');
					if (!optionMap.has(optionName)) {
						optionMap.set(optionName, {
							name: optionName,
							description: optionDescription
						});
					}
				}
			}

			const existing = optionGroupMap.get(groupName);
			if (existing) {
				if (!existing.description && groupDescription) {
					existing.description = groupDescription;
				}
				for (const [optionName, optionValue] of optionMap) {
					if (!existing.optionMap.has(optionName)) {
						existing.optionMap.set(optionName, optionValue);
					}
				}
			} else {
				optionGroupMap.set(groupName, {
					name: groupName,
					description: groupDescription,
					optionMap
				});
			}
		}
	}

	const models = Array.from(modelMap.values()).sort(compareCatalogEntries);

	const layouts = Array.from(layoutMap.values())
		.map((layout) => {
			const variants = Array.from(layout.variantMap.values()).sort(compareCatalogEntries);
			return {
				name: layout.name,
				description: layout.description,
				variants
			};
		})
		.sort(compareCatalogEntries);

	const variantsByLayout = {};
	for (const layout of layouts) {
		variantsByLayout[layout.name] = layout.variants.map((variant) => ({ ...variant }));
	}

	const optionGroups = Array.from(optionGroupMap.values())
		.map((group) => {
			const options = Array.from(group.optionMap.values()).sort(compareCatalogEntries);
			return {
				name: group.name,
				description: group.description,
				options
			};
		})
		.sort(compareCatalogEntries);

	const optionCodes = new Set();
	for (const group of optionGroups) {
		for (const option of group.options) {
			optionCodes.add(`${group.name}:${option.name}`);
		}
	}

	return {
		models,
		layouts,
		optionGroups,
		variantsByLayout,
		optionCodes
	};
}

function alignFormWithCatalog(state) {
	if (!state.catalog) {
		return;
	}

	const modelNames = new Set((state.catalog.models ?? []).map((model) => model.name));
	if (state.form.kb_model && !modelNames.has(state.form.kb_model)) {
		state.form.kb_model = '';
	}

	const { layouts = [], variantsByLayout = {} } = state.catalog;
	const availableLayouts = new Set(layouts.map((layout) => layout.name));
	if (!availableLayouts.has(state.form.kb_layout)) {
		state.form.kb_layout = layouts[0]?.name ?? 'us';
	}

	const variants = variantsByLayout[state.form.kb_layout] ?? [];
	if (state.form.kb_variant) {
		const variantExists = variants.some((variant) => variant.name === state.form.kb_variant);
		if (!variantExists) {
			state.form.kb_variant = '';
		}
	}

	const variantSet = new Set(variants.map((variant) => variant.name));
	const lastSavedVariants = variantsByLayout[state.lastSavedForm?.kb_layout ?? ''] ?? [];
	const lastSavedSet = new Set(lastSavedVariants.map((variant) => variant.name));

	if (state.lastSavedForm) {
		if (state.lastSavedForm.kb_model && !modelNames.has(state.lastSavedForm.kb_model)) {
			state.lastSavedForm.kb_model = '';
		}
		if (!availableLayouts.has(state.lastSavedForm.kb_layout)) {
			state.lastSavedForm.kb_layout = state.form.kb_layout;
		}
		if (state.lastSavedForm.kb_variant && !lastSavedSet.has(state.lastSavedForm.kb_variant)) {
			state.lastSavedForm.kb_variant = '';
		}
	}

	if (state.form.kb_variant && !variantSet.has(state.form.kb_variant)) {
		state.form.kb_variant = '';
	}
}

function validateHyprlandInputFormInternal(form, catalog) {
	const fieldErrors = {};
	const layoutName = normalizeString(form.kb_layout, 'us') || 'us';
	const variantName = normalizeString(form.kb_variant, '');
	const optionList = parseOptionList(form.kb_options);

	if (!layoutName) {
		fieldErrors.kb_layout = 'Layout is required.';
	} else if (catalog?.layouts?.length) {
		const layoutExists = catalog.layouts.some((layout) => layout.name === layoutName);
		if (!layoutExists) {
			fieldErrors.kb_layout = 'Layout is not available on this system.';
		}
	}

	if (variantName) {
		const variants = catalog?.variantsByLayout?.[layoutName] ?? [];
		const variantExists = variants.some((variant) => variant.name === variantName);
		if (!variantExists) {
			fieldErrors.kb_variant = 'Variant is not available for the selected layout.';
		}
	}

	if (optionList.length) {
		const invalidOptions = optionList.filter((option) => {
			if (!catalog?.optionCodes?.size) {
				return false;
			}
			return !catalog.optionCodes.has(option);
		});

		if (invalidOptions.length) {
			fieldErrors.kb_options = `Unknown options: ${invalidOptions.join(', ')}`;
		}
	}

	const repeatRate = Number(form.repeat_rate);
	if (!Number.isFinite(repeatRate) || !Number.isInteger(repeatRate)) {
		fieldErrors.repeat_rate = 'Repeat rate must be a whole number between 1 and 100 repeats/sec.';
	} else if (repeatRate < 1 || repeatRate > 100) {
		fieldErrors.repeat_rate = 'Repeat rate must be between 1 and 100 repeats/sec.';
	}

	const repeatDelay = Number(form.repeat_delay);
	if (!Number.isFinite(repeatDelay) || !Number.isInteger(repeatDelay)) {
		fieldErrors.repeat_delay =
			'Repeat delay must be a whole number between 100 and 10000 milliseconds.';
	} else if (repeatDelay < 100 || repeatDelay > 10000) {
		fieldErrors.repeat_delay = 'Repeat delay must be between 100 and 10000 milliseconds.';
	}

	const sensitivity = Number(form.sensitivity);
	if (!Number.isFinite(sensitivity)) {
		fieldErrors.sensitivity = 'Pointer sensitivity must be a number between -1 and 1.';
	} else if (sensitivity < -1 || sensitivity > 1) {
		fieldErrors.sensitivity = 'Pointer sensitivity must stay between -1 and 1.';
	}

	if (
		form.scroll_factor !== '' &&
		form.scroll_factor !== null &&
		form.scroll_factor !== undefined
	) {
		const scrollFactor = Number(form.scroll_factor);
		if (!Number.isFinite(scrollFactor)) {
			fieldErrors.scroll_factor = 'Scroll factor must be a numeric value.';
		} else if (scrollFactor < 0) {
			fieldErrors.scroll_factor = 'Scroll factor cannot be negative.';
		}
	}

	if (
		form.scroll_button !== '' &&
		form.scroll_button !== null &&
		form.scroll_button !== undefined
	) {
		const scrollButton = Number(form.scroll_button);
		if (!Number.isFinite(scrollButton) || !Number.isInteger(scrollButton)) {
			fieldErrors.scroll_button = 'Scroll button must be an integer value (0 to disable).';
		}
	}

	const followMouse = Number(form.follow_mouse);
	if (!Number.isFinite(followMouse) || !Number.isInteger(followMouse)) {
		fieldErrors.follow_mouse = 'Follow mouse must be a whole number between 0 and 3.';
	} else if (followMouse < 0 || followMouse > 3) {
		fieldErrors.follow_mouse = 'Follow mouse levels range from 0 to 3.';
	}

	const followMouseThreshold = Number(form.follow_mouse_threshold);
	if (!Number.isFinite(followMouseThreshold)) {
		fieldErrors.follow_mouse_threshold =
			'Follow mouse threshold must be a numeric value (milliseconds).';
	} else if (followMouseThreshold < 0) {
		fieldErrors.follow_mouse_threshold = 'Follow mouse threshold cannot be negative.';
	}

	const focusOnClose = Number(form.focus_on_close);
	if (!Number.isFinite(focusOnClose) || !Number.isInteger(focusOnClose)) {
		fieldErrors.focus_on_close = 'Focus on close accepts only 0 or 1.';
	} else if (focusOnClose < 0 || focusOnClose > 1) {
		fieldErrors.focus_on_close = 'Focus on close accepts only 0 or 1.';
	}

	const floatSwitchOverrideFocus = Number(form.float_switch_override_focus);
	if (!Number.isFinite(floatSwitchOverrideFocus) || !Number.isInteger(floatSwitchOverrideFocus)) {
		fieldErrors.float_switch_override_focus = 'Float switch override focus accepts values 0-2.';
	} else if (floatSwitchOverrideFocus < 0 || floatSwitchOverrideFocus > 2) {
		fieldErrors.float_switch_override_focus = 'Float switch override focus accepts values 0-2.';
	}

	const offWindowAxisEvents = Number(form.off_window_axis_events);
	if (!Number.isFinite(offWindowAxisEvents) || !Number.isInteger(offWindowAxisEvents)) {
		fieldErrors.off_window_axis_events = 'Off-window axis events accept values 0-3.';
	} else if (offWindowAxisEvents < 0 || offWindowAxisEvents > 3) {
		fieldErrors.off_window_axis_events = 'Off-window axis events accept values 0-3.';
	}

	const emulateDiscreteScroll = Number(form.emulate_discrete_scroll);
	if (!Number.isFinite(emulateDiscreteScroll) || !Number.isInteger(emulateDiscreteScroll)) {
		fieldErrors.emulate_discrete_scroll = 'Emulate discrete scroll accepts values 0-2.';
	} else if (emulateDiscreteScroll < 0 || emulateDiscreteScroll > 2) {
		fieldErrors.emulate_discrete_scroll = 'Emulate discrete scroll accepts values 0-2.';
	}

	const touchpad = form?.touchpad && typeof form.touchpad === 'object' ? form.touchpad : {};
	const touchpadScrollFactor = Number(touchpad.scroll_factor);
	if (
		touchpad.scroll_factor !== '' &&
		touchpad.scroll_factor !== null &&
		touchpad.scroll_factor !== undefined
	) {
		if (!Number.isFinite(touchpadScrollFactor)) {
			fieldErrors['touchpad.scroll_factor'] = 'Touchpad scroll factor must be a numeric value.';
		} else if (touchpadScrollFactor < 0) {
			fieldErrors['touchpad.scroll_factor'] = 'Touchpad scroll factor cannot be negative.';
		}
	}

	const tapButtonMap = normalizeString(touchpad.tap_button_map ?? '', '');
	if (tapButtonMap && !['lrm', 'lmr'].includes(tapButtonMap)) {
		fieldErrors['touchpad.tap_button_map'] = 'Tap button map must be either lrm or lmr.';
	}

	if (
		touchpad.drag_lock !== '' &&
		touchpad.drag_lock !== null &&
		touchpad.drag_lock !== undefined
	) {
		const dragLock = Number(touchpad.drag_lock);
		if (!Number.isFinite(dragLock) || !Number.isInteger(dragLock)) {
			fieldErrors['touchpad.drag_lock'] = 'Drag lock must be an integer between 0 and 2.';
		} else if (dragLock < 0 || dragLock > 2) {
			fieldErrors['touchpad.drag_lock'] = 'Drag lock must be an integer between 0 and 2.';
		}
	}

	if (touchpad.drag_3fg !== '' && touchpad.drag_3fg !== null && touchpad.drag_3fg !== undefined) {
		const drag3fg = Number(touchpad.drag_3fg);
		if (!Number.isFinite(drag3fg) || !Number.isInteger(drag3fg)) {
			fieldErrors['touchpad.drag_3fg'] = '3-finger drag must be an integer between 0 and 2.';
		} else if (drag3fg < 0 || drag3fg > 2) {
			fieldErrors['touchpad.drag_3fg'] = '3-finger drag must be an integer between 0 and 2.';
		}
	}

	return {
		isValid: Object.keys(fieldErrors).length === 0,
		fieldErrors
	};
}

export function initializeHyprlandInputState() {
	return {
		form: cloneForm(),
		effective: cloneForm(),
		overrides: cloneForm(),
		snapshot: null,
		lastSavedForm: cloneForm(),
		isLoading: false,
		isSaving: false,
		error: null,
		success: null,
		dirty: false,
		autoSaveHandle: null,
		hasHydrated: false,
		validation: { isValid: true, fieldErrors: {} },
		catalog: prepareCatalog({}),
		catalogError: null
	};
}

function applySnapshotToState(state, snapshot) {
	const effective = cloneForm(snapshot?.effective ?? DEFAULT_FORM);
	const overrides = cloneForm(snapshot?.overrides ?? DEFAULT_FORM);

	state.snapshot = snapshot ?? null;
	state.effective = effective;
	state.overrides = overrides;
	state.form = cloneForm(effective);
	state.lastSavedForm = cloneForm(effective);
	state.hasHydrated = true;
	state.dirty = false;
	state.validation = validateHyprlandInputFormInternal(state.form, state.catalog);
}

export function validateHyprlandInputForm(form, catalog) {
	return validateHyprlandInputFormInternal(form, catalog);
}

function buildPayloadFromState(state) {
	const repeatRate = Number(state.form.repeat_rate);
	const repeatDelay = Number(state.form.repeat_delay);
	const safeRepeatRate = Number.isFinite(repeatRate)
		? Math.trunc(repeatRate)
		: DEFAULT_FORM.repeat_rate;
	const safeRepeatDelay = Number.isFinite(repeatDelay)
		? Math.trunc(repeatDelay)
		: DEFAULT_FORM.repeat_delay;

	const safeSensitivity = normalizeFloat(state.form.sensitivity, DEFAULT_FORM.sensitivity, {
		min: -1,
		max: 1,
		precision: 3
	});
	const safeScrollFactor = normalizeFloat(state.form.scroll_factor, DEFAULT_FORM.scroll_factor, {
		min: 0,
		precision: 3
	});
	const safeFollowMouseThreshold = normalizeFloat(
		state.form.follow_mouse_threshold,
		DEFAULT_FORM.follow_mouse_threshold,
		{ min: 0, precision: 3 }
	);

	const safeFollowMouse = normalizeInteger(state.form.follow_mouse, DEFAULT_FORM.follow_mouse, {
		min: 0,
		max: 3
	});
	const safeFocusOnClose = normalizeInteger(
		state.form.focus_on_close,
		DEFAULT_FORM.focus_on_close,
		{ min: 0, max: 1 }
	);
	const safeFloatSwitchOverrideFocus = normalizeInteger(
		state.form.float_switch_override_focus,
		DEFAULT_FORM.float_switch_override_focus,
		{ min: 0, max: 2 }
	);
	const safeOffWindowAxisEvents = normalizeInteger(
		state.form.off_window_axis_events,
		DEFAULT_FORM.off_window_axis_events,
		{ min: 0, max: 3 }
	);
	const safeEmulateDiscreteScroll = normalizeInteger(
		state.form.emulate_discrete_scroll,
		DEFAULT_FORM.emulate_discrete_scroll,
		{ min: 0, max: 2 }
	);
	const safeTouchpad = buildTouchpadPayload(state.form.touchpad);

	return {
		overrides: {
			kb_model: state.form.kb_model,
			kb_layout: state.form.kb_layout,
			kb_variant: state.form.kb_variant,
			kb_options: state.form.kb_options,
			kb_rules: state.form.kb_rules,
			kb_file: state.form.kb_file,
			numlock_by_default: state.form.numlock_by_default,
			resolve_binds_by_sym: state.form.resolve_binds_by_sym,
			repeat_rate: safeRepeatRate,
			repeat_delay: safeRepeatDelay,

			sensitivity: safeSensitivity,
			accel_profile: state.form.accel_profile,
			force_no_accel: state.form.force_no_accel,
			left_handed: state.form.left_handed,

			scroll_points: state.form.scroll_points,
			scroll_method: state.form.scroll_method,
			scroll_button: normalizeInteger(state.form.scroll_button, DEFAULT_FORM.scroll_button),
			scroll_button_lock: state.form.scroll_button_lock,
			scroll_factor: safeScrollFactor,
			natural_scroll: state.form.natural_scroll,

			follow_mouse: safeFollowMouse,
			follow_mouse_threshold: safeFollowMouseThreshold,
			focus_on_close: safeFocusOnClose,
			mouse_refocus: state.form.mouse_refocus,
			float_switch_override_focus: safeFloatSwitchOverrideFocus,
			special_fallthrough: state.form.special_fallthrough,

			off_window_axis_events: safeOffWindowAxisEvents,
			emulate_discrete_scroll: safeEmulateDiscreteScroll,
			touchpad: safeTouchpad
		}
	};
}

export async function loadHyprlandInput(state) {
	state.isLoading = true;
	state.error = null;
	state.catalogError = null;

	try {
		const [catalogResult, snapshotResult] = await Promise.allSettled([
			invoke('get_keyboard_catalog'),
			invoke('get_hyprland_input_settings')
		]);

		if (catalogResult.status === 'fulfilled') {
			state.catalog = prepareCatalog(catalogResult.value ?? {});
		} else {
			console.error('Failed to load keyboard catalog:', catalogResult.reason);
			state.catalog = prepareCatalog({});
			state.catalogError =
				typeof catalogResult.reason === 'string'
					? catalogResult.reason
					: 'Unable to load system keyboard definitions.';
		}

		if (snapshotResult.status === 'fulfilled') {
			applySnapshotToState(state, snapshotResult.value);
			alignFormWithCatalog(state);
			state.validation = validateHyprlandInputFormInternal(state.form, state.catalog);
			state.lastSavedForm = cloneForm(state.form);
			return true;
		}

		const reason = snapshotResult.reason;
		throw typeof reason === 'string'
			? reason
			: (reason?.message ?? 'Unable to read Hyprland input overrides.');
	} catch (error) {
		console.error('Failed to load Hyprland input settings:', error);
		state.error =
			typeof error === 'string'
				? error
				: 'Unable to load Hyprland input settings. Please ensure the backend is running.';
		return false;
	} finally {
		state.isLoading = false;
	}
}

export async function saveHyprlandInput(state, options = {}) {
	if (state.isSaving) {
		return false;
	}

	const { silent = false, message } = options;
	state.validation = validateHyprlandInputFormInternal(state.form, state.catalog);
	if (!state.validation.isValid) {
		state.error = 'Cannot save until all validation errors are resolved.';
		return false;
	}

	state.isSaving = true;
	state.error = null;
	state.success = null;

	try {
		const payload = buildPayloadFromState(state);
		const snapshot = await invoke('update_hyprland_input_settings', { payload });
		applySnapshotToState(state, snapshot);
		alignFormWithCatalog(state);
		state.lastSavedForm = cloneForm(state.form);
		const resolvedMessage = message ?? 'Hyprland input settings saved successfully.';
		state.success = silent ? null : resolvedMessage;
		return true;
	} catch (error) {
		console.error('Failed to save Hyprland input settings:', error);
		state.error =
			typeof error === 'string'
				? error
				: 'Unable to save Hyprland input settings. Please try again.';
		return false;
	} finally {
		state.isSaving = false;
	}
}

export function resetHyprlandInputToDefaults(state) {
	state.form = cloneForm();
	state.validation = validateHyprlandInputFormInternal(state.form, state.catalog);
}

export function recomputeDirty(state, options = {}) {
	const current = options.currentSignature ?? JSON.stringify(state.form ?? {});
	const lastSaved = options.lastSavedSignature ?? JSON.stringify(state.lastSavedForm ?? {});
	state.dirty = current !== lastSaved;
	if (state.dirty) {
		state.success = null;
	}
	return {
		currentSignature: current,
		lastSavedSignature: lastSaved
	};
}

export function markDirty(state) {
	state.dirty = true;
	state.success = null;
}

export function getModels(state) {
	return state.catalog?.models ?? [];
}

export function getLayouts(state) {
	return state.catalog?.layouts ?? [];
}

export function getVariantsForLayout(state, layoutName) {
	return state.catalog?.variantsByLayout?.[layoutName] ?? [];
}

export function getOptionGroups(state) {
	return state.catalog?.optionGroups ?? [];
}

export function getSelectedOptionsSet(form) {
	return new Set(parseOptionList(form?.kb_options ?? ''));
}

export function toggleOptionInForm(state, optionCode) {
	const options = new Set(parseOptionList(state.form.kb_options));
	if (options.has(optionCode)) {
		options.delete(optionCode);
	} else {
		options.add(optionCode);
	}
	state.form.kb_options = Array.from(options).join(',');
}
