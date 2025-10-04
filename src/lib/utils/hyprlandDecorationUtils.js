import { invoke } from '@tauri-apps/api/core';

const DEFAULT_BLUR = Object.freeze({
	enabled: true,
	size: 8,
	passes: 1,
	ignore_opacity: true,
	new_optimizations: true,
	xray: false,
	noise: 0.0117,
	contrast: 0.8916,
	brightness: 0.8172,
	vibrancy: 0.1696,
	vibrancy_darkness: 0,
	special: false,
	popups: false,
	popups_ignorealpha: 0.2,
	input_methods: false,
	input_methods_ignorealpha: 0.2
});

const DEFAULT_SHADOW = {
	enabled: true,
	range: 4,
	render_power: 3,
	sharp: false,
	ignore_window: true,
	color: 'rgba(00000055)',
	color_inactive: null,
	offset: '0 0',
	scale: 1.0
};

const DEFAULT_FORM = Object.freeze({
	rounding: 0,
	rounding_power: 2,
	active_opacity: 1,
	inactive_opacity: 1,
	fullscreen_opacity: 1,
	dim_modal: true,
	dim_inactive: false,
	dim_strength: 0.5,
	dim_special: 0.2,
	dim_around: 0.4,
	screen_shader: '',
	border_part_of_window: true,
	blur: DEFAULT_BLUR,
	shadow: DEFAULT_SHADOW
});

function normalizeBoolean(value, fallback = false) {
	if (typeof value === 'boolean') {
		return value;
	}
	if (typeof value === 'number') {
		return value !== 0;
	}
	return fallback;
}

function normalizeNumber(value, fallback, options = {}) {
	const {
		integer = false,
		min = Number.NEGATIVE_INFINITY,
		max = Number.POSITIVE_INFINITY
	} = options;
	const parsed = Number(value);
	if (!Number.isFinite(parsed)) {
		return fallback;
	}
	const coerced = integer ? Math.trunc(parsed) : parsed;
	if (coerced < min || coerced > max) {
		return fallback;
	}
	return coerced;
}

function normalizeString(value, fallback = '') {
	if (typeof value === 'string') {
		const trimmed = value.trim();
		return trimmed.length ? trimmed : fallback;
	}
	if (value === null || value === undefined) {
		return fallback;
	}
	const stringified = String(value).trim();
	return stringified.length ? stringified : fallback;
}

function cloneBlur(snapshot = DEFAULT_BLUR) {
	return {
		enabled: normalizeBoolean(snapshot.enabled, DEFAULT_BLUR.enabled),
		size: normalizeNumber(snapshot.size, DEFAULT_BLUR.size, { integer: true, min: 1 }),
		passes: normalizeNumber(snapshot.passes, DEFAULT_BLUR.passes, { integer: true, min: 1 }),
		ignore_opacity: normalizeBoolean(snapshot.ignore_opacity, DEFAULT_BLUR.ignore_opacity),
		new_optimizations: normalizeBoolean(snapshot.new_optimizations, DEFAULT_BLUR.new_optimizations),
		xray: normalizeBoolean(snapshot.xray, DEFAULT_BLUR.xray),
		noise: normalizeNumber(snapshot.noise, DEFAULT_BLUR.noise, { min: 0, max: 1 }),
		contrast: normalizeNumber(snapshot.contrast, DEFAULT_BLUR.contrast, { min: 0, max: 2 }),
		brightness: normalizeNumber(snapshot.brightness, DEFAULT_BLUR.brightness, { min: 0, max: 2 }),
		vibrancy: normalizeNumber(snapshot.vibrancy, DEFAULT_BLUR.vibrancy, { min: 0, max: 1 }),
		vibrancy_darkness: normalizeNumber(snapshot.vibrancy_darkness, DEFAULT_BLUR.vibrancy_darkness, {
			min: 0,
			max: 1
		}),
		special: normalizeBoolean(snapshot.special, DEFAULT_BLUR.special),
		popups: normalizeBoolean(snapshot.popups, DEFAULT_BLUR.popups),
		popups_ignorealpha: normalizeNumber(
			snapshot.popups_ignorealpha,
			DEFAULT_BLUR.popups_ignorealpha,
			{ min: 0, max: 1 }
		),
		input_methods: normalizeBoolean(snapshot.input_methods, DEFAULT_BLUR.input_methods),
		input_methods_ignorealpha: normalizeNumber(
			snapshot.input_methods_ignorealpha,
			DEFAULT_BLUR.input_methods_ignorealpha,
			{ min: 0, max: 1 }
		)
	};
}

