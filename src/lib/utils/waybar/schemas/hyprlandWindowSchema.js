/**
 * Hyprland Window module schema definition for Waybar
 * See: https://github.com/Alexays/Waybar/wiki/Module:-Hyprland#window
 */

export const hyprlandWindowSchema = {
	type: 'object',
	title: 'Focused Window',
	description: 'Display the title of the currently focused Hyprland window',
	properties: {
		// General settings
		format: {
			type: 'select',
			title: 'Display Format',
			description: 'How window information should be displayed',
			enum: [
				'__default',
				'{title}',
				'{class}',
				'{class}: {title}',
				'{initialClass}: {title}',
				'👉 {title}',
				'{class} - {title}',
				'__custom'
			],
			enumLabels: [
				'Default ({title})',
				'Window title',
				'Window class',
				'Class: title',
				'Initial class: title',
				'Pointer + title',
				'Class - title',
				'Custom format...'
			],
			default: '__default',
			tab: 'general'
		},
		'format-custom': {
			type: 'string',
			title: 'Custom Format',
			description: 'Enter custom format. Use {title}, {class}, {initialClass}, {initialTitle}.',
			placeholder: '{title}',
			tab: 'general',
			visibleWhen: {
				field: 'format',
				value: '__custom'
			}
		},
		'max-length': {
			type: 'integer',
			title: 'Max Length',
			description: 'Maximum characters to display before truncation',
			minimum: 1,
			maximum: 500,
			placeholder: 'No limit',
			default: 50,
			tab: 'general'
		},
		'separate-outputs': {
			type: 'boolean',
			title: 'Separate Outputs',
			description:
				'Show the active window of the monitor the bar belongs to, instead of the globally focused window',
			default: false,
			tab: 'general'
		},
		icon: {
			type: 'boolean',
			title: 'Show Application Icon',
			description: 'Display the application icon next to the window title',
			default: false,
			tab: 'general'
		},
		'icon-size': {
			type: 'integer',
			title: 'Icon Size',
			description: 'Size of the application icon in pixels',
			minimum: 8,
			maximum: 64,
			default: 24,
			placeholder: '24',
			tab: 'general'
		},

		// Rewrite rules
		'rewrite-rules': {
			type: 'array',
			format: 'textarea',
			title: 'Rewrite Rules',
			description:
				'Transform window titles using regex patterns. Format: "pattern": "replacement" (one per line). Use $1, $2 for capture groups.',
			placeholder:
				'"(.*) — Mozilla Firefox": "🌎 $1"\n"(.*) - Visual Studio Code": "󰨞 $1"\n"(.*) - fish": "> [$1]"',
			default: [],
			tab: 'rewrite'
		},

		// Actions
		'on-click': {
			type: 'string',
			title: 'Left Click Command',
			description: 'Command to execute when left-clicking',
			placeholder: '',
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
			placeholder: '',
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
		rotate: {
			type: 'integer',
			title: 'Rotation',
			description: 'Rotate the module output clockwise (degrees)',
			enum: [0, 90, 180, 270],
			default: 0,
			tab: 'actions'
		},
		'smooth-scrolling-threshold': {
			type: 'number',
			title: 'Smooth Scrolling Threshold',
			description: 'Threshold for smooth scrolling',
			minimum: 0,
			placeholder: '0',
			tab: 'actions'
		},
		tooltip: {
			type: 'boolean',
			title: 'Show Tooltip',
			description: 'Enable tooltip on hover',
			default: true,
			tab: 'actions'
		}
	},
	tabs: [
		{
			id: 'general',
			label: 'General',
			description: 'Display format, length, and icon settings'
		}
		// {
		// 	id: 'rewrite',
		// 	label: 'Rewrite',
		// 	description: 'Transform window titles with regex patterns'
		// },
		// {
		// 	id: 'actions',
		// 	label: 'Actions',
		// 	description: 'Mouse and scroll interactions'
		// }
	]
};
