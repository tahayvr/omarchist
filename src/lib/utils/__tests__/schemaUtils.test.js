import { describe, it, expect } from 'vitest';
import {
	getNestedValue,
	setNestedValue,
	removeNestedValue,
	validateFieldValue,
	cleanModuleConfig,
	hydrateFieldState,
	buildConfigFromFieldState
} from '../waybar/schemaUtils.js';

describe('schemaUtils', () => {
	describe('getNestedValue', () => {
		it('should get top-level values', () => {
			const obj = { foo: 'bar' };
			expect(getNestedValue(obj, 'foo')).toBe('bar');
		});

		it('should get nested values', () => {
			const obj = { calendar: { mode: 'year' } };
			expect(getNestedValue(obj, 'calendar.mode')).toBe('year');
		});

		it('should return undefined for missing paths', () => {
			const obj = { foo: 'bar' };
			expect(getNestedValue(obj, 'missing.path')).toBeUndefined();
		});
	});

	describe('setNestedValue', () => {
		it('should set top-level values', () => {
			const obj = {};
			setNestedValue(obj, 'foo', 'bar');
			expect(obj.foo).toBe('bar');
		});

		it('should set nested values', () => {
			const obj = {};
			setNestedValue(obj, 'calendar.mode', 'month');
			expect(obj.calendar.mode).toBe('month');
		});

		it('should remove value when setting null', () => {
			const obj = { foo: 'bar' };
			setNestedValue(obj, 'foo', null);
			expect(obj.foo).toBeUndefined();
		});
	});

	describe('removeNestedValue', () => {
		it('should remove top-level values', () => {
			const obj = { foo: 'bar', keep: 'this' };
			removeNestedValue(obj, 'foo');
			expect(obj.foo).toBeUndefined();
			expect(obj.keep).toBe('this');
		});

		it('should remove nested values and prune empty parents', () => {
			const obj = { calendar: { mode: 'year' } };
			removeNestedValue(obj, 'calendar.mode');
			expect(obj.calendar).toBeUndefined();
		});

		it('should not prune parents with other properties', () => {
			const obj = { calendar: { mode: 'year', other: 'value' } };
			removeNestedValue(obj, 'calendar.mode');
			expect(obj.calendar).toBeDefined();
			expect(obj.calendar.other).toBe('value');
		});
	});

	describe('validateFieldValue', () => {
		it('should validate string fields', () => {
			const field = { type: 'string' };
			expect(validateFieldValue('hello', field)).toBe('hello');
			expect(validateFieldValue('', field)).toBeNull();
			expect(validateFieldValue(null, field)).toBeNull();
		});

		it('should validate integer fields', () => {
			const field = { type: 'integer', minimum: 1, maximum: 100 };
			expect(validateFieldValue(50, field)).toBe(50);
			expect(validateFieldValue('75', field)).toBe(75);
			expect(validateFieldValue(0, field)).toBeNull(); // below minimum
			expect(validateFieldValue(200, field)).toBeNull(); // above maximum
		});

		it('should validate number fields', () => {
			const field = { type: 'number', minimum: 0 };
			expect(validateFieldValue(3.14, field)).toBe(3.14);
			expect(validateFieldValue('2.5', field)).toBe(2.5);
			expect(validateFieldValue(-1, field)).toBeNull();
		});

		it('should validate boolean fields', () => {
			const field = { type: 'boolean' };
			expect(validateFieldValue(true, field)).toBe(true);
			expect(validateFieldValue(false, field)).toBe(false);
			expect(validateFieldValue('true', field)).toBeNull();
		});

		it('should validate select fields with __default', () => {
			const field = { type: 'select', enum: ['mode', 'tz_up', 'tz_down'] };
			expect(validateFieldValue('mode', field)).toBe('mode');
			expect(validateFieldValue('__default', field)).toBeNull();
			expect(validateFieldValue('', field)).toBeNull();
			expect(validateFieldValue('invalid', field)).toBeNull();
		});

		it('should validate textarea array fields', () => {
			const field = { type: 'array', format: 'textarea' };
			expect(validateFieldValue('line1\nline2\nline3', field)).toEqual(['line1', 'line2', 'line3']);
			expect(validateFieldValue(['a', 'b'], field)).toEqual(['a', 'b']);
			expect(validateFieldValue('', field)).toBeNull();
			expect(validateFieldValue('  \n  \n  ', field)).toBeNull();
		});
	});

	describe('cleanModuleConfig', () => {
		it('should remove invalid and default values', () => {
			const config = {
				format: '{:%H:%M}',
				interval: 60,
				'calendar.mode': '__default',
				empty: '',
				invalid: null
			};

			const schema = {
				properties: {
					format: { type: 'string' },
					interval: { type: 'integer' },
					'calendar.mode': { type: 'select', enum: ['year', 'month'] },
					empty: { type: 'string' },
					invalid: { type: 'string' }
				}
			};

			const cleaned = cleanModuleConfig(config, schema);
			expect(cleaned.format).toBe('{:%H:%M}');
			expect(cleaned.interval).toBe(60);
			expect(cleaned['calendar.mode']).toBeUndefined();
			expect(cleaned.empty).toBeUndefined();
			expect(cleaned.invalid).toBeUndefined();
		});
	});

	describe('hydrateFieldState', () => {
		it('should hydrate field state with config values', () => {
			const config = {
				format: '{:%H:%M}',
				interval: 60
			};

			const schema = {
				properties: {
					format: { type: 'string', default: '' },
					interval: { type: 'integer', default: 1 },
					missing: { type: 'string', default: 'default-value' }
				}
			};

			const state = hydrateFieldState(config, schema);
			expect(state.format).toBe('{:%H:%M}');
			expect(state.interval).toBe(60);
			expect(state.missing).toBe('default-value');
		});

		it('should convert select values to __default when empty', () => {
			const config = {};
			const schema = {
				properties: {
					mode: { type: 'select', enum: ['year', 'month'] }
				}
			};

			const state = hydrateFieldState(config, schema);
			expect(state.mode).toBe('__default');
		});

		it('should convert arrays to textarea format', () => {
			const config = {
				timezones: ['America/New_York', 'Asia/Tokyo']
			};

			const schema = {
				properties: {
					timezones: { type: 'array', format: 'textarea' }
				}
			};

			const state = hydrateFieldState(config, schema);
			expect(state.timezones).toBe('America/New_York\nAsia/Tokyo');
		});
	});

	describe('buildConfigFromFieldState', () => {
		it('should build config from field state', () => {
			const fieldState = {
				format: '{:%H:%M}',
				interval: 60,
				empty: '',
				selectDefault: '__default'
			};

			const schema = {
				properties: {
					format: { type: 'string' },
					interval: { type: 'integer' },
					empty: { type: 'string' },
					selectDefault: { type: 'select', enum: ['a', 'b'] }
				}
			};

			const config = buildConfigFromFieldState(fieldState, schema);
			expect(config.format).toBe('{:%H:%M}');
			expect(config.interval).toBe(60);
			expect(config.empty).toBeUndefined();
			expect(config.selectDefault).toBeUndefined();
		});
	});
});
