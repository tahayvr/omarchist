import { invoke } from '@tauri-apps/api/core';

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
	repeat_delay: 600
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
			repeat_delay: safeRepeatDelay
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
