<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import { Card, Header, Title, Content } from '$lib/components/ui/card/index';
	import { Input } from '$lib/components/ui/input/index';
	import { invoke } from '@tauri-apps/api/core';
	import { Plus, Bell, Trash2, Calendar, Clock, Edit, Settings } from '@lucide/svelte';
	import { onMount, onDestroy } from 'svelte';
	import SettingsComponent from './Settings.svelte';
	import TitleBar from './TitleBar.svelte';
	import { reminders, addReminder, updateReminder as updateReminderStore, deleteReminderFromStore } from '$lib/stores';
	import type { Reminder, ReminderInterval, ReminderColor } from '$lib/stores';

	let showCreateForm = $state(false);
	let showEditForm = $state(false);
	let showSettings = $state(false);
	let editingReminder = $state<Reminder | null>(null);
	let newReminder = $state({
		name: '',
		interval: 'days' as ReminderInterval,
		intervalValue: 1,
		specificDate: '',
		color: 'blue' as ReminderColor
	});

	let reminderTimers = new Map<string, {
		timerId: NodeJS.Timeout;
		nextExecution: Date;
		isActive: boolean;
	}>();
	
	let timerCleanupInterval: NodeJS.Timeout | null = null;

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
			addReminder(reminder);
			resetForm();
			showCreateForm = false;
			setTimeout(() => {
				const newReminderElement = document.getElementById(`reminder-${reminder.id}`);
				if (newReminderElement) {
					newReminderElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
				}
			}, 100);
			
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
		clearReminderTimer(editingReminder.id);
		
		try {
			await invoke('update_reminder', { reminder: editingReminder });
			
			updateReminderStore(editingReminder);
			startReminderTimer(editingReminder);
			
			const updatedReminderName = editingReminder.name;
			showEditForm = false;
			editingReminder = null;
			console.log(`Updated reminder: ${updatedReminderName}`);
		} catch (error) {
			console.error('Failed to update reminder:', error);
			if (editingReminder) {
				startReminderTimer(editingReminder);
			}
		}
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
		clearReminderTimer(id);
		
		try {
			await invoke('delete_reminder', { reminderId: id });
			deleteReminderFromStore(id);
			console.log(`Deleted reminder with ID: ${id}`);
		} catch (error) {
			console.error('Failed to delete reminder:', error);
		}
	}



	function startReminderTimer(reminder: Reminder) {
		clearReminderTimer(reminder.id);
		
		const now = new Date();
		let nextExecution: Date;
		
		switch (reminder.interval) {
			case 'minutes':
				nextExecution = new Date(now.getTime() + reminder.intervalValue * 60 * 1000);
				break;
			case 'hours':
				nextExecution = new Date(now.getTime() + reminder.intervalValue * 60 * 60 * 1000);
				break;
			case 'days':
				nextExecution = new Date(now.getTime() + reminder.intervalValue * 24 * 60 * 60 * 1000);
				break;
			case 'weeks':
				nextExecution = new Date(now.getTime() + reminder.intervalValue * 7 * 24 * 60 * 60 * 1000);
				break;
			case 'months':
				nextExecution = new Date(now);
				nextExecution.setMonth(nextExecution.getMonth() + reminder.intervalValue);
				break;
			case 'specific':
				if (!reminder.specificDate) {
					console.warn(`Reminder ${reminder.id} has no specific date set`);
					return;
				}
				nextExecution = new Date(reminder.specificDate);
				if (nextExecution <= now) {
					console.warn(`Reminder ${reminder.id} scheduled for past date: ${nextExecution}`);
					return;
				}
				break;
			default:
				console.error(`Unknown interval type: ${reminder.interval}`);
				return;
		}
		
		const timeUntilExecution = nextExecution.getTime() - now.getTime();
		if (timeUntilExecution <= 0) {
			console.warn(`Invalid execution time for reminder ${reminder.id}`);
			return;
		}
		
		if (timeUntilExecution > 2147483647) {
			console.warn(`Execution time too far in future for reminder ${reminder.id}, scheduling for max timeout`);
			const timerId = setTimeout(() => {
				startReminderTimer(reminder); 
			}, 2147483647);
			
			reminderTimers.set(reminder.id, {
				timerId,
				nextExecution,
				isActive: true
			});
			return;
		}
		
		const timerId = setTimeout(async () => {
			try {
				await sendNotification(reminder);
				
				if (reminder.interval !== 'specific') {
					startReminderTimer(reminder);
				} else {
					clearReminderTimer(reminder.id);
				}
			} catch (error) {
				console.error(`Failed to execute reminder ${reminder.id}:`, error);
				setTimeout(() => startReminderTimer(reminder), 60000);
			}
		}, timeUntilExecution);
		
		reminderTimers.set(reminder.id, {
			timerId,
			nextExecution,
			isActive: true
		});
		
		console.log(`Scheduled reminder "${reminder.name}" for ${nextExecution.toLocaleString('de-DE')}`);
	}
	
	function clearReminderTimer(reminderId: string) {
		const timerInfo = reminderTimers.get(reminderId);
		if (timerInfo) {
			clearTimeout(timerInfo.timerId);
			reminderTimers.delete(reminderId);
			console.log(`Cleared timer for reminder ${reminderId}`);
		}
	}
	
	function clearAllTimers() {
		reminderTimers.forEach((timerInfo, id) => {
			clearTimeout(timerInfo.timerId);
		});
		reminderTimers.clear();
		console.log('Cleared all reminder timers');
	}

	async function sendNotification(reminder: Reminder) {
		try {

			const audio = new Audio('ms-winsoundevent:Notification.Default');
			audio.play().catch(() => {
				const fallbackAudio = new Audio('data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1fdJivrJBhNjVgodDbq2EcBj+a2/LDciUFLIHO8tiJNwgZaLvt559NEAxQp+PwtmMcBjiR1/LMeSwFJHfH8N2QQAoUXrTp66hVFApGn+DyvmwhBSuBzvLZiTYIG2m98OScTgwOUarm7blmGgU7k9n1unEiBC13yO/eizEIHWq+8+OWT');
				fallbackAudio.play();
			});
			
			await invoke('send_notification', {
				title: 'ReMind',
				body: `Erinnerung: ${reminder.name}`
			});
			console.log(`Notification sent for reminder: ${reminder.name}`);
			
			const timestamp = new Date().toISOString();
			await invoke('update_reminder_last_notified', {
				reminderId: reminder.id,
				timestamp
			});
			reminder.lastNotified = timestamp;
		} catch (error) {
			console.error('Failed to send notification:', error);
			if ('Notification' in window && Notification.permission === 'granted') {
				new Notification('ReMind', {
					body: `Erinnerung: ${reminder.name}`,
					icon: '/favicon.ico'
				});
			} else if ('Notification' in window && Notification.permission !== 'denied') {
				Notification.requestPermission().then(permission => {
					if (permission === 'granted') {
						new Notification('ReMind', {
							body: `Erinnerung: ${reminder.name}`,
							icon: '/favicon.ico'
						});
					}
				});
			}
			throw error; 
		}
	}
	
	function getTimerStatus() {
		const status = {
			activeTimers: reminderTimers.size,
			timers: Array.from(reminderTimers.entries()).map(([id, timerInfo]) => {
				const reminder = $reminders.find(r => r.id === id);
				return {
					id,
					name: reminder?.name || 'Unknown',
					nextExecution: timerInfo.nextExecution.toLocaleString('de-DE'),
					timeUntilExecution: Math.max(0, timerInfo.nextExecution.getTime() - Date.now()),
					isActive: timerInfo.isActive
				};
			})
		};
		console.table(status.timers);
		return status;
	}
	if (typeof window !== 'undefined' && import.meta.env.DEV) {
		(window as any).getTimerStatus = getTimerStatus;
		(window as any).clearAllTimers = clearAllTimers;
		console.log('Debug functions available: getTimerStatus(), clearAllTimers()');
	}

	function formatReminderInfo(reminder: Reminder): string {
		if (reminder.interval === 'specific' && reminder.specificDate) {
			return `Am ${new Date(reminder.specificDate).toLocaleDateString('de-DE')}`;
		}
	return `Alle ${reminder.intervalValue} ${intervalLabels[reminder.interval]}`;
}

