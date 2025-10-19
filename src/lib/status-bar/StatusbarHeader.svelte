<script>
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import MoreIcon from '@lucide/svelte/icons/ellipsis-vertical';
	import { invoke } from '@tauri-apps/api/core';

	let {
		isLoading = false,
		isSaving = false,
		dirty = false,
		isValid = true,
		profiles = [],
		selectedProfileId = null,
		profilesLoading = false,
		profileBusy = false,
		profileInteractionLocked = false,
		onReset = () => {},
		onProfileSelect = async () => {},
		onProfileCreate = async () => {},
		onProfileDelete = async () => {},
		onProfileShare = () => {}
	} = $props();

	const createDialog = $state({ open: false, name: '' });
	const deleteDialog = $state({ open: false, profileId: null });

	const currentProfile = $derived(profiles.find((entry) => entry.id === selectedProfileId) ?? null);
	const currentProfileName = $derived(currentProfile?.name ?? '');
	const isProfileProtected = $derived(
		Boolean(currentProfile?.is_default ?? currentProfile?.is_protected)
	);
	const selectDisabled = $derived(profileInteractionLocked || profilesLoading || profileBusy);
	const profileLabel = $derived(
		profileBusy
			? 'Switching…'
			: profilesLoading
				? 'Loading configurations…'
				: currentProfileName || (profiles.length ? 'Select configuration' : 'Create configuration')
	);
	const status = $derived(
		!isValid
			? { label: 'Needs attention', className: 'text-red-500' }
			: isSaving
				? { label: 'Saving…', className: 'text-muted-foreground' }
				: dirty
					? { label: 'Unsaved changes', className: 'text-amber-400' }
					: { label: 'Up to date', className: 'text-muted-foreground' }
	);
	const deleteTarget = $derived(
		profiles.find((entry) => entry.id === deleteDialog.profileId) ?? null
	);

	function handleReset() {
		onReset?.();
	}

	async function restartApp(appName) {
		await invoke('execute_bash_command_async', {
			command: `omarchy-restart-app ${appName}`
		});
	}

	async function handleProfileSelect(value) {
		if (!value || value === selectedProfileId) {
			return;
		}
		if (selectDisabled) {
			return;
		}
		await onProfileSelect?.(value);
	}

	function openCreateDialog() {
		if (profileInteractionLocked || profileBusy) {
			return;
		}
		createDialog.open = true;
	}

	async function submitCreate() {
		const name = createDialog.name?.trim();
		if (!name) {
			return;
		}
		if (profileInteractionLocked || profileBusy) {
			return;
		}
		const created = await onProfileCreate?.(name);
		if (created) {
			createDialog.open = false;
			createDialog.name = '';
		}
	}

	function handleCreateKeydown(event) {
		if (event.key === 'Enter') {
			event.preventDefault();
			submitCreate();
		}
	}

	function triggerShare() {
		const targetId = currentProfile?.id ?? null;
		onProfileShare?.(targetId);
	}

	function openDeleteDialog() {
		if (!currentProfile || isProfileProtected || profileInteractionLocked || profileBusy) {
			return;
		}
		deleteDialog.profileId = currentProfile.id;
		deleteDialog.open = true;
	}

	async function confirmDelete() {
		const targetId = deleteDialog.profileId;
		if (!targetId) {
			deleteDialog.open = false;
			return;
		}
		if (profileInteractionLocked || profileBusy) {
			return;
		}
		const deleted = await onProfileDelete?.(targetId);
		if (deleted) {
			deleteDialog.open = false;
			deleteDialog.profileId = null;
		}
	}

	$effect(() => {
		if (!createDialog.open && createDialog.name) {
			createDialog.name = '';
		}
	});

	$effect(() => {
		if (!deleteDialog.open && deleteDialog.profileId) {
			deleteDialog.profileId = null;
		}
	});
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="text-accent-foreground uppercase">Status Bar</Card.Title>
		<Card.Description class="text-xs tracking-wide uppercase">
			Waybar configuration
		</Card.Description>
	</Card.Header>
	<Card.Content class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between md:gap-4">
		<div class="flex flex-col gap-2 md:flex-row md:items-center md:gap-3">
			<div class="flex items-center gap-3">
				<Select.Root
					type="single"
					value={selectedProfileId ?? undefined}
					onValueChange={handleProfileSelect}
					disabled={selectDisabled}
				>
					<Select.Trigger class="w-64">
						<span class="truncate text-left text-xs font-semibold tracking-wide uppercase">
							{profileLabel}
						</span>
					</Select.Trigger>
					<Select.Content>
						{#if profiles.length === 0}
							<Select.Item value="__empty" disabled>No configurations yet</Select.Item>
						{:else}
							{#each profiles as profile (profile.id)}
								<Select.Item value={profile.id}>
									<div class="flex w-full items-center justify-between gap-2 text-xs uppercase">
										<span class="truncate font-semibold">{profile.name}</span>
										{#if profile.is_active}
											<span class="text-primary text-[0.6rem] font-medium">Active</span>
										{/if}
									</div>
								</Select.Item>
							{/each}
						{/if}
					</Select.Content>
				</Select.Root>

				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon"
								class="h-9.5 w-8 p-0"
								disabled={profileInteractionLocked || profileBusy}
							>
								<MoreIcon class="h-4 w-4" />
								<span class="sr-only">Open options</span>
							</Button>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="start" class="uppercase">
						<DropdownMenu.Group>
							<DropdownMenu.Item
								onclick={openCreateDialog}
								disabled={profileInteractionLocked || profileBusy}
							>
								New Config
							</DropdownMenu.Item>
							<DropdownMenu.Item
								onclick={triggerShare}
								disabled={!currentProfile || profileInteractionLocked || profileBusy}
							>
								Share Config
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item
								onclick={openDeleteDialog}
								disabled={!currentProfile ||
									isProfileProtected ||
									profileInteractionLocked ||
									profileBusy}
								variant="destructive"
							>
								Delete Config
							</DropdownMenu.Item>
						</DropdownMenu.Group>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
				<span class={`text-xs font-semibold tracking-wide uppercase ${status.className}`}>
					{status.label}
				</span>
			</div>
		</div>
		<div class="flex items-center gap-3">
			<Button
				class="uppercase"
				variant="ghost"
				disabled={isLoading || isSaving}
				onclick={handleReset}
			>
				Reset
			</Button>
			<Button class="uppercase" variant="outline" onclick={() => restartApp('waybar')}>
				Restart Status Bar
			</Button>
		</div>
	</Card.Content>

	<Dialog.Root bind:open={createDialog.open}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title class="text-sm font-semibold tracking-wide uppercase">
					New Waybar Configuration
				</Dialog.Title>
				<Dialog.Description class="text-muted-foreground text-xs uppercase">
					Provide a descriptive name so you can recognize this layout later.
				</Dialog.Description>
			</Dialog.Header>
			<div class="space-y-2">
				<label class="text-xs font-semibold tracking-wide uppercase" for="waybar-profile-name">
					Name
				</label>
				<Input
					id="waybar-profile-name"
					placeholder="e.g. Daily Driver"
					bind:value={createDialog.name}
					disabled={profileInteractionLocked || profileBusy}
					on:keydown={handleCreateKeydown}
				/>
			</div>
			<Dialog.Footer>
				<Dialog.Close asChild>
					<Button
						type="button"
						variant="outline"
						disabled={profileInteractionLocked || profileBusy}
					>
						Cancel
					</Button>
				</Dialog.Close>
				<Button
					type="button"
					onclick={submitCreate}
					disabled={profileInteractionLocked || profileBusy || !createDialog.name?.trim()}
				>
					Create
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<AlertDialog.Root bind:open={deleteDialog.open}>
		<AlertDialog.Content>
			<AlertDialog.Header>
				<AlertDialog.Title class="text-sm font-semibold tracking-wide uppercase">
					Delete Waybar Configuration
				</AlertDialog.Title>
				<AlertDialog.Description class="text-muted-foreground text-xs uppercase">
					This removes
					<span class="text-foreground font-semibold"> {deleteTarget?.name ?? 'this config'} </span>
					from Omarchist. The active profile will switch back to the default layout.
				</AlertDialog.Description>
			</AlertDialog.Header>
			<AlertDialog.Footer>
				<AlertDialog.Cancel disabled={profileInteractionLocked || profileBusy}>
					Keep Config
				</AlertDialog.Cancel>
				<AlertDialog.Action
					disabled={profileInteractionLocked || profileBusy}
					onclick={confirmDelete}
				>
					Delete
				</AlertDialog.Action>
			</AlertDialog.Footer>
		</AlertDialog.Content>
	</AlertDialog.Root>
</Card.Root>
