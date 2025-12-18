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

	for (let i = 0; i < segments.length - 1; i++) {
		const segment = segments[i];
		stack.push({ obj: cursor, key: segment });
		if (!cursor[segment] || typeof cursor[segment] !== 'object') {
			return;
		}
		cursor = cursor[segment];
	}

	delete cursor[segments[segments.length - 1]];

	for (let i = stack.length - 1; i >= 0; i--) {
		const { obj, key } = stack[i];
		if (obj[key] && typeof obj[key] === 'object' && Object.keys(obj[key]).length === 0) {
			delete obj[key];
		} else {
			break;
		}
	}
}

export function validateFieldValue(value, field) {
	if (value === null || value === undefined || value === '') {
		return null;
	}

	const type = field.type;

	if (field.format === 'textarea' && type === 'array') {
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

export function cleanModuleConfig(config, schema) {
	if (!config || typeof config !== 'object') {
		return {};
	}
	if (!schema || !schema.properties) {
		return clone(config);
	}

	const cleaned = {};

	for (const [key, field] of Object.entries(schema.properties)) {
		const value = getNestedValue(config, key);
		const validatedValue = validateFieldValue(value, field);

		if (validatedValue !== null) {
			setNestedValue(cleaned, key, validatedValue);
		}
	}

	return cleaned;
}

export function hydrateFieldState(config, schema) {
	const state = {};

	if (!schema || !schema.properties) {
		return state;
	}

	for (const [key, field] of Object.entries(schema.properties)) {
		if (field.visibleWhen) {
			continue;
		}

		const value = getNestedValue(config, key);

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

		if (field.type === 'select' || (field.enum && Array.isArray(field.enum))) {
			if (value === null || value === undefined || value === '') {
				state[key] = field.default || '__default';
			} else if (typeof value === 'string') {
				if (field.enum && field.enum.includes(value)) {
					state[key] = value;
				} else {
					const customKey = `${key}-custom`;
					if (schema.properties[customKey]) {
						state[key] = '__custom';
						state[customKey] = value;
					} else {
						state[key] = field.default || '__default';
					}
				}
			} else {
				state[key] = field.default || '__default';
			}
			continue;
		}

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

	for (const [key, field] of Object.entries(schema.properties)) {
		if (!field.visibleWhen || key in state) {
			continue;
		}

		const value = getNestedValue(config, key);

		// Handle select types
		if (field.type === 'select' || (field.enum && Array.isArray(field.enum))) {
			if (value === null || value === undefined || value === '') {
				state[key] = field.default || '__default';
			} else if (field.enum && field.enum.includes(value)) {
				state[key] = value;
			} else {
				state[key] = field.default || '__default';
			}
			continue;
		}

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

export function buildConfigFromFieldState(fieldState, schema) {
	if (!fieldState || typeof fieldState !== 'object') {
		return {};
	}
	if (!schema || !schema.properties) {
		return clone(fieldState);
	}

	const config = {};

	for (const [key, field] of Object.entries(schema.properties)) {
		if (field.visibleWhen) {
			const parentField = field.visibleWhen.field;
			const parentValue = fieldState[parentField];
			const expectedValue = field.visibleWhen.value;

			if (parentValue !== expectedValue) {
				continue;
			}
		}

		const value = fieldState[key];

		if (value === '__custom') {
			const customKey = `${key}-custom`;
			const customField = schema.properties[customKey];
			if (customField && fieldState[customKey]) {
				const customValue = validateFieldValue(fieldState[customKey], customField);
				if (customValue !== null) {
					setNestedValue(config, key, customValue);
				}
			}
			continue;
		}

		if (
			value === '__local' ||
			value === '__system' ||
			value === '__default' ||
			value === '__none'
		) {
			continue;
		}

		const validatedValue = validateFieldValue(value, field);

		if (validatedValue !== null) {
			setNestedValue(config, key, validatedValue);
		}
	}

	return config;
}
