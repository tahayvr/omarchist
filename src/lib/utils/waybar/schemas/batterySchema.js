export const batterySchema = {
	type: 'object',
	title: 'Battery',
	description: '',
	properties: {
		bat: {
			type: 'string',
			title: 'Battery Device',
			description:
				'Specific battery to monitor from /sys/class/power_supply/. Leave empty for auto-detection.',
			placeholder: 'Auto-detect (e.g., BAT0, BAT1)',
			default: '',
			tab: 'general'
		},
		adapter: {
			type: 'string',
			title: 'Adapter Device',
			description:
				'Specific adapter to monitor from /sys/class/power_supply/. Leave empty for auto-detection.',
			placeholder: 'Auto-detect (e.g., AC, ADP1)',
			default: '',
			tab: 'general'
		},
		interval: {
			type: 'select',
			title: 'Update Interval',
			description: 'How often to poll battery status (seconds)',
			enum: [1, 2, 5, 10, 30, 60, 120],
			enumLabels: [
				'1 second',
				'2 seconds',
				'5 seconds',
				'10 seconds',
				'30 seconds',
				'60 seconds (default)',
				'2 minutes'
			],
			default: 60,
			tab: 'general'
		},
		'design-capacity': {
			type: 'boolean',
			title: 'Use Design Capacity',
			description:
				'Use design capacity instead of actual maximum. Battery may show less than 100% when full if degraded.',
			default: false,
			tab: 'general'
		},
		'full-at': {
			type: 'integer',
			title: 'Full At Percentage',
			description:
				'Define max battery percentage (useful for old batteries). E.g., 96 means battery shows 100% at 96%.',
			minimum: 1,
			maximum: 100,
			placeholder: '100',
			tab: 'general'
		},
		'weighted-average': {
			type: 'boolean',
			title: 'Weighted Average',
			description:
				'For multiple batteries, calculate average weighted by battery size instead of simple average.',
			default: false,
			tab: 'general'
		},
		'bat-compatibility': {
			type: 'boolean',
			title: 'Battery Compatibility Mode',
			description: 'Enable if battery is not detected automatically.',
			default: false,
			tab: 'general'
		},
		'max-length': {
			type: 'integer',
			title: 'Max Length',
			description: 'Maximum characters to display',
			minimum: 1,
			maximum: 200,
			placeholder: 'No limit',
			tab: 'general'
		},
		rotate: {
			type: 'integer',
			title: 'Rotation',
			description: 'Rotate the module output clockwise (degrees)',
			enum: [0, 90, 180, 270],
			default: 0,
			tab: 'general'
		},

		format: {
			type: 'select',
			title: 'Default Format',
			description: 'Fallback format when status/state-specific formats are not set',
			enum: [
				'__default',
				'{capacity}% {icon}',
				'{icon} {capacity}%',
				'{capacity}%',
				'{icon}',
				'{capacity}% {icon} {time}',
				'{icon} {capacity}% ({time})',
				'{capacity}% ({power}W)',
				'__custom'
			],
			enumLabels: [
				'Default ({capacity}%)',
				'Capacity + icon',
				'Icon + capacity',
				'Capacity only',
				'Icon only',
				'Capacity + icon + time',
				'Icon + capacity (time)',
				'Capacity (power)',
				'Custom format...'
			],
			default: '{capacity}% {icon}',
			tab: 'formats'
		},
		'format-custom': {
			type: 'string',
			title: 'Custom Default Format',
			description: 'Enter custom default format',
			placeholder: '{capacity}% {icon}',
			tab: 'formats',
			visibleWhen: {
				field: 'format',
				value: '__custom'
			}
		},
		'format-time': {
			type: 'select',
			title: 'Time Format',
			description: 'Format for time estimates in {time} placeholder',
			enum: [
				'__default',
				'{H} h {M} min',
				'{H}:{M:02}',
				'{H}h {M}m',
				'{M} min',
				'{H} hours',
				'__custom'
			],
			enumLabels: [
				'Default ({H} h {M} min)',
				'H h M min',
				'H:MM (zero-padded)',
				'Hh Mm',
				'M min (minutes only)',
				'H hours (hours only)',
				'Custom format...'
			],
			default: '{H} h {M} min',
			tab: 'formats'
		},
		'format-time-custom': {
			type: 'string',
			title: 'Custom Time Format',
			description:
				'Enter custom time format. Use {H} for hours, {M} for minutes, {m} for zero-padded minutes.',
			placeholder: '{H} h {M} min',
			tab: 'formats',
			visibleWhen: {
				field: 'format-time',
				value: '__custom'
			}
		},
		'format-icons': {
			type: 'array',
			format: 'textarea',
			title: 'Battery Icons',
			description:
				'Icons from low to high capacity (one per line). Can also be state-based (see docs).',
			placeholder: '\n\n\n\n',
			default: [],
			tab: 'formats'
		},

		// Status-based formats
		'format-charging': {
			type: 'select',
			title: 'Charging Format',
			description: 'Format when battery is charging',
			enum: [
				'__default',
				'󰂄 {capacity}%',
				' {capacity}%',
				'󰂄 {capacity}% ({time})',
				' Charging {capacity}%',
				'{capacity}% 󱐋',
				'__custom'
			],
			enumLabels: [
				'Inherit default format',
				'Charging icon + capacity',
				'Plug icon + capacity',
				'Charging icon + capacity (time)',
				'Plug + "Charging" + capacity',
				'Capacity + bolt icon',
				'Custom format...'
			],
			default: '󰂄 {capacity}%',
			tab: 'formats'
		},
		'format-charging-custom': {
			type: 'string',
			title: 'Custom Charging Format',
			description: 'Enter custom charging format',
			placeholder: '󰂄 {capacity}%',
			tab: 'formats',
			visibleWhen: {
				field: 'format-charging',
				value: '__custom'
			}
		},
		'format-discharging': {
			type: 'select',
			title: 'Discharging Format',
			description: 'Format when battery is discharging (on battery power)',
			enum: [
				'__default',
				'{icon} {capacity}%',
				'{capacity}% {icon}',
				'{icon} {capacity}% ({time})',
				'{capacity}% ({time})',
				'{icon}',
				'__custom'
			],
			enumLabels: [
				'Inherit default format',
				'Icon + capacity',
				'Capacity + icon',
				'Icon + capacity (time)',
				'Capacity (time)',
				'Icon only',
				'Custom format...'
			],
			default: '{icon} {capacity}%',
			tab: 'formats'
		},
		'format-discharging-custom': {
			type: 'string',
			title: 'Custom Discharging Format',
			description: 'Enter custom discharging format',
			placeholder: '{icon} {capacity}%',
			tab: 'formats',
			visibleWhen: {
				field: 'format-discharging',
				value: '__custom'
			}
		},
		'format-full': {
			type: 'select',
			title: 'Full Format',
			description: 'Format when battery is full',
			enum: ['__default', '', ' Full', '󰂅 {capacity}%', '󰂅', ' 100%', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Icon only',
				'Plug + "Full"',
				'Full icon + capacity',
				'Full icon only',
				'Plug + 100%',
				'Custom format...'
			],
			default: '󰂅 {capacity}%',
			tab: 'formats'
		},
		'format-full-custom': {
			type: 'string',
			title: 'Custom Full Format',
			description: 'Enter custom full format',
			placeholder: ' Full',
			tab: 'formats',
			visibleWhen: {
				field: 'format-full',
				value: '__custom'
			}
		},
		'format-plugged': {
			type: 'select',
			title: 'Plugged Format',
			description: 'Format when plugged in but not charging',
			enum: [
				'__default',
				' {capacity}%',
				' Plugged',
				' {capacity}% (Plugged)',
				'{capacity}%',
				'__custom'
			],
			enumLabels: [
				'Inherit default format',
				'Plug + capacity',
				'Plug + "Plugged"',
				'Plug + capacity (Plugged)',
				'Capacity only',
				'Custom format...'
			],
			default: '{capacity}%',
			tab: 'formats'
		},
		'format-plugged-custom': {
			type: 'string',
			title: 'Custom Plugged Format',
			description: 'Enter custom plugged format',
			placeholder: ' {capacity}%',
			tab: 'formats',
			visibleWhen: {
				field: 'format-plugged',
				value: '__custom'
			}
		},
		'format-not-charging': {
			type: 'select',
			title: 'Not Charging Format',
			description: 'Format when plugged but explicitly not charging',
			enum: [
				'__default',
				' {capacity}%',
				' Not Charging',
				'{capacity}% (Not Charging)',
				'__custom'
			],
			enumLabels: [
				'Inherit default format',
				'Plug + capacity',
				'Plug + "Not Charging"',
				'Capacity (Not Charging)',
				'Custom format...'
			],
			default: '{capacity}%',
			tab: 'formats'
		},
		'format-not-charging-custom': {
			type: 'string',
			title: 'Custom Not Charging Format',
			description: 'Enter custom not charging format',
			placeholder: ' {capacity}%',
			tab: 'formats',
			visibleWhen: {
				field: 'format-not-charging',
				value: '__custom'
			}
		},

		// States configuration
		'states.warning': {
			type: 'integer',
			title: 'Warning State Threshold',
			description: 'Battery percentage at or below which warning state activates',
			minimum: 0,
			maximum: 100,
			placeholder: '30',
			tab: 'states'
		},
		'states.critical': {
			type: 'integer',
			title: 'Critical State Threshold',
			description: 'Battery percentage at or below which critical state activates',
			minimum: 0,
			maximum: 100,
			placeholder: '15',
			tab: 'states'
		},
		'format-warning': {
			type: 'select',
			title: 'Warning State Format',
			description: 'Format override when in warning state',
			enum: [
				'__default',
				'󰂃 {capacity}%',
				' {capacity}%',
				'{capacity}% ⚠',
				'{icon} {capacity}% ⚠',
				'__custom'
			],
			enumLabels: [
				'Inherit status format',
				'Low battery icon + capacity',
				'Warning icon + capacity',
				'Capacity + warning symbol',
				'Icon + capacity + warning',
				'Custom format...'
			],
			default: '__default',
			tab: 'states'
		},
		'format-warning-custom': {
			type: 'string',
			title: 'Custom Warning Format',
			description: 'Enter custom warning state format',
			placeholder: ' {capacity}%',
			tab: 'states',
			visibleWhen: {
				field: 'format-warning',
				value: '__custom'
			}
		},
		'format-critical': {
			type: 'select',
			title: 'Critical State Format',
			description: 'Format override when in critical state',
			enum: [
				'__default',
				'󰂃 {capacity}%',
				' {capacity}%',
				'{capacity}% 🔴',
				'{icon} {capacity}% !',
				'__custom'
			],
			enumLabels: [
				'Inherit status format',
				'Empty battery icon + capacity',
				'Alert icon + capacity',
				'Capacity + red circle',
				'Icon + capacity + exclamation',
				'Custom format...'
			],
			default: '__default',
			tab: 'states'
		},
		'format-critical-custom': {
			type: 'string',
			title: 'Custom Critical Format',
			description: 'Enter custom critical state format',
			placeholder: ' {capacity}%',
			tab: 'states',
			visibleWhen: {
				field: 'format-critical',
				value: '__custom'
			}
		},

		tooltip: {
			type: 'boolean',
			title: 'Show Tooltip',
			description: 'Enable tooltip on hover',
			default: true,
			tab: 'tooltip'
		},
		'tooltip-format': {
			type: 'select',
			title: 'Default Tooltip Format',
			description: 'Tooltip format for general state',
			enum: [
				'__default',
				'{timeTo}',
				'{capacity}% - {timeTo}',
				'{capacity}% ({power}W) - {timeTo}',
				'{capacity}% - {cycles} cycles',
				'{capacity}% - Health: {health}%',
				'__custom'
			],
			enumLabels: [
				'Default ({timeTo})',
				'Time to full/empty',
				'Capacity - time',
				'Capacity (power) - time',
				'Capacity - cycles',
				'Capacity - health',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-custom': {
			type: 'string',
			title: 'Custom Tooltip Format',
			description: 'Enter custom tooltip format',
			placeholder: '{timeTo}',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format',
				value: '__custom'
			}
		},
		'tooltip-format-charging': {
			type: 'select',
			title: 'Charging Tooltip',
			description: 'Tooltip when charging',
			enum: [
				'__default',
				'Charging: {timeTo}',
				'{capacity}% - {time} until full',
				'Charging at {power}W',
				'{capacity}% ({power}W) - {time} until full',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Charging: time',
				'Capacity - time until full',
				'Charging at power',
				'Capacity (power) - time until full',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-charging-custom': {
			type: 'string',
			title: 'Custom Charging Tooltip',
			description: 'Enter custom charging tooltip',
			placeholder: 'Charging: {timeTo}',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-charging',
				value: '__custom'
			}
		},
		'tooltip-format-discharging': {
			type: 'select',
			title: 'Discharging Tooltip',
			description: 'Tooltip when discharging',
			enum: [
				'__default',
				'{timeTo}',
				'{capacity}% - {time} remaining',
				'Discharging at {power}W',
				'{capacity}% ({power}W) - {time} remaining',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Time remaining',
				'Capacity - time remaining',
				'Discharging at power',
				'Capacity (power) - time remaining',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-discharging-custom': {
			type: 'string',
			title: 'Custom Discharging Tooltip',
			description: 'Enter custom discharging tooltip',
			placeholder: '{timeTo}',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-discharging',
				value: '__custom'
			}
		},
		'tooltip-format-full': {
			type: 'select',
			title: 'Full Tooltip',
			description: 'Tooltip when battery is full',
			enum: [
				'__default',
				'Full',
				'Battery Full',
				'{capacity}% - Full',
				'{cycles} charge cycles',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Full',
				'Battery Full',
				'Capacity - Full',
				'Charge cycles',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-full-custom': {
			type: 'string',
			title: 'Custom Full Tooltip',
			description: 'Enter custom full tooltip',
			placeholder: 'Full',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-full',
				value: '__custom'
			}
		},

		'on-click': {
			type: 'string',
			title: 'Left Click Command',
			description: 'Command to execute when left-clicking',
			placeholder: 'gnome-power-statistics',
			default: '',
			tab: 'actions'
		},
		'on-click-middle': {
			type: 'string',
			title: 'Middle Click Command',
			description: 'Command to execute when middle-clicking',
			placeholder: 'xfce4-power-manager-settings',
			default: '',
			tab: 'actions'
		},
		'on-click-right': {
			type: 'string',
			title: 'Right Click Command',
			description: 'Command to execute when right-clicking',
			placeholder: 'gnome-power-statistics',
			default: '',
			tab: 'actions'
		},
		'on-scroll-up': {
			type: 'string',
			title: 'Scroll Up Command',
			description: 'Command to execute when scrolling up',
			default: '',
			tab: 'actions'
		},
		'on-scroll-down': {
			type: 'string',
			title: 'Scroll Down Command',
			description: 'Command to execute when scrolling down',
			default: '',
			tab: 'actions'
		},
		'smooth-scrolling-threshold': {
			type: 'number',
			title: 'Smooth Scrolling Threshold',
			description: 'Threshold for smooth scrolling',
			minimum: 0,
			placeholder: '0',
			tab: 'actions'
		}
	},
	tabs: [
		// {
		// 	id: 'general',
		// 	label: 'General',
		// 	description: 'Battery selection, polling, and capacity settings'
		// },
		{
			id: 'formats',
			label: 'Formats',
			description: 'Display formats for different battery statuses'
		}
		// {
		// 	id: 'states',
		// 	label: 'States',
		// 	description: 'Warning and critical state thresholds and formats'
		// },
		// {
		// 	id: 'tooltip',
		// 	label: 'Tooltip',
		// 	description: 'Tooltip formats for each battery status'
		// },
		// {
		// 	id: 'actions',
		// 	label: 'Actions',
		// 	description: 'Mouse and scroll interactions'
		// }
	]
};
