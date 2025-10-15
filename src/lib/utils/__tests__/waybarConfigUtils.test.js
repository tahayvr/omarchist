import { describe, expect, it } from 'vitest';
import {
	initializeWaybarConfigState,
	setModuleField,
	getModuleFieldValue
} from '../waybarConfigUtils.js';

describe('waybarConfigUtils module field updates', () => {
	it('updates simple string fields and marks state dirty', () => {
		const state = initializeWaybarConfigState();
		const original = getModuleFieldValue(state, 'clock', 'format');

		expect(state.dirty).toBe(false);

		setModuleField(state, 'clock', 'format', '{:%H:%M:%S}');

		expect(getModuleFieldValue(state, 'clock', 'format')).toBe('{:%H:%M:%S}');
		expect(getModuleFieldValue(state, 'clock', 'format')).not.toBe(original);
		expect(state.dirty).toBe(true);
		expect(state.validation?.isValid).toBe(true);
	});

	it('still records repeats when the incoming value matches the current value', () => {
		const state = initializeWaybarConfigState();
		const original = getModuleFieldValue(state, 'clock', 'format');

		setModuleField(state, 'clock', 'format', original);

		expect(getModuleFieldValue(state, 'clock', 'format')).toBe(original);
		expect(state.dirty).toBe(true);
	});

	it('removes fields when explicitly set to null', () => {
		const state = initializeWaybarConfigState();

		setModuleField(state, 'clock', 'calendar.mode', 'year');
		expect(getModuleFieldValue(state, 'clock', 'calendar.mode')).toBe('year');

		setModuleField(state, 'clock', 'calendar.mode', null);
		expect(getModuleFieldValue(state, 'clock', 'calendar.mode')).toBeUndefined();
		expect(state.dirty).toBe(true);
	});

	it('handles array values for module fields', () => {
		const state = initializeWaybarConfigState();
		const zones = ['Etc/UTC', 'Asia/Tokyo'];

		setModuleField(state, 'clock', 'timezones', zones);

		expect(getModuleFieldValue(state, 'clock', 'timezones')).toEqual(zones);
		expect(state.dirty).toBe(true);
	});
});
