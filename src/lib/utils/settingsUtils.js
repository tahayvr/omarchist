import { invoke } from '@tauri-apps/api/core';

/**
 * Load settings from backend and update component state
 * @param {Object} state - Component state object with settings properties
 * @returns {Promise<boolean>} Success status
 */
export async function loadSettings(state) {
	console.log('🔧 SettingsUtils: Loading settings from backend');
	state.isLoading = true;
	state.error = null;

	try {
		const result = await invoke('get_app_settings');

		// Validate the loaded settings structure
		if (typeof result !== 'object' || result === null) {
			throw new Error('Invalid settings format received from backend');
		}

		if (typeof result.auto_apply_theme !== 'boolean') {
			console.warn('🔧 SettingsUtils: Invalid auto_apply_theme value, using default');
			result.auto_apply_theme = true;
		}

		state.autoApplyTheme = result.auto_apply_theme;
		state.isInitialized = true;
		console.log('🔧 SettingsUtils: Settings loaded successfully:', result);
		return true;
	} catch (error) {
		console.error('🔧 SettingsUtils: Failed to load settings:', error);

		// Provide user-friendly error messages
		let errorMessage = 'Failed to load settings. Using default values.';
		if (typeof error === 'string') {
			if (error.includes('corrupted')) {
				errorMessage = 'Settings file was corrupted and has been reset to defaults.';
			} else if (error.includes('permissions')) {
				errorMessage = 'Unable to read settings file. Check file permissions.';
			} else if (error.includes('invalid')) {
				errorMessage = 'Settings file format is invalid. Using defaults.';
			} else {
				errorMessage = error;
			}
		}

		state.error = errorMessage;

		// Keep default values on error - graceful fallback
		state.autoApplyTheme = true; // Ensure we have a valid default
		state.isInitialized = true; // Mark as initialized even on error to prevent infinite loading
		return false;
	} finally {
		state.isLoading = false;
	}
}

/**
 * Validate setting value before updating
 * @param {string} key - Setting key
 * @param {any} value - Value to validate
 * @returns {Object} Validation result with isValid and error message
 */
function validateSettingValue(key, value) {
	console.log(`🔧 SettingsUtils: Validating ${key} = ${value} (type: ${typeof value})`);

	switch (key) {
		case 'autoApplyTheme':
			if (typeof value !== 'boolean') {
				return {
					isValid: false,
					error: 'Auto-apply theme setting must be true or false'
				};
			}
			break;
		default:
			console.warn(`🔧 SettingsUtils: Unknown setting key: ${key}`);
			return {
				isValid: false,
				error: `Unknown setting: ${key}`
			};
	}

	console.log(`🔧 SettingsUtils: Validation passed for ${key}`);
	return { isValid: true, error: null };
}

/**
 * Update a specific setting and persist to backend
 * @param {Object} state - Component state object
 * @param {string} key - Setting key to update
 * @param {any} value - New value for the setting
 * @returns {Promise<boolean>} Success status
 */
export async function updateSetting(state, key, value) {
	console.log(`🔧 SettingsUtils: updateSetting called with key=${key}, value=${value}`);

	// Validate the setting value first
	const validation = validateSettingValue(key, value);
	if (!validation.isValid) {
		console.error(`🔧 SettingsUtils: Validation failed for ${key}:`, validation.error);
		state.error = validation.error;
		return false;
	}

	const previousValue = state[key];

	// Optimistically update UI
	state[key] = value;
	state.isLoading = true;
	state.error = null;

	try {
		// Map frontend keys to backend format
		const backendSettings = {
			auto_apply_theme: key === 'autoApplyTheme' ? value : state.autoApplyTheme
		};

		console.log('🔧 SettingsUtils: Sending to backend:', backendSettings);
		await invoke('update_app_settings', {
			settings: backendSettings
		});

		console.log(`🔧 SettingsUtils: Setting ${key} updated successfully to:`, value);
		return true;
	} catch (error) {
		console.error(`🔧 SettingsUtils: Failed to update setting ${key}:`, error);

		// Revert on failure
		state[key] = previousValue;

		// Provide user-friendly error messages
		let errorMessage = 'Failed to save setting. Please try again.';
		if (typeof error === 'string') {
			if (error.includes('Invalid settings provided')) {
				errorMessage = 'Invalid setting value. Please check your input.';
			} else if (error.includes('file permissions')) {
				errorMessage = 'Unable to save settings. Please check file permissions.';
			} else if (error.includes('corrupted')) {
				errorMessage = 'Settings file is corrupted. Using default values.';
			} else {
				errorMessage = error;
			}
		}

		state.error = errorMessage;
		return false;
	} finally {
		state.isLoading = false;
	}
}

/**
 * Reset settings to defaults
 * @param {Object} state - Component state object
 * @returns {Promise<boolean>} Success status
 */
export async function resetSettings(state) {
	console.log('🔧 SettingsUtils: Resetting settings to defaults');
	state.isLoading = true;
	state.error = null;

	try {
		const result = await invoke('reset_app_settings');

		// Validate the reset result
		if (typeof result !== 'object' || result === null) {
			throw new Error('Invalid response from reset operation');
		}

		if (typeof result.auto_apply_theme !== 'boolean') {
			console.warn('🔧 SettingsUtils: Invalid reset result, using hardcoded default');
			result.auto_apply_theme = true;
		}

		state.autoApplyTheme = result.auto_apply_theme;
		console.log('🔧 SettingsUtils: Settings reset to defaults successfully:', result);
		return true;
	} catch (error) {
		console.error('🔧 SettingsUtils: Failed to reset settings:', error);

		// Provide user-friendly error message
		let errorMessage = 'Failed to reset settings. Please try again.';
		if (typeof error === 'string') {
			if (error.includes('permissions')) {
				errorMessage = 'Unable to reset settings. Check file permissions.';
			} else {
				errorMessage = error;
			}
		}

		state.error = errorMessage;
		return false;
	} finally {
		state.isLoading = false;
	}
}

/**
 * Clear error message from state
 * @param {Object} state - Component state object
 */
export function clearError(state) {
	console.log('🔧 SettingsUtils: Clearing error message');
	state.error = null;
}

/**
 * Validate the entire settings state for consistency
 * @param {Object} state - Component state object
 * @returns {Object} Validation result with isValid and errors array
 */
export function validateSettingsState(state) {
	console.log('🔧 SettingsUtils: Validating settings state:', state);

	const errors = [];

	// Check if state is properly initialized
	if (!state.isInitialized) {
		errors.push('Settings not initialized');
	}

	// Validate individual settings
	if (typeof state.autoApplyTheme !== 'boolean') {
		errors.push('Auto-apply theme must be true or false');
	}

	const isValid = errors.length === 0;
	console.log(
		`🔧 SettingsUtils: State validation result: ${isValid ? 'VALID' : 'INVALID'}`,
		errors
	);

	return {
		isValid,
		errors
	};
}

/**
 * Get a user-friendly description for a setting
 * @param {string} key - Setting key
 * @returns {string} Human-readable description
 */
export function getSettingDescription(key) {
	const descriptions = {
		autoApplyTheme:
			'Automatically applies custom themes when you enter theme edit mode. When enabled, themes will be applied to your system immediately when you start editing them.'
	};

	return descriptions[key] || `Setting: ${key}`;
}
