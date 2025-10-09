import { describe, it, expect, beforeEach } from 'vitest';
import {
	initializeHyprlandInputState,
	validateHyprlandInputForm,
	toggleOptionInForm,
	getVariantsForLayout,
	getSelectedOptionsSet,
	prepareCatalog
} from '../hyprlandInputUtils.js';

describe('hyprlandInputUtils', () => {
	let state;

	beforeEach(() => {
		state = initializeHyprlandInputState();
		state.catalog = {
			models: [{ name: 'pc105', description: 'Generic 105-key PC' }],
			layouts: [
				{
					name: 'us',
					description: 'English (US)',
					variants: [{ name: 'intl', description: 'English (US, intl.)' }]
				},
				{
					name: 'gb',
					description: 'English (UK)',
					variants: []
				}
			],
			optionGroups: [
				{
					name: 'grp',
					description: 'Group switching',
					options: [{ name: 'toggle', description: 'Toggle group' }]
				}
			],
			variantsByLayout: {
				us: [{ name: 'intl', description: 'English (US, intl.)' }],
				gb: []
			},
			optionCodes: new Set(['grp:toggle'])
		};
	});

	it('initializes with default form values', () => {
		expect(state.form).toMatchObject({
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
			sensitivity: 0,
			accel_profile: '',
			force_no_accel: false,
			left_handed: false,
			scroll_points: '',
			scroll_method: '',
			scroll_button: 0,
			scroll_button_lock: false,
			scroll_factor: 1,
			natural_scroll: false,
			follow_mouse: 1,
			follow_mouse_threshold: 0,
			focus_on_close: 0,
			mouse_refocus: true,
			float_switch_override_focus: 1,
			special_fallthrough: false,
			off_window_axis_events: 1,
			emulate_discrete_scroll: 1
		});
		expect(state.validation.isValid).toBe(true);
	});

	it('validates mouse and touchpad ranges', () => {
		let result = validateHyprlandInputForm(
			{
				...state.form,
				sensitivity: 2,
				scroll_factor: -0.5,
				scroll_button: 1.5,
				follow_mouse: 5,
				follow_mouse_threshold: -10,
				focus_on_close: 3,
				float_switch_override_focus: 9,
				off_window_axis_events: -1,
				emulate_discrete_scroll: 4
			},
			state.catalog
		);

		expect(result.isValid).toBe(false);
		expect(result.fieldErrors.sensitivity).toBeDefined();
		expect(result.fieldErrors.scroll_factor).toBeDefined();
		expect(result.fieldErrors.scroll_button).toBeDefined();
		expect(result.fieldErrors.follow_mouse).toBeDefined();
		expect(result.fieldErrors.follow_mouse_threshold).toBeDefined();
		expect(result.fieldErrors.focus_on_close).toBeDefined();
		expect(result.fieldErrors.float_switch_override_focus).toBeDefined();
		expect(result.fieldErrors.off_window_axis_events).toBeDefined();
		expect(result.fieldErrors.emulate_discrete_scroll).toBeDefined();

		result = validateHyprlandInputForm(
			{
				...state.form,
				sensitivity: -0.75,
				scroll_factor: 1.5,
				scroll_button: 274,
				follow_mouse: 2,
				follow_mouse_threshold: 150,
				focus_on_close: 1,
				float_switch_override_focus: 2,
				off_window_axis_events: 3,
				emulate_discrete_scroll: 2
			},
			state.catalog
		);

		expect(result.isValid).toBe(true);
	});

	it('validates layout and variant selections against catalog', () => {
		let result = validateHyprlandInputForm(state.form, state.catalog);
		expect(result.isValid).toBe(true);

		result = validateHyprlandInputForm({ ...state.form, kb_layout: 'unknown' }, state.catalog);
		expect(result.isValid).toBe(false);
		expect(result.fieldErrors.kb_layout).toBeDefined();

		result = validateHyprlandInputForm({ ...state.form, kb_variant: 'intl' }, state.catalog);
		expect(result.isValid).toBe(true);

		result = validateHyprlandInputForm({ ...state.form, kb_variant: 'colemak' }, state.catalog);
		expect(result.isValid).toBe(false);
		expect(result.fieldErrors.kb_variant).toBeDefined();
	});

	it('flags unknown keyboard option codes', () => {
		const result = validateHyprlandInputForm(
			{ ...state.form, kb_options: 'grp:toggle,unknown:option' },
			state.catalog
		);
		expect(result.isValid).toBe(false);
		expect(result.fieldErrors.kb_options).toMatch(/unknown/i);
	});

	it('validates repeat rate and delay ranges', () => {
		let result = validateHyprlandInputForm({ ...state.form, repeat_rate: 0 }, state.catalog);
		expect(result.isValid).toBe(false);
		expect(result.fieldErrors.repeat_rate).toBeDefined();

		result = validateHyprlandInputForm({ ...state.form, repeat_delay: 50 }, state.catalog);
		expect(result.isValid).toBe(false);
		expect(result.fieldErrors.repeat_delay).toBeDefined();

		result = validateHyprlandInputForm(
			{ ...state.form, repeat_rate: 60, repeat_delay: 500 },
			state.catalog
		);
		expect(result.isValid).toBe(true);

		result = validateHyprlandInputForm(
			{ ...state.form, repeat_rate: 20.7, repeat_delay: 750.3 },
			state.catalog
		);
		expect(result.isValid).toBe(false);
		expect(result.fieldErrors.repeat_rate).toBeDefined();
		expect(result.fieldErrors.repeat_delay).toBeDefined();
	});

	it('supports toggling options in the form value', () => {
		toggleOptionInForm(state, 'grp:toggle');
		expect(state.form.kb_options).toBe('grp:toggle');
		toggleOptionInForm(state, 'grp:toggle');
		expect(state.form.kb_options).toBe('');
	});

	it('returns variants for a given layout', () => {
		const variants = getVariantsForLayout(state, 'us');
		expect(Array.isArray(variants)).toBe(true);
		expect(variants[0]).toMatchObject({ name: 'intl' });
	});

	it('builds a selected options set from form value', () => {
		state.form.kb_options = 'grp:toggle,grp:other';
		const options = getSelectedOptionsSet(state.form);
		expect(options.has('grp:toggle')).toBe(true);
		expect(options.has('grp:other')).toBe(true);
	});

	it('normalizes keyboard catalog data with prepareCatalog', () => {
		const catalog = prepareCatalog({
			models: [
				{ name: 'pc105', description: 'Generic 105-key PC' },
				{ name: 'pc101', description: 'Generic 101-key PC' },
				{ name: 'pc105', description: 'Duplicate entry' },
				{ name: '', description: 'Invalid entry' },
				{ name: 'pc110', description: '' }
			],
			layouts: [
				{
					name: 'us',
					description: 'English (US)',
					variants: [
						{ name: 'intl', description: 'English (US, intl.)' },
						{ name: 'intl', description: 'Duplicate variant' },
						{ name: '', description: 'Invalid variant' }
					]
				},
				{
					name: 'gb',
					description: 'English (UK)',
					variants: []
				},
				{
					name: 'gb',
					description: '',
					variants: [{ name: 'colemak', description: 'Colemak' }]
				}
			],
			option_groups: [
				{
					name: 'grp',
					description: 'Group switching',
					options: [
						{ name: 'toggle', description: 'Toggle group' },
						{ name: 'toggle', description: 'Duplicate option' }
					]
				},
				{
					name: 'grp',
					description: '',
					options: [{ name: 'ctrl_alt_toggle', description: 'Ctrl+Alt Toggle' }]
				},
				{
					name: '',
					description: 'Invalid group',
					options: [{ name: 'ignored', description: 'Ignored' }]
				}
			]
		});

		expect(catalog.models.map((model) => model.name)).toEqual(['pc101', 'pc105', 'pc110']);
		expect(catalog.layouts.map((layout) => layout.name)).toEqual(['gb', 'us']);
		expect(catalog.variantsByLayout.us.map((variant) => variant.name)).toEqual(['intl']);
		expect(catalog.variantsByLayout.gb.map((variant) => variant.name)).toEqual(['colemak']);
		expect(catalog.optionGroups[0].name).toBe('grp');
		expect(catalog.optionGroups[0].options.map((option) => option.name)).toEqual([
			'ctrl_alt_toggle',
			'toggle'
		]);
		expect(catalog.optionCodes.has('grp:toggle')).toBe(true);
		expect(catalog.optionCodes.has('grp:ctrl_alt_toggle')).toBe(true);
	});
});