onMount(() => {
	// Start timers for existing reminders from store
	const unsubscribe = reminders.subscribe(currentReminders => {
		// Clear existing timers
		clearAllTimers();
		
		// Start timers for all current reminders
		currentReminders.forEach(reminder => {
			startReminderTimer(reminder);
		});
		
		console.log(`Initialized ${currentReminders.length} reminders with timers`);
	});
	
	// Setup timer cleanup interval
	timerCleanupInterval = setInterval(() => {
		const now = new Date();
		reminderTimers.forEach((timerInfo, id) => {
			if (timerInfo.nextExecution <= now && !timerInfo.isActive) {
				clearReminderTimer(id);
			}
		});
	}, 5 * 60 * 1000);
	
	// Handle dropdown clicks
	const handleClickOutside = (event: MouseEvent) => {
		const editDropdown = document.getElementById('edit-interval-dropdown');
		const newDropdown = document.getElementById('new-interval-dropdown');
		const target = event.target as Element;
		
		if (editDropdown && !editDropdown.closest('.custom-dropdown')?.contains(target)) {
			editDropdown.classList.add('hidden');
		}
		if (newDropdown && !newDropdown.closest('.custom-dropdown')?.contains(target)) {
			newDropdown.classList.add('hidden');
		}
	};
	
	document.addEventListener('click', handleClickOutside);
	
	return () => {
		unsubscribe();
		document.removeEventListener('click', handleClickOutside);
	};
});

