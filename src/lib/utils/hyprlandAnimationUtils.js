import { invoke } from '@tauri-apps/api/core';

const DEFAULT_FORM = Object.freeze({
	enabled: true,
	workspace_wraparound: false
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

function cloneForm(source = DEFAULT_FORM) {
	return {
		enabled: normalizeBoolean(source.enabled, DEFAULT_FORM.enabled),
		workspace_wraparound: normalizeBoolean(
			source.workspace_wraparound,
			DEFAULT_FORM.workspace_wraparound
		)
	};
}

function applySnapshotToState(state, snapshot) {
	const effective = snapshot?.effective ?? {};
	const overrides = snapshot?.overrides ?? {};

	state.snapshot = snapshot ?? null;
	state.effective = { ...effective };
	state.overrides = { ...overrides };
	state.hasHydrated = true;

	state.form = cloneForm(state.effective);
	state.lastSavedForm = cloneForm(state.form);
	state.dirty = false;
	state.validation = validateHyprlandAnimationForm(state.form);
}

export function initializeHyprlandAnimationState() {
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
		validation: validateHyprlandAnimationForm(DEFAULT_FORM),
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

export function validateHyprlandAnimationForm(form) {
	const fieldErrors = {};

	const requireBoolean = (field) => {
		if (typeof form[field] !== 'boolean') {
			fieldErrors[field] = 'Must be true or false.';
		}
	};

	requireBoolean('enabled');
	requireBoolean('workspace_wraparound');

	return {
		isValid: Object.keys(fieldErrors).length === 0,
		fieldErrors
	};
}

export async function loadHyprlandAnimation(state) {
	if (state.isLoading) {
		return false;
	}

	state.isLoading = true;
	state.error = null;

	try {
		const snapshot = await invoke('get_hyprland_animation_settings');
		applySnapshotToState(state, snapshot);
		return true;
	} catch (err) {
		state.error = `Failed to load Hyprland animation settings: ${err}`;
		return false;
	} finally {
		state.isLoading = false;
	}
}

export async function saveHyprlandAnimation(state, options = {}) {
	const { silent = false, message = null } = options;

	if (state.isSaving) {
		return false;
	}

	const validation = validateHyprlandAnimationForm(state.form);
	if (!validation.isValid) {
		if (!silent) {
			state.error = 'Please fix validation errors before saving.';
		}
		return false;
	}

	state.isSaving = true;
	state.error = null;
	state.success = null;

	try {
		const overridesPayload = buildOverridesFromForm(state.form);
		const snapshot = await invoke('update_hyprland_animation_settings', {
			payload: { overrides: overridesPayload }
		});

		applySnapshotToState(state, snapshot);

		if (!silent) {
			state.success = message ?? 'Hyprland animation settings saved successfully.';
		}

		return true;
	} catch (err) {
		if (!silent) {
			state.error = `Failed to save Hyprland animation settings: ${err}`;
		}
		return false;
	} finally {
		state.isSaving = false;
	}
}

export function resetHyprlandAnimationToDefaults(state) {
	state.form = cloneForm(DEFAULT_FORM);
	state.dirty = true;
	state.validation = validateHyprlandAnimationForm(state.form);
}

function buildOverridesFromForm(form) {
	const overrides = {};

	const hasChanged = (field, defaultValue) => {
		return form[field] !== defaultValue;
	};

	if (hasChanged('enabled', DEFAULT_FORM.enabled)) {
		overrides.enabled = form.enabled;
	}

	if (hasChanged('workspace_wraparound', DEFAULT_FORM.workspace_wraparound)) {
		overrides.workspace_wraparound = form.workspace_wraparound;
	}

	return overrides;
}
