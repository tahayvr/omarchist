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
			type: 'select',
			title: 'Time Format',
			description: 'Primary format string for the clock display',
			enum: [
				'{:%H:%M}',
				'{:%I:%M %p}',
				'{:%H:%M:%S}',
				'{:%I:%M:%S %p}',
				'{:%R}',
				'{:%T}',
				'__custom'
			],
			enumLabels: [
				'24-hour (14:30)',
				'12-hour with AM/PM (02:30 PM)',
				'24-hour with seconds (14:30:45)',
				'12-hour with seconds (02:30:45 PM)',
				'Short 24-hour (%R)',
				'Short with seconds (%T)',
				'Custom format...'
			],
			default: '{:%H:%M}',
			tab: 'general'
		},
		'format-custom': {
			type: 'string',
			title: 'Custom Time Format',
			description: 'Enter custom format when "Custom format..." is selected above',
			placeholder: '{:%H:%M}',
			tab: 'general',
			visibleWhen: {
				field: 'format',
				value: '__custom'
			}
		},
		'format-alt': {
			type: 'select',
			title: 'Alternate Format',
			description: 'Format displayed when clicking the clock',
			enum: [
				'{:%a %d %b}',
				'{:%A, %B %d, %Y}',
				'{:%x}',
				'{:%d/%m/%Y}',
				'{:%m/%d/%Y}',
				'{:%Y-%m-%d}',
				'{:%B %d}',
				'__custom'
			],
			enumLabels: [
				'Short date (Mon 17 Oct)',
				'Full date (Monday, October 17, 2025)',
				'Locale date (%x)',
				'DD/MM/YYYY (17/10/2025)',
				'MM/DD/YYYY (10/17/2025)',
				'ISO date (2025-10-17)',
				'Month day (October 17)',
				'Custom format...'
			],
			default: '{:%a %d %b}',
			tab: 'general'
		},
		'format-alt-custom': {
			type: 'string',
			title: 'Custom Alternate Format',
			description: 'Enter custom format when "Custom format..." is selected above',
			placeholder: '{:%a %d %b}',
			tab: 'general',
			visibleWhen: {
				field: 'format-alt',
				value: '__custom'
			}
		},
		timezone: {
			type: 'select',
			title: 'Timezone',
			description: 'Timezone identifier for time display',
			enum: [
				'__local',
				'UTC',
				'America/New_York',
				'America/Chicago',
				'America/Denver',
				'America/Los_Angeles',
				'America/Anchorage',
				'America/Toronto',
				'America/Mexico_City',
				'America/Sao_Paulo',
				'Europe/London',
				'Europe/Paris',
				'Europe/Berlin',
				'Europe/Rome',
				'Europe/Madrid',
				'Europe/Moscow',
				'Europe/Istanbul',
				'Asia/Dubai',
				'Asia/Kolkata',
				'Asia/Bangkok',
				'Asia/Shanghai',
				'Asia/Tokyo',
				'Asia/Seoul',
				'Australia/Sydney',
				'Australia/Melbourne',
				'Pacific/Auckland',
				'__custom'
			],
			enumLabels: [
				'Local timezone',
				'UTC',
				'New York (EST/EDT)',
				'Chicago (CST/CDT)',
				'Denver (MST/MDT)',
				'Los Angeles (PST/PDT)',
				'Anchorage (AKST/AKDT)',
				'Toronto (EST/EDT)',
				'Mexico City (CST/CDT)',
				'São Paulo (BRT)',
				'London (GMT/BST)',
				'Paris (CET/CEST)',
				'Berlin (CET/CEST)',
				'Rome (CET/CEST)',
				'Madrid (CET/CEST)',
				'Moscow (MSK)',
				'Istanbul (TRT)',
				'Dubai (GST)',
				'India (IST)',
				'Bangkok (ICT)',
				'Shanghai (CST)',
				'Tokyo (JST)',
				'Seoul (KST)',
				'Sydney (AEDT/AEST)',
				'Melbourne (AEDT/AEST)',
				'Auckland (NZDT/NZST)',
				'Custom timezone...'
			],
			default: '__local',
			tab: 'general'
		},
		'timezone-custom': {
			type: 'string',
			title: 'Custom Timezone',
			description: 'Enter custom timezone identifier (e.g., America/Argentina/Buenos_Aires)',
			placeholder: 'America/New_York',
			tab: 'general',
			visibleWhen: {
				field: 'timezone',
				value: '__custom'
			}
		},
		timezones: {
			type: 'array',
			title: 'Additional Timezones',
			description:
				'List of timezones to cycle through (one per line). Do not use with single timezone option.',
			items: {
				type: 'string'
			},
			format: 'textarea',
			placeholder: 'Etc/UTC\nAmerica/New_York\nAsia/Tokyo',
			tab: 'general'
		},
		locale: {
			type: 'select',
			title: 'Locale',
			description: 'Locale for date formatting',
			enum: [
				'__system',
				'C',
				'en_US.UTF-8',
				'en_GB.UTF-8',
				'de_DE.UTF-8',
				'fr_FR.UTF-8',
				'es_ES.UTF-8',
				'it_IT.UTF-8',
				'pt_BR.UTF-8',
				'ru_RU.UTF-8',
				'ja_JP.UTF-8',
				'zh_CN.UTF-8',
				'ko_KR.UTF-8',
				'ar_SA.UTF-8',
				'__custom'
			],
			enumLabels: [
				'System locale',
				'C (default)',
				'English (US)',
				'English (UK)',
				'German',
				'French',
				'Spanish',
				'Italian',
				'Portuguese (Brazil)',
				'Russian',
				'Japanese',
				'Chinese (Simplified)',
				'Korean',
				'Arabic (Saudi Arabia)',
				'Custom locale...'
			],
			default: '__system',
			tab: 'general'
		},
		'locale-custom': {
			type: 'string',
			title: 'Custom Locale',
			description: 'Enter custom locale identifier (e.g., en_AU.UTF-8)',
			placeholder: 'en_US.UTF-8',
			tab: 'general',
			visibleWhen: {
				field: 'locale',
				value: '__custom'
			}
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
			type: 'select',
			title: 'Tooltip Format',
			description: 'Format for tooltip text',
			enum: [
				'__default',
				'{:%Y-%m-%d}',
				'{:%A, %B %d, %Y}',
				'{:%c}',
				'<tt><small>{calendar}</small></tt>',
				'{tz_list}',
				'__custom'
			],
			enumLabels: [
				'Default (same as format)',
				'ISO date (2025-10-17)',
				'Full date (Monday, October 17, 2025)',
				'Locale date and time',
				'Calendar popup',
				'Timezone list',
				'Custom format...'
			],
			default: '__default',
			tab: 'general'
		},
		'tooltip-format-custom': {
			type: 'string',
			title: 'Custom Tooltip Format',
			description: 'Enter custom tooltip format',
			placeholder: '{:%Y-%m-%d}',
			tab: 'general',
			visibleWhen: {
				field: 'tooltip-format',
				value: '__custom'
			}
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
			title: 'Left Click Command',
			description: 'Command to execute when left-clicking the clock',
			placeholder: 'gnome-calendar',
			default: '',
			tab: 'actions'
		},
		'on-click-middle': {
			type: 'string',
			title: 'Middle Click Command',
			description: 'Command to execute when middle-clicking (scroll button)',
			placeholder: 'gnome-clocks',
			default: '',
			tab: 'actions'
		},
		'on-click-right': {
			type: 'select',
			title: 'Right Click Action',
			description: 'Built-in action for right-click',
			enum: ['__none', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'None',
				'Switch Calendar Mode (year/month)',
				'Next Timezone',
				'Previous Timezone',
				'Calendar Forward (next month/year)',
				'Calendar Back (previous month/year)',
				'Calendar Reset (today)'
			],
			default: '__none',
			tab: 'actions'
		},
		'on-scroll-up': {
			type: 'select',
			title: 'Scroll Up Action',
			description: 'Built-in action when scrolling up',
			enum: ['__none', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'None',
				'Switch Calendar Mode (year/month)',
				'Next Timezone',
				'Previous Timezone',
				'Calendar Forward (next month/year)',
				'Calendar Back (previous month/year)',
				'Calendar Reset (today)'
			],
			default: '__none',
			tab: 'actions'
		},
		'on-scroll-down': {
			type: 'select',
			title: 'Scroll Down Action',
			description: 'Built-in action when scrolling down',
			enum: ['__none', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'None',
				'Switch Calendar Mode (year/month)',
				'Next Timezone',
				'Previous Timezone',
				'Calendar Forward (next month/year)',
				'Calendar Back (previous month/year)',
				'Calendar Reset (today)'
			],
			default: '__none',
			tab: 'actions'
		},
		'on-click-forward': {
			type: 'select',
			title: 'Mouse Forward Button',
			description: 'Built-in action for forward mouse button',
			enum: ['__none', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'None',
				'Switch Calendar Mode (year/month)',
				'Next Timezone',
				'Previous Timezone',
				'Calendar Forward (next month/year)',
				'Calendar Back (previous month/year)',
				'Calendar Reset (today)'
			],
			default: '__none',
			tab: 'actions'
		},
		'on-click-backward': {
			type: 'select',
			title: 'Mouse Back Button',
			description: 'Built-in action for back mouse button',
			enum: ['__none', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'None',
				'Switch Calendar Mode (year/month)',
				'Next Timezone',
				'Previous Timezone',
				'Calendar Forward (next month/year)',
				'Calendar Back (previous month/year)',
				'Calendar Reset (today)'
			],
			default: '__none',
			tab: 'actions'
		},

		// Calendar settings
		'calendar.mode': {
			type: 'select',
			title: 'Calendar Mode',
			description: 'Initial calendar display mode',
			enum: ['month', 'year'],
			enumLabels: ['Month View', 'Year View'],
			default: 'month',
			tab: 'calendar'
		},
		'calendar.mode-mon-col': {
			type: 'select',
			title: 'Months Per Column',
			description: 'Number of months per column in year view',
			enum: [1, 2, 3, 4, 6, 12],
			enumLabels: ['1', '2', '3 (default)', '4', '6', '12'],
			default: 3,
			tab: 'calendar'
		},
		'calendar.weeks-pos': {
			type: 'select',
			title: 'Week Numbers Position',
			description: 'Where to show week numbers in calendar',
			enum: ['__none', 'left', 'right'],
			enumLabels: ['Hidden', 'Left Side', 'Right Side'],
			default: '__none',
			tab: 'calendar'
		},
		'calendar.on-scroll': {
			type: 'select',
			title: 'Calendar Scroll Amount',
			description: 'Number of months to scroll per action',
			enum: [1, 2, 3, 6, 12],
			enumLabels: ['1 month', '2 months', '3 months', '6 months', '1 year'],
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
			title: 'Calendar Right Click',
			description: 'Calendar popup action on right-click',
			enum: ['__none', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'None',
				'Switch Calendar Mode (year/month)',
				'Next Timezone',
				'Previous Timezone',
				'Forward (next month/year)',
				'Back (previous month/year)',
				'Reset (today)'
			],
			default: '__none',
			tab: 'calendar'
		},
		'actions.on-click-middle': {
			type: 'select',
			title: 'Calendar Middle Click',
			description: 'Calendar popup action on middle-click',
			enum: ['__none', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'None',
				'Switch Calendar Mode (year/month)',
				'Next Timezone',
				'Previous Timezone',
				'Forward (next month/year)',
				'Back (previous month/year)',
				'Reset (today)'
			],
			default: '__none',
			tab: 'calendar'
		},
		'actions.on-scroll-up': {
			type: 'select',
			title: 'Calendar Scroll Up',
			description: 'Calendar popup action on scroll up',
			enum: ['__none', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'None',
				'Switch Calendar Mode (year/month)',
				'Next Timezone',
				'Previous Timezone',
				'Forward (next month/year)',
				'Back (previous month/year)',
				'Reset (today)'
			],
			default: '__none',
			tab: 'calendar'
		},
		'actions.on-scroll-down': {
			type: 'select',
			title: 'Calendar Scroll Down',
			description: 'Calendar popup action on scroll down',
			enum: ['__none', 'mode', 'tz_up', 'tz_down', 'shift_up', 'shift_down', 'shift_reset'],
			enumLabels: [
				'None',
				'Switch Calendar Mode (year/month)',
				'Next Timezone',
				'Previous Timezone',
				'Forward (next month/year)',
				'Back (previous month/year)',
				'Reset (today)'
			],
			default: '__none',
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
