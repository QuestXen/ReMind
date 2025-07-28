<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import ReminderApp from '$lib/components/ReminderApp.svelte';
	import Loading from '$lib/components/Loading.svelte';
	import { reminders, settings, isLoading, loadingError } from '$lib/stores';
	import type { Reminder, AppSettings } from '$lib/stores';

	let retryCount = 0;
	const MAX_RETRIES = 3;

	async function loadAppData() {
		try {
			isLoading.set(true);
			loadingError.set(null);

			// Parallel loading of settings and reminders
			const [loadedSettings, loadedReminders] = await Promise.all([
				invoke<Record<string, any>>('load_settings'),
				invoke<Reminder[]>('load_reminders')
			]);
			
			// Convert loaded settings to AppSettings format
			const appSettings: AppSettings = {
				autostartEnabled: loadedSettings.autostart_enabled || false,
				theme: loadedSettings.theme || null,
				notificationSound: loadedSettings.notification_sound !== false,
				...loadedSettings // Include any additional dynamic settings
			};

			// Update stores
			settings.set(appSettings);
			reminders.set(loadedReminders);

			// Small delay to show loading animation
			await new Promise(resolve => setTimeout(resolve, 500));

			isLoading.set(false);
			retryCount = 0;

			console.log('App data loaded successfully:', {
				settings: appSettings,
				remindersCount: loadedReminders.length
			});
		} catch (error) {
			console.error('Failed to load app data:', error);
			loadingError.set(`Fehler beim Laden der Anwendungsdaten: ${error}`);
			isLoading.set(false);
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

	onMount(() => {
		// Delay the data loading to allow update check first
		setTimeout(() => {
			loadAppData();
		}, 2000); // Give update system time to check and install
	});
</script>

<main class="bg-background min-h-screen">
	{#if $isLoading}
		<Loading 
			message="Lade Erinnerungen und Einstellungen" 
			error={$loadingError}
			retryCallback={$loadingError ? retryLoading : undefined}
		/>
	{:else}
		<ReminderApp />
	{/if}
</main>
