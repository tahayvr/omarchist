import { defineConfig } from 'vitepress';

// https://vitepress.dev/reference/site-config
export default defineConfig({
	title: 'Omarchist Docs',
	description: 'Documentation for Omarchist. A GUI app for managing & theming Omarchy Linux.',
	head: [
		['link', { rel: 'icon', href: '/images/icon.png' }],
		['link', { rel: 'canonical', href: 'https://omarchist.com' }],

		// Basic SEO
		[
			'meta',
			{
				name: 'keywords',
				content:
					'Omarchist, Omarchy Linux, Hyprland, Waybar, Linux theming, GUI app, Linux customization'
			}
		],
		['meta', { name: 'author', content: 'Taha Nejad' }],
		['meta', { name: 'robots', content: 'index, follow' }],

		// Open Graph / Facebook
		['meta', { property: 'og:type', content: 'website' }],
		['meta', { property: 'og:url', content: 'https://omarchist.com' }],
		[
			'meta',
			{
				property: 'og:title',
				content: 'Omarchist Docs'
			}
		],
		[
			'meta',
			{
				property: 'og:description',
				content: 'Documentation for Omarchist. A GUI app for managing & theming Omarchy Linux.'
			}
		],
		['meta', { property: 'og:image', content: '/images/omarchist-social.png' }],
		['meta', { property: 'og:image:width', content: '1200' }],
		['meta', { property: 'og:image:height', content: '630' }],
		['meta', { property: 'og:site_name', content: 'Omarchist Docs' }],
		['meta', { property: 'og:locale', content: 'en_US' }],

		// Twitter
		['meta', { name: 'twitter:card', content: 'summary_large_image' }],
		['meta', { name: 'twitter:site', content: '@tahayvr' }],
		['meta', { name: 'twitter:creator', content: '@tahayvr' }],
		[
			'meta',
			{
				name: 'twitter:title',
				content: 'Omarchist Docs'
			}
		],
		[
			'meta',
			{
				name: 'twitter:description',
				content: 'Documentation for Omarchist. A GUI app for managing & theming Omarchy Linux.'
			}
		],
		['meta', { name: 'twitter:image', content: '/images/omarchist-social.png' }]
	],
	themeConfig: {
		// https://vitepress.dev/reference/default-theme-config
		logo: '/images/icon.png',
		siteTitle: 'OMARCHIST',
		nav: [{ text: 'Themes', link: 'https://omarchist.com/themes' }],

		sidebar: [
			{
				text: 'GETTING STARTED',
				items: [
					{ text: 'Introduction', link: '/' },
					{ text: 'Installation', link: '/#installation' },
					{ text: 'CLI', link: '/cli' }
				]
			},
			{
				text: 'THEMING',
				items: [
					{ text: 'Overview', link: '/theming/' },
					{ text: 'Theme Designer', link: '/theming/#theme-designer' },
					{ text: 'Sharing Themes', link: '/theming/sharing' }
				]
			},
			{
				text: 'CONFIGURATION',
				items: [
					{ text: 'Hyprland', link: '/configuration/' },
					{ text: 'Status Bar', link: '/status-bar' }
				]
			},
			{
				text: 'TOOLS',
				items: [{ text: 'System Monitor', link: '/system-monitor' }]
			}
		],

		editLink: {
			pattern: 'https://github.com/tahayvr/omarchist/docs/edit/dev/:path',
			text: 'Edit this page on GitHub'
		},

		socialLinks: [
			{ icon: 'github', link: 'https://github.com/tahayvr/omarchist' },
			{ icon: 'x', link: 'https://x.com/tahayvr' }
		],
		search: {
			provider: 'local'
		},
		footer: {
			message: 'Omarchist is released under the MIT License.',
			copyright: 'Copyright © 2026 <a href="https://taha.gg">Taha Nejad</a>'
		}
	},

	transformHead({ assets }) {
		const headLinks = [];

		// Preload JetBrains Mono font
		const jetBrainsFont = assets.find((file) => /JetBrainsMono.*\.(woff2|woff|ttf)/.test(file));
		if (jetBrainsFont) {
			const fontType = jetBrainsFont.endsWith('.woff2')
				? 'font/woff2'
				: jetBrainsFont.endsWith('.woff')
					? 'font/woff'
					: 'font/ttf';
			headLinks.push([
				'link',
				{
					rel: 'preload',
					href: jetBrainsFont,
					as: 'font',
					type: fontType,
					crossorigin: ''
				}
			]);
		}

		// Preload Open Sans font
		const openSansFont = assets.find((file) => /OpenSans.*\.(woff2|woff|ttf)/.test(file));
		if (openSansFont) {
			const fontType = openSansFont.endsWith('.woff2')
				? 'font/woff2'
				: openSansFont.endsWith('.woff')
					? 'font/woff'
					: 'font/ttf';
			headLinks.push([
				'link',
				{
					rel: 'preload',
					href: openSansFont,
					as: 'font',
					type: fontType,
					crossorigin: ''
				}
			]);
		}

		return headLinks;
	}
});
