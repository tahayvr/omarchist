import { invoke } from '@tauri-apps/api/core';

const clone = (value) => {
	if (typeof structuredClone === 'function') {
		return structuredClone(value);
	}
	return JSON.parse(JSON.stringify(value ?? null));
};

const DEFAULT_LAYOUT = Object.freeze({
	left: Object.freeze(['hyprland/workspaces', 'hyprland/window']),
	center: Object.freeze(['clock']),
	right: Object.freeze(['network', 'battery', 'tray']),
	hidden: Object.freeze([])
});

const DEFAULT_GLOBALS = Object.freeze({
	layer: 'top',
	position: 'top',
	height: 32,
	margin: '0px',
	padding: '0px 12px',
	border_radius: 8,
	background: '#1e1e1e',
	foreground: '#d4d4d8'
});

const DEFAULT_MODULE_SETTINGS = Object.freeze({
	clock: Object.freeze({
		format: '{:%H:%M}',
		'tooltip-format': '{:%A %d %B %Y}',
		timezone: 'local'
	}),
	battery: Object.freeze({
		format: '{capacity}%',
		'tooltip-format': 'Battery {capacity}%',
		'critical-threshold': 15
	}),
	network: Object.freeze({
		'format-wifi': 'WiFi {essid}',
		'format-ethernet': 'Ethernet {ifname}',
		tooltip: true
	}),
	tray: Object.freeze({
		'icon-size': 18,
		spacing: 6
	}),
	'hyprland/workspaces': Object.freeze({
		format: '{name}',
		'active-only': false
	}),
	'hyprland/window': Object.freeze({
		format: '{title}',
		'max-length': 50
	})
});

export const KNOWN_MODULES = [
	{
		id: 'hyprland/workspaces',
		title: 'Workspaces',
		description: 'Displays active Hyprland workspaces.'
	},
	{
		id: 'hyprland/window',
		title: 'Focused Window',
		description: 'Shows the focused window title with truncation.'
	},
	{
		id: 'clock',
		title: 'Clock',
		description: 'Time display with alternate tooltip format.'
	},
	{
		id: 'battery',
		title: 'Battery',
		description: 'Battery status with percentage and critical threshold.'
	},
	{
		id: 'network',
		title: 'Network',
		description: 'Wi-Fi and Ethernet information.'
	},
	{
		id: 'tray',
		title: 'System Tray',
		description: 'Hosts status icons and background services.'
	}
];

function cloneModules(source = DEFAULT_MODULE_SETTINGS) {
	return Object.entries(source).reduce((acc, [key, value]) => {
		acc[key] = clone(value);
		return acc;
	}, {});
}

function cloneLayout(source = DEFAULT_LAYOUT) {
	return {
		left: [...(source.left ?? [])],
		center: [...(source.center ?? [])],
		right: [...(source.right ?? [])],
		hidden: [...(source.hidden ?? [])]
	};
}

function cloneGlobals(source = DEFAULT_GLOBALS) {
	return {
		layer: source.layer ?? DEFAULT_GLOBALS.layer,
		position: source.position ?? DEFAULT_GLOBALS.position,
		height: Number.isFinite(source.height) ? source.height : DEFAULT_GLOBALS.height,
		margin: typeof source.margin === 'string' ? source.margin : DEFAULT_GLOBALS.margin,
		padding: typeof source.padding === 'string' ? source.padding : DEFAULT_GLOBALS.padding,
		border_radius: Number.isFinite(source.border_radius)
			? source.border_radius
			: DEFAULT_GLOBALS.border_radius,
		background:
			typeof source.background === 'string' ? source.background : DEFAULT_GLOBALS.background,
		foreground:
			typeof source.foreground === 'string' ? source.foreground : DEFAULT_GLOBALS.foreground
	};
}

export function initializeWaybarConfigState() {
	return {
		layout: cloneLayout(),
		modules: cloneModules(),
		globals: cloneGlobals(),
		passthrough: {},
		raw: null,
		dirty: false,
		isLoading: false,
		isSaving: false,
		hasHydrated: false,
		error: null,
		success: null,
		validation: { isValid: true, fieldErrors: {} }
	};
}

function ensureArray(value, fallback = []) {
	if (Array.isArray(value)) {
		return value.filter((entry) => typeof entry === 'string' && entry.length);
	}
	return [...fallback];
}

function normalizeModules(layout, modules) {
	const known = new Set(KNOWN_MODULES.map((entry) => entry.id));
	const next = cloneModules();

	for (const moduleId of Object.keys(modules ?? {})) {
		if (!known.has(moduleId)) {
			continue;
		}
		const value = modules[moduleId];
		if (value && typeof value === 'object') {
			next[moduleId] = clone(value);
		}
	}

	const all = new Set([...layout.left, ...layout.center, ...layout.right, ...layout.hidden]);

	for (const moduleId of KNOWN_MODULES.map((entry) => entry.id)) {
		if (!all.has(moduleId) && !layout.hidden.includes(moduleId)) {
			layout.hidden.push(moduleId);
		}
	}

	layout.hidden = Array.from(new Set(layout.hidden));

	return next;
}

function computeHidden(layout) {
	const seen = new Set([...layout.left, ...layout.center, ...layout.right]);
	return KNOWN_MODULES.map((entry) => entry.id).filter((id) => !seen.has(id));
}

