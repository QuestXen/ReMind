<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import { Card, Header, Title, Content } from '$lib/components/ui/card/index';
	import { Input } from '$lib/components/ui/input/index';
	import { invoke } from '@tauri-apps/api/core';
	import { Plus, Bell, Trash2, Calendar, Clock, Edit, Settings } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import SettingsComponent from './Settings.svelte';
	import TitleBar from './TitleBar.svelte';

	type ReminderInterval = 'minutes' | 'hours' | 'days' | 'weeks' | 'months' | 'specific';
	type ReminderColor = 'blue' | 'green' | 'purple' | 'red' | 'orange' | 'pink';

	interface Reminder {
		id: string;
		name: string;
		interval: ReminderInterval;
		intervalValue: number;
		specificDate?: string;
		color: ReminderColor;
		createdAt: string;
		lastNotified?: string;
	}

	let showCreateForm = $state(false);
	let showEditForm = $state(false);
	let showSettings = $state(false);
	let editingReminder = $state<Reminder | null>(null);
	let reminders = $state<Reminder[]>([]);
	let newReminder = $state({
		name: '',
		interval: 'days' as ReminderInterval,
		intervalValue: 1,
		specificDate: '',
		color: 'blue' as ReminderColor
	});

	let reminderTimers = new Map<string, NodeJS.Timeout>();

	const colorClasses = {
		blue: 'bg-blue-500 border-blue-200 text-blue-50',
		green: 'bg-green-500 border-green-200 text-green-50',
		purple: 'bg-purple-500 border-purple-200 text-purple-50',
		red: 'bg-red-500 border-red-200 text-red-50',
		orange: 'bg-orange-500 border-orange-200 text-orange-50',
		pink: 'bg-pink-500 border-pink-200 text-pink-50'
	};

	const intervalLabels = {
		minutes: 'Minuten',
		hours: 'Stunden',
		days: 'Tage',
		weeks: 'Wochen',
		months: 'Monate',
		specific: 'Bestimmtes Datum'
	};

	async function createReminder() {
		if (!newReminder.name.trim()) return;

		const reminder: Reminder = {
			id: crypto.randomUUID(),
			name: newReminder.name,
			interval: newReminder.interval,
			intervalValue: newReminder.intervalValue,
			specificDate: newReminder.interval === 'specific' ? newReminder.specificDate : undefined,
			color: newReminder.color,
			createdAt: new Date().toISOString()
		};

		try {
			await invoke('add_reminder', { reminder });
			reminders = [...reminders, reminder];
			resetForm();
			showCreateForm = false;
			setTimeout(() => {
				const newReminderElement = document.getElementById(`reminder-${reminder.id}`);
				if (newReminderElement) {
					newReminderElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
				}
			}, 100);

			// Start notification timer
			startReminderTimer(reminder);
		} catch (error) {
			console.error('Failed to create reminder:', error);
		}
	}

	function resetForm() {
		newReminder = {
			name: '',
			interval: 'days',
			intervalValue: 1,
			specificDate: '',
			color: 'blue'
		};
	}

	function startEditReminder(reminder: Reminder) {
		editingReminder = { ...reminder };

		showEditForm = true;
		showCreateForm = false;
	}

	async function updateReminder() {
		if (!editingReminder || !editingReminder.name.trim()) return;

		// Clear existing timer
		const existingTimer = reminderTimers.get(editingReminder!.id);
		if (existingTimer) {
			clearTimeout(existingTimer);
			reminderTimers.delete(editingReminder!.id);
		}

		// Update reminder via Rust backend
		await invoke('update_reminder', { reminder: editingReminder });

		// Update local state
		const index = reminders.findIndex((r) => r.id === editingReminder!.id);
		if (index !== -1) {
			reminders[index] = editingReminder;
			reminders = reminders; // Trigger reactivity

			// Start new timer
			startReminderTimer(editingReminder!);
		}

		showEditForm = false;
		editingReminder = null;
	}

	function cancelEdit() {
		showEditForm = false;
		editingReminder = null;
	}

	function openSettings() {
		showSettings = true;
	}

	function closeSettings() {
		showSettings = false;
	}

	async function deleteReminder(id: string) {
		try {
			await invoke('delete_reminder', { reminderId: id });
			reminders = reminders.filter((r) => r.id !== id);

			// Clear timer if exists
			const timerId = reminderTimers.get(id);
			if (timerId) {
				clearTimeout(timerId);
				reminderTimers.delete(id);
			}
		} catch (error) {
			console.error('Failed to delete reminder:', error);
		}
	}

	async function loadReminders() {
		try {
			const loadedReminders = await invoke<Reminder[]>('load_reminders');
			reminders = loadedReminders;

			// Start timers for all loaded reminders
			reminders.forEach((reminder) => {
				startReminderTimer(reminder);
			});
		} catch (error) {
			console.error('Failed to load reminders:', error);
		}
	}

	function startReminderTimer(reminder: Reminder) {
		let intervalMs = 0;

		switch (reminder.interval) {
			case 'minutes':
				intervalMs = reminder.intervalValue * 60 * 1000;
				break;
			case 'hours':
				intervalMs = reminder.intervalValue * 60 * 60 * 1000;
				break;
			case 'days':
				intervalMs = reminder.intervalValue * 24 * 60 * 60 * 1000;
				break;
			case 'weeks':
				intervalMs = reminder.intervalValue * 7 * 24 * 60 * 60 * 1000;
				break;
			case 'months':
				intervalMs = reminder.intervalValue * 30 * 24 * 60 * 60 * 1000;
				break;
			case 'specific':
				if (reminder.specificDate) {
					const targetDate = new Date(reminder.specificDate);
					const now = new Date();
					intervalMs = targetDate.getTime() - now.getTime();
					if (intervalMs <= 0) return;
				}
				break;
		}

		if (intervalMs > 0) {
			const timerId = setTimeout(() => {
				sendNotification(reminder);
				if (reminder.interval !== 'specific') {
					startReminderTimer(reminder); // Restart for recurring reminders
				}
			}, intervalMs);

			reminderTimers.set(reminder.id, timerId);
		}
	}

	async function sendNotification(reminder: Reminder) {
		try {
			// Play Windows notification sound
			const audio = new Audio('ms-winsoundevent:Notification.Default');
			audio.play().catch(() => {
				// Fallback to system beep if Windows sound fails
				const fallbackAudio = new Audio(
					'data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1fdJivrJBhNjVgodDbq2EcBj+a2/LDciUFLIHO8tiJNwgZaLvt559NEAxQp+PwtmMcBjiR1/LMeSwFJHfH8N2QQAoUXrTp66hVFApGn+DyvmwhBSuBzvLZiTYIG2m98OScTgwOUarm7blmGgU7k9n1unEiBC13yO/eizEIHWq+8+OWT'
				);
				fallbackAudio.play();
			});

			await invoke('send_notification', {
				title: 'Erinnerung',
				body: reminder.name
			});

			const timestamp = new Date().toISOString();
			await invoke('update_reminder_last_notified', {
				reminderId: reminder.id,
				timestamp
			});

			// Update local state
			reminder.lastNotified = timestamp;
		} catch (error) {
			console.error('Failed to send notification:', error);
		}
	}

	function formatReminderInfo(reminder: Reminder): string {
		if (reminder.interval === 'specific' && reminder.specificDate) {
			return `Am ${new Date(reminder.specificDate).toLocaleDateString('de-DE')}`;
		}
		return `Alle ${reminder.intervalValue} ${intervalLabels[reminder.interval]}`;
	}

	onMount(() => {
		loadReminders();
	});
