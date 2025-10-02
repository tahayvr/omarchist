import { describe, it, expect, beforeEach } from 'vitest';
import {
	initializeHyprlandGeneralState,
	validateHyprlandGeneralForm,
	recomputeDirty,
	resetHyprlandGeneralToDefaults
} from '../hyprlandGeneralUtils.js';

describe('hyprlandGeneralUtils', () => {
	let baseState;

	beforeEach(() => {
		baseState = initializeHyprlandGeneralState();
	});

	describe('initializeHyprlandGeneralState', () => {
		it('returns defaults with valid initial validation', () => {
			expect(baseState.form).toMatchObject({
				layout: 'dwindle',
				extend_border_grab_area: 15,
				resize_corner: 0
			});
			expect(baseState.validation.isValid).toBe(true);
			expect(baseState.dirty).toBe(false);
		});
	});

	describe('validateHyprlandGeneralForm', () => {
		it('flags invalid layout values', () => {
			const result = validateHyprlandGeneralForm({
				...baseState.form,
				layout: 'spiral'
			});
			expect(result.isValid).toBe(false);
			expect(result.fieldErrors.layout).toBeDefined();
		});

		it('requires non-negative integer for extend_border_grab_area', () => {
			const result = validateHyprlandGeneralForm({
				...baseState.form,
				extend_border_grab_area: -5
			});
			expect(result.isValid).toBe(false);
			expect(result.fieldErrors.extend_border_grab_area).toMatch(/non-negative/);
		});

		it('requires resize_corner to be within range', () => {
			const result = validateHyprlandGeneralForm({
				...baseState.form,
				resize_corner: 7
			});
			expect(result.isValid).toBe(false);
			expect(result.fieldErrors.resize_corner).toBeDefined();
		});
	});

	describe('recomputeDirty', () => {
		it('sets dirty when form differs from last saved snapshot', () => {
			recomputeDirty(baseState);
			expect(baseState.dirty).toBe(false);

			baseState.form.layout = 'master';
			recomputeDirty(baseState);
			expect(baseState.dirty).toBe(true);
		});
	});

	describe('resetHyprlandGeneralToDefaults', () => {
		it('restores default values and invalidates dirty flag', () => {
			baseState.form.layout = 'master';
			baseState.form.extend_border_grab_area = 42;
			recomputeDirty(baseState);
			expect(baseState.dirty).toBe(true);

			resetHyprlandGeneralToDefaults(baseState);
			expect(baseState.form.layout).toBe('dwindle');
			expect(baseState.form.extend_border_grab_area).toBe(15);
			expect(baseState.validation.isValid).toBe(true);
			expect(baseState.dirty).toBe(true);
		});
	});
});
