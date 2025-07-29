<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import { Input } from '$lib/components/ui/input/index';
	import * as Select from '$lib/components/ui/select/index';
	import * as Popover from '$lib/components/ui/popover/index';
	import * as Dialog from '$lib/components/ui/dialog/index';
	import { Bell, Palette, Clock, Repeat, Calendar as CalendarIcon } from '@lucide/svelte';
import ColorPicker from 'svelte-awesome-color-picker';
import type { Reminder, ReminderInterval, ReminderColor, ReminderMode, AppointmentRecurrence } from '$lib/stores';
    import { DateFormatter, type DateValue, getLocalTimeZone, parseDate } from "@internationalized/date";
    import { cn } from "$lib/utils.js";
    import { buttonVariants } from "$lib/components/ui/button/index.js";
    import { Calendar as DateCalendar } from "$lib/components/ui/calendar/index";

	interface Props {
		open: boolean;
		mode: 'create' | 'edit';
		initialData?: Reminder | null;
		onclose: () => void;
		onsave: (reminder: Partial<Reminder>) => void;
	}

	let { open = $bindable(), mode, initialData, onclose, onsave }: Props = $props();

	let editingReminder = $derived(mode === 'edit' ? initialData : null);

	let formData = $state({
		name: '',
		reminderMode: 'interval' as ReminderMode,
		// Interval mode
		interval: 'days' as ReminderInterval,
		intervalValue: 1,
		// Scheduled mode
		specificDate: undefined as DateValue | undefined,
		specificTime: '09:00',
		beginAt: undefined as DateValue | undefined,
		beginAtTime: '09:00',
		// Appointment mode
		appointmentDate: undefined as DateValue | undefined,
		appointmentTime: '09:00',
		recurrence: 'once' as AppointmentRecurrence,
		weekdays: [] as number[],
		recurrenceEndDate: undefined as DateValue | undefined,
		// Common
		color: 'blue' as ReminderColor
	});

	let showColorPicker = $state(false);
	let customColor = $state('#7f5af0');

	let beginAtDateOpen = $state(false);
	let appointmentDateOpen = $state(false);
	let recurrenceEndDateOpen = $state(false);

	const colorClasses = {
		blue: 'bg-blue-500 border-blue-200 text-blue-50',
		green: 'bg-green-500 border-green-200 text-green-50',
		purple: 'bg-purple-500 border-purple-200 text-purple-50',
		red: 'bg-red-500 border-red-200 text-red-50',
		orange: 'bg-orange-500 border-orange-200 text-orange-50',
		pink: 'bg-pink-500 border-pink-200 text-pink-50'
	};

	const modeOptions = [
		{ value: 'interval', label: 'Intervall-Reminder', icon: Repeat, description: 'Regelmäßige Erinnerungen in festgelegten Abständen' },
		{ value: 'scheduled', label: 'Geplanter Reminder', icon: Clock, description: 'Erinnerung zu einem bestimmten Datum und Uhrzeit' },
		{ value: 'appointment', label: 'Termin-Reminder', icon: CalendarIcon, description: 'Einmalige oder wiederkehrende Termine' }
	];

	const intervalOptions = [
		{ value: 'minutes', label: 'Minuten' },
		{ value: 'hours', label: 'Stunden' },
		{ value: 'days', label: 'Tage' },
		{ value: 'weeks', label: 'Wochen' },
		{ value: 'months', label: 'Monate' }
	];

	const recurrenceOptions = [
		{ value: 'once', label: 'Einmalig' },
		{ value: 'daily', label: 'Täglich' },
		{ value: 'weekly', label: 'Wöchentlich' },
		{ value: 'monthly', label: 'Monatlich' },
		{ value: 'custom', label: 'Benutzerdefiniert' }
	];

	const weekdayOptions = [
		{ value: 0, label: 'Sonntag' },
		{ value: 1, label: 'Montag' },
		{ value: 2, label: 'Dienstag' },
		{ value: 3, label: 'Mittwoch' },
		{ value: 4, label: 'Donnerstag' },
		{ value: 5, label: 'Freitag' },
		{ value: 6, label: 'Samstag' }
	];

	const intervalTriggerContent = $derived(
		intervalOptions.find((option) => option.value === formData.interval)?.label ?? 'Intervall wählen'
	);

	const recurrenceTriggerContent = $derived(
		recurrenceOptions.find((option) => option.value === formData.recurrence)?.label ?? 'Wiederholung wählen'
	);

	const selectedMode = $derived(
		modeOptions.find((option) => option.value === formData.reminderMode)
	);

	// Initialize form data when dialog opens
	$effect(() => {
		if (open) {
			if (editingReminder) {
				// Edit mode - populate with existing data
				formData.name = editingReminder.title || editingReminder.name || '';
				formData.reminderMode = editingReminder.mode || 'interval';
				formData.interval = editingReminder.interval || 'minutes';
				formData.intervalValue = editingReminder.intervalValue || 1;
				formData.specificDate = editingReminder.specificDate ? parseDate(editingReminder.specificDate.split('T')[0]) : undefined;
				formData.specificTime = editingReminder.specificTime || '09:00';
				formData.beginAt = editingReminder.beginAt ? parseDate(editingReminder.beginAt.split('T')[0]) : undefined;
				formData.beginAtTime = editingReminder.beginAtTime || '09:00';
				formData.appointmentDate = editingReminder.appointmentDate ? parseDate(editingReminder.appointmentDate.split('T')[0]) : undefined;
				formData.appointmentTime = editingReminder.appointmentTime || '09:00';
				formData.recurrence = editingReminder.recurrence || 'once';
				formData.weekdays = editingReminder.weekdays || [];
				formData.recurrenceEndDate = editingReminder.recurrenceEndDate ? parseDate(editingReminder.recurrenceEndDate.split('T')[0]) : undefined;
				formData.color = editingReminder.color;
			} else {
				// Create mode - reset to defaults
				formData.name = '';
				formData.reminderMode = 'interval';
				formData.interval = 'minutes';
				formData.intervalValue = 1;
				formData.specificDate = undefined;
				formData.specificTime = '09:00';
				formData.beginAt = undefined;
				formData.beginAtTime = '09:00';
				formData.appointmentDate = undefined;
				formData.appointmentTime = '09:00';
				formData.recurrence = 'once';
				formData.weekdays = [];
				formData.recurrenceEndDate = undefined;
				formData.color = 'blue';
			}
			beginAtDateOpen = false;
			appointmentDateOpen = false;
			recurrenceEndDateOpen = false;
		}
	});

	$effect(() => {
		if (formData.beginAt && beginAtDateOpen) beginAtDateOpen = false;
	});

	$effect(() => {
		if (formData.appointmentDate && appointmentDateOpen) appointmentDateOpen = false;
	});

	$effect(() => {
		if (formData.recurrenceEndDate && recurrenceEndDateOpen) recurrenceEndDateOpen = false;
	});

	function handleSave() {
		if (!formData.name.trim()) return;
		
		const reminderToSave: Partial<Reminder> = {
			title: formData.name,
			message: '', // Default empty message
			color: formData.color,
			mode: formData.reminderMode,
		};
		
		// Add mode-specific properties
		if (formData.reminderMode === 'interval') {
			reminderToSave.interval = formData.interval;
			reminderToSave.intervalValue = formData.intervalValue;
		} else if (formData.reminderMode === 'scheduled') {
			reminderToSave.interval = formData.interval;
			reminderToSave.intervalValue = formData.intervalValue;
			if (formData.beginAt && formData.beginAtTime) {
				const beginDateTime = formData.beginAt.toDate(getLocalTimeZone());
				const [hours, minutes] = formData.beginAtTime.split(':');
				beginDateTime.setHours(parseInt(hours), parseInt(minutes));
				reminderToSave.beginAt = beginDateTime.toISOString();
				reminderToSave.beginAtTime = formData.beginAtTime;
			}
		} else if (formData.reminderMode === 'appointment') {
			if (formData.appointmentDate) {
				const dateTime = formData.appointmentDate.toDate(getLocalTimeZone());
				const [hours, minutes] = formData.appointmentTime.split(':');
				dateTime.setHours(parseInt(hours), parseInt(minutes));
				reminderToSave.appointmentDate = dateTime.toISOString();
				reminderToSave.appointmentTime = formData.appointmentTime;
			}
			reminderToSave.recurrence = formData.recurrence;
			if (formData.recurrence === 'custom') {
				reminderToSave.weekdays = formData.weekdays;
			}
			if (formData.recurrenceEndDate) {
				reminderToSave.recurrenceEndDate = formData.recurrenceEndDate.toDate(getLocalTimeZone()).toISOString();
			}
		}
		
		if (mode === 'edit' && initialData) {
			reminderToSave.id = initialData.id;
			reminderToSave.createdAt = initialData.createdAt;
			reminderToSave.active = initialData.active;
		}
		
		onsave(reminderToSave);
	}

	function toggleWeekday(day: number) {
		if (formData.weekdays.includes(day)) {
			formData.weekdays = formData.weekdays.filter(d => d !== day);
		} else {
			formData.weekdays = [...formData.weekdays, day];
		}
	}

	function handleClose() {
		onclose();
	}
