<script lang="ts">
	import { onMount } from 'svelte';
        import m from '../../paraglide/messages';

	interface Props {
		message?: string;
		retryCallback?: (() => void) | (() => Promise<void>);
		error?: string | null;
		isUpdating?: boolean;
	}

	let {
		message = m.loading_data(),
		retryCallback,
		error = null,
		isUpdating = false
	}: Props = $props();

	let progress = $state(0);
	let progressInterval: ReturnType<typeof setInterval> | null = null;

	onMount(() => {
		if (!error) {
			progressInterval = setInterval(() => {
				if (progress < 100) {
					progress += Math.random() * 15 + 10;
					if (progress > 100) progress = 100;
				}
			}, 100);
		}

		return () => {
			if (progressInterval !== null) {
				clearInterval(progressInterval);
				progressInterval = null;
			}
		};
	});
</script>

<div class="bg-background flex h-screen w-full items-center justify-center">
	<div class="mx-auto w-full max-w-md px-6">
		{#if error}
			<div class="space-y-6 text-center">
				<div class="text-destructive">
					<svg class="mx-auto mb-4 h-16 w-16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"
						></path>
					</svg>
				</div>
				<h2 class="text-heading text-foreground mb-2 text-xl">{m.loading_error()}</h2>
				<p class="text-muted-foreground text-body text-sm">{error}</p>
				{#if retryCallback}
					<button
						onclick={retryCallback}
						class="bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg px-6 py-2 font-medium transition-colors"
					>
						{m.retry()}
					</button>
				{/if}
			</div>
		{:else}
			<div class="space-y-6 text-center">
				{#if isUpdating}
					<div class="text-primary">
						<svg
							class="mx-auto mb-4 h-16 w-16 animate-spin"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
							></path>
						</svg>
					</div>
				{:else}
					<div class="text-primary">
						<svg
							class="mx-auto mb-4 h-16 w-16 animate-spin"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707"
							></path>
						</svg>
					</div>
				{/if}
				<div class="bg-muted h-3 w-full overflow-hidden rounded-full">
					<div
						class="from-primary to-primary/80 h-full rounded-full bg-gradient-to-r transition-all duration-300 ease-out"
						style="width: {progress}%"
					></div>
				</div>
				<p class="text-muted-foreground text-lg">{message}</p>

				{#if isUpdating}
					<p class="text-muted-foreground text-sm">{m.warning_update()}</p>
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
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	.space-y-6 {
		animation: fade-in 0.6s ease-out;
	}
</style>
