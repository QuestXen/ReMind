<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import { Input } from '$lib/components/ui/input/index';
	import * as Select from '$lib/components/ui/select/index';
	import * as Popover from '$lib/components/ui/popover/index';
	import * as Dialog from '$lib/components/ui/dialog/index';
	import { Bell, Palette } from '@lucide/svelte';
	import ColorPicker from 'svelte-awesome-color-picker';
	import type { Reminder, ReminderInterval, ReminderColor } from '$lib/stores';

	interface Props {
		open: boolean;
		mode: 'create' | 'edit';
		initialData?: Reminder | null;
		onclose: () => void;
		onsave: (reminder: Partial<Reminder>) => void;
	}

	let { open = $bindable(), mode, initialData, onclose, onsave }: Props = $props();

	let formData = $state({
		name: '',
		interval: 'days' as ReminderInterval,
		intervalValue: 1,
		color: 'blue' as ReminderColor
	});

	let showColorPicker = $state(false);
	let customColor = $state('#7f5af0');

	const colorClasses = {
		blue: 'bg-blue-500 border-blue-200 text-blue-50',
		green: 'bg-green-500 border-green-200 text-green-50',
		purple: 'bg-purple-500 border-purple-200 text-purple-50',
		red: 'bg-red-500 border-red-200 text-red-50',
		orange: 'bg-orange-500 border-orange-200 text-orange-50',
		pink: 'bg-pink-500 border-pink-200 text-pink-50'
	};

	const intervalOptions = [
		{ value: 'minutes', label: 'Minuten' },
		{ value: 'hours', label: 'Stunden' },
		{ value: 'days', label: 'Tage' },
		{ value: 'weeks', label: 'Wochen' },
		{ value: 'months', label: 'Monate' }
	];

	const triggerContent = $derived(
		intervalOptions.find((option) => option.value === formData.interval)?.label ?? 'Intervall wählen'
	);

	// Initialize form data when dialog opens
	$effect(() => {
		if (open) {
			if (mode === 'edit' && initialData) {
				formData = {
					name: initialData.name,
					interval: initialData.interval,
					intervalValue: initialData.intervalValue,
					color: initialData.color
				};
			} else {
				// Reset form for create mode
				formData = {
					name: '',
					interval: 'days',
					intervalValue: 1,
					color: 'blue'
				};
			}
			showColorPicker = false;
			customColor = '#7f5af0';
		}
	});

	function handleSave() {
		if (!formData.name.trim()) return;
		
		const reminderToSave: Partial<Reminder> = {
			name: formData.name,
			interval: formData.interval,
			intervalValue: formData.intervalValue,
			color: formData.color
		};
		
		if (mode === 'edit' && initialData) {
			reminderToSave.id = initialData.id;
			reminderToSave.createdAt = initialData.createdAt;
			reminderToSave.active = initialData.active;
		}
		
		onsave(reminderToSave);
	}

	function handleClose() {
		onclose();
	}
</script>

