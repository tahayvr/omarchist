import { invoke } from '@tauri-apps/api/core';

const DEFAULT_FORM = {
	no_border_on_floating: false,
	layout: 'dwindle',
	no_focus_fallback: false,
	resize_on_border: false,
	extend_border_grab_area: 15,
	hover_icon_on_border: true,
	allow_tearing: false,
	resize_corner: 0
};

const VALID_LAYOUTS = new Set(['master', 'dwindle']);

function normalizeBoolean(value, fallback = false) {
	if (typeof value === 'boolean') {
		return value;
	}
	if (typeof value === 'number') {
		return value !== 0;
	}
	return fallback;
}

function normalizeLayout(value) {
	if (typeof value === 'string') {
		const lower = value.toLowerCase();
		if (lower === 'master' || lower === 'dwindle') {
			return lower;
		}
	}
	return 'dwindle';
}

function normalizeInteger(value, fallback = 0) {
	const parsed = Number(value);
	if (Number.isFinite(parsed)) {
		return Math.trunc(parsed);
	}
	return fallback;
}

function applySnapshotToState(state, snapshot) {
	const effective = snapshot?.effective ?? {};
	const overrides = snapshot?.overrides ?? {};

	state.snapshot = snapshot ?? null;
	state.effective = effective;
	state.overrides = overrides;
	state.hasHydrated = true;

	state.form = {
		no_border_on_floating: normalizeBoolean(
			effective.no_border_on_floating,
			DEFAULT_FORM.no_border_on_floating
		),
		layout: normalizeLayout(effective.layout ?? DEFAULT_FORM.layout),
		no_focus_fallback: normalizeBoolean(
			effective.no_focus_fallback,
			DEFAULT_FORM.no_focus_fallback
		),
		resize_on_border: normalizeBoolean(effective.resize_on_border, DEFAULT_FORM.resize_on_border),
		extend_border_grab_area: normalizeInteger(
			effective.extend_border_grab_area,
			DEFAULT_FORM.extend_border_grab_area
		),
		hover_icon_on_border: normalizeBoolean(
			effective.hover_icon_on_border,
			DEFAULT_FORM.hover_icon_on_border
		),
		allow_tearing: normalizeBoolean(effective.allow_tearing, DEFAULT_FORM.allow_tearing),
		resize_corner: normalizeInteger(effective.resize_corner, DEFAULT_FORM.resize_corner)
	};

	state.lastSavedForm = { ...state.form };
	state.dirty = false;
	state.validation = validateHyprlandGeneralForm(state.form);
}

export function initializeHyprlandGeneralState() {
	return {
		form: { ...DEFAULT_FORM },
		effective: { ...DEFAULT_FORM },
		overrides: {},
		snapshot: null,
		lastSavedForm: { ...DEFAULT_FORM },
		isLoading: false,
		isSaving: false,
		error: null,
		success: null,
		dirty: false,
		validation: validateHyprlandGeneralForm(DEFAULT_FORM),
		autoSaveHandle: null,
		hasHydrated: false
	};
}

export function markDirty(state) {
	state.dirty = true;
	state.success = null;
}

export function recomputeDirty(state) {
	const current = JSON.stringify(state.form ?? {});
	const lastSaved = JSON.stringify(state.lastSavedForm ?? {});
	state.dirty = current !== lastSaved;
	if (state.dirty) {
		state.success = null;
	}
}

export function validateHyprlandGeneralForm(form) {
	const fieldErrors = {};

	const requireBoolean = (field) => {
		if (typeof form[field] !== 'boolean') {
			fieldErrors[field] = 'Must be true or false.';
		}
	};

	requireBoolean('no_border_on_floating');
	requireBoolean('no_focus_fallback');
	requireBoolean('resize_on_border');
	requireBoolean('hover_icon_on_border');
	requireBoolean('allow_tearing');

	if (!VALID_LAYOUTS.has(String(form.layout).toLowerCase())) {
		fieldErrors.layout = 'Layout must be either master or dwindle.';
	}

	const extendValue = Number(form.extend_border_grab_area);
	if (!Number.isInteger(extendValue) || extendValue < 0) {
		fieldErrors.extend_border_grab_area = 'Value must be a non-negative integer.';
	}

	const cornerValue = Number(form.resize_corner);
	if (!Number.isInteger(cornerValue) || cornerValue < 0 || cornerValue > 4) {
		fieldErrors.resize_corner = 'Value must be between 0 and 4.';
	}

	return {
		isValid: Object.keys(fieldErrors).length === 0,
		fieldErrors
	};
}

export async function loadHyprlandGeneral(state) {
	state.isLoading = true;
	state.error = null;
	state.success = null;

	try {
		const snapshot = await invoke('get_hyprland_general_settings');
		applySnapshotToState(state, snapshot);
		return true;
	} catch (error) {
		console.error('Failed to load Hyprland general settings:', error);
		state.error =
			typeof error === 'string'
				? error
				: 'Unable to load Hyprland general settings. Please ensure the backend is running.';
		return false;
	} finally {
		state.isLoading = false;
	}
}

function buildPayloadFromState(state) {
	return {
		overrides: {
			no_border_on_floating: state.form.no_border_on_floating,
			layout: state.form.layout,
			no_focus_fallback: state.form.no_focus_fallback,
			resize_on_border: state.form.resize_on_border,
			extend_border_grab_area: state.form.extend_border_grab_area,
			hover_icon_on_border: state.form.hover_icon_on_border,
			allow_tearing: state.form.allow_tearing,
			resize_corner: state.form.resize_corner
		}
	};
}

export async function saveHyprlandGeneral(state, options = {}) {
	if (state.isSaving) {
		return false;
	}

	const { silent = false, message } = options;

	const validation = validateHyprlandGeneralForm(state.form);
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
		const snapshot = await invoke('update_hyprland_general_settings', {
			payload
		});

		applySnapshotToState(state, snapshot);
		const resolvedMessage = message ?? 'Hyprland general settings saved successfully.';
		state.success = silent ? null : resolvedMessage;
		return true;
	} catch (error) {
		console.error('Failed to save Hyprland general settings:', error);
		state.error =
			typeof error === 'string'
				? error
				: 'Unable to save Hyprland general settings. Please try again.';
		return false;
	} finally {
		state.isSaving = false;
	}
}

export function resetHyprlandGeneralToDefaults(state) {
	state.form = { ...DEFAULT_FORM };
	state.validation = validateHyprlandGeneralForm(state.form);
	markDirty(state);
}

export function getDefaultFormValues() {
	return { ...DEFAULT_FORM };
}