function cloneShadow(snapshot = DEFAULT_SHADOW) {
	return {
		enabled: normalizeBoolean(snapshot.enabled, DEFAULT_SHADOW.enabled),
		range: normalizeNumber(snapshot.range, DEFAULT_SHADOW.range, { integer: true, min: 0 }),
		render_power: normalizeNumber(snapshot.render_power, DEFAULT_SHADOW.render_power, {
			integer: true,
			min: 1,
			max: 4
		}),
		sharp: normalizeBoolean(snapshot.sharp, DEFAULT_SHADOW.sharp),
		ignore_window: normalizeBoolean(snapshot.ignore_window, DEFAULT_SHADOW.ignore_window),
		color: normalizeString(snapshot.color, DEFAULT_SHADOW.color),
		color_inactive:
			snapshot.color_inactive === null || snapshot.color_inactive === undefined
				? null
				: normalizeString(snapshot.color_inactive, null),
		offset: normalizeString(snapshot.offset, DEFAULT_SHADOW.offset),
		scale: normalizeNumber(snapshot.scale, DEFAULT_SHADOW.scale, { min: 0, max: 1 })
	};
}

function cloneForm(source = DEFAULT_FORM) {
	return {
		rounding: normalizeNumber(source.rounding, DEFAULT_FORM.rounding, { integer: true, min: 0 }),
		rounding_power: normalizeNumber(source.rounding_power, DEFAULT_FORM.rounding_power, {
			min: 1,
			max: 10
		}),
		active_opacity: normalizeNumber(source.active_opacity, DEFAULT_FORM.active_opacity, {
			min: 0,
			max: 1
		}),
		inactive_opacity: normalizeNumber(source.inactive_opacity, DEFAULT_FORM.inactive_opacity, {
			min: 0,
			max: 1
		}),
		fullscreen_opacity: normalizeNumber(
			source.fullscreen_opacity,
			DEFAULT_FORM.fullscreen_opacity,
			{ min: 0, max: 1 }
		),
		dim_modal: normalizeBoolean(source.dim_modal, DEFAULT_FORM.dim_modal),
		dim_inactive: normalizeBoolean(source.dim_inactive, DEFAULT_FORM.dim_inactive),
		dim_strength: normalizeNumber(source.dim_strength, DEFAULT_FORM.dim_strength, {
			min: 0,
			max: 1
		}),
		dim_special: normalizeNumber(source.dim_special, DEFAULT_FORM.dim_special, {
			min: 0,
			max: 1
		}),
		dim_around: normalizeNumber(source.dim_around, DEFAULT_FORM.dim_around, {
			min: 0,
			max: 1
		}),
		screen_shader: normalizeString(source.screen_shader, DEFAULT_FORM.screen_shader),
		border_part_of_window: normalizeBoolean(
			source.border_part_of_window,
			DEFAULT_FORM.border_part_of_window
		),
		blur: cloneBlur(source.blur ?? DEFAULT_BLUR),
		shadow: cloneShadow(source.shadow ?? DEFAULT_SHADOW)
	};
}

function applySnapshotToState(state, snapshot) {
	const effective = snapshot?.effective ?? {};
	const overrides = snapshot?.overrides ?? {};

	state.snapshot = snapshot ?? null;
	state.effective = cloneForm({
		...effective,
		blur: effective.blur ?? DEFAULT_BLUR,
		shadow: effective.shadow ?? DEFAULT_SHADOW
	});
	state.overrides = cloneForm({
		...overrides,
		blur: overrides.blur ?? DEFAULT_BLUR,
		shadow: overrides.shadow ?? DEFAULT_SHADOW
	});
	state.hasHydrated = true;

	state.form = cloneForm(state.effective);
	state.lastSavedForm = cloneForm(state.form);
	state.dirty = false;
	state.validation = validateHyprlandDecorationForm(state.form);
}

export function initializeHyprlandDecorationState() {
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
		validation: validateHyprlandDecorationForm(DEFAULT_FORM),
		autoSaveHandle: null,
		hasHydrated: false
	};
}