onDestroy(() => {
	clearAllTimers();
	if (timerCleanupInterval) {
		clearInterval(timerCleanupInterval);
	}
	console.log('ReminderApp component destroyed, all timers cleared');
});
</script>

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

	@keyframes slide-in-left {
		from {
			opacity: 0;
			transform: translateX(20px);
		}
		to {
			opacity: 1;
			transform: translateX(0);
		}
	}

	@keyframes slide-up-staggered {
		from {
			opacity: 0;
			transform: translateY(30px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.animate-fade-in {
		animation: fade-in 0.6s ease-out;
	}

	.animate-slide-up {
		animation: slide-up 0.4s ease-out;
	}

	.animate-slide-up-staggered {
		animation: slide-up-staggered 0.5s ease-out;
	}

	.animate-slide-out-left {
		animation: slide-out-left 0.3s ease-out;
	}

	.animate-slide-in-left {
		animation: slide-in-left 0.3s ease-out;
	}

	* {
		scroll-behavior: smooth;
	}

	html, body {
		scrollbar-width: none;
		-ms-overflow-style: none;
	}

	::-webkit-scrollbar {
		display: none;
	}

	input:focus,
	select:focus,
	button:focus {
		outline: none;
		box-shadow: 0 0 0 3px rgba(0, 0, 0, 0.1);
	}

	.custom-dropdown {
		position: relative;
	}

	.custom-dropdown button:focus {
		outline: none;
		box-shadow: 0 0 0 3px rgba(127, 90, 240, 0.2);
	}

	.dropdown-list {
		position: fixed;
		z-index: 9999;
		border: 1px solid #374151;
		border-radius: 0.375rem;
		background-color: #1f2937 !important;
		background: #1f2937 !important;
		box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
		max-height: 200px;
		overflow-y: auto;
		min-width: 200px;
	}

	.dropdown-item {
		padding: 0.5rem 0.75rem;
		cursor: pointer;
		color: #f9fafb !important;
		border: none;
		background: transparent;
		width: 100%;
		text-align: left;
		transition: background-color 0.2s;
		flex-shrink: 0;
	}

	.dropdown-item:hover {
		background-color: #374151 !important;
		color: #f9fafb !important;
	}


</style>

{#if showSettings}
	<SettingsComponent on:close={closeSettings} />
{:else}
<div class="flex flex-col h-screen overflow-hidden">
	<TitleBar title="Re:Mind" icon="R" />

  <div id="content-scroll" class="flex-1 overflow-y-auto bg-background relative {showSettings ? 'animate-slide-out-left' : 'animate-slide-in-left'}">
	<div class="p-6">
		<div class="max-w-4xl mx-auto">
		<div class="flex justify-center mb-12">
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
				class="bg-primary hover:bg-primary/90 text-primary-foreground px-8 py-4 rounded-2xl shadow-lg transition-all duration-300 hover:scale-105 hover:shadow-xl border-0 text-subheading"
			>
				<Plus class="w-5 h-5 mr-3" />
				Neue Erinnerung
			</Button>
		</div>

	{#if showCreateForm || showEditForm}
		<Card id="create-form" class="mb-12 bg-card border border-border shadow-xl rounded-3xl overflow-hidden transition-all duration-500 animate-slide-up">
			<Content class="space-y-8 p-8">
				<h2 class="text-3xl text-heading text-card-foreground mb-6">
					{showEditForm ? 'Erinnerung bearbeiten' : 'Neue Erinnerung erstellen'}
				</h2>
				<div class="transition-all duration-300">
					<label for="reminder-name" class="block text-sm text-subheading text-card-foreground mb-3">Erinnerungsname</label>
					{#if showEditForm}
						<Input 
								id="reminder-name"
								bind:value={editingReminder!.name}
								placeholder="z.B. Medikamente nehmen, Meeting vorbereiten..."
								class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
							/>
					{:else}
						<Input 
								id="reminder-name"
								bind:value={newReminder.name}
								placeholder="z.B. Medikamente nehmen, Wasser trinken..."
								class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
							/>
					{/if}
				</div>

				<div class="grid grid-cols-1 md:grid-cols-2 gap-8">
					<div class="transition-all duration-300">
						<label for="reminder-interval" class="block text-sm text-subheading text-card-foreground mb-3">Intervall</label>
						{#if showEditForm && editingReminder}
							<div class="custom-dropdown relative">
								<button 
									type="button"
									onclick={(event) => {
										const dropdown = document.getElementById('edit-interval-dropdown');
										if (dropdown) {
											const rect = event.currentTarget.getBoundingClientRect();
											dropdown.style.top = `${rect.bottom + 4}px`;
											dropdown.style.left = `${rect.left}px`;
											dropdown.style.width = `${rect.width}px`;
											dropdown.classList.toggle('hidden');
										}
									}}
									class="w-full h-12 px-4 py-3 bg-input border border-border rounded-xl text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 cursor-pointer shadow-sm hover:shadow-md flex items-center justify-between text-left"
								>
									<span>{intervalLabels[editingReminder.interval]}</span>
									<svg class="w-5 h-5 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
									</svg>
								</button>
								<div id="edit-interval-dropdown" class="dropdown-list hidden">
									{#each Object.entries(intervalLabels) as [value, label]}
										<button
												type="button"
												onclick={() => {
													if (editingReminder) {
														editingReminder.interval = value as ReminderInterval;
													}
													const dropdown = document.getElementById('edit-interval-dropdown');
													if (dropdown) {
														dropdown.classList.add('hidden');
													}
												}}
												class="dropdown-item {editingReminder?.interval === value ? 'bg-primary text-primary-foreground' : ''}"
											>
											{label}
										</button>
									{/each}
								</div>
							</div>
						{:else}
							<div class="custom-dropdown relative">
								<button 
									type="button"
									onclick={(event) => {
										const dropdown = document.getElementById('new-interval-dropdown');
										if (dropdown) {
											const rect = event.currentTarget.getBoundingClientRect();
											dropdown.style.top = `${rect.bottom + 4}px`;
											dropdown.style.left = `${rect.left}px`;
											dropdown.style.width = `${rect.width}px`;
											dropdown.classList.toggle('hidden');
										}
									}}
									class="w-full h-12 px-4 py-3 bg-input border border-border rounded-xl text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 cursor-pointer shadow-sm hover:shadow-md flex items-center justify-between text-left"
								>
									<span>{intervalLabels[newReminder.interval]}</span>
									<svg class="w-5 h-5 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
									</svg>
								</button>
								<div id="new-interval-dropdown" class="dropdown-list hidden">
									{#each Object.entries(intervalLabels) as [value, label]}
										<button
												type="button"
												onclick={() => {
													newReminder.interval = value as ReminderInterval;
													const dropdown = document.getElementById('new-interval-dropdown');
													if (dropdown) {
														dropdown.classList.add('hidden');
													}
												}}
												class="dropdown-item {newReminder.interval === value ? 'bg-primary text-primary-foreground' : ''}"
											>
											{label}
										</button>
									{/each}
								</div>
							</div>
						{/if}
					</div>

					{#if showEditForm && editingReminder}
						{#if editingReminder.interval !== 'specific'}
							<div class="transition-all duration-300">
								<label for="reminder-interval-value" class="block text-sm text-subheading text-card-foreground mb-3">Anzahl</label>
								<Input 
									id="reminder-interval-value"
									type="number"
									bind:value={editingReminder.intervalValue}
									min="1"
									class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
								/>
							</div>
						{:else}
							<div class="transition-all duration-300">
								<label for="reminder-specific-date" class="block text-sm text-subheading text-card-foreground mb-3">Datum & Zeit</label>
								<Input 
									id="reminder-specific-date"
									type="datetime-local"
									bind:value={editingReminder.specificDate}
									class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
								/>
							</div>
						{/if}
					{:else}
						{#if newReminder.interval !== 'specific'}
							<div class="transition-all duration-300">
								<label for="new-reminder-interval-value" class="block text-sm text-subheading text-card-foreground mb-3">Anzahl</label>
								<Input 
									id="new-reminder-interval-value"
									type="number"
									bind:value={newReminder.intervalValue}
									min="1"
									class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
								/>
							</div>
						{:else}
							<div class="transition-all duration-300">
								<label for="new-reminder-specific-date" class="block text-sm text-subheading text-card-foreground mb-3">Datum & Zeit</label>
								<Input 
									id="new-reminder-specific-date"
									type="datetime-local"
									bind:value={newReminder.specificDate}
									class="w-full h-12 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
								/>
							</div>
						{/if}
					{/if}
				</div>

				<fieldset class="transition-all duration-300">
					<legend class="block text-sm text-subheading text-card-foreground mb-4">Akzentfarbe wählen</legend>
					<div class="flex gap-4">
						{#each Object.entries(colorClasses) as [color, classes]}
							{#if showEditForm}
								<button
									type="button"
									onclick={() => { if (editingReminder) editingReminder.color = color as ReminderColor; }}
									aria-label="Farbe {color} auswählen"
									class="w-10 h-10 rounded-full {classes} {editingReminder?.color === color ? 'ring-4 ring-primary scale-110 shadow-lg' : 'ring-2 ring-border'} transition-all duration-300 hover:scale-105 hover:shadow-md"
								></button>
							{:else}
								<button
									type="button"
									onclick={() => newReminder.color = color as ReminderColor}
									aria-label="Farbe {color} auswählen"
									class="w-10 h-10 rounded-full {classes} {newReminder.color === color ? 'ring-4 ring-primary scale-110 shadow-lg' : 'ring-2 ring-border'} transition-all duration-300 hover:scale-105 hover:shadow-md"
								></button>
							{/if}
						{/each}
					</div>
				</fieldset>

				<div class="flex gap-6 pt-8">
					<Button 
						onclick={showEditForm ? updateReminder : createReminder}
						class="flex-1 bg-primary hover:bg-primary/90 text-primary-foreground py-4 rounded-xl text-subheading transition-all duration-300 hover:scale-105 hover:shadow-lg border-0"
					>
						<Bell class="w-5 h-5 mr-3" />
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
						class="flex-1 border-2 border-border text-muted-foreground hover:bg-destructive/10 hover:text-destructive py-4 rounded-xl text-subheading transition-all duration-300 hover:scale-105 hover:border-destructive"
					>
						Abbrechen
					</Button>
				</div>
			</Content>
		</Card>
	{/if}

	<div class="grid gap-6">
		{#if $reminders.length === 0}
			<Card class="bg-card border border-border shadow-lg rounded-3xl overflow-hidden animate-slide-up-staggered">
				<Content class="text-center py-16">
					<h3 class="text-2xl text-heading text-card-foreground mb-3">Noch keine Erinnerungen</h3>
					<p class="text-muted-foreground text-lg text-caption">Erstelle deine erste Erinnerung</p>
				</Content>
			</Card>
		{:else}
			{#each $reminders as reminder, index (reminder.id)}
			<Card id={`reminder-${reminder.id}`} class="bg-card border border-border shadow-lg hover:shadow-xl rounded-3xl overflow-hidden transition-all duration-300 group hover:scale-[1.02] animate-slide-up-staggered" style="animation-delay: {index * 0.1}s;">
					<Content class="p-8">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-6">
								<div class="w-6 h-6 rounded-full {colorClasses[reminder.color]} flex-shrink-0 shadow-md transition-all duration-300 group-hover:scale-110"></div>
								<div>
									<h3 class="text-xl text-subheading text-card-foreground mb-2">{reminder.name}</h3>
									<div class="flex items-center gap-6 text-sm text-muted-foreground">
										<span class="flex items-center gap-2 text-body">
											{#if reminder.interval === 'specific'}
												<Calendar class="w-4 h-4" />
											{:else}
												<Clock class="w-4 h-4" />
											{/if}
											{formatReminderInfo(reminder)}
										</span>
										{#if reminder.lastNotified}
											<span class="text-muted-foreground text-caption">Zuletzt: {new Date(reminder.lastNotified).toLocaleString('de-DE')}</span>
										{/if}
									</div>
								</div>
							</div>
							<div class="flex gap-3">
								<Button 
									onclick={() => startEditReminder(reminder)}
									variant="outline"
									class="opacity-0 group-hover:opacity-100 transition-all duration-300 border-2 border-border text-muted-foreground hover:bg-accent hover:border-primary rounded-xl p-3"
								>
									<Edit class="w-4 h-4" />
								</Button>
								<Button 
									onclick={() => deleteReminder(reminder.id)}
									variant="outline"
									class="opacity-0 group-hover:opacity-100 transition-all duration-300 border-2 border-destructive/30 text-destructive hover:bg-destructive/10 hover:border-destructive rounded-xl p-3"
								>
									<Trash2 class="w-4 h-4" />
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

{#if !showSettings}
	<div class="fixed bottom-6 right-6 z-50">
		<Button
			onclick={openSettings}
			variant="default"
			size="icon"
			class="w-12 h-12 rounded-xl bg-primary hover:bg-primary/90 text-primary-foreground shadow-lg hover:shadow-xl transition-all duration-300 hover:scale-105"
		>
			<Settings class="w-5 h-5" />
		</Button>
	</div>
{/if}