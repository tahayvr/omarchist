import { clockSchema } from './schemas/clockSchema.js';
import { batterySchema } from './schemas/batterySchema.js';
import { networkSchema } from './schemas/networkSchema.js';
import { bluetoothSchema } from './schemas/bluetoothSchema.js';
import { cpuSchema } from './schemas/cpuSchema.js';
import { pulseaudioSchema } from './schemas/pulseaudioSchema.js';
import { hyprlandWindowSchema } from './schemas/hyprlandWindowSchema.js';
import { memorySchema } from './schemas/memorySchema.js';
import ClockModuleForm from '../../status-bar/modules/singleModules/ClockModuleForm.svelte';
import NetworkModuleForm from '../../status-bar/modules/singleModules/NetworkModuleForm.svelte';
import BluetoothModuleForm from '../../status-bar/modules/singleModules/BluetoothModuleForm.svelte';
import BatteryModuleForm from '../../status-bar/modules/singleModules/BatteryModuleForm.svelte';
import CpuModuleForm from '../../status-bar/modules/singleModules/CpuModuleForm.svelte';
import PulseaudioModuleForm from '../../status-bar/modules/singleModules/PulseaudioModuleForm.svelte';
import HyprlandWindowModuleForm from '../../status-bar/modules/singleModules/HyprlandWindowModuleForm.svelte';
import MemoryModuleForm from '../../status-bar/modules/singleModules/MemoryModuleForm.svelte';

export const moduleRegistry = {
	clock: {
		schema: clockSchema,
		component: ClockModuleForm,
		validator: null,
		configurable: true
	},
	battery: {
		schema: batterySchema,
		component: BatteryModuleForm,
		validator: null,
		configurable: true
	},
	bluetooth: {
		schema: bluetoothSchema,
		component: BluetoothModuleForm,
		validator: null,
		configurable: true
	},
	cpu: {
		schema: cpuSchema,
		component: CpuModuleForm,
		validator: null,
		configurable: true
	},
	pulseaudio: {
		schema: pulseaudioSchema,
		component: PulseaudioModuleForm,
		validator: null,
		configurable: true
	},
	network: {
		schema: networkSchema,
		component: NetworkModuleForm,
		validator: null,
		configurable: true
	},
	'hyprland/window': {
		schema: hyprlandWindowSchema,
		component: HyprlandWindowModuleForm,
		validator: null,
		configurable: true
	},
	'hyprland/workspaces': {
		schema: null,
		component: null,
		validator: null,
		configurable: true,
		defaultConfig: {
			'on-click': 'activate',
			format: '{icon}',
			'format-icons': {
				default: '',
				1: '1',
				2: '2',
				3: '3',
				4: '4',
				5: '5',
				6: '6',
				7: '7',
				8: '8',
				9: '9',
				10: '0',
				active: '󱓻'
			},
			'persistent-workspaces': {
				1: [],
				2: [],
				3: [],
				4: [],
				5: []
			}
		}
	},
	memory: {
		schema: memorySchema,
		component: MemoryModuleForm,
		validator: null,
		configurable: true
	},
	'custom/omarchy': {
		schema: null,
		component: null,
		validator: null,
		configurable: false,
		defaultConfig: {
			format: "<span font='omarchy'>\ue900</span>",
			'on-click': 'omarchy-menu',
			'on-click-right': 'xdg-terminal-exec',
			'tooltip-format': 'Omarchy Menu\n\nSuper + Alt + Space'
		}
	},
	'custom/update': {
		schema: null,
		component: null,
		validator: null,
		configurable: false,
		defaultConfig: {
			format: '',
			exec: 'omarchy-update-available',
			'on-click': 'omarchy-launch-floating-terminal-with-presentation omarchy-update',
			'tooltip-format': 'Omarchy update available',
			signal: 7,
			interval: 21600
		}
	},
	'custom/voxtype': {
		schema: null,
		component: null,
		validator: null,
		configurable: false,
		defaultConfig: {
			exec: 'omarchy-voxtype-status',
			'return-type': 'json',
			format: '{icon}',
			'format-icons': {
				idle: '',
				recording: '󰍬',
				transcribing: '󰔟'
			},
			tooltip: true,
			'on-click-right': 'omarchy-voxtype-config',
			'on-click': 'omarchy-voxtype-model'
		}
	},
	'custom/screenrecording-indicator': {
		schema: null,
		component: null,
		validator: null,
		configurable: false,
		defaultConfig: {
			'on-click': 'omarchy-cmd-screenrecord',
			exec: '$OMARCHY_PATH/default/waybar/indicators/screen-recording.sh',
			signal: 8,
			'return-type': 'json'
		}
	},
	'group/tray-expander': {
		schema: null,
		component: null,
		validator: null,
		configurable: false,
		defaultConfig: {
			orientation: 'inherit',
			drawer: {
				'transition-duration': 600,
				'children-class': 'tray-group-item'
			},
			modules: ['custom/expand-icon', 'tray']
		}
	},
	'custom/expand-icon': {
		schema: null,
		component: null,
		validator: null,
		configurable: false,
		defaultConfig: {
			format: '',
			tooltip: false,
			'on-scroll-up': '',
			'on-scroll-down': '',
			'on-scroll-left': '',
			'on-scroll-right': ''
		}
	},
	tray: {
		schema: null,
		component: null,
		validator: null,
		configurable: false,
		defaultConfig: {
			'icon-size': 12,
			spacing: 17
		}
	}
};

export function getModuleDefinition(moduleId) {
	if (!moduleId || typeof moduleId !== 'string') {
		return null;
	}
	return moduleRegistry[moduleId] || null;
}

export function hasModuleSchema(moduleId) {
	const def = getModuleDefinition(moduleId);
	return def && def.schema ? true : false;
}

export function getModuleSchema(moduleId) {
	const def = getModuleDefinition(moduleId);
	return def?.schema || null;
}

export function getModuleComponent(moduleId) {
	const def = getModuleDefinition(moduleId);
	return def?.component || null;
}

export function getModuleValidator(moduleId) {
	const def = getModuleDefinition(moduleId);
	return def?.validator || null;
}

export function isModuleConfigurable(moduleId) {
	const def = getModuleDefinition(moduleId);
	if (!def) {
		return false;
	}
	if (def.configurable !== undefined) {
		return def.configurable;
	}
	return def.schema ? true : false;
}
