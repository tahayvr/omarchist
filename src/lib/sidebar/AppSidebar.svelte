<script>
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { page } from '$app/stores';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import ThemeIcon from '@lucide/svelte/icons/swatch-book';
	import DocsIcon from '@lucide/svelte/icons/library';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import SidebarToggle from './SidebarToggle.svelte';
	import InfoIcon from '@lucide/svelte/icons/info';
	import OmarchyIcon from '$lib/icons/OmarchyIcon.svelte';
	import BoltIcon from '@lucide/svelte/icons/bolt';
	import KeyboardIcon from '@lucide/svelte/icons/keyboard';
	import MouseIcon from '@lucide/svelte/icons/mouse';
	import StatusbarIcon from '@lucide/svelte/icons/rectangle-ellipsis';

	$: currentPath = $page.url.pathname;

	const isActive = (href) => {
		if (!href) return false;
		if (href === '/') {
			return currentPath === '/';
		}

		return currentPath === href || currentPath.startsWith(`${href}/`);
	};

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
		},
		{
			title: 'Status Bar',
			url: '/status-bar',
			icon: StatusbarIcon
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
								<a {...props} href="/themes">
									<ThemeIcon class={isActive('/themes') ? 'text-accent-foreground' : ''} />
									<span
										class="font-semibold uppercase"
										class:text-accent-foreground={isActive('/themes')}
									>
										Themes
									</span>
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
									<a {...props} href={item.url}>
										<item.icon class={isActive(item.url) ? 'text-accent-foreground' : ''} />
										<span
											class="font-semibold uppercase"
											class:text-accent-foreground={isActive(item.url)}
										>
											{item.title}
										</span>
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
						<a {...props} href="/settings">
							<SettingsIcon class={isActive('/settings') ? 'text-accent-foreground' : ''} />
							<span class="font-semibold" class:text-accent-foreground={isActive('/settings')}>
								Settings
							</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton>
					{#snippet child({ props })}
						<a {...props} href="/about">
							<InfoIcon class={isActive('/about') ? 'text-accent-foreground' : ''} />
							<span class="font-semibold" class:text-accent-foreground={isActive('/about')}>
								About
							</span>
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
