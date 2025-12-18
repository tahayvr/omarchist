export const pulseaudioSchema = {
	type: 'object',
	title: 'Pulseaudio',
	description: '',
	properties: {
		'scroll-step': {
			type: 'number',
			title: 'Scroll Step',
			description: 'Volume change percentage when scrolling (can be decimal)',
			minimum: 0.1,
			maximum: 100,
			step: 0.5,
			default: 1.0,
			placeholder: '1.0',
			tab: 'general'
		},
		'max-volume': {
			type: 'integer',
			title: 'Maximum Volume',
			description: 'Maximum volume that can be set (percentage)',
			minimum: 100,
			maximum: 200,
			default: 100,
			placeholder: '100',
			tab: 'general'
		},
		'reverse-scrolling': {
			type: 'boolean',
			title: 'Reverse Scrolling (Touchpad)',
			description: 'Reverse scroll direction for touchpad/trackpad',
			default: false,
			tab: 'general'
		},
		'reverse-mouse-scrolling': {
			type: 'boolean',
			title: 'Reverse Mouse Scrolling',
			description: 'Reverse scroll direction for mice',
			default: false,
			tab: 'general'
		},
		'ignored-sinks': {
			type: 'array',
			format: 'textarea',
			title: 'Ignored Sinks',
			description:
				'List of sink descriptions to ignore (one per line). Use "pactl list sinks" to find descriptions.',
			placeholder: 'Easy Effects Sink\nNull Output',
			default: [],
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
			title: 'Default Format',
			description: 'Fallback format when device-specific formats are not set',
			enum: [
				'__default',
				'{volume}% {icon}',
				'{icon} {volume}%',
				'{volume}%',
				'{icon}',
				'{icon} {volume}% {format_source}',
				'{volume}% {icon} {desc}',
				'__custom'
			],
			enumLabels: [
				'Default ({volume}%)',
				'Volume + icon',
				'Icon + volume',
				'Volume only',
				'Icon only',
				'Icon + volume + source',
				'Volume + icon + device',
				'Custom format...'
			],
			default: '{volume}% {icon}',
			tab: 'formats'
		},
		'format-custom': {
			type: 'string',
			title: 'Custom Default Format',
			description: 'Enter custom default format',
			placeholder: '{volume}% {icon}',
			tab: 'formats',
			visibleWhen: {
				field: 'format',
				value: '__custom'
			}
		},
		'format-bluetooth': {
			type: 'select',
			title: 'Bluetooth Format',
			description: 'Format when using Bluetooth speakers',
			enum: [
				'__default',
				'{volume}% ',
				' {volume}%',
				'{volume}% {icon}',
				'{icon} {volume}% {desc}',
				'__custom'
			],
			enumLabels: [
				'Inherit default format',
				'Volume + Bluetooth icon',
				'Bluetooth icon + volume',
				'Volume + device icon',
				'Icon + volume + device name',
				'Custom format...'
			],
			default: '{volume}% ',
			tab: 'formats'
		},
		'format-bluetooth-custom': {
			type: 'string',
			title: 'Custom Bluetooth Format',
			description: 'Enter custom Bluetooth format',
			placeholder: '{volume}% ',
			tab: 'formats',
			visibleWhen: {
				field: 'format-bluetooth',
				value: '__custom'
			}
		},
		'format-muted': {
			type: 'select',
			title: 'Muted Format',
			description: 'Format when sound is muted',
			enum: ['__default', '', ' Muted', ' {volume}%', '', 'Muted', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Muted icon',
				'Muted icon + text',
				'Muted icon + volume',
				'Strikethrough icon',
				'Text only',
				'Custom format...'
			],
			default: '󰝟 {volume}%',
			tab: 'formats'
		},
		'format-muted-custom': {
			type: 'string',
			title: 'Custom Muted Format',
			description: 'Enter custom muted format',
			placeholder: '',
			tab: 'formats',
			visibleWhen: {
				field: 'format-muted',
				value: '__custom'
			}
		},
		'format-source': {
			type: 'select',
			title: 'Source Format',
			description: 'Format for input source (microphone)',
			enum: ['__default', '{volume}% ', ' {volume}%', '{volume}%', '', '__custom'],
			enumLabels: [
				'Default ({volume}%)',
				'Volume + mic icon',
				'Mic icon + volume',
				'Volume only',
				'Mic icon only',
				'Custom format...'
			],
			default: '{volume}% ',
			tab: 'formats'
		},
		'format-source-custom': {
			type: 'string',
			title: 'Custom Source Format',
			description: 'Enter custom source format',
			placeholder: '{volume}% ',
			tab: 'formats',
			visibleWhen: {
				field: 'format-source',
				value: '__custom'
			}
		},
		'format-source-muted': {
			type: 'select',
			title: 'Source Muted Format',
			description: 'Format when microphone is muted',
			enum: ['__default', '', ' Muted', ' {volume}%', '', '__custom'],
			enumLabels: [
				'Inherit source format',
				'Muted mic icon',
				'Muted mic icon + text',
				'Muted mic icon + volume',
				'Strikethrough mic icon',
				'Custom format...'
			],
			default: '',
			tab: 'formats'
		},
		'format-source-muted-custom': {
			type: 'string',
			title: 'Custom Source Muted Format',
			description: 'Enter custom source muted format',
			placeholder: '',
			tab: 'formats',
			visibleWhen: {
				field: 'format-source-muted',
				value: '__custom'
			}
		},
		'format-icons': {
			type: 'array',
			format: 'textarea',
			title: 'Volume Icons',
			description:
				'Icons from low to high volume (one per line). Can also be device-specific (see docs).',
			placeholder: '\n\n',
			default: [],
			tab: 'formats'
		},

		// States configuration
		'states.warning': {
			type: 'integer',
			title: 'Warning State Threshold',
			description: 'Volume percentage at or above which warning state activates',
			minimum: 0,
			maximum: 200,
			placeholder: '100',
			tab: 'states'
		},
		'states.critical': {
			type: 'integer',
			title: 'Critical State Threshold',
			description: 'Volume percentage at or above which critical state activates',
			minimum: 0,
			maximum: 200,
			placeholder: '150',
			tab: 'states'
		},
		'format-warning': {
			type: 'select',
			title: 'Warning State Format',
			description: 'Format override when in warning state (high volume)',
			enum: ['__default', '{volume}% {icon} ⚠', '{icon} {volume}% ⚠', '{volume}% 🔊', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Volume + icon + warning',
				'Icon + volume + warning',
				'Volume + loud speaker',
				'Custom format...'
			],
			default: '__default',
			tab: 'states'
		},
		'format-warning-custom': {
			type: 'string',
			title: 'Custom Warning Format',
			description: 'Enter custom warning state format',
			placeholder: '{volume}% {icon} ⚠',
			tab: 'states',
			visibleWhen: {
				field: 'format-warning',
				value: '__custom'
			}
		},
		'format-critical': {
			type: 'select',
			title: 'Critical State Format',
			description: 'Format override when in critical state (very high volume)',
			enum: [
				'__default',
				'{volume}% {icon} 🔴',
				'{icon} {volume}% !',
				'{volume}% 🔊 !',
				'__custom'
			],
			enumLabels: [
				'Inherit default format',
				'Volume + icon + red circle',
				'Icon + volume + exclamation',
				'Volume + loud speaker + exclamation',
				'Custom format...'
			],
			default: '__default',
			tab: 'states'
		},
		'format-critical-custom': {
			type: 'string',
			title: 'Custom Critical Format',
			description: 'Enter custom critical state format',
			placeholder: '{volume}% {icon} 🔴',
			tab: 'states',
			visibleWhen: {
				field: 'format-critical',
				value: '__custom'
			}
		},

		// Tooltip settings
		'tooltip-format': {
			type: 'select',
			title: 'Tooltip Format',
			description: 'Tooltip format on hover',
			enum: [
				'__default',
				'{desc}',
				'{desc} - {volume}%',
				'{volume}% on {desc}',
				'{desc}\n{volume}%',
				'__custom'
			],
			enumLabels: [
				'Default ({desc})',
				'Device description',
				'Device - volume',
				'Volume on device',
				'Device + volume (multiline)',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-custom': {
			type: 'string',
			title: 'Custom Tooltip Format',
			description: 'Enter custom tooltip format',
			placeholder: '{desc}',
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
			placeholder: 'pavucontrol',
			default: '',
			tab: 'actions'
		},
		'on-click-middle': {
			type: 'string',
			title: 'Middle Click Command',
			description: 'Command to execute when middle-clicking',
			placeholder: 'pactl set-sink-mute @DEFAULT_SINK@ toggle',
			default: '',
			tab: 'actions'
		},
		'on-click-right': {
			type: 'string',
			title: 'Right Click Command',
			description: 'Command to execute when right-clicking',
			placeholder: 'pavucontrol',
			default: '',
			tab: 'actions'
		},
		'on-scroll-up': {
			type: 'string',
			title: 'Scroll Up Command',
			description: 'Command to execute when scrolling up (replaces default volume control)',
			placeholder: 'Leave empty for default volume up',
			default: '',
			tab: 'actions'
		},
		'on-scroll-down': {
			type: 'string',
			title: 'Scroll Down Command',
			description: 'Command to execute when scrolling down (replaces default volume control)',
			placeholder: 'Leave empty for default volume down',
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
			description: 'Volume control, scroll behavior, and device settings'
		},
		{
			id: 'formats',
			label: 'Formats',
			description: 'Display formats for different audio states'
		}
		// {
		// 	id: 'states',
		// 	label: 'States',
		// 	description: 'Warning and critical volume thresholds'
		// },
		// {
		// 	id: 'tooltip',
		// 	label: 'Tooltip',
		// 	description: 'Tooltip format configuration'
		// },
		// {
		// 	id: 'actions',
		// 	label: 'Actions',
		// 	description: 'Mouse and scroll interactions'
		// }
	]
};
