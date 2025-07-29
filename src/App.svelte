<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import ReminderApp from '$lib/components/ReminderApp.svelte';
	import Loading from '$lib/components/Loading.svelte';
	import { reminders, settings, isLoading, loadingError } from '$lib/stores';
	import type { Reminder, AppSettings } from '$lib/stores';

	let retryCount = 0;
	const MAX_RETRIES = 3;
	let updateStatus = $state('checking'); // 'checking', 'updating', 'done'
	let updateMessage = $state('Update Checken...');
	let trayUpdateTriggered = $state(false);

	interface UpdateInfo {
		available: boolean;
		version?: string;
		notes?: string;
		pubDate?: string;
	}

	interface LoadedSettings {
		autostart_enabled?: boolean;
		theme?: string | null;
		notification_sound?: boolean;
		[key: string]: unknown;
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
				invoke<LoadedSettings>('load_settings'),
				invoke<Reminder[]>('load_reminders')
			]);

			const appSettings: AppSettings = {
				autostartEnabled: Boolean(loadedSettings.autostart_enabled),
				theme: typeof loadedSettings.theme === 'string' ? loadedSettings.theme : null,
				notificationSound: loadedSettings.notification_sound !== false,
				...loadedSettings
			};

			// Update stores
			settings.set(appSettings);
			reminders.set(loadedReminders);

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
			loadingError.set(
				'Maximale Anzahl von Wiederholungsversuchen erreicht. Bitte starten Sie die Anwendung neu.'
			);
		}
	}

	onMount(() => {
		// Store cleanup functions
		let unlistenUpdateStart: (() => void) | undefined;
		let unlistenUpdateInstalling: (() => void) | undefined;
		let unlistenUpdateNotAvailable: (() => void) | undefined;
		let unlistenUpdateError: (() => void) | undefined;

		// Async initialization function
		async function initializeApp() {
			// Listen for tray update events
			unlistenUpdateStart = await listen('update-check-started', () => {
				trayUpdateTriggered = true;
				updateStatus = 'checking';
				updateMessage = 'Update wird geprüft...';
				isLoading.set(true);
			});

			unlistenUpdateInstalling = await listen('update-installing', (event) => {
				updateStatus = 'updating';
				const updateInfo = event.payload as UpdateInfo;
				updateMessage = `Update ${updateInfo.version} wird installiert...`;
			});

			unlistenUpdateNotAvailable = await listen('update-not-available', () => {
				updateStatus = 'done';
				updateMessage = 'Keine Updates verfügbar';
				setTimeout(() => {
					isLoading.set(false);
					trayUpdateTriggered = false;
				}, 2000);
			});

			unlistenUpdateError = await listen('update-error', (event) => {
				updateStatus = 'done';
				loadingError.set(`Update-Fehler: ${event.payload}`);
				isLoading.set(false);
				trayUpdateTriggered = false;
			});

			// Initial app startup logic
			if (!trayUpdateTriggered) {
				const updateInfo = await checkForUpdates();

				if (updateInfo?.available) {
					console.log(`Update available: ${updateInfo.version}`);
					updateMessage = `Update ${updateInfo.version} verfügbar. Installation wird gestartet...`;

					await new Promise((resolve) => setTimeout(resolve, 1000));

					const updateSuccess = await installUpdate();

					if (!updateSuccess) {
						updateStatus = 'done';
						loadAppData();
					}
				} else {
					updateStatus = 'done';
					loadAppData();
				}
			}
		}

		// Start async initialization
		initializeApp().catch(console.error);

		// Return cleanup function
		return () => {
			unlistenUpdateStart?.();
			unlistenUpdateInstalling?.();
			unlistenUpdateNotAvailable?.();
			unlistenUpdateError?.();
		};
	});
</script>

<main class="bg-background min-h-screen">
	{#if $isLoading || updateStatus !== 'done' || trayUpdateTriggered}
		<Loading
			message={trayUpdateTriggered ? updateMessage : (updateStatus === 'done' ? 'Lade Erinnerungen und Einstellungen' : updateMessage)}
			error={$loadingError}
			retryCallback={$loadingError ? retryLoading : undefined}
			isUpdating={updateStatus === 'updating'}
		/>
	{:else}
		<ReminderApp />
	{/if}
</main>
