/**
 * Module Registry - Maps module IDs to their schemas and configurations
 * This provides a centralized place to define all Waybar module behaviors
 */

import { clockSchema } from './schemas/clockSchema.js';
import { batterySchema } from './schemas/batterySchema.js';
import { networkSchema } from './schemas/networkSchema.js';
import { bluetoothSchema } from './schemas/bluetoothSchema.js';
import { cpuSchema } from './schemas/cpuSchema.js';
import { pulseaudioSchema } from './schemas/pulseaudioSchema.js';
import ClockModuleForm from '../../status-bar/modules/ClockModuleForm.svelte';
import NetworkModuleForm from '../../status-bar/modules/NetworkModuleForm.svelte';
import BluetoothModuleForm from '../../status-bar/modules/BluetoothModuleForm.svelte';
import BatteryModuleForm from '../../status-bar/modules/BatteryModuleForm.svelte';
import CpuModuleForm from '../../status-bar/modules/CpuModuleForm.svelte';
import PulseaudioModuleForm from '../../status-bar/modules/PulseaudioModuleForm.svelte';

/**
 * Registry of all supported Waybar modules with their schemas
 * Each entry can specify:
 * - schema: JSON schema definition for the module
 * - component: Optional custom Svelte component (null = use generic renderer)
 * - validator: Optional custom validation function
 * - configurable: Whether the module has user-facing configuration (default: true if schema exists)
 */
export const moduleRegistry = {
	clock: {
		schema: clockSchema,
		component: ClockModuleForm,
		validator: null,
		configurable: true
	},
	battery: {
		schema: batterySchema,
		component: BatteryModuleForm,
		validator: null,
		configurable: true
	},
	bluetooth: {
		schema: bluetoothSchema,
		component: BluetoothModuleForm,
		validator: null,
		configurable: true
	},
	cpu: {
		schema: cpuSchema,
		component: CpuModuleForm,
		validator: null,
		configurable: true
	},
	pulseaudio: {
		schema: pulseaudioSchema,
		component: PulseaudioModuleForm,
		validator: null,
		configurable: true
	},
	network: {
		schema: networkSchema,
		component: NetworkModuleForm,
		validator: null,
		configurable: true
	},
	// Non-configurable modules (no user settings)
	'custom/omarchy-menu': {
		schema: null,
		component: null,
		validator: null,
		configurable: false
	},
	'custom/updates': {
		schema: null,
		component: null,
		validator: null,
		configurable: false
	},
	'custom/screen-recorder': {
		schema: null,
		component: null,
		validator: null,
		configurable: false
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

/**
 * Check if a module is user-configurable
 * @param {string} moduleId - The Waybar module identifier
 * @returns {boolean} True if the module can be configured by users
 */
export function isModuleConfigurable(moduleId) {
	const def = getModuleDefinition(moduleId);
	if (!def) {
		return false;
	}
	// Explicitly set configurable flag, or default to true if schema exists
	if (def.configurable !== undefined) {
		return def.configurable;
	}
	return def.schema ? true : false;
}
