<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';

	interface Props {
		message?: string;
		retryCallback?: (() => void) | (() => Promise<void>);
		error?: string | null;
	}

	let { message = 'Lade Daten...', retryCallback, error = null }: Props = $props();

	let progress = $state(0);
	let progressInterval: NodeJS.Timeout;
	let currentMessage = $state('Update Checken...');
	let isUpdating = $state(false);

	interface UpdateInfo {
		available: boolean;
		version?: string;
		notes?: string;
		pubDate?: string;
	}

	async function checkForUpdates(): Promise<UpdateInfo | null> {
		try {
			currentMessage = 'Update Checken...';
			const updateInfo = await invoke<UpdateInfo>('check_for_updates');
			return updateInfo;
		} catch (error) {
			console.error('Failed to check for updates:', error);
			return null;
		}
	}

	async function installUpdate(): Promise<boolean> {
		try {
			isUpdating = true;
			currentMessage = 'Update wird installiert...';
			progress = 0;
			
			// Simulate progress during update
			const updateProgressInterval = setInterval(() => {
				if (progress < 90) {
					progress += Math.random() * 10 + 5;
				}
			}, 500);

			await invoke('install_update');
			
			clearInterval(updateProgressInterval);
			progress = 100;
			currentMessage = 'Update abgeschlossen. Anwendung wird neu gestartet...';
			
			return true;
		} catch (error) {
			console.error('Failed to install update:', error);
			isUpdating = false;
			return false;
		}
	}

	onMount(() => {
		// Async logic in separate function
		async function handleMount() {
			if (!error) {
				// First check for updates
				const updateInfo = await checkForUpdates();
				
				if (updateInfo?.available) {
					console.log(`Update available: ${updateInfo.version}`);
					currentMessage = `Update ${updateInfo.version} verfügbar. Installation wird gestartet...`;
					
					// Wait a moment to show the message
					await new Promise(resolve => setTimeout(resolve, 1000));
					
					// Install update automatically
					const updateSuccess = await installUpdate();
					
					if (!updateSuccess) {
						// If update failed, continue with normal loading
						currentMessage = 'Lade Daten...';
						startNormalLoading();
					}
					// If update succeeded, the app will restart automatically
				} else {
					// No update available, continue with normal loading
					currentMessage = 'Lade Daten...';
					startNormalLoading();
				}
			}
		}
		
		// Execute async logic
		handleMount();
		
		// Return cleanup function synchronously
		return () => {
			if (progressInterval) clearInterval(progressInterval);
		};
	});

	function startNormalLoading() {
		if (!isUpdating) {
			progressInterval = setInterval(() => {
				if (progress < 100) {
					progress += Math.random() * 15 + 5;
					if (progress > 100) progress = 100;
				}
			}, 200);
		}
	}
</script>

<div class="flex h-screen w-full items-center justify-center bg-background">
	<div class="w-full max-w-md mx-auto px-6">
		{#if error}
			<!-- Error State -->
			<div class="text-center space-y-6">
				<div class="text-destructive">
					<svg class="w-16 h-16 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
					</svg>
				</div>
				<h2 class="text-heading text-foreground text-xl mb-2">Fehler beim Laden</h2>
				<p class="text-muted-foreground text-body text-sm">{error}</p>
				{#if retryCallback}
					<button 
						onclick={retryCallback}
						class="bg-primary hover:bg-primary/90 text-primary-foreground px-6 py-2 rounded-lg transition-colors font-medium"
					>
						Erneut versuchen
					</button>
				{/if}
			</div>
		{:else}
			<!-- Loading/Update State -->
			<div class="text-center space-y-6">
				{#if isUpdating}
					<!-- Update Icon -->
					<div class="text-primary">
						<svg class="w-16 h-16 mx-auto mb-4 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
						</svg>
					</div>
				{:else}
					<!-- Loading Icon -->
					<div class="text-primary">
						<svg class="w-16 h-16 mx-auto mb-4 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707"></path>
						</svg>
					</div>
				{/if}
				
				<!-- Progress Bar -->
				<div class="w-full bg-muted rounded-full h-3 overflow-hidden">
					<div 
						class="h-full bg-gradient-to-r from-primary to-primary/80 rounded-full transition-all duration-300 ease-out"
						style="width: {progress}%"
					></div>
				</div>
				
				<!-- Loading Text -->
				<p class="text-muted-foreground text-lg">{currentMessage}</p>
				
				{#if isUpdating}
					<p class="text-muted-foreground text-sm">Bitte schließen Sie die Anwendung nicht während des Updates.</p>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	@keyframes fade-in {
		from {
			opacity: 0;
			transform: translateY(20px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@keyframes pulse {
		0%, 100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	.space-y-8 {
		animation: fade-in 0.8s ease-out;
	}

	.space-y-6 {
		animation: fade-in 0.6s ease-out;
	}

	/* Smooth progress bar animation */
	.progress-bar {
		transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}

	/* Step indicator animation */
	.step-indicator {
		transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
	}
</style>