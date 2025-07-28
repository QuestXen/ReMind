<script lang="ts">
	import { onMount } from 'svelte';

	interface Props {
		message?: string;
		retryCallback?: (() => void) | (() => Promise<void>);
		error?: string | null;
	}

	let { message = 'Lade Daten...', retryCallback, error = null }: Props = $props();

	let progress = $state(0);
	let progressInterval: NodeJS.Timeout;

	onMount(() => {
		if (!error) {
			progressInterval = setInterval(() => {
				if (progress < 100) {
					progress += Math.random() * 15 + 5; // Random increment between 5-20
					if (progress > 100) progress = 100;
				}
			}, 200);
		}

		return () => {
			if (progressInterval) clearInterval(progressInterval);
		};
	});
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
			<!-- Simple Loading State -->
			<div class="text-center space-y-6">
				<!-- Progress Bar -->
				<div class="w-full bg-muted rounded-full h-3 overflow-hidden">
					<div 
						class="h-full bg-gradient-to-r from-primary to-primary/80 rounded-full transition-all duration-300 ease-out"
						style="width: {progress}%"
					></div>
				</div>
				
				<!-- Loading Text -->
				<p class="text-muted-foreground text-lg">Daten werden geladen...</p>
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