</script>

<Dialog.Root bind:open onOpenChange={(isOpen) => { if (!isOpen) handleClose(); }}>
	<Dialog.Content class="{formData.reminderMode === 'interval' ? 'max-w-2xl' : formData.reminderMode === 'scheduled' ? 'max-w-3xl' : 'max-w-4xl'} max-h-[95vh] overflow-y-auto" showCloseButton={false}>
		<div class="space-y-6 p-6">
			<div class="text-center mb-8">
				<h2 class="text-3xl text-heading text-card-foreground mb-2">
					{mode === 'edit' ? 'Erinnerung bearbeiten' : 'Neue Erinnerung erstellen'}
				</h2>
				{#if selectedMode}
					<p class="text-sm text-muted-foreground">{selectedMode.description}</p>
				{/if}
			</div>
			
			<!-- Erinnerungsname -->
			<div class="transition-all duration-300">
				<label for="reminder-name" class="block text-sm text-subheading text-card-foreground mb-3">Erinnerungsname</label>
				<Input 
					id="reminder-name"
					bind:value={formData.name}
					placeholder="z.B. Medikamente nehmen, Meeting vorbereiten..."
					class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
				/>
			</div>

			<!-- Modus-Auswahl -->
			<div class="transition-all duration-300">
				<label class="block text-sm text-subheading text-card-foreground mb-4">Reminder-Typ</label>
				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					{#each modeOptions as option}
						<button
							type="button"
							onclick={() => formData.reminderMode = option.value as ReminderMode}
							class="p-4 border-2 rounded-xl transition-all duration-300 hover:scale-105 {formData.reminderMode === option.value ? 'border-primary bg-primary/10 shadow-lg' : 'border-border hover:border-primary/50'}"
						>
							<div class="flex flex-col items-center space-y-2">
								<svelte:component this={option.icon} class="w-6 h-6 {formData.reminderMode === option.value ? 'text-primary' : 'text-muted-foreground'}" />
								<span class="text-sm font-medium {formData.reminderMode === option.value ? 'text-primary' : 'text-foreground'}">{option.label}</span>
							</div>
						</button>
					{/each}
				</div>
			</div>

			<!-- Dynamische Felder basierend auf Modus -->
			<div class="mode-fields-container transition-all duration-500 ease-in-out" style="{formData.reminderMode === 'interval' ? 'min-height: 150px;' : formData.reminderMode === 'scheduled' ? 'min-height: 300px;' : 'min-height: 400px;'}">
				{#if formData.reminderMode === 'interval'}
					<div class="space-y-6 animate-fade-in {formData.recurrence === 'custom' ? 'space-y-6' : 'space-y-2'}">
						<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
							<div class="transition-all duration-300">
								<label for="reminder-interval" class="block text-sm text-subheading text-card-foreground mb-3">Intervall</label>
								<Select.Root type="single" name="reminderInterval" bind:value={formData.interval}>
									<Select.Trigger class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground flex items-center">
										{intervalTriggerContent}
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
									class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
								/>
							</div>
						</div>

						
					</div>

				{:else if formData.reminderMode === 'scheduled'}
					<div class="space-y-{formData.recurrence === 'custom' ? 6 : 2} animate-fade-in">
						<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
							<div class="transition-all duration-300">
								<label for="reminder-interval" class="block text-sm text-subheading text-card-foreground mb-3">Intervall</label>
								<Select.Root type="single" name="reminderInterval" bind:value={formData.interval}>
									<Select.Trigger class="w-full h-12 px-4 bg-input border border-border rounded-xl text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 flex items-center">
										{intervalTriggerContent}
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
									class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
								/>
							</div>
						</div>

						<hr class="border-border my-6" />
						<h3 class="text-lg font-medium text-card-foreground mb-4">Beginn des Reminders</h3>
						
						<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
							<div class="transition-all duration-300">
								<label class="block text-sm text-subheading text-card-foreground mb-3">Start-Datum</label>
								<Popover.Root bind:open={beginAtDateOpen}>
									<Popover.Trigger class={cn(buttonVariants({ variant: "outline", class: "w-full h-12 justify-start text-left font-normal rounded-xl px-4 bg-input border-border text-foreground focus:border-ring focus:ring-ring transition-all duration-300 hover:border-ring/50" }), !formData.beginAt && "text-muted-foreground")}>
								{formData.beginAt ? new DateFormatter("de-DE", { dateStyle: "long" }).format(formData.beginAt.toDate(getLocalTimeZone())) : "Datum wählen"}
							</Popover.Trigger>
									<Popover.Content class="w-auto p-0 bg-card border-border rounded-xl shadow-xl">
										<DateCalendar type="single" initialFocus bind:value={formData.beginAt} />
									</Popover.Content>
								</Popover.Root>
							</div>
							<div class="transition-all duration-300">
								<label class="block text-sm text-subheading text-card-foreground mb-3">Start-Uhrzeit</label>
								<Popover.Root>
									<Popover.Trigger class={cn(buttonVariants({ variant: "outline", class: "w-full h-12 justify-start text-left font-normal rounded-xl px-4 bg-input border-border text-foreground focus:border-ring focus:ring-ring transition-all duration-300 hover:border-ring/50" }))}>
										{formData.beginAtTime || "Uhrzeit wählen"}
									</Popover.Trigger>
									<Popover.Content class="w-64 p-4 bg-card border-border rounded-xl shadow-xl">
										<div class="grid grid-cols-2 gap-4">
											<div>
												<p class="text-sm font-medium mb-2">Stunde</p>
												<div class="max-h-32 overflow-y-auto rounded-md border border-border">
													{#each Array.from({length: 24}, (_, i) => i.toString().padStart(2, '0')) as hour}
																<button type="button" class="w-full py-1 hover:bg-muted text-foreground" onclick={() => { const [_, m] = (formData.beginAtTime || '00:00').split(':'); formData.beginAtTime = `${hour}:${m}`; }}>
																	{hour}
																</button>
															{/each}
												</div>
											</div>
											<div>
												<p class="text-sm font-medium mb-2">Minute</p>
												<div class="max-h-32 overflow-y-auto rounded-md border border-border">
													{#each Array.from({length: 60}, (_, i) => i.toString().padStart(2, '0')) as minute}
																<button type="button" class="w-full py-1 hover:bg-muted text-foreground" onclick={() => { const [h, _] = (formData.beginAtTime || '00:00').split(':'); formData.beginAtTime = `${h}:${minute}`; }}>
																	{minute}
																</button>
															{/each}
												</div>
											</div>
										</div>
									</Popover.Content>
								</Popover.Root>
							</div>
						</div>
					</div>

				{:else if formData.reminderMode === 'appointment'}
					<div class="space-y-6 animate-fade-in">
						<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
						<div class="transition-all duration-300">
						<label class="block text-sm text-subheading text-card-foreground mb-3">Termin-Datum</label>
						<Popover.Root bind:open={appointmentDateOpen}>
							<Popover.Trigger class={cn(buttonVariants({ variant: "outline", class: "w-full h-12 justify-start text-left font-normal rounded-xl px-4 bg-input border-border text-foreground focus:border-ring focus:ring-ring transition-all duration-300 hover:border-ring/50" }), !formData.appointmentDate && "text-muted-foreground")}>
								{formData.appointmentDate ? new DateFormatter("de-DE", { dateStyle: "long" }).format(formData.appointmentDate.toDate(getLocalTimeZone())) : "Datum wählen"}
							</Popover.Trigger>
							<Popover.Content class="w-auto p-0 bg-card border-border rounded-xl shadow-xl">
								<DateCalendar type="single" initialFocus bind:value={formData.appointmentDate} />
							</Popover.Content>
						</Popover.Root>
					</div>
						<div class="transition-all duration-300">
							<label class="block text-sm text-subheading text-card-foreground mb-3">Termin-Uhrzeit</label>
							<Popover.Root>
								<Popover.Trigger class={cn(buttonVariants({ variant: "outline", class: "w-full h-12 justify-start text-left font-normal rounded-xl px-4 bg-input border-border text-foreground focus:border-ring focus:ring-ring transition-all duration-300 hover:border-ring/50" }))}>
									{formData.appointmentTime || "Uhrzeit wählen"}
								</Popover.Trigger>
								<Popover.Content class="w-64 p-4 bg-card border-border rounded-xl shadow-xl">
									<div class="grid grid-cols-2 gap-4">
										<div>
											<p class="text-sm font-medium mb-2">Stunde</p>
											<div class="max-h-32 overflow-y-auto rounded-md border border-border">
												{#each Array.from({length: 24}, (_, i) => i.toString().padStart(2, '0')) as hour}
																	<button type="button" class="w-full py-1 hover:bg-muted text-foreground" onclick={() => { const [_, m] = (formData.appointmentTime || '00:00').split(':'); formData.appointmentTime = `${hour}:${m}`; }}>
																		{hour}
																	</button>
																{/each}
											</div>
										</div>
										<div>
											<p class="text-sm font-medium mb-2">Minute</p>
											<div class="max-h-32 overflow-y-auto rounded-md border border-border">
												{#each Array.from({length: 60}, (_, i) => i.toString().padStart(2, '0')) as minute}
																	<button type="button" class="w-full py-1 hover:bg-muted text-foreground" onclick={() => { const [h, _] = (formData.appointmentTime || '00:00').split(':'); formData.appointmentTime = `${h}:${minute}`; }}>
																		{minute}
																	</button>
																{/each}
											</div>
										</div>
									</div>
								</Popover.Content>
							</Popover.Root>
						</div>
					</div>

						<div class="transition-all duration-300">
							<label class="block text-sm text-subheading text-card-foreground mb-3">Wiederholung</label>
							<Select.Root type="single" name="recurrence" bind:value={formData.recurrence}>
								<Select.Trigger class="w-full h-12 px-4 bg-input border border-border rounded-xl text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 flex items-center">
									{recurrenceTriggerContent}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										<Select.Label>Wiederholung wählen</Select.Label>
										{#each recurrenceOptions as option (option.value)}
											<Select.Item value={option.value} label={option.label}>
												{option.label}
											</Select.Item>
										{/each}
									</Select.Group>
								</Select.Content>
							</Select.Root>
						</div>

						{#if formData.recurrence === 'custom'}
							<div class="transition-all duration-300 animate-fade-in">
								<Label class="block text-sm text-subheading text-card-foreground mb-3">Wochentage auswählen</Label>
								<div class="grid grid-cols-2 md:grid-cols-4 gap-3">
									{#each weekdayOptions as day}
										<button
											type="button"
											onclick={() => toggleWeekday(day.value)}
											class="p-3 border-2 rounded-lg transition-all duration-300 hover:scale-105 {formData.weekdays.includes(day.value) ? 'border-primary bg-primary/10 text-primary' : 'border-border hover:border-primary/50'}"
										>
											<span class="text-sm font-medium">{day.label}</span>
										</button>
									{/each}
								</div>
							</div>
						{/if}

						{#if formData.recurrence !== 'once'}
					<div class="transition-all duration-300 {formData.recurrence === 'custom' ? 'mt-6' : 'mt-0'}">
						<label class="block text-sm text-subheading text-card-foreground mb-3">Ende der Wiederholung (optional)</label>
						<Popover.Root bind:open={recurrenceEndDateOpen}>
							<Popover.Trigger class={cn(buttonVariants({ variant: "outline", class: "w-full h-12 justify-start text-left font-normal rounded-xl px-4 bg-input border-border text-foreground focus:border-ring focus:ring-ring transition-all duration-300 hover:border-ring/50" }), !formData.recurrenceEndDate && "text-muted-foreground")}>
						{formData.recurrenceEndDate ? new DateFormatter("de-DE", { dateStyle: "long" }).format(formData.recurrenceEndDate.toDate(getLocalTimeZone())) : "Datum wählen"}
					</Popover.Trigger>
							<Popover.Content class="w-auto p-0 bg-card border-border rounded-xl shadow-xl">
								<DateCalendar type="single" initialFocus bind:value={formData.recurrenceEndDate} />
							</Popover.Content>
						</Popover.Root>
					</div>
				{/if}
					</div>
				{/if}
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

<style>
	.color-option {
		transition: all 0.3s ease;
	}

	.color-option:hover {
		transform: scale(1.1);
	}

	.color-option.selected {
		transform: scale(1.2);
		box-shadow: 0 0 0 3px rgba(var(--primary), 0.3);
	}

	.mode-fields-container {
		overflow: hidden;
	}

	@keyframes fade-in {
		from {
			opacity: 0;
			transform: translateY(10px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.animate-fade-in {
		animation: fade-in 0.3s ease-out;
    }
</style>