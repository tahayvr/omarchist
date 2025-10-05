import { describe, it, expect, beforeEach } from 'vitest';
import {
	initializeHyprlandDecorationState,
	validateHyprlandDecorationForm,
	recomputeDirty,
	resetHyprlandDecorationToDefaults
} from '../hyprlandDecorationUtils.js';

describe('hyprlandDecorationUtils', () => {
	let baseState;

	beforeEach(() => {
		baseState = initializeHyprlandDecorationState();
	});

	describe('initializeHyprlandDecorationState', () => {
		it('returns defaults with valid initial validation', () => {
			expect(baseState.form).toMatchObject({
				rounding: 0,
				border_part_of_window: true,
				blur: {
					enabled: true,
					size: 8,
					passes: 1
				},
				shadow: {
					enabled: true,
					range: 4,
					render_power: 3
				}
			});
			expect(baseState.validation.isValid).toBe(true);
			expect(baseState.dirty).toBe(false);
		});
	});

	describe('validateHyprlandDecorationForm', () => {
		it('flags invalid numeric ranges', () => {
			const result = validateHyprlandDecorationForm({
				...baseState.form,
				rounding: -2,
				active_opacity: 1.4,
				blur: {
					...baseState.form.blur,
					size: 0,
					noise: 1.8
				},
				shadow: {
					...baseState.form.shadow,
					render_power: 8,
					offset: ''
				}
			});

			expect(result.isValid).toBe(false);
			expect(result.fieldErrors.rounding).toBeDefined();
			expect(result.fieldErrors.active_opacity).toBeDefined();
			expect(result.fieldErrors['blur.size']).toBeDefined();
			expect(result.fieldErrors['blur.noise']).toBeDefined();
			expect(result.fieldErrors['shadow.render_power']).toBeDefined();
			expect(result.fieldErrors['shadow.offset']).toBeDefined();
		});

		it('accepts valid override payload', () => {
			const valid = validateHyprlandDecorationForm(baseState.form);
			expect(valid.isValid).toBe(true);
		});
	});

	describe('recomputeDirty', () => {
		it('sets dirty when form changes', () => {
			recomputeDirty(baseState);
			expect(baseState.dirty).toBe(false);

			baseState.form.rounding = 6;
			recomputeDirty(baseState);
			expect(baseState.dirty).toBe(true);

			baseState.dirty = false;
			baseState.lastSavedForm = {
				...baseState.form,
				blur: { ...baseState.form.blur },
				shadow: { ...baseState.form.shadow }
			};
			baseState.form.blur.size = 12;
			recomputeDirty(baseState);
			expect(baseState.dirty).toBe(true);
		});
	});

	describe('resetHyprlandDecorationToDefaults', () => {
		it('restores default values and marks state dirty', () => {
			baseState.form.rounding = 12;
			baseState.form.blur.enabled = false;
			baseState.form.shadow.range = 10;
			recomputeDirty(baseState);
			expect(baseState.dirty).toBe(true);

			resetHyprlandDecorationToDefaults(baseState);
			expect(baseState.form.rounding).toBe(0);
			expect(baseState.form.blur.enabled).toBe(true);
			expect(baseState.form.shadow.range).toBe(4);
			expect(baseState.validation.isValid).toBe(true);
			expect(baseState.dirty).toBe(true);
		});
	});
});
