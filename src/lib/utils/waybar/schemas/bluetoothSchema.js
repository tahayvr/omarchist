/**
 * Bluetooth module schema definition for Waybar
 * See: https://github.com/Alexays/Waybar/wiki/Module:-Bluetooth
 */

export const bluetoothSchema = {
	type: 'object',
	title: 'Bluetooth',
	description: 'Display Bluetooth controller and device information',
	properties: {
		// General settings
		controller: {
			type: 'string',
			title: 'Controller Alias',
			description:
				'Specify which Bluetooth controller to monitor by alias. Leave empty to use any available controller. Recommended when multiple controllers exist.',
			placeholder: 'Auto-detect',
			default: '',
			tab: 'general'
		},
		'format-device-preference': {
			type: 'array',
			format: 'textarea',
			title: 'Device Display Priority',
			description:
				'List device aliases in order of display preference (one per line). First connected device in the list will be shown. Falls back to last connected device if none match.',
			placeholder: 'device1\ndevice2\ndevice3',
			default: [],
			tab: 'general'
		},
		'max-length': {
			type: 'integer',
			title: 'Max Length',
			description: 'Maximum characters to display before truncation',
			minimum: 1,
			maximum: 200,
			placeholder: 'No limit',
			tab: 'general'
		},
		'min-length': {
			type: 'integer',
			title: 'Min Length',
			description: 'Minimum characters the module should occupy',
			minimum: 0,
			maximum: 200,
			placeholder: '0',
			tab: 'general'
		},
		align: {
			type: 'number',
			title: 'Text Alignment',
			description: 'Text alignment: 0 (left) to 1 (right)',
			minimum: 0,
			maximum: 1,
			step: 0.1,
			placeholder: '0',
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

		// Format settings
		format: {
			type: 'select',
			title: 'Default Format',
			description: 'Fallback format used when state-specific formats are not set',
			enum: [
				'__default',
				' {status}',
				'󰂯 {status}',
				'󰂯',
				' {num_connections}',
				'󰂯 {num_connections} connected',
				'{controller_alias}',
				'__custom'
			],
			enumLabels: [
				'Default ( {status})',
				'Icon + status',
				'Alt icon + status',
				'Alt icon only',
				'Icon + connection count',
				'Alt icon + count + text',
				'Controller alias',
				'Custom format...'
			],
			default: '__default',
			tab: 'formats'
		},
		'format-custom': {
			type: 'string',
			title: 'Custom Default Format',
			description: 'Enter a custom default format string',
			placeholder: ' {status}',
			tab: 'formats',
			visibleWhen: {
				field: 'format',
				value: '__custom'
			}
		},
		'format-disabled': {
			type: 'select',
			title: 'Disabled Format',
			description: 'Format when Bluetooth is disabled (empty hides module)',
			enum: ['__default', '', '󰂲', '󰂲 Disabled', '󰂲 Off', '󰂯 Disabled', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Hide module (empty)',
				'Disabled icon',
				'Disabled icon + text',
				'Disabled icon + "Off"',
				'Alt disabled icon + text',
				'Custom format...'
			],
			default: '__default',
			tab: 'formats'
		},
		'format-disabled-custom': {
			type: 'string',
			title: 'Custom Disabled Format',
			description: 'Enter custom format for disabled state',
			placeholder: '󰂲',
			tab: 'formats',
			visibleWhen: {
				field: 'format-disabled',
				value: '__custom'
			}
		},
		'format-off': {
			type: 'select',
			title: 'Off Format',
			description: 'Format when controller is turned off',
			enum: ['__default', '', '󰂲', '󰂲 Off', ' Off', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Hide module (empty)',
				'Disabled icon',
				'Disabled icon + Off',
				'Icon + Off',
				'Custom format...'
			],
			default: '__default',
			tab: 'formats'
		},
		'format-off-custom': {
			type: 'string',
			title: 'Custom Off Format',
			description: 'Enter custom format for off state',
			placeholder: '󰂲 Off',
			tab: 'formats',
			visibleWhen: {
				field: 'format-off',
				value: '__custom'
			}
		},
		'format-on': {
			type: 'select',
			title: 'On Format',
			description: 'Format when controller is on with no devices connected',
			enum: ['__default', '', '', ' On', '󰂯', '󰂯 On', ' Ready', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Hide module (empty)',
				'Icon only',
				'Icon + On',
				'Alt icon only',
				'Alt icon + On',
				'Icon + Ready',
				'Custom format...'
			],
			default: '__default',
			tab: 'formats'
		},
		'format-on-custom': {
			type: 'string',
			title: 'Custom On Format',
			description: 'Enter custom format for on state',
			placeholder: ' On',
			tab: 'formats',
			visibleWhen: {
				field: 'format-on',
				value: '__custom'
			}
		},
		'format-connected': {
			type: 'select',
			title: 'Connected Format',
			description: 'Format when at least one device is connected',
			enum: [
				'__default',
				' {device_alias}',
				'󰂯 {device_alias}',
				' {num_connections}',
				' {num_connections} connected',
				'󰂯 {num_connections} connected',
				'{device_alias}',
				'__custom'
			],
			enumLabels: [
				'Inherit default format',
				'Icon + device name',
				'Alt icon + device name',
				'Icon + connection count',
				'Icon + count + text',
				'Alt icon + count + text',
				'Device name only',
				'Custom format...'
			],
			default: '__default',
			tab: 'formats'
		},
		'format-connected-custom': {
			type: 'string',
			title: 'Custom Connected Format',
			description: 'Enter custom format for connected state',
			placeholder: ' {device_alias}',
			tab: 'formats',
			visibleWhen: {
				field: 'format-connected',
				value: '__custom'
			}
		},
		'format-connected-battery': {
			type: 'select',
			title: 'Connected with Battery Format',
			description:
				'⚠️ Experimental: Format when device provides battery percentage (requires BlueZ experimental features)',
			enum: [
				'__default',
				' {device_alias} {device_battery_percentage}%',
				'󰂯 {device_alias} {device_battery_percentage}%',
				' {device_alias} 󰥉 {device_battery_percentage}%',
				'{device_alias} ({device_battery_percentage}%)',
				'__custom'
			],
			enumLabels: [
				'Inherit connected format',
				'Icon + device + battery %',
				'Alt icon + device + battery %',
				'Icon + device + battery icon + %',
				'Device (battery %)',
				'Custom format...'
			],
			default: '__default',
			tab: 'formats'
		},
		'format-connected-battery-custom': {
			type: 'string',
			title: 'Custom Connected Battery Format',
			description: 'Enter custom format for connected device with battery',
			placeholder: ' {device_alias} {device_battery_percentage}%',
			tab: 'formats',
			visibleWhen: {
				field: 'format-connected-battery',
				value: '__custom'
			}
		},
		'format-no-controller': {
			type: 'select',
			title: 'No Controller Format',
			description: 'Format when no Bluetooth controller is available',
			enum: ['__default', '', '󰂲 No BT', '󰂲', 'No Bluetooth', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Hide module (empty)',
				'Disabled icon + text',
				'Disabled icon only',
				'Text only',
				'Custom format...'
			],
			default: '__default',
			tab: 'formats'
		},
		'format-no-controller-custom': {
			type: 'string',
			title: 'Custom No Controller Format',
			description: 'Enter custom format for no controller state',
			placeholder: '󰂲',
			tab: 'formats',
			visibleWhen: {
				field: 'format-no-controller',
				value: '__custom'
			}
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
			type: 'select',
			title: 'Default Tooltip Format',
			description: 'Tooltip format for general state',
			enum: [
				'__default',
				'{controller_alias}\t{controller_address}',
				'{controller_alias}',
				'Bluetooth: {status}',
				'{controller_alias} - {num_connections} connected',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Controller alias + address',
				'Controller alias only',
				'Bluetooth: status',
				'Controller + connection count',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-custom': {
			type: 'string',
			title: 'Custom Tooltip Format',
			description: 'Enter custom tooltip format',
			placeholder: '{controller_alias}\t{controller_address}',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format',
				value: '__custom'
			}
		},
		'tooltip-format-disabled': {
			type: 'select',
			title: 'Disabled Tooltip',
			description: 'Tooltip when Bluetooth is disabled',
			enum: [
				'__default',
				'Bluetooth Disabled',
				'{controller_alias} - Disabled',
				'Bluetooth is turned off',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Bluetooth Disabled',
				'Controller - Disabled',
				'Bluetooth is turned off',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-disabled-custom': {
			type: 'string',
			title: 'Custom Disabled Tooltip',
			description: 'Enter custom tooltip for disabled state',
			placeholder: 'Bluetooth Disabled',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-disabled',
				value: '__custom'
			}
		},
		'tooltip-format-off': {
			type: 'select',
			title: 'Off Tooltip',
			description: 'Tooltip when controller is turned off',
			enum: [
				'__default',
				'Bluetooth Off',
				'{controller_alias} - Off',
				'Bluetooth controller is off',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Bluetooth Off',
				'Controller - Off',
				'Bluetooth controller is off',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-off-custom': {
			type: 'string',
			title: 'Custom Off Tooltip',
			description: 'Enter custom tooltip for off state',
			placeholder: 'Bluetooth Off',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-off',
				value: '__custom'
			}
		},
		'tooltip-format-on': {
			type: 'select',
			title: 'On Tooltip',
			description: 'Tooltip when controller is on with no devices',
			enum: [
				'__default',
				'{controller_alias} - Ready',
				'Bluetooth On',
				'{controller_alias}\nNo devices connected',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Controller - Ready',
				'Bluetooth On',
				'Controller + No devices',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-on-custom': {
			type: 'string',
			title: 'Custom On Tooltip',
			description: 'Enter custom tooltip for on state',
			placeholder: 'Bluetooth On',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-on',
				value: '__custom'
			}
		},
		'tooltip-format-connected': {
			type: 'select',
			title: 'Connected Tooltip',
			description: 'Tooltip when devices are connected',
			enum: [
				'__default',
				'{controller_alias}\t{controller_address}\n\n{device_enumerate}',
				'{controller_alias} - {num_connections} connected\n\n{device_enumerate}',
				'{num_connections} device(s) connected\n\n{device_enumerate}',
				'{device_enumerate}',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Controller + device list',
				'Controller + count + device list',
				'Count + device list',
				'Device list only',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-connected-custom': {
			type: 'string',
			title: 'Custom Connected Tooltip',
			description: 'Enter custom tooltip for connected state',
			placeholder: '{controller_alias}\n\n{device_enumerate}',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-connected',
				value: '__custom'
			}
		},
		'tooltip-format-enumerate-connected': {
			type: 'select',
			title: 'Device List Item Format',
			description: 'Format for each device in the {device_enumerate} list',
			enum: [
				'__default',
				'{device_alias}\t{device_address}',
				'{device_alias}',
				'• {device_alias}',
				'󰂱 {device_alias} ({device_address})',
				'__custom'
			],
			enumLabels: [
				'Default format',
				'Device alias + address',
				'Device alias only',
				'Bullet + device alias',
				'Icon + device alias (address)',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-enumerate-connected-custom': {
			type: 'string',
			title: 'Custom Device List Item Format',
			description: 'Enter custom format for device list items',
			placeholder: '{device_alias}\t{device_address}',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-enumerate-connected',
				value: '__custom'
			}
		},
		'tooltip-format-connected-battery': {
			type: 'select',
			title: 'Connected Battery Tooltip',
			description: '⚠️ Experimental: Tooltip when device has battery info',
			enum: [
				'__default',
				'{controller_alias} - {num_connections} connected\n\n{device_enumerate}',
				'{device_alias} - {device_battery_percentage}%',
				'{num_connections} device(s)\n\n{device_enumerate}',
				'__custom'
			],
			enumLabels: [
				'Inherit connected tooltip',
				'Controller + count + device list',
				'Device + battery %',
				'Count + device list',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-connected-battery-custom': {
			type: 'string',
			title: 'Custom Connected Battery Tooltip',
			description: 'Enter custom tooltip for connected device with battery',
			placeholder: '{device_alias} - {device_battery_percentage}%',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-connected-battery',
				value: '__custom'
			}
		},
		'tooltip-format-enumerate-connected-battery': {
			type: 'select',
			title: 'Battery Device List Item Format',
			description: '⚠️ Experimental: Format for devices with battery in {device_enumerate} list',
			enum: [
				'__default',
				'{device_alias}\t{device_address}\t{device_battery_percentage}%',
				'{device_alias} - {device_battery_percentage}%',
				'• {device_alias} (󰥉 {device_battery_percentage}%)',
				'󰂱 {device_alias} 󰥉 {device_battery_percentage}%',
				'__custom'
			],
			enumLabels: [
				'Inherit enumerate-connected format',
				'Device + address + battery %',
				'Device - battery %',
				'Bullet + device (battery icon + %)',
				'BT icon + device + battery icon + %',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-enumerate-connected-battery-custom': {
			type: 'string',
			title: 'Custom Battery Device List Format',
			description: 'Enter custom format for battery device list items',
			placeholder: '{device_alias}\t{device_battery_percentage}%',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-enumerate-connected-battery',
				value: '__custom'
			}
		},

		// Actions
		'on-click': {
			type: 'string',
			title: 'Left Click Command',
			description: 'Command to execute when left-clicking the module',
			placeholder: 'blueberry',
			default: '',
			tab: 'actions'
		},
		'on-click-middle': {
			type: 'string',
			title: 'Middle Click Command',
			description: 'Command to execute when middle-clicking',
			placeholder: 'blueman-manager',
			default: '',
			tab: 'actions'
		},
		'on-click-right': {
			type: 'string',
			title: 'Right Click Command',
			description: 'Command to execute when right-clicking',
			placeholder: 'blueman-manager',
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
		// 	description: 'Controller selection and display settings'
		// },
		{
			id: 'formats',
			label: 'Formats',
			description: 'Display formats for different Bluetooth states'
		}
		// {
		// 	id: 'tooltip',
		// 	label: 'Tooltip',
		// 	description: 'Tooltip formats for each state and device list'
		// },
		// {
		// 	id: 'actions',
		// 	label: 'Actions',
		// 	description: 'Mouse and scroll interactions'
		// }
	]
};
