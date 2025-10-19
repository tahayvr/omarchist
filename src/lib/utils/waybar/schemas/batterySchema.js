/**
 * Battery module schema definition for Waybar
 * See: https://github.com/Alexays/Waybar/wiki/Module:-Battery
 */

export const batterySchema = {
	type: 'object',
	title: 'Battery',
	description: 'Display battery status and charge level',
	properties: {
		// General settings
		format: {
			type: 'string',
			title: 'Format',
			description: 'Default format string for battery display',
			default: '{capacity}% {icon}',
			tab: 'general'
		},
		'format-charging': {
			type: 'string',
			title: 'Charging Format',
			description: 'Format when battery is charging',
			default: '{icon}',
			tab: 'general'
		},
		'format-discharging': {
			type: 'string',
			title: 'Discharging Format',
			description: 'Format when battery is discharging',
			default: '{icon}',
			tab: 'general'
		},
		'format-plugged': {
			type: 'string',
			title: 'Plugged Format',
			description: 'Format when plugged in and fully charged',
			default: '',
			tab: 'general'
		},
		'format-full': {
			type: 'string',
			title: 'Full Format',
			description: 'Format when battery is full',
			default: '󰂅',
			tab: 'general'
		},
		interval: {
			type: 'integer',
			title: 'Update Interval',
			description: 'How often to check battery status (seconds)',
			default: 5,
			minimum: 1,
			maximum: 60,
			tab: 'general'
		},
		'max-length': {
			type: 'integer',
			title: 'Max Length',
			description: 'Maximum characters to display',
			minimum: 1,
			maximum: 100,
			tab: 'general'
		},
		rotate: {
			type: 'integer',
			title: 'Rotation',
			description: 'Degrees to rotate the module',
			enum: [0, 90, 180, 270],
			tab: 'general'
		},

		// Tooltip settings
		tooltip: {
			type: 'boolean',
			title: 'Show Tooltip',
			description: 'Enable tooltip on hover',
			default: true,
			tab: 'tooltip'
		},
		'tooltip-format': {
			type: 'string',
			title: 'Tooltip Format',
			description: 'Custom format for tooltip text',
			default: '',
			tab: 'tooltip'
		},
		'tooltip-format-charging': {
			type: 'string',
			title: 'Charging Tooltip',
			description: 'Tooltip format when charging',
			default: '{power:>1.0f}W↑ {capacity}%',
			tab: 'tooltip'
		},
		'tooltip-format-discharging': {
			type: 'string',
			title: 'Discharging Tooltip',
			description: 'Tooltip format when discharging',
			default: '{power:>1.0f}W↓ {capacity}%',
			tab: 'tooltip'
		},

		// Actions
		'on-click': {
			type: 'string',
			title: 'Click Action',
			description: 'Command to run on left click',
			default: '',
			tab: 'actions'
		},
		'on-click-middle': {
			type: 'string',
			title: 'Middle Click',
			description: 'Command to run on middle click',
			default: '',
			tab: 'actions'
		},
		'on-click-right': {
			type: 'string',
			title: 'Right Click',
			description: 'Command to run on right click',
			default: '',
			tab: 'actions'
		},
		'on-scroll-up': {
			type: 'string',
			title: 'Scroll Up',
			description: 'Command to run on scroll up',
			default: '',
			tab: 'actions'
		},
		'on-scroll-down': {
			type: 'string',
			title: 'Scroll Down',
			description: 'Command to run on scroll down',
			default: '',
			tab: 'actions'
		},

		// States (thresholds)
		'states.warning': {
			type: 'integer',
			title: 'Warning Threshold',
			description: 'Battery percentage for warning state',
			default: 20,
			minimum: 0,
			maximum: 100,
			tab: 'states'
		},
		'states.critical': {
			type: 'integer',
			title: 'Critical Threshold',
			description: 'Battery percentage for critical state',
			default: 10,
			minimum: 0,
			maximum: 100,
			tab: 'states'
		}
	},
	tabs: [
		{
			id: 'general',
			label: 'General',
			description: 'Basic battery display and format settings'
		},
		{
			id: 'tooltip',
			label: 'Tooltip',
			description: 'Tooltip format and display options'
		},
		{
			id: 'actions',
			label: 'Actions',
			description: 'Mouse and keyboard interactions'
		},
		{
			id: 'states',
			label: 'States',
			description: 'Battery level thresholds for visual states'
		}
	]
};
