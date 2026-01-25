export function generateWaybarStyles(globals, moduleStyles) {
	let css = '';

	css += "@import '../omarchy/current/theme/waybar.css';\n\n";

	css += '* {\n';
	css += `  background-color: ${globals.background};\n`;
	css += `  color: ${globals.foreground};\n`;
	css += '  border: none;\n';
	css += '  border-radius: 0;\n';
	css += '  min-height: 0;\n';
	css += "  font-family: 'JetBrainsMono Nerd Font';\n";
	css += '  font-size: 12px;\n';
	css += '}\n\n';

	css += '.modules-left {\n';
	css += `  margin-left: ${globals.leftMargin}px;\n`;
	if (globals.leftPadding > 0) {
		css += `  padding: ${globals.leftPadding}px;\n`;
	}
	if (globals.leftBackground) {
		css += `  background: ${globals.leftBackground};\n`;
	}
	css += '}\n\n';

	if (globals.centerMargin > 0 || globals.centerPadding > 0 || globals.centerBackground) {
		css += '.modules-center {\n';
		css += `  margin-left: ${globals.centerMargin}px;\n`;
		css += `  margin-right: ${globals.centerMargin}px;\n`;
		if (globals.centerPadding > 0) {
			css += `  padding: ${globals.centerPadding}px;\n`;
		}
		if (globals.centerBackground) {
			css += `  background: ${globals.centerBackground};\n`;
		}
		css += '}\n\n';
	}

	css += '.modules-right {\n';
	css += `  margin-right: ${globals.rightMargin}px;\n`;
	if (globals.rightPadding > 0) {
		css += `  padding: ${globals.rightPadding}px;\n`;
	}
	if (globals.rightBackground) {
		css += `  background: ${globals.rightBackground};\n`;
	}
	css += '}\n\n';

	css += '#workspaces button {\n';
	css += '  all: initial;\n';
	css += '  padding: 0 6px;\n';
	css += '  margin: 0 1.5px;\n';
	css += '  min-width: 9px;\n';
	css += '}\n\n';

	css += '#workspaces button.empty {\n';
	css += '  opacity: 0.5;\n';
	css += '}\n\n';

	const commonModules = [
		'#cpu',
		'#battery',
		'#pulseaudio',
		'#custom-omarchy',
		'#custom-screenrecording-indicator',
		'#custom-update'
	];
	css += commonModules.join(',\n') + ' {\n';
	css += '  min-width: 12px;\n';
	css += '  margin: 0 7.5px;\n';
	css += '}\n\n';

	css += '#tray {\n';
	css += '  margin-right: 16px;\n';
	css += '}\n\n';

	css += '#bluetooth {\n';
	css += '  margin-right: 17px;\n';
	css += '}\n\n';

	css += '#network {\n';
	css += '  margin-right: 13px;\n';
	css += '}\n\n';

	css += '#custom-expand-icon {\n';
	css += '  margin-right: 18px;\n';
	css += '}\n\n';

	css += 'tooltip {\n';
	css += '  padding: 2px;\n';
	css += '}\n\n';

	css += '#custom-update {\n';
	css += '  font-size: 10px;\n';
	css += '}\n\n';

	css += '#clock {\n';
	css += '  margin-left: 8.75px;\n';
	css += '}\n\n';

	css += '.hidden {\n';
	css += '  opacity: 0;\n';
	css += '}\n\n';

	css += '#custom-screenrecording-indicator {\n';
	css += '  min-width: 12px;\n';
	css += '  margin-left: 5px;\n';
	css += '  font-size: 10px;\n';
	css += '  padding-bottom: 1px;\n';
	css += '}\n\n';

	css += '#custom-screenrecording-indicator.active {\n';
	css += '  color: #a55555;\n';
	css += '}\n\n';

	css += '#custom-voxtype {\n';
	css += '  min-width: 12px;\n';
	css += '  margin: 0 0 0 7.5px;\n';
	css += '}\n\n';

	css += '#custom-voxtype.recording {\n';
	css += '  color: #a55555;\n';
	css += '}\n\n';

	css += generateModuleStylesCss(moduleStyles);

	return css;
}

function generateModuleStylesCss(moduleStyles) {
	let css = '';
	const buttonProps = new Set([
		'background',
		'background-color',
		'color',
		'border',
		'border-radius',
		'padding',
		'margin'
	]);

	for (const [moduleId, styles] of Object.entries(moduleStyles)) {
		if (!styles || Object.keys(styles).length === 0) continue;

		const cssId = moduleId.replace(/\//g, '-');
		const isWorkspaces = moduleId === 'hyprland/workspaces';

		const containerStyles = [];
		const buttonStyles = [];

		for (const [prop, val] of Object.entries(styles)) {
			if (val === undefined || val === null || val === '') continue;

			const kebabProp = prop.replace(/[A-Z]/g, (m) => `-${m.toLowerCase()}`);

			if (isWorkspaces && buttonProps.has(kebabProp)) {
				buttonStyles.push(`${kebabProp}: ${val}`);
			} else {
				containerStyles.push(`${kebabProp}: ${val}`);
			}
		}

		if (containerStyles.length > 0) {
			css += `#${cssId} {\n`;
			containerStyles.forEach((s) => (css += `  ${s};\n`));
			css += '}\n\n';
		}

		if (isWorkspaces && buttonStyles.length > 0) {
			css += `#${cssId} button {\n`;
			buttonStyles.forEach((s) => (css += `  ${s};\n`));
			css += '}\n\n';
		}
	}

	return css;
}
