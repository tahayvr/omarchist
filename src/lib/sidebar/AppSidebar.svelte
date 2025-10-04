<script>
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import ThemeIcon from '@lucide/svelte/icons/palette';
	import DocsIcon from '@lucide/svelte/icons/library';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import SidebarToggle from './SidebarToggle.svelte';
	import InfoIcon from '@lucide/svelte/icons/info';
	import DropIcon from '@lucide/svelte/icons/droplet';
	import SquareIcon from '@lucide/svelte/icons/square';

	// Menu items.
	const items = [
		{
			title: 'Hyprland',
			url: '/hyprland',
			icon: DropIcon
		},
		{
			title: 'Omarchy',
			url: '/omarchy',
			icon: SquareIcon
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
									<span class="uppercase">Themes</span>
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
						<a href="/settings" {...props}>
							<SettingsIcon />
							<span>Settings</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton>
					{#snippet child({ props })}
						<a href="/about" {...props}>
							<InfoIcon />
							<span>About</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
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
