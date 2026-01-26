import { invoke } from '@tauri-apps/api/core';
import { moduleRegistry } from './moduleRegistry.js';

const clone = (value) => {
	if (typeof structuredClone === 'function') {
		try {
			return structuredClone(value);
		} catch {
			// Fall through to JSON clone
		}
	}
	if (value === undefined) {
		return undefined;
	}
	try {
		return JSON.parse(JSON.stringify(value));
	} catch {
		if (Array.isArray(value)) {
			return [...value];
		}
		if (value && typeof value === 'object') {
			return { ...value };
		}
		return value;
	}
};

const DEFAULT_LAYOUT = Object.freeze({
	left: Object.freeze(['custom/omarchy', 'hyprland/workspaces']),
	center: Object.freeze([
		'clock',
		'custom/update',
		'custom/voxtype',
		'custom/screenrecording-indicator'
	]),
	right: Object.freeze([
		'group/tray-expander',
		'bluetooth',
		'network',
		'pulseaudio',
		'cpu',
		'battery'
	]),
	hidden: Object.freeze(['hyprland/window'])
});

const DEFAULT_GLOBALS = Object.freeze({
	layer: 'top',
	position: 'top',
	height: 26,
	background: '@background',
	foreground: '@foreground',
	spacing: 0,
	leftMargin: 8,
	leftPadding: 0,
	leftBackground: '',
	centerMargin: 0,
	centerPadding: 0,
	centerBackground: '',
	rightMargin: 8,
	rightPadding: 0,
	rightBackground: ''
});

const DEFAULT_PASSTHROUGH = Object.freeze({
	reload_style_on_change: true
});

function getModuleDefaultConfig(moduleId) {
	const registryEntry = moduleRegistry[moduleId];
	if (!registryEntry) return {};

	if (registryEntry.defaultConfig) {
		return clone(registryEntry.defaultConfig);
	}

	if (registryEntry.schema && registryEntry.schema.properties) {
		const config = {};
		for (const [key, prop] of Object.entries(registryEntry.schema.properties)) {
			if (prop.default !== undefined) {
				config[key] = clone(prop.default);
			}
		}
		return config;
	}

	return {};
}

const DEFAULT_MODULE_SETTINGS = Object.freeze(
	Object.keys(moduleRegistry).reduce((acc, moduleId) => {
		acc[moduleId] = Object.freeze(getModuleDefaultConfig(moduleId));
		return acc;
	}, {})
);

export const KNOWN_MODULES = Object.entries(moduleRegistry).map(([id, entry]) => ({
	id,
	title: entry.schema?.title || id,
	description: entry.schema?.description || ''
}));

const MODULE_LOOKUP = new Map(KNOWN_MODULES.map((entry) => [entry.id, entry]));
const LAYOUT_SECTIONS = ['left', 'center', 'right', 'hidden'];

export const GLOBAL_FIELD_DEFINITIONS = Object.freeze([
	{
		key: 'layer',
		label: 'Layer',
		type: 'select',
		options: [
			{ label: 'Top', value: 'top' },
			{ label: 'Bottom', value: 'bottom' },
			{ label: 'Overlay', value: 'overlay' }
		]
	},
	{
		key: 'position',
		label: 'Position',
		type: 'select',
		options: [
			{ label: 'Top', value: 'top' },
			{ label: 'Bottom', value: 'bottom' }
		]
	},
	{ key: 'height', label: 'Height (px)', type: 'number', min: 16, max: 128, step: 1 },
	{ key: 'spacing', label: 'Spacing (px)', type: 'number', min: 0, max: 64, step: 1 }
]);

