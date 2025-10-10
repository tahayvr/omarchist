<script>
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import ThemeIcon from '@lucide/svelte/icons/swatch-book';
	import DocsIcon from '@lucide/svelte/icons/library';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import SidebarToggle from './SidebarToggle.svelte';
	import InfoIcon from '@lucide/svelte/icons/info';
	import DropIcon from '@lucide/svelte/icons/droplet';
	import OmarchyIcon from '$lib/icons/OmarchyIcon.svelte';
	import BoltIcon from '@lucide/svelte/icons/bolt';
	import KeyboardIcon from '@lucide/svelte/icons/keyboard';
	import MouseIcon from '@lucide/svelte/icons/mouse';
	import Bolt from '@lucide/svelte/icons/bolt';

	// Menu items.
	const items = [
		{
			title: 'General',
			url: '/general',
			icon: BoltIcon
		},
		{
			title: 'Keyboard',
			url: '/keyboard',
			icon: KeyboardIcon
		},
		{
			title: 'Mouse & Touchpad',
			url: '/mouse',
			icon: MouseIcon
		},
		{
			title: 'Omarchy',
			url: '/omarchy',
			icon: OmarchyIcon
		}
	];
</script>

<Sidebar.Root collapsible="icon">
	<Sidebar.Header class="border-b">
		<SidebarToggle />
	</Sidebar.Header>
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					<Sidebar.MenuItem>
						<Sidebar.MenuButton>
							{#snippet child({ props })}
								<a href="/themes" {...props}>
									<ThemeIcon />
									<span class="font-semibold uppercase">Themes</span>
								</a>
							{/snippet}
						</Sidebar.MenuButton>
					</Sidebar.MenuItem>
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
		<Sidebar.Group>
			<Sidebar.GroupLabel>Configs</Sidebar.GroupLabel>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each items as item (item.title)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a href={item.url} {...props}>
										<item.icon />
										<span class="font-semibold uppercase">{item.title}</span>
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
						<a href="/settings" {...props}>
							<SettingsIcon />
							<span class="font-semibold">Settings</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton>
					{#snippet child({ props })}
						<a href="/about" {...props}>
							<InfoIcon />
							<span class="font-semibold">About</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton>
					{#snippet child({ props })}
						<a href="https://manuals.omamix.org/2/the-omarchy-manual" target="_blank" {...props}>
							<DocsIcon />
							<span class="font-semibold">Omarchy Docs</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
				<Sidebar.MenuBadge>
					<ExternalLinkIcon class="h-3 w-3" />
				</Sidebar.MenuBadge>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>
	<Sidebar.Rail />
</Sidebar.Root>
