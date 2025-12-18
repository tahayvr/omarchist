export const cpuSchema = {
	type: 'object',
	title: 'CPU',
	description: '',
	properties: {
		interval: {
			type: 'select',
			title: 'Update Interval',
			description: 'How often to poll CPU usage (seconds)',
			enum: [1, 2, 3, 5, 10, 15, 30],
			enumLabels: [
				'1 second',
				'2 seconds',
				'3 seconds',
				'5 seconds',
				'10 seconds (default)',
				'15 seconds',
				'30 seconds'
			],
			default: 10,
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
		tooltip: {
			type: 'boolean',
			title: 'Show Tooltip',
			description: 'Enable tooltip on hover',
			default: true,
			tab: 'general'
		},

		format: {
			type: 'select',
			title: 'Display Format',
			description: 'How CPU information should be displayed',
			enum: [
				'__default',
				'{usage}% 󰍛',
				'󰍛 {usage}%',
				'{usage}%',
				'󰍛',
				'{icon}',
				'{usage}% ({avg_frequency} GHz)',
				'󰍛 {usage}% {load}',
				'{icon0}{icon1}{icon2}{icon3}',
				'__custom'
			],
			enumLabels: [
				'Default ({usage}%)',
				'Usage + icon',
				'Icon + usage',
				'Usage only',
				'Icon only',
				'Usage icon (from format-icons)',
				'Usage (frequency)',
				'Icon + usage + load',
				'Per-core icons (4 cores)',
				'Custom format...'
			],
			default: '{usage}% 󰍛',
			tab: 'formats'
		},
		'format-custom': {
			type: 'string',
			title: 'Custom Format',
			description:
				'Enter custom format. Use {usage}, {load}, {icon}, {usageN}, {iconN}, {avg_frequency}, etc.',
			placeholder: '{usage}% 󰍛',
			tab: 'formats',
			visibleWhen: {
				field: 'format',
				value: '__custom'
			}
		},
		'format-icons': {
			type: 'array',
			format: 'textarea',
			title: 'Usage Icons',
			description:
				'Icons from low to high CPU usage (one per line). Used with {icon} or {iconN} placeholders.',
			placeholder: '▁\n▂\n▃\n▄\n▅\n▆\n▇\n█',
			default: [],
			tab: 'formats'
		},

		'states.warning': {
			type: 'integer',
			title: 'Warning State Threshold',
			description: 'CPU usage percentage at or above which warning state activates',
			minimum: 0,
			maximum: 100,
			placeholder: '70',
			tab: 'states'
		},
		'states.critical': {
			type: 'integer',
			title: 'Critical State Threshold',
			description: 'CPU usage percentage at or above which critical state activates',
			minimum: 0,
			maximum: 100,
			placeholder: '90',
			tab: 'states'
		},
		'format-warning': {
			type: 'select',
			title: 'Warning State Format',
			description: 'Format override when in warning state',
			enum: ['__default', '{usage}% ⚠', '󰍛 {usage}% ⚠', '{usage}% 󰀦', '{icon} ⚠', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Usage + warning symbol',
				'Icon + usage + warning',
				'Usage + warning icon',
				'Usage icon + warning',
				'Custom format...'
			],
			default: '__default',
			tab: 'states'
		},
		'format-warning-custom': {
			type: 'string',
			title: 'Custom Warning Format',
			description: 'Enter custom warning state format',
			placeholder: '{usage}% ⚠',
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
			enum: ['__default', '{usage}% 🔴', '󰍛 {usage}% !', '{usage}% 󰀨', '{icon} !', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Usage + red circle',
				'Icon + usage + exclamation',
				'Usage + critical icon',
				'Usage icon + exclamation',
				'Custom format...'
			],
			default: '__default',
			tab: 'states'
		},
		'format-critical-custom': {
			type: 'string',
			title: 'Custom Critical Format',
			description: 'Enter custom critical state format',
			placeholder: '{usage}% 🔴',
			tab: 'states',
			visibleWhen: {
				field: 'format-critical',
				value: '__custom'
			}
		},

		'on-click': {
			type: 'string',
			title: 'Left Click Command',
			description: 'Command to execute when left-clicking',
			placeholder: 'gnome-system-monitor',
			default: '',
			tab: 'actions'
		},
		'on-click-middle': {
			type: 'string',
			title: 'Middle Click Command',
			description: 'Command to execute when middle-clicking',
			placeholder: 'htop',
			default: '',
			tab: 'actions'
		},
		'on-click-right': {
			type: 'string',
			title: 'Right Click Command',
			description: 'Command to execute when right-clicking',
			placeholder: 'btop',
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
		{
			id: 'general',
			label: 'General',
			description: 'Polling interval and display settings'
		},
		{
			id: 'formats',
			label: 'Formats',
			description: 'Display format and usage icons'
		}
		// {
		// 	id: 'states',
		// 	label: 'States',
		// 	description: 'Warning and critical state thresholds and formats'
		// },
		// {
		// 	id: 'actions',
		// 	label: 'Actions',
		// 	description: 'Mouse and scroll interactions'
		// }
	]
};