export function applySnapshotToState(state, snapshot) {
	const layout = snapshot?.layout ?? {};
	state.layout = cloneLayout({
		left: ensureArray(layout.left, DEFAULT_LAYOUT.left),
		center: ensureArray(layout.center, DEFAULT_LAYOUT.center),
		right: ensureArray(layout.right, DEFAULT_LAYOUT.right),
		hidden: ensureArray(layout.hidden, DEFAULT_LAYOUT.hidden)
	});

	state.layout.hidden = computeHidden(state.layout);

	state.globals = cloneGlobals(snapshot?.globals);
	state.modules = normalizeModules(state.layout, snapshot?.modules);
	state.passthrough =
		snapshot?.passthrough && typeof snapshot.passthrough === 'object'
			? clone(snapshot.passthrough)
			: {};
	state.raw = snapshot?.raw_json ?? null;
	state.dirty = false;
	state.hasHydrated = true;
	state.validation = validateWaybarConfig(state);
	state.error = null;
	state.success = null;
}

export function markWaybarDirty(state) {
	state.dirty = true;
	state.success = null;
}

export function validateWaybarConfig(state) {
	const fieldErrors = {};
	const allModules = new Set();

	const requireDistinct = (sectionName, list) => {
		for (const moduleId of list) {
			if (!KNOWN_MODULES.find((entry) => entry.id === moduleId)) {
				fieldErrors[`${sectionName}`] = 'Contains an unsupported module.';
				break;
			}
			if (allModules.has(moduleId)) {
				fieldErrors[`${sectionName}`] = 'Module already placed in another region.';
				break;
			}
			allModules.add(moduleId);
		}
	};

	requireDistinct('layout.left', state.layout.left);
	requireDistinct('layout.center', state.layout.center);
	requireDistinct('layout.right', state.layout.right);

	if (!['top', 'bottom'].includes(state.globals.position)) {
		fieldErrors['globals.position'] = 'Position must be either top or bottom.';
	}

	if (!['top', 'bottom', 'overlay'].includes(state.globals.layer)) {
		fieldErrors['globals.layer'] = 'Layer must be top, bottom, or overlay.';
	}

	if (!Number.isFinite(state.globals.height) || state.globals.height <= 0) {
		fieldErrors['globals.height'] = 'Height must be a positive number.';
	}

	if (!Number.isFinite(state.globals.border_radius) || state.globals.border_radius < 0) {
		fieldErrors['globals.border_radius'] = 'Border radius must be zero or greater.';
	}

	if (typeof state.globals.margin !== 'string') {
		fieldErrors['globals.margin'] = 'Margin must be a string.';
	}

	if (typeof state.globals.padding !== 'string') {
		fieldErrors['globals.padding'] = 'Padding must be a string.';
	}

	return {
		isValid: Object.keys(fieldErrors).length === 0,
		fieldErrors
	};
}

function buildSavePayload(state) {
	return {
		layout: {
			left: [...state.layout.left],
			center: [...state.layout.center],
			right: [...state.layout.right],
			hidden: computeHidden(state.layout)
		},
		modules: state.modules,
		globals: state.globals,
		passthrough: state.passthrough ?? {}
	};
}

export async function loadWaybarConfig(state) {
	state.isLoading = true;
	state.error = null;
	state.success = null;

	try {
		const snapshot = await invoke('get_waybar_config_snapshot');
		applySnapshotToState(state, snapshot);
		return true;
	} catch (error) {
		console.error('Failed to load Waybar config:', error);
		state.error =
			typeof error === 'string'
				? error
				: 'Unable to load Waybar configuration. Please ensure the config file exists.';
		return false;
	} finally {
		state.isLoading = false;
	}
}

export async function saveWaybarConfig(state, options = {}) {
	if (state.isSaving) {
		return false;
	}

	state.validation = validateWaybarConfig(state);
	if (!state.validation.isValid) {
		state.error = 'Please resolve validation errors before saving.';
		return false;
	}

	const { silent = false, message } = options;

	state.isSaving = true;
	state.error = null;
	state.success = null;

	try {
		const payload = buildSavePayload(state);
		const snapshot = await invoke('save_waybar_config_snapshot', { payload });
		applySnapshotToState(state, snapshot);
		const resolvedMessage = message ?? 'Waybar configuration saved.';
		state.success = silent ? null : resolvedMessage;
		return true;
	} catch (error) {
		console.error('Failed to save Waybar config:', error);
		state.error =
			typeof error === 'string' ? error : 'Unable to save Waybar configuration. Please try again.';
		return false;
	} finally {
		state.isSaving = false;
	}
}

export function resetWaybarConfigToDefaults(state) {
	state.layout = cloneLayout();
	state.modules = cloneModules();
	state.globals = cloneGlobals();
	state.validation = validateWaybarConfig(state);
	markWaybarDirty(state);
}

export function updateWaybarLayoutSection(state, section, modules) {
	if (!['left', 'center', 'right', 'hidden'].includes(section)) {
		return;
	}
	state.layout[section] = modules.filter((id) => KNOWN_MODULES.find((entry) => entry.id === id));
	state.layout.hidden = computeHidden(state.layout);
	markWaybarDirty(state);
}

export function updateWaybarGlobals(state, key, value) {
	if (!(key in state.globals)) {
		return;
	}
	state.globals[key] = value;
	state.validation = validateWaybarConfig(state);
	markWaybarDirty(state);
}

export function updateWaybarModule(state, moduleId, updater) {
	if (!state.modules[moduleId]) {
		return;
	}
	const nextValue =
		typeof updater === 'function' ? updater(clone(state.modules[moduleId])) : updater;
	if (!nextValue || typeof nextValue !== 'object') {
		return;
	}
	state.modules[moduleId] = nextValue;
	markWaybarDirty(state);
}