<Dialog.Root bind:open onOpenChange={(isOpen) => { if (!isOpen) handleClose(); }}>
	<Dialog.Content class="max-w-2xl max-h-[90vh] overflow-y-auto" showCloseButton={false}>
		<div class="space-y-8 p-8">
			<h2 class="text-3xl text-heading text-card-foreground mb-6">
				{mode === 'edit' ? 'Erinnerung bearbeiten' : 'Neue Erinnerung erstellen'}
			</h2>
			
			<div class="transition-all duration-300">
				<label for="reminder-name" class="block text-sm text-subheading text-card-foreground mb-3">Erinnerungsname</label>
				<Input 
					id="reminder-name"
					bind:value={formData.name}
					placeholder="z.B. Medikamente nehmen, Meeting vorbereiten..."
					class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
				/>
			</div>

			<div class="grid grid-cols-1 md:grid-cols-2 gap-8">
				<div class="transition-all duration-300">
					<label for="reminder-interval" class="block text-sm text-subheading text-card-foreground mb-3">Intervall</label>
					<Select.Root type="single" name="reminderInterval" bind:value={formData.interval}>
						<Select.Trigger class="w-full h-12 px-4 bg-input border border-border rounded-xl text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 flex items-center">
							{triggerContent}
						</Select.Trigger>
						<Select.Content>
							<Select.Group>
								<Select.Label>Intervall wählen</Select.Label>
								{#each intervalOptions as option (option.value)}
									<Select.Item value={option.value} label={option.label}>
										{option.label}
									</Select.Item>
								{/each}
							</Select.Group>
						</Select.Content>
					</Select.Root>
				</div>

				<div class="transition-all duration-300">
					<label for="reminder-interval-value" class="block text-sm text-subheading text-card-foreground mb-3">Anzahl</label>
					<Input 
						id="reminder-interval-value"
						type="number"
						bind:value={formData.intervalValue}
						min="1"
						class="w-full h-10 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
					/>
				</div>
			</div>

			<fieldset class="transition-all duration-300">
				<legend class="block text-sm text-subheading text-card-foreground mb-4">Akzentfarbe wählen</legend>
				<div class="flex gap-4 items-center">
					{#each Object.entries(colorClasses) as [color, classes]}
						<button
							type="button"
							onclick={() => {
								formData.color = color as ReminderColor;
								showColorPicker = false;
							}}
							aria-label="Farbe {color} auswählen"
							class="w-10 h-10 rounded-full {classes} {formData.color === color ? 'ring-4 ring-primary scale-110 shadow-lg' : 'ring-2 ring-border'} transition-all duration-300 hover:scale-105 hover:shadow-md"
						></button>
					{/each}
					
					<Popover.Root bind:open={showColorPicker}>
						<Popover.Trigger>
							<button
								type="button"
								aria-label="Benutzerdefinierte Farbe wählen"
								class="w-10 h-10 rounded-full transition-all duration-300 hover:scale-105 hover:shadow-md overflow-hidden flex items-center justify-center {formData.color?.startsWith('#') ? 'ring-4 ring-primary scale-110 shadow-lg' : 'ring-2 ring-border'}"
								style="{formData.color?.startsWith('#') ? `background: ${formData.color};` : 'background: linear-gradient(45deg, #7f5af0, #50fa7b, #e8a87c, #ff6b6b);'}"
							>
								{#if !formData.color?.startsWith('#')}
									<Palette class="w-4 h-4 text-white drop-shadow-sm" />
								{/if}
							</button>
						</Popover.Trigger>
						<Popover.Content class="p-3 bg-card border border-border rounded-xl shadow-xl">
							<div class="flex flex-col items-center space-y-3">
								<h4 class="text-sm font-medium text-card-foreground">Benutzerdefinierte Farbe</h4>
								<ColorPicker 
									bind:hex={customColor}
									position="responsive"
									isDialog={false}
									--picker-height="120px"
									--picker-width="180px"
									--slider-width="18px"
									--cp-bg-color="hsl(var(--card))"
									--cp-border-color="hsl(var(--border))"
									--cp-text-color="hsl(var(--card-foreground))"
									--cp-input-color="hsl(var(--input))"
									--cp-button-hover-color="hsl(var(--accent))"
								/>
								<Button 
									onclick={() => {
										formData.color = customColor;
										showColorPicker = false;
									}}
									class="px-3 py-1.5 bg-primary hover:bg-primary/90 text-primary-foreground text-xs rounded-lg"
								>
									Übernehmen
								</Button>
							</div>
						</Popover.Content>
					</Popover.Root>
				</div>
			</fieldset>

			<div class="flex gap-6 pt-8">
				<Button 
					onclick={handleSave}
					disabled={!formData.name.trim()}
					class="flex-1 bg-primary hover:bg-primary/90 text-primary-foreground py-4 rounded-xl text-subheading transition-all duration-300 hover:scale-105 hover:shadow-lg border-0"
				>
					<Bell class="w-5 h-5 mr-3" />
					{mode === 'edit' ? 'Erinnerung aktualisieren' : 'Erinnerung erstellen'}
				</Button>
				<Button 
					onclick={handleClose}
					variant="outline"
					class="flex-1 border-2 border-border text-muted-foreground hover:bg-destructive/10 hover:text-destructive py-4 rounded-xl text-subheading transition-all duration-300 hover:scale-105 hover:border-destructive"
				>
					Abbrechen
				</Button>
			</div>
		</div>
	</Dialog.Content>
</Dialog.Root>