<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import { Switch } from '$lib/components/ui/switch/index';
	import { ArrowLeft } from '@lucide/svelte';
	import { createEventDispatcher, onMount } from 'svelte';
	import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
	import { invoke } from '@tauri-apps/api/core';
	import TitleBar from './TitleBar.svelte';
	import { settings, updateSetting } from '$lib/stores';

	interface SystemInfo {
		app_version: string;
		os_name: string;
		os_version: string;
		arch: string;
	}

	const dispatch = createEventDispatcher<{ close: void }>();

	let loading = $state(false);
	let updating = $state(false);
	let systemInfo = $state<SystemInfo | null>(null);

	function handleClose() {
		dispatch('close');
	}

	async function toggleAutostart(checked: boolean) {
		if (updating) {
			console.log('[Frontend] Ignoring toggle - updating');
			return;
		}
		
		try {
			console.log('[Frontend] Toggling autostart to:', checked);
			updating = true;
			
			// Update store immediately for responsive UI
			updateSetting('autostartEnabled', checked);
			
			if (checked) {
				await enable();
				console.log('[Frontend] Autostart enabled in system');
			} else {
				await disable();
				console.log('[Frontend] Autostart disabled in system');
			}
			
			// Save to backend using new generic function
			await invoke('update_setting', { 
				key: 'autostartEnabled', 
				value: checked 
			});
			console.log('[Frontend] Autostart setting saved to backend');
			
		} catch (error) {
			console.error('[Frontend] Failed to toggle autostart:', error);
			// Revert store on error
			updateSetting('autostartEnabled', !checked);
			console.log('[Frontend] Reverted autostart setting due to error');
		} finally {
			updating = false;
		}
	}

	async function toggleNotificationSound(checked: boolean) {
		if (updating) {
			console.log('[Frontend] Ignoring toggle - updating');
			return;
		}
		
		try {
			console.log('[Frontend] Toggling notification sound to:', checked);
			updating = true;
			
			// Update store immediately for responsive UI
			updateSetting('notificationSound', checked);
			
			// Save to backend
			await invoke('update_setting', { 
				key: 'notificationSound', 
				value: checked 
			});
			console.log('[Frontend] Notification sound setting saved to backend');
			
		} catch (error) {
			console.error('[Frontend] Failed to toggle notification sound:', error);
			// Revert store on error
			updateSetting('notificationSound', !checked);
			console.log('[Frontend] Reverted notification sound setting due to error');
		} finally {
			updating = false;
		}
	}

	async function loadSystemInfo() {
		try {
			const info = await invoke<SystemInfo>('get_system_info');
			systemInfo = info;
		} catch (error) {
			console.error('[Frontend] Failed to load system info:', error);
		}
	}

	onMount(() => {
		loadSystemInfo();
	});
</script>

<div class="flex h-screen flex-col overflow-hidden">
	<TitleBar title="Einstellungen" icon="S" />

	<div class="bg-background animate-slide-in-right flex-1 overflow-y-auto relative">
		<div class="p-4">
			<Button
				onclick={handleClose}
				variant="ghost"
				class="text-muted-foreground hover:text-muted-foreground/80 hover:bg-muted/50 flex items-center gap-2 transition-colors"
			>
				<ArrowLeft class="h-4 w-4" />
				Zurück
			</Button>
		</div>

		<div class="px-6 pb-6">
			<div class="mx-auto max-w-4xl">
				<div class="mb-8 text-center">
					<h1 class="text-heading text-foreground mb-4 text-3xl">Einstellungen</h1>
				</div>

				<!-- Autostart Setting -->
				<div class="bg-card border-border rounded-3xl border p-8 mb-6">
					<div class="space-y-6">
						<div class="flex items-center justify-between">
							<div class="space-y-1">
								<h3 class="text-heading text-card-foreground text-lg">Autostart</h3>
								<p class="text-muted-foreground text-body text-sm">App automatisch beim Systemstart starten</p>
							</div>
							<div class="flex items-center">
								{#if loading}
									<div class="animate-spin rounded-full h-5 w-5 border-b-2 border-primary"></div>
								{:else}
									<Switch 
									bind:checked={$settings.autostartEnabled} 
									disabled={updating}
									onCheckedChange={toggleAutostart}
								/>
								{/if}
							</div>
						</div>
					</div>
				</div>

				<!-- Notification Sound Setting -->
				<div class="bg-card border-border rounded-3xl border p-8">
					<div class="space-y-6">
						<div class="flex items-center justify-between">
							<div class="space-y-1">
								<h3 class="text-heading text-card-foreground text-lg">Benachrichtigungssound</h3>
								<p class="text-muted-foreground text-body text-sm">Windows-Systemsound bei Benachrichtigungen abspielen</p>
							</div>
							<div class="flex items-center">
								{#if loading}
									<div class="animate-spin rounded-full h-5 w-5 border-b-2 border-primary"></div>
								{:else}
									<Switch 
									bind:checked={$settings.notificationSound} 
									disabled={updating}
									onCheckedChange={toggleNotificationSound}
								/>
								{/if}
							</div>
						</div>
					</div>
				</div>

			</div>
		</div>
	</div>

	{#if systemInfo}
		<div class="bg-background border-t border-border px-6 py-3">
			<div class="text-center space-y-1">
				<p class="text-muted-foreground text-xs">
					ReMind v{systemInfo.app_version} (Beta)
				</p>
				<p class="text-muted-foreground text-xs">
					{systemInfo.os_version} {systemInfo.arch}
				</p>
				<p class="text-muted-foreground text-xs italic">
					Dies ist eine unfertige Version - Fehler und Performance-Probleme können auftreten
				</p>
			</div>
		</div>
	{/if}
</div>

<style>
	@keyframes slide-in-right {
		from {
			opacity: 0;
			transform: translateX(20px);
		}
		to {
			opacity: 1;
			transform: translateX(0);
		}
	}

	.animate-slide-in-right {
		animation: slide-in-right 0.3s ease-out;
	}
</style>
