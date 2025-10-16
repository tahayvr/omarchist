/**
 * Clock module schema definition for Waybar
 * See: https://github.com/Alexays/Waybar/wiki/Module:-Clock
 */

export const clockSchema = {
	type: 'object',
	title: 'Clock',
	description: 'Localized time display with alternate format',
	properties: {
		// General settings
		format: {
			type: 'string',
			title: 'Time Format',
			description: 'Primary format string for the clock display',
			default: '{:%H:%M}',
			tab: 'general'
		},
		'format-alt': {
			type: 'string',
			title: 'Alternate Format',
			description: 'Format displayed when clicking the clock',
			default: '{:%a %d %b}',
			tab: 'general'
		},
		timezone: {
			type: 'string',
			title: 'Timezone',
			description: 'Timezone identifier (e.g., America/New_York)',
			default: '',
			placeholder: 'Local timezone',
			tab: 'general'
		},
		timezones: {
			type: 'array',
			title: 'Additional Timezones',
			description: 'List of timezones to cycle through (one per line)',
			items: {
				type: 'string'
			},
			format: 'textarea',
			tab: 'general'
		},
		locale: {
			type: 'string',
			title: 'Locale',
			description: 'Locale for date formatting (e.g., en_US.UTF-8)',
			default: '',
			placeholder: 'System locale',
			tab: 'general'
		},
		interval: {
			type: 'integer',
			title: 'Update Interval',
			description: 'How often to update the clock (seconds)',
			default: 60,
			minimum: 1,
			maximum: 3600,
			tab: 'general'
		},
		'max-length': {
			type: 'integer',
			title: 'Max Length',
			description: 'Maximum characters to display',
			minimum: 1,
			maximum: 200,
			tab: 'general'
		},
		rotate: {
			type: 'integer',
			title: 'Rotation',
			description: 'Degrees to rotate the module (0, 90, 180, 270)',
			enum: [0, 90, 180, 270],
			tab: 'general'
		},
		tooltip: {
			type: 'boolean',
			title: 'Show Tooltip',
			description: 'Enable tooltip on hover',
			default: true,
			tab: 'general'
		},
		'tooltip-format': {
			type: 'string',
			title: 'Tooltip Format',
			description: 'Custom format for tooltip text',
			default: '',
			tab: 'general'
		},
		'smooth-scrolling-threshold': {
			type: 'number',
			title: 'Smooth Scrolling Threshold',
			description: 'Threshold for smooth scrolling',
			minimum: 0,
			tab: 'general'
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
			type: 'select',
			title: 'Right Click',
			description: 'Action for right click',
			enum: ['__default', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'Default',
				'Switch Calendar Mode',
				'Time Zone Next',
				'Time Zone Previous',
				'Calendar Forward',
				'Calendar Back',
				'Calendar Reset'
			],
			default: '__default',
			tab: 'actions'
		},
		'on-scroll-up': {
			type: 'select',
			title: 'Scroll Up',
			description: 'Action for scroll up',
			enum: ['__default', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'Default',
				'Switch Calendar Mode',
				'Time Zone Next',
				'Time Zone Previous',
				'Calendar Forward',
				'Calendar Back',
				'Calendar Reset'
			],
			default: '__default',
			tab: 'actions'
		},
		'on-scroll-down': {
			type: 'select',
			title: 'Scroll Down',
			description: 'Action for scroll down',
			enum: ['__default', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'Default',
				'Switch Calendar Mode',
				'Time Zone Next',
				'Time Zone Previous',
				'Calendar Forward',
				'Calendar Back',
				'Calendar Reset'
			],
			default: '__default',
			tab: 'actions'
		},
		'on-click-forward': {
			type: 'select',
			title: 'Mouse Forward Button',
			description: 'Action for mouse forward button',
			enum: ['__default', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'Default',
				'Switch Calendar Mode',
				'Time Zone Next',
				'Time Zone Previous',
				'Calendar Forward',
				'Calendar Back',
				'Calendar Reset'
			],
			default: '__default',
			tab: 'actions'
		},
		'on-click-backward': {
			type: 'select',
			title: 'Mouse Back Button',
			description: 'Action for mouse back button',
			enum: ['__default', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'Default',
				'Switch Calendar Mode',
				'Time Zone Next',
				'Time Zone Previous',
				'Calendar Forward',
				'Calendar Back',
				'Calendar Reset'
			],
			default: '__default',
			tab: 'actions'
		},

		// Calendar settings
		'calendar.mode': {
			type: 'select',
			title: 'Calendar Mode',
			description: 'Initial calendar display mode',
			enum: ['__default', 'year', 'month'],
			enumLabels: ['Default (Month)', 'Year', 'Month'],
			default: '__default',
			tab: 'calendar'
		},
		'calendar.mode-mon-col': {
			type: 'integer',
			title: 'Months Per Column',
			description: 'Number of months per column in year view',
			minimum: 1,
			maximum: 12,
			default: 3,
			tab: 'calendar'
		},
		'calendar.weeks-pos': {
			type: 'select',
			title: 'Week Numbers Position',
			description: 'Where to show week numbers',
			enum: ['__default', 'left', 'right'],
			enumLabels: ['Default (Hidden)', 'Left', 'Right'],
			default: '__default',
			tab: 'calendar'
		},
		'calendar.on-scroll': {
			type: 'integer',
			title: 'Calendar Scroll',
			description: 'Number of months to scroll per action',
			minimum: 1,
			maximum: 12,
			default: 1,
			tab: 'calendar'
		},
		'calendar.format.months': {
			type: 'string',
			title: 'Month Format',
			description: 'Format for month headers',
			default: '',
			placeholder: '<span color="#ffead3"><b>{}</b></span>',
			tab: 'calendar'
		},
		'calendar.format.days': {
			type: 'string',
			title: 'Day Format',
			description: 'Format for day numbers',
			default: '',
			placeholder: '<span color="#ecc6d9"><b>{}</b></span>',
			tab: 'calendar'
		},
		'calendar.format.weeks': {
			type: 'string',
			title: 'Week Format',
			description: 'Format for week numbers',
			default: '',
			placeholder: '<span color="#99ffdd"><b>W{}</b></span>',
			tab: 'calendar'
		},
		'calendar.format.weekdays': {
			type: 'string',
			title: 'Weekday Format',
			description: 'Format for weekday headers',
			default: '',
			placeholder: '<span color="#ffcc66"><b>{}</b></span>',
			tab: 'calendar'
		},
		'calendar.format.today': {
			type: 'string',
			title: 'Today Format',
			description: 'Format for current day',
			default: '',
			placeholder: '<span color="#ff6699"><b><u>{}</u></b></span>',
			tab: 'calendar'
		},
		'actions.on-click-right': {
			type: 'select',
			title: 'Right Click Calendar',
			description: 'Calendar action on right click',
			enum: ['__default', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'Default',
				'Switch Calendar Mode',
				'Time Zone Next',
				'Time Zone Previous',
				'Calendar Forward',
				'Calendar Back',
				'Calendar Reset'
			],
			default: '__default',
			tab: 'calendar'
		},
		'actions.on-click-middle': {
			type: 'select',
			title: 'Middle Click Calendar',
			description: 'Calendar action on middle click',
			enum: ['__default', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'Default',
				'Switch Calendar Mode',
				'Time Zone Next',
				'Time Zone Previous',
				'Calendar Forward',
				'Calendar Back',
				'Calendar Reset'
			],
			default: '__default',
			tab: 'calendar'
		},
		'actions.on-scroll-up': {
			type: 'select',
			title: 'Scroll Up Calendar',
			description: 'Calendar action on scroll up',
			enum: ['__default', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'Default',
				'Switch Calendar Mode',
				'Time Zone Next',
				'Time Zone Previous',
				'Calendar Forward',
				'Calendar Back',
				'Calendar Reset'
			],
			default: '__default',
			tab: 'calendar'
		},
		'actions.on-scroll-down': {
			type: 'select',
			title: 'Scroll Down Calendar',
			description: 'Calendar action on scroll down',
			enum: ['__default', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'Default',
				'Switch Calendar Mode',
				'Time Zone Next',
				'Time Zone Previous',
				'Calendar Forward',
				'Calendar Back',
				'Calendar Reset'
			],
			default: '__default',
			tab: 'calendar'
		}
	},
	tabs: [
		{
			id: 'general',
			label: 'General',
			description: 'Basic clock display and update settings'
		},
		{
			id: 'actions',
			label: 'Actions',
			description: 'Mouse and keyboard interactions'
		},
		{
			id: 'calendar',
			label: 'Calendar',
			description: 'Calendar popup configuration'
		}
	]
};
