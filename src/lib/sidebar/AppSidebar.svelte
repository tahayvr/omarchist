<script>
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import HouseIcon from '@lucide/svelte/icons/house';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import ThemeIcon from '@lucide/svelte/icons/palette';
	import DocsIcon from '@lucide/svelte/icons/library';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import SidebarToggle from './SidebarToggle.svelte';

	// Menu items.
	const items = [
		{
			title: 'Themes',
			url: '/themes',
			icon: ThemeIcon
		},
		{
			title: 'Settings',
			url: '/settings',
			icon: SettingsIcon
		}
	];

	async function openDocs() {
		await openUrl('https://manuals.omamix.org/2/the-omarchy-manual');
	}
</script>

<Sidebar.Root collapsible="icon">
	<Sidebar.Header class="border-b">
		<SidebarToggle />
	</Sidebar.Header>
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each items as item (item.title)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a href={item.url} {...props}>
										<item.icon />
										<span class="uppercase">{item.title}</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>
	<Sidebar.Footer>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton>
					{#snippet child({ props })}
						<a href="https://manuals.omamix.org/2/the-omarchy-manual" target="_blank" {...props}>
							<DocsIcon />
							<span>Omarchy Docs</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
				<Sidebar.MenuBadge>
					<ExternalLinkIcon class="h-4 w-4" />
				</Sidebar.MenuBadge>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>
	<Sidebar.Rail />
</Sidebar.Root>
