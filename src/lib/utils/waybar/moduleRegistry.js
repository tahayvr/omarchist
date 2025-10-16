/**
 * Module Registry - Maps module IDs to their schemas and configurations
 * This provides a centralized place to define all Waybar module behaviors
 */

import { clockSchema } from './schemas/clockSchema.js';
import { batterySchema } from './schemas/batterySchema.js';
import { networkSchema } from './schemas/networkSchema.js';

/**
 * Registry of all supported Waybar modules with their schemas
 * Each entry can specify:
 * - schema: JSON schema definition for the module
 * - component: Optional custom Svelte component (null = use generic renderer)
 * - validator: Optional custom validation function
 */
export const moduleRegistry = {
	clock: {
		schema: clockSchema,
		component: null,
		validator: null
	},
	battery: {
		schema: batterySchema,
		component: null,
		validator: null
	},
	network: {
		schema: networkSchema,
		component: null,
		validator: null
	}
	// Add more modules as needed
};

/**
 * Get the module definition for a given module ID
 * @param {string} moduleId - The Waybar module identifier
 * @returns {object|null} Module definition with schema, component, and validator
 */
export function getModuleDefinition(moduleId) {
	if (!moduleId || typeof moduleId !== 'string') {
		return null;
	}
	return moduleRegistry[moduleId] || null;
}

/**
 * Check if a module has a schema definition
 * @param {string} moduleId - The Waybar module identifier
 * @returns {boolean} True if the module has a schema
 */
export function hasModuleSchema(moduleId) {
	const def = getModuleDefinition(moduleId);
	return def && def.schema ? true : false;
}

/**
 * Get the schema for a module
 * @param {string} moduleId - The Waybar module identifier
 * @returns {object|null} The module's JSON schema
 */
export function getModuleSchema(moduleId) {
	const def = getModuleDefinition(moduleId);
	return def?.schema || null;
}

/**
 * Get the custom component for a module (if any)
 * @param {string} moduleId - The Waybar module identifier
 * @returns {object|null} The Svelte component or null
 */
export function getModuleComponent(moduleId) {
	const def = getModuleDefinition(moduleId);
	return def?.component || null;
}

/**
 * Get the validator function for a module (if any)
 * @param {string} moduleId - The Waybar module identifier
 * @returns {Function|null} The validator function or null
 */
export function getModuleValidator(moduleId) {
	const def = getModuleDefinition(moduleId);
	return def?.validator || null;
}