const MODULE_FIELD_DEFINITIONS = Object.freeze({
	clock: [
		{ key: 'format', label: 'Format', type: 'text', placeholder: '{:L%A %H:%M}' },
		{ key: 'format-alt', label: 'Alt Format', type: 'text', placeholder: '{:L%d %B W%V %Y}' }
	],
	battery: [
		{ key: 'format', label: 'Format', type: 'text', placeholder: '{capacity}% {icon}' },
		{ key: 'format-full', label: 'Full Icon', type: 'text', placeholder: '󰂅' }
	],
	network: [
		{ key: 'format', label: 'Format', type: 'text', placeholder: '{icon}' },
		{ key: 'format-wifi', label: 'Wi-Fi Format', type: 'text', placeholder: '{icon}' },
		{ key: 'format-ethernet', label: 'Ethernet Format', type: 'text', placeholder: '󰀂' },
		{ key: 'on-click', label: 'On Click Command', type: 'text', placeholder: 'omarchy-launch-wifi' }
	],
	tray: [
		{ key: 'icon-size', label: 'Icon Size', type: 'number', min: 8, max: 64, step: 1 },
		{ key: 'spacing', label: 'Spacing', type: 'number', min: 0, max: 32, step: 1 }
	],
	'hyprland/workspaces': [{ key: 'format', label: 'Format', type: 'text', placeholder: '{icon}' }],
	'hyprland/window': [
		{ key: 'format', label: 'Format', type: 'text', placeholder: '{title}' },
		{
			key: 'max-length',
			label: 'Max Length',
			type: 'number',
			min: 10,
			max: 200,
			step: 5
		}
	],
	'custom/update': [
		{
			key: 'interval',
			label: 'Refresh Interval (s)',
			type: 'number',
			min: 60,
			max: 86400,
			step: 60
		},
		{ key: 'signal', label: 'Signal ID', type: 'number', min: 1, max: 64, step: 1 }
	],
	'custom/screenrecording-indicator': [
		{ key: 'signal', label: 'Signal ID', type: 'number', min: 1, max: 64, step: 1 }
	],
	bluetooth: [
		{ key: 'on-click', label: 'On Click Command', type: 'text', placeholder: 'blueberry' }
	],
	pulseaudio: [
		{ key: 'scroll-step', label: 'Scroll Step', type: 'number', min: 1, max: 10, step: 1 }
	],
	cpu: [
		{ key: 'interval', label: 'Refresh Interval (s)', type: 'number', min: 1, max: 60, step: 1 }
	]
});

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
		spacing: Number.isFinite(source.spacing) ? source.spacing : DEFAULT_GLOBALS.spacing,
		background:
			typeof source.background === 'string' ? source.background : DEFAULT_GLOBALS.background,
		foreground:
			typeof source.foreground === 'string' ? source.foreground : DEFAULT_GLOBALS.foreground,
		leftMargin: Number.isFinite(source.leftMargin) ? source.leftMargin : DEFAULT_GLOBALS.leftMargin,
		leftPadding: Number.isFinite(source.leftPadding)
			? source.leftPadding
			: DEFAULT_GLOBALS.leftPadding,
		leftBackground:
			typeof source.leftBackground === 'string'
				? source.leftBackground
				: DEFAULT_GLOBALS.leftBackground,
		centerMargin: Number.isFinite(source.centerMargin)
			? source.centerMargin
			: DEFAULT_GLOBALS.centerMargin,
		centerPadding: Number.isFinite(source.centerPadding)
			? source.centerPadding
			: DEFAULT_GLOBALS.centerPadding,
		centerBackground:
			typeof source.centerBackground === 'string'
				? source.centerBackground
				: DEFAULT_GLOBALS.centerBackground,
		rightMargin: Number.isFinite(source.rightMargin)
			? source.rightMargin
			: DEFAULT_GLOBALS.rightMargin,
		rightPadding: Number.isFinite(source.rightPadding)
			? source.rightPadding
			: DEFAULT_GLOBALS.rightPadding,
		rightBackground:
			typeof source.rightBackground === 'string'
				? source.rightBackground
				: DEFAULT_GLOBALS.rightBackground
	};
}