</script>

{#if showSettings}
	<SettingsComponent on:close={closeSettings} />
{:else}
	<!-- ReminderApp -->
	<div
		class="flex h-screen flex-col overflow-hidden {showSettings ? 'animate-slide-out-left' : ''}"
	>
		<TitleBar title="Re:Mind" icon="R" />

		<div id="content-scroll" class="bg-background relative flex-1 overflow-y-auto">
			<div class="p-6">
				<div class="mx-auto max-w-4xl">
					<div class="animate-fade-in mb-8 text-center">
						<p class="text-muted-foreground text-caption text-lg">Erstelle Erinnerungen</p>
					</div>

					<div class="mb-12 flex justify-center">
						<Button
							onclick={() => {
								showCreateForm = true;
								setTimeout(() => {
									const formElement = document.getElementById('create-form');
									if (formElement) {
										formElement.scrollIntoView({ behavior: 'smooth', block: 'start' });
									}
								}, 100);
							}}
							class="bg-primary hover:bg-primary/90 text-primary-foreground text-subheading rounded-2xl border-0 px-8 py-4 shadow-lg transition-all duration-300 hover:scale-105 hover:shadow-xl"
						>
							<Plus class="mr-3 h-5 w-5" />
							Neue Erinnerung
						</Button>
					</div>

					{#if showCreateForm || showEditForm}
						<Card
							id="create-form"
							class="bg-card border-border animate-slide-up mb-12 overflow-hidden rounded-3xl border shadow-xl transition-all duration-500"
						>
							<Content class="space-y-8 p-8">
								<h2 class="text-heading text-card-foreground mb-6 text-3xl">
									{showEditForm ? 'Erinnerung bearbeiten' : 'Neue Erinnerung erstellen'}
								</h2>
								<div class="transition-all duration-300">
									<label
										for="reminder-name"
										class="text-subheading text-card-foreground mb-3 block text-sm"
										>Erinnerungsname</label
									>
									{#if showEditForm}
										<Input
											id="reminder-name"
											bind:value={editingReminder!.name}
											placeholder="z.B. Medikamente nehmen, Meeting vorbereiten..."
											class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-12 w-full rounded-xl px-4 transition-all duration-300"
										/>
									{:else}
										<Input
											id="reminder-name"
											bind:value={newReminder.name}
											placeholder="z.B. Medikamente nehmen, Wasser trinken..."
											class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-12 w-full rounded-xl px-4 transition-all duration-300"
										/>
									{/if}
								</div>

								<div class="grid grid-cols-1 gap-8 md:grid-cols-2">
									<div class="transition-all duration-300">
										<label
											for="reminder-interval"
											class="text-subheading text-card-foreground mb-3 block text-sm"
											>Intervall</label
										>
										{#if showEditForm}
											<select
												bind:value={editingReminder.interval}
												class="h-12 w-full cursor-pointer rounded-xl border-2 border-gray-200 bg-white px-4 py-3 text-gray-900 shadow-sm transition-all duration-300 hover:border-gray-300 hover:shadow-md focus:border-blue-500 focus:ring-4 focus:ring-blue-500/20 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100 dark:hover:border-gray-500"
											>
												{#each Object.entries(intervalLabels) as [value, label]}
													<option {value}>{label}</option>
												{/each}
											</select>
										{:else}
											<select
												bind:value={newReminder.interval}
												class="h-12 w-full cursor-pointer rounded-xl border-2 border-gray-200 bg-white px-4 py-3 text-gray-900 shadow-sm transition-all duration-300 hover:border-gray-300 hover:shadow-md focus:border-blue-500 focus:ring-4 focus:ring-blue-500/20 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100 dark:hover:border-gray-500"
											>
												{#each Object.entries(intervalLabels) as [value, label]}
													<option {value}>{label}</option>
												{/each}
											</select>
										{/if}
									</div>

									{#if showEditForm}
										{#if editingReminder!.interval !== 'specific'}
											<div class="transition-all duration-300">
												<label
													for="reminder-interval-value"
													class="text-subheading text-card-foreground mb-3 block text-sm"
													>Anzahl</label
												>
												<Input
													id="reminder-interval-value"
													type="number"
													bind:value={editingReminder!.intervalValue}
													min="1"
													class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-12 w-full rounded-xl px-4 transition-all duration-300"
												/>
											</div>
										{:else}
											<div class="transition-all duration-300">
												<label
													for="reminder-specific-date"
													class="text-subheading text-card-foreground mb-3 block text-sm"
													>Datum & Zeit</label
												>
												<Input
													id="reminder-specific-date"
													type="datetime-local"
													bind:value={editingReminder!.specificDate}
													class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-12 w-full rounded-xl px-4 transition-all duration-300"
												/>
											</div>
										{/if}
									{:else if newReminder.interval !== 'specific'}
										<div class="transition-all duration-300">
											<label
												for="new-reminder-interval-value"
												class="text-subheading text-card-foreground mb-3 block text-sm"
												>Anzahl</label
											>
											<Input
												id="new-reminder-interval-value"
												type="number"
												bind:value={newReminder.intervalValue}
												min="1"
												class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-12 w-full rounded-xl px-4 transition-all duration-300"
											/>
										</div>
									{:else}
										<div class="transition-all duration-300">
											<label
												for="new-reminder-specific-date"
												class="text-subheading text-card-foreground mb-3 block text-sm"
												>Datum & Zeit</label
											>
											<Input
												id="new-reminder-specific-date"
												type="datetime-local"
												bind:value={newReminder.specificDate}
												class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-12 w-full rounded-xl px-4 transition-all duration-300"
											/>
										</div>
									{/if}
								</div>

								<fieldset class="transition-all duration-300">
									<legend class="text-subheading text-card-foreground mb-4 block text-sm"
										>Akzentfarbe wählen</legend
									>
									<div class="flex gap-4">
										{#each Object.entries(colorClasses) as [color, classes]}
											{#if showEditForm}
												<button
													type="button"
													onclick={() => {
														if (editingReminder) editingReminder.color = color as ReminderColor;
													}}
													aria-label="Farbe {color} auswählen"
													class="h-10 w-10 rounded-full {classes} {editingReminder?.color === color
														? 'ring-primary scale-110 shadow-lg ring-4'
														: 'ring-border ring-2'} transition-all duration-300 hover:scale-105 hover:shadow-md"
												></button>
											{:else}
												<button
													type="button"
													onclick={() => (newReminder.color = color as ReminderColor)}
													aria-label="Farbe {color} auswählen"
													class="h-10 w-10 rounded-full {classes} {newReminder.color === color
														? 'ring-primary scale-110 shadow-lg ring-4'
														: 'ring-border ring-2'} transition-all duration-300 hover:scale-105 hover:shadow-md"
												></button>
											{/if}
										{/each}
									</div>
								</fieldset>

								<div class="flex gap-6 pt-8">
									<Button
										onclick={showEditForm ? updateReminder : createReminder}
										class="bg-primary hover:bg-primary/90 text-primary-foreground text-subheading flex-1 rounded-xl border-0 py-4 transition-all duration-300 hover:scale-105 hover:shadow-lg"
									>
										<Bell class="mr-3 h-5 w-5" />
										{showEditForm ? 'Erinnerung aktualisieren' : 'Erinnerung erstellen'}
									</Button>
									<Button
										onclick={() => {
											showCreateForm = false;
											showEditForm = false;
											resetForm();
											cancelEdit();
											const contentScroll = document.getElementById('content-scroll');
											if (contentScroll) {
												contentScroll.scrollTo({ top: 0, behavior: 'smooth' });
											}
										}}
										variant="outline"
										class="border-border text-muted-foreground hover:bg-destructive/10 hover:text-destructive text-subheading hover:border-destructive flex-1 rounded-xl border-2 py-4 transition-all duration-300 hover:scale-105"
									>
										Abbrechen
									</Button>
								</div>
							</Content>
						</Card>
					{/if}

					<div class="grid gap-6">
						{#if reminders.length === 0}
							<Card
								class="bg-card border-border animate-fade-in overflow-hidden rounded-3xl border shadow-lg"
							>
								<Content class="py-16 text-center">
									<h3 class="text-heading text-card-foreground mb-3 text-2xl">
										Noch keine Erinnerungen
									</h3>
									<p class="text-muted-foreground text-caption text-lg">
										Erstelle deine erste Erinnerung
									</p>
								</Content>
							</Card>
						{:else}
							{#each reminders as reminder (reminder.id)}
								<Card
									id={`reminder-${reminder.id}`}
									class="bg-card border-border group animate-slide-up overflow-hidden rounded-3xl border shadow-lg transition-all duration-300 hover:scale-[1.02] hover:shadow-xl"
								>
									<Content class="p-8">
										<div class="flex items-center justify-between">
											<div class="flex items-center gap-6">
												<div
													class="h-6 w-6 rounded-full {colorClasses[
														reminder.color
													]} flex-shrink-0 shadow-md transition-all duration-300 group-hover:scale-110"
												></div>
												<div>
													<h3 class="text-subheading text-card-foreground mb-2 text-xl">
														{reminder.name}
													</h3>
													<div class="text-muted-foreground flex items-center gap-6 text-sm">
														<span class="text-body flex items-center gap-2">
															{#if reminder.interval === 'specific'}
																<Calendar class="h-4 w-4" />
															{:else}
																<Clock class="h-4 w-4" />
															{/if}
															{formatReminderInfo(reminder)}
														</span>
														{#if reminder.lastNotified}
															<span class="text-muted-foreground text-caption"
																>Zuletzt: {new Date(reminder.lastNotified).toLocaleString(
																	'de-DE'
																)}</span
															>
														{/if}
													</div>
												</div>
											</div>
											<div class="flex gap-3">
												<Button
													onclick={() => startEditReminder(reminder)}
													variant="outline"
													class="border-border text-muted-foreground hover:bg-accent hover:border-primary rounded-xl border-2 p-3 opacity-0 transition-all duration-300 group-hover:opacity-100"
												>
													<Edit class="h-4 w-4" />
												</Button>
												<Button
													onclick={() => deleteReminder(reminder.id)}
													variant="outline"
													class="border-destructive/30 text-destructive hover:bg-destructive/10 hover:border-destructive rounded-xl border-2 p-3 opacity-0 transition-all duration-300 group-hover:opacity-100"
												>
													<Trash2 class="h-4 w-4" />
												</Button>
											</div>
										</div>
									</Content>
								</Card>
							{/each}
						{/if}
					</div>
				</div>
			</div>
		</div>
	</div>
{/if}

<!-- Settings Button - Always visible -->
<div class="fixed right-6 bottom-6 z-50">
	<Button
		onclick={openSettings}
		variant="default"
		size="icon"
		class="bg-primary hover:bg-primary/90 text-primary-foreground h-12 w-12 rounded-xl shadow-lg transition-all duration-300 hover:scale-105 hover:shadow-xl"
	>
		<Settings class="h-5 w-5" />
	</Button>
</div>

<style>
	@keyframes fade-in {
		from {
			opacity: 0;
			transform: translateY(-10px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@keyframes slide-up {
		from {
			opacity: 0;
			transform: translateY(20px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@keyframes slide-out-left {
		from {
			opacity: 1;
			transform: translateX(0);
		}
		to {
			opacity: 0;
			transform: translateX(-20px);
		}
	}

	.animate-fade-in {
		animation: fade-in 0.6s ease-out;
	}

	.animate-slide-up {
		animation: slide-up 0.4s ease-out;
	}

	.animate-slide-out-left {
		animation: slide-out-left 0.3s ease-out;
	}

	/* Smooth scrolling */
	* {
		scroll-behavior: smooth;
	}

	/* Hide scrollbars */
	html,
	body {
		scrollbar-width: none;
		-ms-overflow-style: none;
	}

	::-webkit-scrollbar {
		display: none;
	}

	/* Custom focus styles */
	input:focus,
	select:focus,
	button:focus {
		outline: none;
		box-shadow: 0 0 0 3px rgba(0, 0, 0, 0.1);
	}
</style>
