<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';

	interface Props {
		title: string;
		icon: string;
		showWindowControls?: boolean;
	}

	let { title, icon, showWindowControls = true }: Props = $props();

	// Window control functions
	let appWindow: any = null;
	try {
		appWindow = getCurrentWindow();
	} catch (error) {
		// Ignore error when running in web mode
	}

	const minimize = () => appWindow?.minimize();
	const toggleMaximize = () => appWindow?.toggleMaximize();
	const close = () => appWindow?.close();
</script>

<!-- Custom Title Bar -->
<div
	class="bg-background border-border flex flex-shrink-0 items-center justify-between border-b p-4 select-none"
	data-tauri-drag-region
	ondblclick={toggleMaximize}
	role="button"
	tabindex="0"
>
	<!-- Linke Seite: Icon + Titel -->
	<div class="flex items-center gap-3">
		<div class="bg-primary flex h-6 w-6 items-center justify-center rounded-full">
			<span class="text-primary-foreground text-xs font-bold">{icon}</span>
		</div>
		<span class="text-foreground font-medium">{title}</span>
	</div>

	<!-- Rechte Seite: Window Controls -->
	{#if showWindowControls}
		<div class="flex items-center gap-2" data-tauri-drag-region="false">
			<button
				class="hover:bg-muted text-muted-foreground hover:text-foreground flex h-8 w-8 items-center justify-center rounded transition-colors"
				onclick={minimize}
				aria-label="Minimize"
			>
				<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
					<path d="M4 10h12v2H4z"></path>
				</svg>
			</button>
			<button
				class="hover:bg-muted text-muted-foreground hover:text-foreground flex h-8 w-8 items-center justify-center rounded transition-colors"
				onclick={toggleMaximize}
				aria-label="Toggle Maximize"
			>
				<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
					<path d="M4 4h12v12H4V4zm2 2v8h8V6H6z"></path>
				</svg>
			</button>
			<button
				class="hover:bg-destructive hover:text-destructive-foreground text-muted-foreground flex h-8 w-8 items-center justify-center rounded transition-colors"
				onclick={close}
				aria-label="Close"
			>
				<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
					<path
						d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 
						   111.414 1.414L11.414 10l4.293 4.293a1 1 0 
						   01-1.414 1.414L10 11.414l-4.293 
						   4.293a1 1 0 01-1.414-1.414L8.586 10 
						   4.293 5.707a1 1 0 010-1.414z"
					></path>
				</svg>
			</button>
		</div>
	{/if}
</div>

<style>
	/* Ziehbarer Bereich */
	[data-tauri-drag-region] {
		-webkit-app-region: drag;
	}
	/* Klickbare Buttons ausnehmen */
	[data-tauri-drag-region='false'],
	[data-tauri-drag-region='false'] * {
		-webkit-app-region: no-drag;
	}
</style>