export function initializeWaybarConfigState() {
	return {
		layout: cloneLayout(),
		modules: cloneModules(),
		moduleStyles: {},
		globals: cloneGlobals(),
		passthrough: clone(DEFAULT_PASSTHROUGH),
		styleCss: '',
		raw: null,
		profileId: null,
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
	const known = new Set([
		...KNOWN_MODULES.map((entry) => entry.id),
		...Object.keys(DEFAULT_MODULE_SETTINGS)
	]);
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
	if (snapshot?.passthrough && typeof snapshot.passthrough === 'object') {
		const incoming = clone(snapshot.passthrough);
		state.passthrough = { ...clone(DEFAULT_PASSTHROUGH), ...(incoming ?? {}) };
	} else {
		state.passthrough = clone(DEFAULT_PASSTHROUGH);
	}

	const omarchist = snapshot?.passthrough?._omarchist;
	if (omarchist && typeof omarchist === 'object' && omarchist.moduleStyles) {
		state.moduleStyles = clone(omarchist.moduleStyles);
	} else {
		state.moduleStyles = {};
	}

	state.styleCss = typeof snapshot?.style_css === 'string' ? snapshot.style_css : '';
	state.raw = snapshot?.raw_json ?? null;
	state.profileId = snapshot?.profile_id ?? state.profileId ?? null;
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

	if (!Number.isFinite(state.globals.spacing) || state.globals.spacing < 0) {
		fieldErrors['globals.spacing'] = 'Spacing must be zero or greater.';
	}

	return {
		isValid: Object.keys(fieldErrors).length === 0,
		fieldErrors
	};
}

import { generateWaybarStyles } from './styleGenerator.js';

function sanitizeModulesForSave(modules) {
	const cleaned = {};
	for (const [moduleId, config] of Object.entries(modules)) {
		const modConfig = { ...config };

		for (const key of Object.keys(modConfig)) {
			const value = modConfig[key];

			if (value === '__default') {
				delete modConfig[key];
				continue;
			}

			if (value === '__custom') {
				const customKey = `${key}-custom`;
				if (modConfig[customKey] !== undefined) {
					modConfig[key] = modConfig[customKey];
				} else {
					delete modConfig[key];
				}
			}
		}

		for (const key of Object.keys(modConfig)) {
			if (key.endsWith('-custom')) {
				delete modConfig[key];
			}
		}

		// Transform rewrite-rules to rewrite object
		if (modConfig['rewrite-rules']) {
			const rules = {};
			let rulesSource = modConfig['rewrite-rules'];

			// Handle if it's a string (textarea) or array
			const lines = Array.isArray(rulesSource)
				? rulesSource
				: typeof rulesSource === 'string'
					? rulesSource.split('\n')
					: [];

			for (const line of lines) {
				if (!line || typeof line !== 'string') continue;
				const trimmed = line.trim();
				if (!trimmed) continue;

				// Match "pattern": "replacement"
				const match = trimmed.match(/^"(.*)"\s*:\s*"(.*)"$/);
				if (match) {
					rules[match[1]] = match[2];
				}
			}

			if (Object.keys(rules).length > 0) {
				modConfig['rewrite'] = rules;
			}
			delete modConfig['rewrite-rules'];
		}

		// Un-flatten dot-notation keys (e.g. "states.warning" -> states: { warning: ... })
		// We iterate over a snapshot of keys to avoid issues while modifying the object
		for (const key of Object.keys(modConfig)) {
			if (key.includes('.')) {
				const value = modConfig[key];
				const parts = key.split('.');

				let current = modConfig;
				for (let i = 0; i < parts.length - 1; i++) {
					const part = parts[i];
					if (!current[part] || typeof current[part] !== 'object') {
						current[part] = {};
					}
					current = current[part];
				}

				current[parts[parts.length - 1]] = value;
				delete modConfig[key];
			}
		}

		cleaned[moduleId] = modConfig;
	}
	return cleaned;
}

function buildSavePayload(state) {
	const generatedCss = generateWaybarStyles(state.globals, state.moduleStyles);
	const sanitizedModules = sanitizeModulesForSave(state.modules);

	return {
		layout: {
			left: [...state.layout.left],
			center: [...state.layout.center],
			right: [...state.layout.right],
			hidden: computeHidden(state.layout)
		},
		modules: sanitizedModules,
		globals: state.globals,
		passthrough: state.passthrough ?? {},
		style_css: generatedCss,
		module_styles: state.moduleStyles || {}
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
	state.moduleStyles = {};
	state.globals = cloneGlobals();
	state.passthrough = clone(DEFAULT_PASSTHROUGH);
	state.styleCss = '';
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

export function updateWaybarStyleCss(state, styleCss) {
	state.styleCss = typeof styleCss === 'string' ? styleCss : '';
	markWaybarDirty(state);
}

export function updateModuleStyle(state, moduleId, style) {
	if (!moduleId) {
		return;
	}
	state.moduleStyles = {
		...state.moduleStyles,
		[moduleId]: style || {}
	};
	markWaybarDirty(state);
}

export function getModuleStyle(state, moduleId) {
	return state.moduleStyles?.[moduleId] || {};
}

export function updateWaybarModule(state, moduleId, updater) {
	const currentValue = state.modules?.[moduleId];
	const base = isPlainObject(currentValue) ? currentValue : {};
	const nextValue = typeof updater === 'function' ? updater(clone(base)) : updater;
	if (!isPlainObject(nextValue)) {
		return;
	}
	state.modules = {
		...state.modules,
		[moduleId]: nextValue
	};
	state.validation = validateWaybarConfig(state);
	markWaybarDirty(state);
}

function coerceNumber(value, fallback) {
	const parsed = Number(value);
	return Number.isFinite(parsed) ? parsed : fallback;
}

function normalizeRegion(region) {
	if (LAYOUT_SECTIONS.includes(region)) {
		return region;
	}
	return 'hidden';
}

function removeModuleFromSections(layout, moduleId) {
	for (const section of LAYOUT_SECTIONS) {
		layout[section] = layout[section].filter((id) => id !== moduleId);
	}
}

function isPlainObject(value) {
	return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function splitFieldPath(fieldKey) {
	return fieldKey
		.split('.')
		.map((segment) => segment.trim())
		.filter(Boolean);
}

function pruneEmptyAncestors(target, segments) {
	const stack = [];
	let cursor = target;
	for (const segment of segments) {
		if (!isPlainObject(cursor)) {
			return;
		}
		stack.push({ parent: cursor, key: segment });
		cursor = cursor[segment];
	}

	for (let index = stack.length - 1; index >= 0; index -= 1) {
		const { parent, key } = stack[index];
		const value = parent[key];
		if (isPlainObject(value) && Object.keys(value).length === 0) {
			delete parent[key];
			continue;
		}
		break;
	}
}

function setNestedModuleValue(target, fieldKey, value) {
	if (typeof fieldKey !== 'string' || !fieldKey.includes('.')) {
		if (value === null) {
			delete target[fieldKey];
			return;
		}
		target[fieldKey] = value;
		return;
	}

	const segments = splitFieldPath(fieldKey);
	if (!segments.length) {
		return;
	}

	const lastIndex = segments.length - 1;
	let cursor = target;

	for (let index = 0; index < lastIndex; index += 1) {
		const key = segments[index];
		if (!isPlainObject(cursor[key])) {
			cursor[key] = {};
		}
		cursor = cursor[key];
	}

	const lastKey = segments[lastIndex];
	if (value === null) {
		if (isPlainObject(cursor)) {
			delete cursor[lastKey];
			pruneEmptyAncestors(target, segments.slice(0, -1));
		}
		return;
	}

	if (isPlainObject(cursor)) {
		cursor[lastKey] = value;
	}
}

function getNestedModuleValue(source, fieldKey) {
	if (typeof fieldKey !== 'string') {
		return undefined;
	}
	if (!fieldKey.includes('.')) {
		return source?.[fieldKey];
	}
	const segments = splitFieldPath(fieldKey);
	let cursor = source;
	for (const segment of segments) {
		if (!isPlainObject(cursor) && !Array.isArray(cursor)) {
			return undefined;
		}
		cursor = cursor?.[segment];
		if (cursor === undefined) {
			return undefined;
		}
	}
	return cursor;
}

function determineRegion(layout, moduleId) {
	if (layout.left.includes(moduleId)) {
		return 'left';
	}
	if (layout.center.includes(moduleId)) {
		return 'center';
	}
	if (layout.right.includes(moduleId)) {
		return 'right';
	}
	return 'hidden';
}

export function getModuleMeta(moduleId) {
	return MODULE_LOOKUP.get(moduleId) ?? null;
}

export function getModuleRegion(state, moduleId) {
	return determineRegion(state.layout, moduleId);
}

export function setModuleRegion(state, moduleId, region) {
	if (!MODULE_LOOKUP.has(moduleId)) {
		return;
	}

	const target = normalizeRegion(region);
	removeModuleFromSections(state.layout, moduleId);

	if (target !== 'hidden') {
		state.layout[target] = [...state.layout[target], moduleId];
	}

	state.layout.hidden = computeHidden(state.layout);
	state.validation = validateWaybarConfig(state);
	markWaybarDirty(state);
}

export function getModuleFields(moduleId) {
	return MODULE_FIELD_DEFINITIONS[moduleId] ?? [];
}

export function setModuleConfig(state, moduleId, moduleConfig) {
	if (!MODULE_LOOKUP.has(moduleId)) {
		return;
	}
	const nextValue = isPlainObject(moduleConfig) ? clone(moduleConfig) : {};
	updateWaybarModule(state, moduleId, () => nextValue);
}

export function setModuleField(state, moduleId, fieldKey, value) {
	if (!MODULE_LOOKUP.has(moduleId)) {
		return;
	}
	updateWaybarModule(state, moduleId, (current) => {
		const next = current && typeof current === 'object' ? current : {};
		setNestedModuleValue(next, fieldKey, value);
		return next;
	});
}

export function getModuleFieldValue(state, moduleId, fieldKey) {
	return getNestedModuleValue(state.modules?.[moduleId], fieldKey);
}

export function getGlobalFieldDefinitions() {
	return GLOBAL_FIELD_DEFINITIONS;
}

export function sanitizeGlobalInput(key, value) {
	switch (key) {
		case 'height':
		case 'spacing':
		case 'leftMargin':
		case 'leftPadding':
		case 'centerMargin':
		case 'centerPadding':
		case 'rightMargin':
		case 'rightPadding':
			return coerceNumber(value, DEFAULT_GLOBALS[key]);
		default:
			return value;
	}
}

export async function listWaybarProfiles() {
	return invoke('list_waybar_profiles');
}

export async function createWaybarProfile(name) {
	return invoke('create_waybar_profile', { name });
}

export async function selectWaybarProfile(profileId) {
	return invoke('select_waybar_profile', { profileId });
}

export async function deleteWaybarProfile(profileId) {
	return invoke('delete_waybar_profile', { profileId });
}

export async function getWaybarStyleCss() {
	return invoke('get_waybar_style_css');
}

export async function saveWaybarStyleCss(styleCss) {
	return invoke('save_waybar_style_css', { styleCss });
}
