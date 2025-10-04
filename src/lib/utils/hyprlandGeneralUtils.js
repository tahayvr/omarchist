import { invoke } from '@tauri-apps/api/core';

const DEFAULT_SNAP = Object.freeze({
	enabled: false,
	window_gap: 10,
	monitor_gap: 10,
	border_overlap: false,
	respect_gaps: false
});

const DEFAULT_FORM = Object.freeze({
	border_size: 1,
	no_border_on_floating: false,
	gaps_in: '5',
	gaps_out: '20',
	float_gaps: '0',
	gaps_workspaces: 0,
	layout: 'dwindle',
	no_focus_fallback: false,
	resize_on_border: false,
	extend_border_grab_area: 15,
	hover_icon_on_border: true,
	allow_tearing: false,
	resize_corner: 0,
	snap: DEFAULT_SNAP
});

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

function normalizeUnsignedInteger(value, fallback = 0) {
	const parsed = Number(value);
	if (Number.isFinite(parsed)) {
		return Math.max(0, Math.trunc(parsed));
	}
	return fallback;
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

function cloneSnap(snapshot = DEFAULT_SNAP) {
	return {
		enabled: normalizeBoolean(snapshot.enabled, DEFAULT_SNAP.enabled),
		window_gap: normalizeUnsignedInteger(snapshot.window_gap, DEFAULT_SNAP.window_gap),
		monitor_gap: normalizeUnsignedInteger(snapshot.monitor_gap, DEFAULT_SNAP.monitor_gap),
		border_overlap: normalizeBoolean(snapshot.border_overlap, DEFAULT_SNAP.border_overlap),
		respect_gaps: normalizeBoolean(snapshot.respect_gaps, DEFAULT_SNAP.respect_gaps)
	};
}

function cloneForm(source = DEFAULT_FORM) {
	return {
		border_size: normalizeUnsignedInteger(source.border_size, DEFAULT_FORM.border_size),
		no_border_on_floating: normalizeBoolean(
			source.no_border_on_floating,
			DEFAULT_FORM.no_border_on_floating
		),
		gaps_in: normalizeString(source.gaps_in, DEFAULT_FORM.gaps_in),
		gaps_out: normalizeString(source.gaps_out, DEFAULT_FORM.gaps_out),
		float_gaps: normalizeString(source.float_gaps, DEFAULT_FORM.float_gaps),
		gaps_workspaces: normalizeUnsignedInteger(source.gaps_workspaces, DEFAULT_FORM.gaps_workspaces),
		layout: normalizeLayout(source.layout ?? DEFAULT_FORM.layout),
		no_focus_fallback: normalizeBoolean(source.no_focus_fallback, DEFAULT_FORM.no_focus_fallback),
		resize_on_border: normalizeBoolean(source.resize_on_border, DEFAULT_FORM.resize_on_border),
		extend_border_grab_area: normalizeInteger(
			source.extend_border_grab_area,
			DEFAULT_FORM.extend_border_grab_area
		),
		hover_icon_on_border: normalizeBoolean(
			source.hover_icon_on_border,
			DEFAULT_FORM.hover_icon_on_border
		),
		allow_tearing: normalizeBoolean(source.allow_tearing, DEFAULT_FORM.allow_tearing),
		resize_corner: normalizeInteger(source.resize_corner, DEFAULT_FORM.resize_corner),
		snap: cloneSnap(source.snap ?? DEFAULT_SNAP)
	};
}

function applySnapshotToState(state, snapshot) {
	const effective = snapshot?.effective ?? {};
	const overrides = snapshot?.overrides ?? {};

	state.snapshot = snapshot ?? null;
	state.effective = {
		...effective,
		snap: {
			...(effective.snap ?? {})
		}
	};
	state.overrides = {
		...overrides,
		snap: {
			...(overrides.snap ?? {})
		}
	};
	state.hasHydrated = true;

	state.form = cloneForm(state.effective);

	state.lastSavedForm = cloneForm(state.form);
	state.dirty = false;
	state.validation = validateHyprlandGeneralForm(state.form);
}

export function initializeHyprlandGeneralState() {
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
		validation: validateHyprlandGeneralForm(DEFAULT_FORM),
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

export function validateHyprlandGeneralForm(form) {
	const fieldErrors = {};

	const requireBoolean = (field) => {
		if (typeof form[field] !== 'boolean') {
			fieldErrors[field] = 'Must be true or false.';
		}
	};

	const requireNonEmptyString = (field) => {
		if (typeof form[field] !== 'string' || form[field].trim() === '') {
			fieldErrors[field] = 'Value is required.';
		}
	};

	requireBoolean('no_border_on_floating');
	requireBoolean('no_focus_fallback');
	requireBoolean('resize_on_border');
	requireBoolean('hover_icon_on_border');
	requireBoolean('allow_tearing');
	requireNonEmptyString('gaps_in');
	requireNonEmptyString('gaps_out');
	requireNonEmptyString('float_gaps');

	if (!VALID_LAYOUTS.has(String(form.layout).toLowerCase())) {
		fieldErrors.layout = 'Layout must be either master or dwindle.';
	}

	const borderSizeValue = Number(form.border_size);
	if (!Number.isInteger(borderSizeValue) || borderSizeValue < 0) {
		fieldErrors.border_size = 'Value must be a non-negative integer.';
	}

	const extendValue = Number(form.extend_border_grab_area);
	if (!Number.isInteger(extendValue) || extendValue < 0) {
		fieldErrors.extend_border_grab_area = 'Value must be a non-negative integer.';
	}

	const gapsWorkspacesValue = Number(form.gaps_workspaces);
	if (!Number.isInteger(gapsWorkspacesValue) || gapsWorkspacesValue < 0) {
		fieldErrors.gaps_workspaces = 'Value must be a non-negative integer.';
	}

	const cornerValue = Number(form.resize_corner);
	if (!Number.isInteger(cornerValue) || cornerValue < 0 || cornerValue > 4) {
		fieldErrors.resize_corner = 'Value must be between 0 and 4.';
	}

	const snap = form.snap ?? {};

	const requireSnapBoolean = (field) => {
		if (typeof snap[field] !== 'boolean') {
			fieldErrors[`snap.${field}`] = 'Must be true or false.';
		}
	};

	requireSnapBoolean('enabled');
	requireSnapBoolean('border_overlap');
	requireSnapBoolean('respect_gaps');

	const windowGapRaw = snap.window_gap;
	const windowGap = Number(windowGapRaw);
	if (
		windowGapRaw === '' ||
		windowGapRaw === null ||
		windowGapRaw === undefined ||
		!Number.isInteger(windowGap) ||
		windowGap < 0
	) {
		fieldErrors['snap.window_gap'] = 'Value must be a non-negative integer.';
	}

	const monitorGapRaw = snap.monitor_gap;
	const monitorGap = Number(monitorGapRaw);
	if (
		monitorGapRaw === '' ||
		monitorGapRaw === null ||
		monitorGapRaw === undefined ||
		!Number.isInteger(monitorGap) ||
		monitorGap < 0
	) {
		fieldErrors['snap.monitor_gap'] = 'Value must be a non-negative integer.';
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
			border_size: Number(state.form.border_size),
			no_border_on_floating: state.form.no_border_on_floating,
			gaps_in: state.form.gaps_in?.trim() ?? DEFAULT_FORM.gaps_in,
			gaps_out: state.form.gaps_out?.trim() ?? DEFAULT_FORM.gaps_out,
			float_gaps: state.form.float_gaps?.trim() ?? DEFAULT_FORM.float_gaps,
			gaps_workspaces: Number(state.form.gaps_workspaces),
			layout: state.form.layout,
			no_focus_fallback: state.form.no_focus_fallback,
			resize_on_border: state.form.resize_on_border,
			extend_border_grab_area: Number(state.form.extend_border_grab_area),
			hover_icon_on_border: state.form.hover_icon_on_border,
			allow_tearing: state.form.allow_tearing,
			resize_corner: Number(state.form.resize_corner),
			snap: {
				enabled: state.form.snap.enabled,
				window_gap: Number(state.form.snap.window_gap),
				monitor_gap: Number(state.form.snap.monitor_gap),
				border_overlap: state.form.snap.border_overlap,
				respect_gaps: state.form.snap.respect_gaps
			}
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
	state.form = cloneForm();
	state.validation = validateHyprlandGeneralForm(state.form);
	markDirty(state);
}

export function getDefaultFormValues() {
	return cloneForm();
}

export function getDefaultSnapValues() {
	return cloneSnap();
}
