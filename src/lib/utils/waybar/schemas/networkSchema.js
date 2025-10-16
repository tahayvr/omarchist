/**
 * Network module schema definition for Waybar
 * See: https://github.com/Alexays/Waybar/wiki/Module:-Network
 */

export const networkSchema = {
	type: 'object',
	title: 'Network',
	description: 'Display network connection status and information',
	properties: {
		// General settings
		format: {
			type: 'string',
			title: 'Default Format',
			description: 'Format when connected',
			default: '{icon}',
			tab: 'general'
		},
		'format-wifi': {
			type: 'string',
			title: 'WiFi Format',
			description: 'Format for WiFi connections',
			default: '{icon}',
			tab: 'general'
		},
		'format-ethernet': {
			type: 'string',
			title: 'Ethernet Format',
			description: 'Format for wired connections',
			default: '󰀂',
			tab: 'general'
		},
		'format-disconnected': {
			type: 'string',
			title: 'Disconnected Format',
			description: 'Format when no connection',
			default: '󰤮',
			tab: 'general'
		},
		'format-linked': {
			type: 'string',
			title: 'Linked Format',
			description: 'Format when connected but no internet',
			default: '{icon} (No IP)',
			tab: 'general'
		},
		interval: {
			type: 'integer',
			title: 'Update Interval',
			description: 'How often to check network status (seconds)',
			default: 3,
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
			description: 'Custom tooltip format',
			default: '',
			tab: 'tooltip'
		},
		'tooltip-format-wifi': {
			type: 'string',
			title: 'WiFi Tooltip',
			description: 'Tooltip format for WiFi',
			default: '{essid} ({frequency} GHz)\n⇣{bandwidthDownBytes}  ⇡{bandwidthUpBytes}',
			tab: 'tooltip'
		},
		'tooltip-format-ethernet': {
			type: 'string',
			title: 'Ethernet Tooltip',
			description: 'Tooltip format for ethernet',
			default: '⇣{bandwidthDownBytes}  ⇡{bandwidthUpBytes}',
			tab: 'tooltip'
		},
		'tooltip-format-disconnected': {
			type: 'string',
			title: 'Disconnected Tooltip',
			description: 'Tooltip when disconnected',
			default: 'Disconnected',
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
		}
	},
	tabs: [
		{
			id: 'general',
			label: 'General',
			description: 'Network display and format settings'
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
		}
	]
};
