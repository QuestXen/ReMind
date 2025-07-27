<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import { ArrowLeft } from '@lucide/svelte';
	import { createEventDispatcher } from 'svelte';
	import TitleBar from './TitleBar.svelte';

	const dispatch = createEventDispatcher<{ close: void }>();

	function handleClose() {
		dispatch('close');
	}
</script>

<div class="flex h-screen flex-col overflow-hidden">
	<TitleBar title="Einstellungen" icon="S" />

	<!-- Settings Content -->
	<div class="bg-background animate-slide-in-right flex-1 overflow-y-auto">
		<!-- Zurück Button -->
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

				<!-- Placeholder für zukünftige Einstellungen -->
				<div class="bg-card border-border rounded-3xl border p-8 text-center">
					<p class="text-muted-foreground text-body">Einstellungen werden hier angezeigt</p>
				</div>
			</div>
		</div>
	</div>
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
