export const networkSchema = {
	type: 'object',
	title: 'Network',
	description: '',
	properties: {
		interface: {
			type: 'string',
			title: 'Network Interface',
			description:
				'Select a specific interface to monitor (e.g., wlan0, eth0). Leave empty for auto-detection. Wildcards are allowed.',
			placeholder: 'Auto-detect',
			default: '__default',
			tab: 'general'
		},
		family: {
			type: 'select',
			title: 'IP Address Family',
			description: 'Which IP version to display and use for connection detection',
			enum: ['ipv4', 'ipv6', 'ipv4_6'],
			enumLabels: ['IPv4 (default)', 'IPv6', 'Dual stack (IPv4 + IPv6)'],
			default: 'ipv4',
			tab: 'general'
		},
		interval: {
			type: 'select',
			title: 'Update Interval',
			description: 'How often to poll network status (seconds)',
			enum: [1, 2, 3, 5, 10, 30, 60],
			enumLabels: [
				'1 second',
				'2 seconds',
				'3 seconds',
				'5 seconds',
				'10 seconds',
				'30 seconds',
				'60 seconds (default)'
			],
			default: 3,
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
		rotate: {
			type: 'integer',
			title: 'Rotation',
			description: 'Rotate the module output clockwise (degrees)',
			minimum: 0,
			maximum: 360,
			placeholder: '0',
			tab: 'general'
		},

		format: {
			type: 'select',
			title: 'Default Format',
			description: 'Fallback format used when connection-specific formats are not set',
			enum: [
				'__default',
				'{ifname}',
				'{icon}',
				'{icon} {ifname}',
				'{ifname} {ipaddr}',
				'{icon} {ipaddr}',
				'{ipaddr}',
				'__custom'
			],
			enumLabels: [
				'Default ({ifname})',
				'Interface name only',
				'Signal icon only',
				'Signal icon + interface',
				'Interface + IP address',
				'Signal icon + IP address',
				'IP address only',
				'Custom format...'
			],
			default: '{ifname}',
			tab: 'formats'
		},
		'format-custom': {
			type: 'string',
			title: 'Custom Default Format',
			description: 'Enter a custom default format string',
			placeholder: '{ifname}',
			tab: 'formats',
			visibleWhen: {
				field: 'format',
				value: '__custom'
			}
		},
		'format-icons': {
			type: 'array',
			format: 'textarea',
			title: 'Signal Strength Icons',
			description: 'Provide one icon per line from weakest to strongest when using {icon}.',
			placeholder: '󰤯\n󰤟\n󰤢\n󰤥\n󰤨',
			default: [],
			tab: 'formats'
		},
		'format-wifi': {
			type: 'select',
			title: 'WiFi Format',
			description: 'Format applied when a wireless interface is displayed',
			enum: [
				'__default',
				'{icon}',
				'{icon} {essid}',
				'{icon} {signalStrength}%',
				'{icon} {essid} ({signalStrength}%)',
				'{essid}',
				'{essid} ({signalStrength}%)',
				'{signalStrength}%',
				'{ipaddr}',
				'{essid} ({signalStrength}%) {bandwidthDownBytes}',
				' ',
				'__custom'
			],
			enumLabels: [
				'Inherit default format',
				'Signal icon only',
				'Signal icon + SSID',
				'Signal icon + signal percent',
				'Signal icon + SSID + percent',
				'SSID only',
				'SSID with signal percent',
				'Signal percent only',
				'IP address only',
				'SSID + percent + download speed',
				'Icon only (Omarchy)',
				'Custom format...'
			],
			default: ' ',
			tab: 'formats'
		},
		'format-wifi-custom': {
			type: 'string',
			title: 'Custom WiFi Format',
			description: 'Enter a custom WiFi format string',
			placeholder: '{essid} ({signalStrength}%)',
			tab: 'formats',
			visibleWhen: {
				field: 'format-wifi',
				value: '__custom'
			}
		},
		'format-ethernet': {
			type: 'select',
			title: 'Ethernet Format',
			description: 'Format applied when a wired interface is displayed',
			enum: [
				'__default',
				'󰊗 {ipaddr}/{cidr}',
				'󰊗',
				'󰊗 {ifname}',
				'󰊗 {ipaddr}',
				'󰊗 {bandwidthDownBytes}',
				'{ifname}',
				'󰈀',
				'__custom'
			],
			enumLabels: [
				'Inherit default format',
				'Icon + IP/CIDR',
				'Icon only',
				'Icon + interface',
				'Icon + IP address',
				'Icon + download speed',
				'Interface name only',
				'Icon only (Omarchy)',
				'Custom format...'
			],
			default: '󰈀',
			tab: 'formats'
		},
		'format-ethernet-custom': {
			type: 'string',
			title: 'Custom Ethernet Format',
			description: 'Enter a custom ethernet format string',
			placeholder: '󰊗 {ipaddr}/{cidr}',
			tab: 'formats',
			visibleWhen: {
				field: 'format-ethernet',
				value: '__custom'
			}
		},
		'format-linked': {
			type: 'select',
			title: 'Linked Format',
			description: 'Format applied when the interface is linked but has no IP address',
			enum: ['__default', '{ifname} (No IP)', '󰌘 {ifname}', '󰌘', 'No IP', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Interface (No IP)',
				'Warning icon + interface',
				'Warning icon only',
				'Text "No IP"',
				'Custom format...'
			],
			default: '{ifname} (No IP)',
			tab: 'formats'
		},
		'format-linked-custom': {
			type: 'string',
			title: 'Custom Linked Format',
			description: 'Enter a custom linked format string',
			placeholder: '{ifname} (No IP)',
			tab: 'formats',
			visibleWhen: {
				field: 'format-linked',
				value: '__custom'
			}
		},
		'format-disconnected': {
			type: 'select',
			title: 'Disconnected Format',
			description: 'Format applied when the interface is disconnected (empty hides the module)',
			enum: ['__default', '󰤮', '󰤮 Disconnected', '', '󰤭', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Disconnected icon',
				'Disconnected icon + text',
				'Hide module (empty)',
				'Alternative disconnected icon',
				'Custom format...'
			],
			default: '',
			tab: 'formats'
		},
		'format-disconnected-custom': {
			type: 'string',
			title: 'Custom Disconnected Format',
			description: 'Enter a custom disconnected format string (empty hides the module)',
			placeholder: '󰤮',
			tab: 'formats',
			visibleWhen: {
				field: 'format-disconnected',
				value: '__custom'
			}
		},
		'format-disabled': {
			type: 'select',
			title: 'Disabled Format',
			description: 'Format applied when the interface is disabled',
			enum: ['__default', '󰤮', '󰤮 Disabled', '', '󰖪', '__custom'],
			enumLabels: [
				'Inherit default format',
				'Disconnected icon',
				'Disconnected icon + text',
				'Hide module (empty)',
				'Airplane mode icon',
				'Custom format...'
			],
			default: '󰤮 Disabled',
			tab: 'formats'
		},
		'format-disabled-custom': {
			type: 'string',
			title: 'Custom Disabled Format',
			description: 'Enter a custom disabled format string',
			placeholder: '󰤮',
			tab: 'formats',
			visibleWhen: {
				field: 'format-disabled',
				value: '__custom'
			}
		},
		'format-alt': {
			type: 'string',
			title: 'Alternate Format',
			description: 'Format toggled via click',
			placeholder: '{ifname} {ipaddr}',
			tab: 'formats'
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
			description: 'Tooltip format for the general state',
			enum: [
				'__default',
				'{ifname} via {gwaddr}',
				'{ifname}',
				'{ipaddr}/{cidr}',
				'{ifname} ({essid}): {ipaddr}',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Interface via gateway',
				'Interface name only',
				'IP address / CIDR',
				'Interface (SSID): IP',
				'Custom format...'
			],
			default: '{ifname} ({essid}): {ipaddr}',
			tab: 'tooltip'
		},
		'tooltip-format-custom': {
			type: 'string',
			title: 'Custom Tooltip Format',
			description: 'Enter a custom tooltip format string',
			placeholder: '{ifname} via {gwaddr}',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format',
				value: '__custom'
			}
		},
		'tooltip-format-wifi': {
			type: 'select',
			title: 'WiFi Tooltip Format',
			description: 'Tooltip format used for wireless interfaces',
			enum: [
				'__default',
				'{essid} ({signalStrength}%)',
				'{essid} ({signalStrength}%) - {frequency} GHz',
				'{essid}\n{ipaddr}/{cidr}\n⇣{bandwidthDownBytes}  ⇡{bandwidthUpBytes}',
				'{essid} - {signaldBm} dBm',
				'{ifname} ({essid}): {ipaddr}',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'SSID with signal percent',
				'SSID with signal percent and frequency',
				'SSID + IP + bandwidth',
				'SSID with signal strength in dBm',
				'Interface (SSID): IP',
				'Custom format...'
			],
			default: '{ifname} ({essid}): {ipaddr}',
			tab: 'tooltip'
		},
		'tooltip-format-wifi-custom': {
			type: 'string',
			title: 'Custom WiFi Tooltip Format',
			description: 'Enter a custom WiFi tooltip format string',
			placeholder: '{essid} ({signalStrength}%)',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-wifi',
				value: '__custom'
			}
		},
		'tooltip-format-ethernet': {
			type: 'select',
			title: 'Ethernet Tooltip Format',
			description: 'Tooltip format used for wired interfaces',
			enum: [
				'__default',
				'{ifname}',
				'{ifname} - {ipaddr}/{cidr}',
				'⇣{bandwidthDownBytes}  ⇡{bandwidthUpBytes}',
				'{ifname}\n{ipaddr}/{cidr}\n⇣{bandwidthDownBytes}  ⇡{bandwidthUpBytes}',
				'{ifname}: {ipaddr}',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Interface name',
				'Interface with IP/CIDR',
				'Download / upload speeds',
				'Interface + IP + bandwidth',
				'Interface: IP',
				'Custom format...'
			],
			default: '{ifname}: {ipaddr}',
			tab: 'tooltip'
		},
		'tooltip-format-ethernet-custom': {
			type: 'string',
			title: 'Custom Ethernet Tooltip Format',
			description: 'Enter a custom ethernet tooltip format string',
			placeholder: '{ifname}',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-ethernet',
				value: '__custom'
			}
		},
		'tooltip-format-disconnected': {
			type: 'select',
			title: 'Disconnected Tooltip',
			description: 'Tooltip shown when the interface is disconnected',
			enum: [
				'__default',
				'Disconnected',
				'No network connection',
				'{ifname} - Disconnected',
				'__custom'
			],
			enumLabels: [
				'Inherit default tooltip',
				'Disconnected',
				'No network connection',
				'Interface - Disconnected',
				'Custom format...'
			],
			default: 'Disconnected',
			tab: 'tooltip'
		},
		'tooltip-format-disconnected-custom': {
			type: 'string',
			title: 'Custom Disconnected Tooltip',
			description: 'Enter a custom tooltip string for disconnected state',
			placeholder: 'Disconnected',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-disconnected',
				value: '__custom'
			}
		},
		'tooltip-format-disabled': {
			type: 'select',
			title: 'Disabled Tooltip',
			description: 'Tooltip shown when the interface is disabled',
			enum: ['__default', 'Disabled', 'Network disabled', '{ifname} - Disabled', '__custom'],
			enumLabels: [
				'Inherit default tooltip',
				'Disabled',
				'Network disabled',
				'Interface - Disabled',
				'Custom format...'
			],
			default: '__default',
			tab: 'tooltip'
		},
		'tooltip-format-disabled-custom': {
			type: 'string',
			title: 'Custom Disabled Tooltip',
			description: 'Enter a custom tooltip string for disabled state',
			placeholder: 'Disabled',
			tab: 'tooltip',
			visibleWhen: {
				field: 'tooltip-format-disabled',
				value: '__custom'
			}
		},

		'on-click': {
			type: 'string',
			title: 'Left Click Command',
			description: 'Command to execute when the module is left-clicked',
			placeholder: 'nm-connection-editor',
			default: 'nm-connection-editor',
			tab: 'actions'
		},
		'on-click-middle': {
			type: 'string',
			title: 'Middle Click Command',
			description: 'Command to execute when the module is middle-clicked',
			placeholder: 'nm-applet',
			default: '',
			tab: 'actions'
		},
		'on-click-right': {
			type: 'string',
			title: 'Right Click Command',
			description: 'Command to execute when the module is right-clicked',
			placeholder: 'networkmanager_dmenu',
			default: '',
			tab: 'actions'
		},
		'on-scroll-up': {
			type: 'string',
			title: 'Scroll Up Command',
			description: 'Command to execute when scrolling up on the module',
			default: '',
			tab: 'actions'
		},
		'on-scroll-down': {
			type: 'string',
			title: 'Scroll Down Command',
			description: 'Command to execute when scrolling down on the module',
			default: '',
			tab: 'actions'
		},
		'smooth-scrolling-threshold': {
			type: 'number',
			title: 'Smooth Scrolling Threshold',
			description: 'Threshold used for smoothing scroll events',
			minimum: 0,
			placeholder: '0',
			tab: 'actions'
		}
	},
	tabs: [
		{
			id: 'general',
			label: 'General',
			description: 'Interface selection and polling behaviour'
		},
		{
			id: 'formats',
			label: 'Formats',
			description: 'Display formats for different connection states'
		},
		{
			id: 'tooltip',
			label: 'Tooltip',
			description: 'Tooltip formats for each connection state'
		},
		{
			id: 'actions',
			label: 'Actions',
			description: 'Mouse and scroll interactions'
		}
	]
};
