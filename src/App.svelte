<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
        import ReminderApp from '$lib/components/ReminderApp.svelte';
        import Loading from '$lib/components/Loading.svelte';
        import { reminders, settings, isLoading, loadingError } from '$lib/stores';
        import { sanitizeReminderList, sanitizeAppSettings } from '$lib/utils/validation';
        import { setLocale } from './paraglide/runtime';
        import m from './paraglide/messages';

	let retryCount = 0;
	const MAX_RETRIES = 3;
	let updateStatus = $state('checking'); // 'checking', 'updating', 'done'
	let updateMessage = $state(m.update_checking());
	let trayUpdateTriggered = $state(false);
	let isAutostart = $state(false);
	let skipUpdateCheck = $state(false);
	let isLanguageChanging = $state(false);

	function detectAutostart(): boolean {
		// If keepSettingsOpen is set, it's a language change reload
		if (localStorage.getItem('keepSettingsOpen') === 'true') {
			return false;
		}
		// Check if app was opened without user interaction (autostart)
		return document.referrer === '' && !window.opener && !document.hasFocus();
	}

	interface UpdateInfo {
		available: boolean;
		version?: string;
		notes?: string;
		pubDate?: string;
	}

        interface AppStatePayload {
                reminders: unknown;
                settings: unknown;
        }

	async function checkForUpdates(): Promise<UpdateInfo | null> {
		try {
			updateMessage = m.update_checking();
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
			updateMessage = m.update_installing();

			await invoke('install_update');

			updateMessage = m.update_finished();
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

			// Add delay only if autostart
			if (isAutostart) {
				await new Promise((resolve) => setTimeout(resolve, 500));
			}

                        const { reminders: rawReminders, settings: rawSettings } =
                                await invoke<AppStatePayload>('load_app_state');

                        const sanitizedReminders = sanitizeReminderList(rawReminders);
                        const appSettings = sanitizeAppSettings(rawSettings);

                        // Update stores
                        settings.set(appSettings);
                        reminders.set(sanitizedReminders);
                        setLocale(appSettings.language as 'en' | 'de');

			isLoading.set(false);
			retryCount = 0;
			updateStatus = 'done';

			console.log('App data loaded successfully:', {
				settings: appSettings,
				remindersCount: sanitizedReminders.length
			});
		} catch (error) {
			console.error('Failed to load app data:', error);

			// Set error message for retry
			if (retryCount < MAX_RETRIES) {
				loadingError.set(m.connection_error_retry());
			} else {
				loadingError.set(m.connection_error_final());
			}

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
			loadingError.set(m.max_retries_reached());
		}
	}

	onMount(() => {
		// Detect if app was started via autostart
		isAutostart = detectAutostart();

		// Check if this is a reload after language change
		if (localStorage.getItem('keepSettingsOpen') === 'true') {
			skipUpdateCheck = true;
			isLanguageChanging = true;
			localStorage.removeItem('keepSettingsOpen');
			// Hide language changing state after UI is ready
			setTimeout(() => {
				isLanguageChanging = false;
			}, 300);
		}

		let unlistenUpdateStart: (() => void) | undefined;
		let unlistenUpdateInstalling: (() => void) | undefined;
		let unlistenUpdateNotAvailable: (() => void) | undefined;
		let unlistenUpdateError: (() => void) | undefined;

		async function initializeApp() {
			unlistenUpdateStart = await listen('update-check-started', () => {
				trayUpdateTriggered = true;
				updateStatus = 'checking';
				updateMessage = m.update_checking();
				isLoading.set(true);
			});

			unlistenUpdateInstalling = await listen('update-installing', (event) => {
				updateStatus = 'updating';
				const updateInfo = event.payload as UpdateInfo;
				updateMessage = updateInfo.version
					? m.update_installing_version({ version: updateInfo.version })
					: m.update_installing();
			});

			unlistenUpdateNotAvailable = await listen('update-not-available', () => {
				updateStatus = 'done';
				updateMessage = m.update_not_available();
				setTimeout(() => {
					isLoading.set(false);
					trayUpdateTriggered = false;
				}, 2000);
			});

			unlistenUpdateError = await listen('update-error', (event) => {
				updateStatus = 'done';
				loadingError.set(m.update_error({ error: event.payload as string }));
				isLoading.set(false);
				trayUpdateTriggered = false;
			});

			if (!trayUpdateTriggered) {
				// Skip update check if requested (e.g., language change)
				if (skipUpdateCheck) {
					updateStatus = 'done';
					loadAppData();
					return;
				}

				// Add delay only if autostart
				if (isAutostart) {
					await new Promise((resolve) => setTimeout(resolve, 1500));
				}

				const updateInfo = await checkForUpdates();

				if (updateInfo?.available) {
					console.log(`Update available: ${updateInfo.version}`);
					updateMessage = updateInfo.version
						? m.update_available({ version: updateInfo.version })
						: m.update_checking();

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

		initializeApp().catch(console.error);

		return () => {
			unlistenUpdateStart?.();
			unlistenUpdateInstalling?.();
			unlistenUpdateNotAvailable?.();
			unlistenUpdateError?.();
		};
	});
</script>

<main class="bg-background min-h-screen">
	{#if isLanguageChanging}
		<!-- Show only background during language change -->
	{:else if $isLoading || updateStatus !== 'done' || trayUpdateTriggered}
		<Loading
			message={trayUpdateTriggered
				? updateMessage
				: updateStatus === 'done'
					? m.loading_reminders()
					: updateMessage}
			error={$loadingError}
			retryCallback={$loadingError ? retryLoading : undefined}
			isUpdating={updateStatus === 'updating'}
		/>
	{:else}
		<ReminderApp />
	{/if}
</main>