export function markDirty(state) {
	state.dirty = true;
	state.success = null;
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

export function validateHyprlandDecorationForm(form) {
	const fieldErrors = {};

	const requireBoolean = (object, field, keyOverride) => {
		if (typeof object[field] !== 'boolean') {
			fieldErrors[keyOverride ?? field] = 'Must be true or false.';
		}
	};

	const requireNumberInRange = (object, field, min, max, options = {}) => {
		const raw = object[field];
		const value = Number(raw);
		const { integer = false } = options;
		const key = options.keyOverride ?? field;

		if (!Number.isFinite(value)) {
			fieldErrors[key] = 'Value must be a number.';
			return;
		}

		if (integer && !Number.isInteger(value)) {
			fieldErrors[key] = 'Value must be an integer.';
			return;
		}

		if (value < min || value > max) {
			let message;
			if (min !== Number.NEGATIVE_INFINITY && max !== Number.POSITIVE_INFINITY) {
				message = `Value must be between ${min} and ${max}.`;
			} else if (min !== Number.NEGATIVE_INFINITY) {
				message = `Value must be greater than or equal to ${min}.`;
			} else if (max !== Number.POSITIVE_INFINITY) {
				message = `Value must be less than or equal to ${max}.`;
			} else {
				message = 'Value must be within the allowed range.';
			}
			fieldErrors[key] = message;
		}
	};

	const requireNonNegativeInteger = (object, field, keyOverride) => {
		const raw = object[field];
		const value = Number(raw);
		const key = keyOverride ?? field;
		if (!Number.isFinite(value) || value < 0 || !Number.isInteger(value)) {
			fieldErrors[key] = 'Value must be a non-negative integer.';
		}
	};

	requireNonNegativeInteger(form, 'rounding');
	requireNumberInRange(form, 'rounding_power', 1, 10, { keyOverride: 'rounding_power' });
	requireNumberInRange(form, 'active_opacity', 0, 1);
	requireNumberInRange(form, 'inactive_opacity', 0, 1, {
		keyOverride: 'inactive_opacity'
	});
	requireNumberInRange(form, 'fullscreen_opacity', 0, 1, {
		keyOverride: 'fullscreen_opacity'
	});
	requireBoolean(form, 'dim_modal');
	requireBoolean(form, 'dim_inactive');
	requireNumberInRange(form, 'dim_strength', 0, 1, { keyOverride: 'dim_strength' });
	requireNumberInRange(form, 'dim_special', 0, 1, { keyOverride: 'dim_special' });
	requireNumberInRange(form, 'dim_around', 0, 1, { keyOverride: 'dim_around' });
	requireBoolean(form, 'border_part_of_window');

	const screenShaderValid = typeof form.screen_shader === 'string';
	if (!screenShaderValid) {
		fieldErrors.screen_shader = 'Value must be text.';
	}

	const ensureBlurBoolean = (field) => requireBoolean(form.blur, field, `blur.${field}`);
	const ensureBlurFloat = (field, min, max) =>
		requireNumberInRange(form.blur, field, min, max, { keyOverride: `blur.${field}` });
	const ensureBlurInteger = (field, min = 1) =>
		requireNumberInRange(form.blur, field, min, Number.POSITIVE_INFINITY, {
			integer: true,
			keyOverride: `blur.${field}`
		});

	ensureBlurBoolean('enabled');
	ensureBlurInteger('size');
	ensureBlurInteger('passes');
	ensureBlurBoolean('ignore_opacity');
	ensureBlurBoolean('new_optimizations');
	ensureBlurBoolean('xray');
	ensureBlurFloat('noise', 0, 1);
	ensureBlurFloat('contrast', 0, 2);
	ensureBlurFloat('brightness', 0, 2);
	ensureBlurFloat('vibrancy', 0, 1);
	ensureBlurFloat('vibrancy_darkness', 0, 1);
	ensureBlurBoolean('special');
	ensureBlurBoolean('popups');
	ensureBlurFloat('popups_ignorealpha', 0, 1);
	ensureBlurBoolean('input_methods');
	ensureBlurFloat('input_methods_ignorealpha', 0, 1);

	const ensureShadowBoolean = (field) => requireBoolean(form.shadow, field, `shadow.${field}`);
	const ensureShadowInteger = (field, min, max, options = {}) =>
		requireNumberInRange(form.shadow, field, min, max, {
			integer: true,
			keyOverride: `shadow.${field}`,
			...options
		});
	const ensureShadowFloat = (field, min, max) =>
		requireNumberInRange(form.shadow, field, min, max, { keyOverride: `shadow.${field}` });

	ensureShadowBoolean('enabled');
	ensureShadowInteger('range', 0, Number.POSITIVE_INFINITY);
	ensureShadowInteger('render_power', 1, 4);
	ensureShadowBoolean('sharp');
	ensureShadowBoolean('ignore_window');

	// Validate required string fields (color and offset)
	['color', 'offset'].forEach((field) => {
		const value = form.shadow?.[field];
		if (typeof value !== 'string' || value.trim() === '') {
			fieldErrors[`shadow.${field}`] = 'Value is required.';
		}
	});

	// color_inactive is optional - only validate if provided
	if (form.shadow?.color_inactive !== null && form.shadow?.color_inactive !== undefined) {
		const value = form.shadow.color_inactive;
		if (typeof value === 'string' && value.trim() !== '') {
			// Valid non-empty string, no error
		} else if (typeof value === 'string' && value.trim() === '') {
			// Empty string should be treated as null (unset)
		} else {
			fieldErrors['shadow.color_inactive'] = 'Value must be a valid color string or left empty.';
		}
	}

	ensureShadowFloat('scale', 0, 1);

	return {
		isValid: Object.keys(fieldErrors).length === 0,
		fieldErrors
	};
}

function buildPayloadFromState(state) {
	const shadowPayload = {
		enabled: state.form.shadow.enabled,
		range: Number(state.form.shadow.range),
		render_power: Number(state.form.shadow.render_power),
		sharp: state.form.shadow.sharp,
		ignore_window: state.form.shadow.ignore_window,
		color: state.form.shadow.color?.trim() ?? DEFAULT_SHADOW.color,
		offset: state.form.shadow.offset?.trim() ?? DEFAULT_SHADOW.offset,
		scale: Number(state.form.shadow.scale)
	};

	// Only include color_inactive if it has a non-empty value
	if (
		state.form.shadow.color_inactive &&
		typeof state.form.shadow.color_inactive === 'string' &&
		state.form.shadow.color_inactive.trim() !== ''
	) {
		shadowPayload.color_inactive = state.form.shadow.color_inactive.trim();
	}

	return {
		overrides: {
			rounding: Number(state.form.rounding),
			rounding_power: Number(state.form.rounding_power),
			active_opacity: Number(state.form.active_opacity),
			inactive_opacity: Number(state.form.inactive_opacity),
			fullscreen_opacity: Number(state.form.fullscreen_opacity),
			dim_modal: state.form.dim_modal,
			dim_inactive: state.form.dim_inactive,
			dim_strength: Number(state.form.dim_strength),
			dim_special: Number(state.form.dim_special),
			dim_around: Number(state.form.dim_around),
			screen_shader: state.form.screen_shader ?? DEFAULT_FORM.screen_shader,
			border_part_of_window: state.form.border_part_of_window,
			blur: {
				enabled: state.form.blur.enabled,
				size: Number(state.form.blur.size),
				passes: Number(state.form.blur.passes),
				ignore_opacity: state.form.blur.ignore_opacity,
				new_optimizations: state.form.blur.new_optimizations,
				xray: state.form.blur.xray,
				noise: Number(state.form.blur.noise),
				contrast: Number(state.form.blur.contrast),
				brightness: Number(state.form.blur.brightness),
				vibrancy: Number(state.form.blur.vibrancy),
				vibrancy_darkness: Number(state.form.blur.vibrancy_darkness),
				special: state.form.blur.special,
				popups: state.form.blur.popups,
				popups_ignorealpha: Number(state.form.blur.popups_ignorealpha),
				input_methods: state.form.blur.input_methods,
				input_methods_ignorealpha: Number(state.form.blur.input_methods_ignorealpha)
			},
			shadow: shadowPayload
		}
	};
}

export async function loadHyprlandDecoration(state) {
	state.isLoading = true;
	state.error = null;
	state.success = null;

	try {
		const snapshot = await invoke('get_hyprland_decoration_settings');
		applySnapshotToState(state, snapshot);
		return true;
	} catch (error) {
		console.error('Failed to load Hyprland decoration settings:', error);
		state.error =
			typeof error === 'string'
				? error
				: 'Unable to load Hyprland decoration settings. Please ensure the backend is running.';
		return false;
	} finally {
		state.isLoading = false;
	}
}

export async function saveHyprlandDecoration(state, options = {}) {
	if (state.isSaving) {
		return false;
	}

	const { silent = false, message } = options;

	const validation = validateHyprlandDecorationForm(state.form);
	state.validation = validation;
	if (!validation.isValid) {
		state.error = 'Cannot save until all validation errors are resolved.';
		return false;
	}

	state.isSaving = true;
	state.error = null;
	state.success = null;

	try {
		const payload = buildPayloadFromState(state);
		const snapshot = await invoke('update_hyprland_decoration_settings', {
			payload
		});

		applySnapshotToState(state, snapshot);
		const resolvedMessage = message ?? 'Hyprland decoration settings saved successfully.';
		state.success = silent ? null : resolvedMessage;
		return true;
	} catch (error) {
		console.error('Failed to save Hyprland decoration settings:', error);
		state.error =
			typeof error === 'string'
				? error
				: 'Unable to save Hyprland decoration settings. Please try again.';
		return false;
	} finally {
		state.isSaving = false;
	}
}

export function resetHyprlandDecorationToDefaults(state) {
	state.form = cloneForm();
	state.validation = validateHyprlandDecorationForm(state.form);
	markDirty(state);
}

export function getDefaultFormValues() {
	return cloneForm();
}

export function getDefaultBlurValues() {
	return cloneBlur();
}

export function getDefaultShadowValues() {
	return cloneShadow();
}
