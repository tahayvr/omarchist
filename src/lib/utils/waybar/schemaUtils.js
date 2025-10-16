/**
 * Schema utilities for validating and cleaning module configurations
 */

/**
 * Clone a value safely
 */
function clone(value) {
	if (typeof structuredClone === 'function') {
		try {
			return structuredClone(value);
		} catch {
			/* no-op */
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
}

/**
 * Get a nested value from an object using dot notation
 * @param {object} target - The object to read from
 * @param {string} path - The dot-separated path (e.g., 'calendar.mode')
 * @returns {*} The value at the path, or undefined
 */
export function getNestedValue(target, path) {
	if (!target || typeof path !== 'string') {
		return undefined;
	}
	if (!path.includes('.')) {
		return target[path];
	}
	const segments = path.split('.');
	let cursor = target;
	for (const segment of segments) {
		if (!cursor || typeof cursor !== 'object') {
			return undefined;
		}
		cursor = cursor[segment];
	}
	return cursor;
}

/**
 * Set a nested value in an object using dot notation
 * @param {object} target - The object to modify
 * @param {string} path - The dot-separated path
 * @param {*} value - The value to set
 */
export function setNestedValue(target, path, value) {
	if (!path || typeof path !== 'string') {
		return;
	}
	if (value === null || value === undefined) {
		removeNestedValue(target, path);
		return;
	}
	if (!path.includes('.')) {
		target[path] = value;
		return;
	}
	const segments = path.split('.');
	let cursor = target;
	for (let i = 0; i < segments.length - 1; i++) {
		const segment = segments[i];
		if (!cursor[segment] || typeof cursor[segment] !== 'object') {
			cursor[segment] = {};
		}
		cursor = cursor[segment];
	}
	cursor[segments[segments.length - 1]] = value;
}

/**
 * Remove a nested value from an object and prune empty branches
 * @param {object} target - The object to modify
 * @param {string} path - The dot-separated path
 */
export function removeNestedValue(target, path) {
	if (!target || typeof target !== 'object' || !path) {
		return;
	}
	if (!path.includes('.')) {
		delete target[path];
		return;
	}
	const segments = path.split('.');
	let cursor = target;
	const stack = [];

	// Navigate to the parent of the target field
	for (let i = 0; i < segments.length - 1; i++) {
		const segment = segments[i];
		stack.push({ obj: cursor, key: segment });
		if (!cursor[segment] || typeof cursor[segment] !== 'object') {
			return;
		}
		cursor = cursor[segment];
	}

	// Delete the target field
	delete cursor[segments[segments.length - 1]];

	// Prune empty parent objects
	for (let i = stack.length - 1; i >= 0; i--) {
		const { obj, key } = stack[i];
		if (obj[key] && typeof obj[key] === 'object' && Object.keys(obj[key]).length === 0) {
			delete obj[key];
		} else {
			break;
		}
	}
}

/**
 * Validate and clean a config value based on schema field definition
 * @param {*} value - The value to validate
 * @param {object} field - The schema field definition
 * @returns {*} The cleaned value or null if invalid
 */
export function validateFieldValue(value, field) {
	if (value === null || value === undefined || value === '') {
		return null;
	}

	const type = field.type;

	// Handle special format types
	if (field.format === 'textarea' && type === 'array') {
		// Textarea for arrays (e.g., timezones)
		if (typeof value === 'string') {
			const items = value
				.split('\n')
				.map((line) => line.trim())
				.filter((line) => line.length > 0);
			return items.length > 0 ? items : null;
		}
		if (Array.isArray(value)) {
			const items = value.filter((item) => typeof item === 'string' && item.trim().length > 0);
			return items.length > 0 ? items : null;
		}
		return null;
	}

	// Handle select types with __default sentinel
	if (field.type === 'select' || (field.enum && Array.isArray(field.enum))) {
		if (typeof value === 'string') {
			if (value === '__default' || value === '') {
				return null;
			}
			if (field.enum.includes(value)) {
				return value;
			}
		}
		return null;
	}

	// Handle primitive types
	switch (type) {
		case 'string':
			return typeof value === 'string' && value.length > 0 ? value : null;

		case 'integer':
		case 'number': {
			const num = type === 'integer' ? parseInt(value, 10) : parseFloat(value);
			if (!Number.isFinite(num)) {
				return null;
			}
			if (field.minimum !== undefined && num < field.minimum) {
				return null;
			}
			if (field.maximum !== undefined && num > field.maximum) {
				return null;
			}
			if (field.enum && !field.enum.includes(num)) {
				return null;
			}
			return num;
		}

		case 'boolean':
			return typeof value === 'boolean' ? value : null;

		case 'array':
			return Array.isArray(value) && value.length > 0 ? value : null;

		default:
			return value;
	}
}

/**
 * Clean a module config by removing default values and invalid fields
 * @param {object} config - The raw config object
 * @param {object} schema - The module schema
 * @returns {object} The cleaned config object
 */
export function cleanModuleConfig(config, schema) {
	if (!config || typeof config !== 'object') {
		return {};
	}
	if (!schema || !schema.properties) {
		return clone(config);
	}

	const cleaned = {};

	// Process each field in the schema
	for (const [key, field] of Object.entries(schema.properties)) {
		const value = getNestedValue(config, key);
		const validatedValue = validateFieldValue(value, field);

		if (validatedValue !== null) {
			setNestedValue(cleaned, key, validatedValue);
		}
	}

	return cleaned;
}

/**
 * Hydrate a field state object from a config, filling in defaults from schema
 * @param {object} config - The config object
 * @param {object} schema - The module schema
 * @returns {object} Field state object with values for all schema fields
 */
export function hydrateFieldState(config, schema) {
	const state = {};

	if (!schema || !schema.properties) {
		return state;
	}

	for (const [key, field] of Object.entries(schema.properties)) {
		const value = getNestedValue(config, key);

		// Handle special textarea format for arrays
		if (field.format === 'textarea' && field.type === 'array') {
			if (Array.isArray(value)) {
				state[key] = value.join('\n');
			} else if (typeof value === 'string') {
				state[key] = value;
			} else {
				state[key] = '';
			}
			continue;
		}

		// Handle select types with __default sentinel
		if (field.type === 'select' || (field.enum && Array.isArray(field.enum))) {
			if (value === null || value === undefined || value === '') {
				state[key] = '__default';
			} else if (typeof value === 'string') {
				state[key] = value;
			} else {
				state[key] = '__default';
			}
			continue;
		}

		// Handle primitive types
		if (value !== null && value !== undefined) {
			state[key] = value;
		} else if (field.default !== undefined) {
			state[key] = field.default;
		} else {
			// Set appropriate empty value based on type
			switch (field.type) {
				case 'string':
					state[key] = '';
					break;
				case 'integer':
				case 'number':
					state[key] = '';
					break;
				case 'boolean':
					state[key] = false;
					break;
				case 'array':
					state[key] = [];
					break;
				default:
					state[key] = '';
			}
		}
	}

	return state;
}

/**
 * Build a module config from field state
 * @param {object} fieldState - The field state object
 * @param {object} schema - The module schema
 * @returns {object} The resulting config object
 */
export function buildConfigFromFieldState(fieldState, schema) {
	if (!fieldState || typeof fieldState !== 'object') {
		return {};
	}
	if (!schema || !schema.properties) {
		return clone(fieldState);
	}

	const config = {};

	for (const [key, field] of Object.entries(schema.properties)) {
		const value = fieldState[key];
		const validatedValue = validateFieldValue(value, field);

		if (validatedValue !== null) {
			setNestedValue(config, key, validatedValue);
		}
	}

	return config;
}
