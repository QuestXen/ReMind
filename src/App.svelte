<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import ReminderApp from '$lib/components/ReminderApp.svelte';
	import Loading from '$lib/components/Loading.svelte';
	import { reminders, settings, isLoading, loadingError } from '$lib/stores';
	import type { Reminder, AppSettings } from '$lib/stores';

	let retryCount = 0;
	const MAX_RETRIES = 3;
	let updateStatus = $state('checking'); // 'checking', 'updating', 'done'
	let updateMessage = $state('Update Checken...');

	interface UpdateInfo {
		available: boolean;
		version?: string;
		notes?: string;
		pubDate?: string;
	}

	async function checkForUpdates(): Promise<UpdateInfo | null> {
		try {
			updateMessage = 'Update Checken...';
			const updateInfo = await invoke<UpdateInfo>('check_for_updates');
			return updateInfo;
		} catch (error) {
			console.error('Failed to check for updates:', error);
			return null;
		}
	}

	async function installUpdate(): Promise<boolean> {
		try {
			updateStatus = 'updating';
			updateMessage = 'Update wird installiert...';
			
			await invoke('install_update');
			
			updateMessage = 'Update abgeschlossen. Anwendung wird neu gestartet...';
			return true;
		} catch (error) {
			console.error('Failed to install update:', error);
			return false;
		}
	}

	async function loadAppData() {
		try {
			isLoading.set(true);
			loadingError.set(null);

			const [loadedSettings, loadedReminders] = await Promise.all([
				invoke<Record<string, any>>('load_settings'),
				invoke<Reminder[]>('load_reminders')
			]);
			
			const appSettings: AppSettings = {
				autostartEnabled: loadedSettings.autostart_enabled || false,
				theme: loadedSettings.theme || null,
				notificationSound: loadedSettings.notification_sound !== false,
				...loadedSettings 
			};

			// Update stores
			settings.set(appSettings);
			reminders.set(loadedReminders);

			await new Promise(resolve => setTimeout(resolve, 300));

			isLoading.set(false);
			retryCount = 0;
			updateStatus = 'done';

			console.log('App data loaded successfully:', {
				settings: appSettings,
				remindersCount: loadedReminders.length
			});
		} catch (error) {
			console.error('Failed to load app data:', error);

			isLoading.set(false);
			updateStatus = 'done';
		}
	}

	async function retryLoading() {
		if (retryCount < MAX_RETRIES) {
			retryCount++;
			console.log(`Retry attempt ${retryCount}/${MAX_RETRIES}`);
			await loadAppData();
		} else {
			loadingError.set('Maximale Anzahl von Wiederholungsversuchen erreicht. Bitte starten Sie die Anwendung neu.');
		}
	}

	onMount(async () => {
		const updateInfo = await checkForUpdates();
		
		if (updateInfo?.available) {
			console.log(`Update available: ${updateInfo.version}`);
			updateMessage = `Update ${updateInfo.version} verfügbar. Installation wird gestartet...`;
			
			await new Promise(resolve => setTimeout(resolve, 1000));
			
			const updateSuccess = await installUpdate();
			
			if (!updateSuccess) {
				updateStatus = 'done';
				loadAppData();
			}
		} else {
			updateStatus = 'done';
			loadAppData();
		}
	});
</script>

<main class="bg-background min-h-screen">
	{#if $isLoading || updateStatus !== 'done'}
		<Loading 
			message={updateStatus === 'done' ? 'Lade Erinnerungen und Einstellungen' : updateMessage}
			error={$loadingError}
			retryCallback={$loadingError ? retryLoading : undefined}
			isUpdating={updateStatus === 'updating'}
		/>
	{:else}
		<ReminderApp />
	{/if}
</main>
