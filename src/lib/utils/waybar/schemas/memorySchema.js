export const memorySchema = {
	type: 'object',
	title: 'Memory',
	description: '',
	properties: {
		interval: {
			type: 'integer',
			title: 'Update Interval',
			description: 'How often to poll memory information (in seconds)',
			minimum: 1,
			maximum: 300,
			default: 30,
			placeholder: '30',
			tab: 'general'
		},
		'max-length': {
			type: 'integer',
			title: 'Max Length',
			description: 'Maximum characters to display before truncation',
			minimum: 1,
			maximum: 500,
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
			title: 'Display Format',
			description: 'How memory information should be displayed',
			enum: [
				'__default',
				'{icon}',
				'{icon} {percentage}%',
				'{used:0.1f}G/{total:0.1f}G',
				'{icon} {used:0.1f}G',
				'{percentage}% ({used:0.1f}G)',
				'{icon} {used:0.1f}G/{total:0.1f}G',
				'__custom'
			],
			enumLabels: [
				'Default ( icon)',
				'Icon only',
				'Icon + percentage',
				'Used/Total (GB)',
				'Icon + used (GB)',
				'Percentage + used (GB)',
				'Icon + used/total (GB)',
				'Custom format...'
			],
			default: '{percentage}%',
			tab: 'formats'
		},
		'format-custom': {
			type: 'string',
			title: 'Custom Format',
			description:
				'Enter custom format. Use {percentage}, {used}, {total}, {avail}, {swapPercentage}, etc.',
			placeholder: '{percentage}%',
			tab: 'formats',
			visibleWhen: {
				field: 'format',
				value: '__custom'
			}
		},
		'format-icons': {
			type: 'array',
			format: 'textarea',
			title: 'Memory Icons',
			description:
				'Icons from low to high memory usage (one per line). Used with {icon} placeholder.',
			placeholder: '▁\n▂\n▃\n▄\n▅\n▆\n▇\n█',
			default: [],
			tab: 'formats'
		},

		'states.warning': {
			type: 'integer',
			title: 'Warning Threshold',
			description: 'Memory usage percentage to trigger warning state',
			minimum: 0,
			maximum: 100,
			placeholder: '80',
			tab: 'states'
		},
		'states.critical': {
			type: 'integer',
			title: 'Critical Threshold',
			description: 'Memory usage percentage to trigger critical state',
			minimum: 0,
			maximum: 100,
			placeholder: '90',
			tab: 'states'
		},

		// Tooltip
		tooltip: {
			type: 'boolean',
			title: 'Show Tooltip',
			description: 'Enable tooltip on hover',
			default: true,
			tab: 'tooltip'
		},
		'tooltip-format': {
			type: 'select',
			title: 'Tooltip Format',
			description: 'Format of the text to display in the tooltip',
			enum: [
				'__default',
				'{used:0.1f}GiB used',
				'{used:0.1f}G / {total:0.1f}G',
				'RAM: {used:0.1f}G / {total:0.1f}G\nSwap: {swapUsed:0.1f}G / {swapTotal:0.1f}G',
				'{percentage}% used ({used:0.1f}G / {total:0.1f}G)',
				'__custom'
			],
			enumLabels: [
				'Default ({used:0.1f}GiB used)',
				'Used memory',
				'Used / Total',
				'RAM + Swap details',
				'Percentage + used/total',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-custom': {
			type: 'string',
			title: 'Custom Tooltip Format',
			description: 'Enter custom tooltip format',
			placeholder: '{used:0.1f}GiB used',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format',
				value: '__custom'
			}
		},

		// Actions
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
			placeholder: '',
			default: '',
			tab: 'actions'
		},
		'on-click-right': {
			type: 'string',
			title: 'Right Click Command',
			description: 'Command to execute when right-clicking',
			placeholder: 'htop',
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
			description: 'Update interval and display settings'
		},
		{
			id: 'formats',
			label: 'Formats',
			description: 'Configure how memory information is displayed'
		}
		// {
		// 	id: 'states',
		// 	label: 'States',
		// 	description: 'Define warning and critical thresholds'
		// },
		// {
		// 	id: 'tooltip',
		// 	label: 'Tooltip',
		// 	description: 'Tooltip display configuration'
		// },
		// {
		// 	id: 'actions',
		// 	label: 'Actions',
		// 	description: 'Mouse and scroll interactions'
		// }
	]
};
