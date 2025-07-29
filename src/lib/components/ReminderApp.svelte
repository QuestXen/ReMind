<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import { Card, Header, Title, Content } from '$lib/components/ui/card/index';
	import { invoke } from '@tauri-apps/api/core';
	import { Plus, Bell, Trash2, Calendar as CalendarIcon, Clock, Edit, Settings } from '@lucide/svelte';
	import { onMount, onDestroy } from 'svelte';
	
	import SettingsComponent from './Settings.svelte';
	import TitleBar from './TitleBar.svelte';
	import ReminderDialog from './ReminderDialog.svelte';
	import { reminders, addReminder, updateReminder as updateReminderStore, deleteReminderFromStore } from '$lib/stores';
	import type { Reminder, ReminderInterval, ReminderColor } from '$lib/stores';

	let showSettings = $state(false);
	let showDialog = $state(false);
	let dialogMode = $state<'create' | 'edit'>('create');
	let editingReminder = $state<Reminder | null>(null);

	let reminderTimers = new Map<string, {
		timerId: NodeJS.Timeout;
		nextExecution: Date;
		isActive: boolean;
	}>();
	
	let timerCleanupInterval: NodeJS.Timeout | null = null;
	let timeUpdateInterval: NodeJS.Timeout | null = null;
	let timeUpdateTrigger = $state(0);

	const colorClasses = {
		blue: 'bg-blue-500 border-blue-200 text-blue-50',
		green: 'bg-green-500 border-green-200 text-green-50',
		purple: 'bg-purple-500 border-purple-200 text-purple-50',
		red: 'bg-red-500 border-red-200 text-red-50',
		orange: 'bg-orange-500 border-orange-200 text-orange-50',
		pink: 'bg-pink-500 border-pink-200 text-pink-50'
	};

	function getColorStyle(color: string): string {
		if (color in colorClasses) {
			return colorClasses[color as keyof typeof colorClasses];
		}
		return '';
	}

	function getCustomColorStyle(color: string): string {
		if (color.startsWith('#')) {
			return `background-color: ${color}; border-color: ${color}40; color: white;`;
		}
		return '';
	}

	const intervalLabels = {
		minutes: 'Minuten',
		hours: 'Stunden',
		days: 'Tage',
		weeks: 'Wochen',
		months: 'Monate',
		specific: 'Bestimmtes Datum'
	};


	async function handleDialogSave(reminderData: Partial<Reminder>) {
		if (dialogMode === 'create') {
			const reminder: Reminder = {
				id: crypto.randomUUID(),
				title: reminderData.title!,
				message: reminderData.message || '',
				color: reminderData.color!,
				mode: reminderData.mode || 'interval',
				// Interval mode properties
				interval: reminderData.interval,
				intervalValue: reminderData.intervalValue,
				// Scheduled mode properties (uses interval and intervalValue)
				specificDate: reminderData.specificDate,
				specificTime: reminderData.specificTime,
				beginAt: reminderData.beginAt,
				beginAtTime: reminderData.beginAtTime,
				// Appointment mode properties
				appointmentDate: reminderData.appointmentDate,
				appointmentTime: reminderData.appointmentTime,
				recurrence: reminderData.recurrence,
				weekdays: reminderData.weekdays,
				recurrenceEndDate: reminderData.recurrenceEndDate,
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString()
			};
			
			try {
				await invoke('add_reminder', { reminder });
				addReminder(reminder);
				startReminderTimer(reminder);
			} catch (error) {
				console.error('Failed to create reminder:', error);
			}
		} else if (dialogMode === 'edit' && editingReminder) {
			const updatedReminder: Reminder = {
				...editingReminder,
				...reminderData
			} as Reminder;
			
			clearReminderTimer(updatedReminder.id);
			
			try {
				await invoke('update_reminder', { reminder: updatedReminder });
				updateReminderStore(updatedReminder);
				startReminderTimer(updatedReminder);
				console.log(`Updated reminder: ${updatedReminder.title || updatedReminder.name}`);
			} catch (error) {
				console.error('Failed to update reminder:', error);
				if (editingReminder) {
					startReminderTimer(editingReminder);
				}
			}
		}
		
		// Dialog schließen und State zurücksetzen
		showDialog = false;
		editingReminder = null;
	}



	function startEditReminder(reminder: Reminder) {
		editingReminder = { ...reminder };
		dialogMode = 'edit';
		showDialog = true;
	}



	function openSettings() {
		showSettings = true;
	}

	function closeSettings() {
		showSettings = false;
	}

	async function deleteReminder(id: string) {
		clearReminderTimer(id);
		
		const reminderElement = document.getElementById(`reminder-${id}`);
		if (reminderElement) {
			// Erste Phase: Inhalte ausblenden
			const contentElements = reminderElement.querySelectorAll('.reminder-content, .reminder-text, .reminder-info');
			contentElements.forEach(element => {
				element.classList.add('animate-content-fade-out');
			});
			
			// Zweite Phase: Karte animieren
			setTimeout(() => {
				reminderElement.classList.add('animate-delete-smooth');
				
				// Nachfolgende Elemente nach oben bewegen
				const allReminders = document.querySelectorAll('[id^="reminder-"]');
				let foundTarget = false;
				allReminders.forEach((element) => {
					if (foundTarget && element !== reminderElement) {
						element.classList.add('animate-smooth-move-up');
						setTimeout(() => {
							element.classList.remove('animate-smooth-move-up');
						}, 600);
					}
					if (element === reminderElement) {
						foundTarget = true;
					}
				});
			}, 200);
			
			// Dritte Phase: Element aus DOM entfernen
			setTimeout(async () => {
				try {
					await invoke('delete_reminder', { reminderId: id });
					deleteReminderFromStore(id);
					console.log(`Deleted reminder with ID: ${id}`);
				} catch (error) {
					console.error('Failed to delete reminder:', error);
					// Fehlerbehandlung: Animation rückgängig machen
					if (reminderElement) {
						reminderElement.classList.remove('animate-delete-smooth');
						contentElements.forEach(element => {
							element.classList.remove('animate-content-fade-out');
						});
					}
				}
			}, 800); 
		} else {
			try {
				await invoke('delete_reminder', { reminderId: id });
				deleteReminderFromStore(id);
				console.log(`Deleted reminder with ID: ${id}`);
			} catch (error) {
				console.error('Failed to delete reminder:', error);
			}
		}
	}

	async function toggleReminderActive(reminder: Reminder) {
		const updatedReminder = { ...reminder, active: !reminder.active };
		
		try {
			await invoke('update_reminder', { reminder: updatedReminder });
			updateReminderStore(updatedReminder);
			
			if (updatedReminder.active) {
				startReminderTimer(updatedReminder);
				console.log(`Activated reminder: ${updatedReminder.name}`);
			} else {
				clearReminderTimer(updatedReminder.id);
				console.log(`Deactivated reminder: ${updatedReminder.name}`);
			}
		} catch (error) {
			console.error('Failed to toggle reminder active status:', error);
		}
	}



	function startReminderTimer(reminder: Reminder) {
		if (!reminder.active) {
			console.log(`Skipping timer for inactive reminder: ${reminder.name}`);
			return;
		}
		
		clearReminderTimer(reminder.id);
		
		const now = new Date();
		let nextExecution: Date | null = null;
		
		// Handle different reminder modes
		switch (reminder.mode) {
			case 'interval':
				nextExecution = calculateIntervalExecution(reminder, now);
				break;
			case 'scheduled':
				nextExecution = calculateScheduledExecution(reminder, now);
				break;
			case 'appointment':
				nextExecution = calculateAppointmentExecution(reminder, now);
				break;
			default:
				// Fallback for old reminders without mode
				nextExecution = calculateLegacyExecution(reminder, now);
				break;
		}
		
		if (!nextExecution) {
			console.warn(`Could not calculate next execution for reminder ${reminder.id}`);
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

	// Helper functions for calculating next execution times
	function calculateIntervalExecution(reminder: Reminder, now: Date): Date | null {
		if (!reminder.interval || !reminder.intervalValue) {
			console.warn(`Interval reminder ${reminder.id} missing interval or intervalValue`);
			return null;
		}

		// Check if BeginAt is set and we haven't reached it yet
		if (reminder.beginAt) {
			const beginAtDate = new Date(reminder.beginAt);
			if (reminder.beginAtTime) {
				const [hours, minutes] = reminder.beginAtTime.split(':').map(Number);
				beginAtDate.setHours(hours, minutes, 0, 0);
			}
			
			if (now < beginAtDate) {
				// Schedule for BeginAt time
				return beginAtDate;
			}
		}

		switch (reminder.interval) {
			case 'minutes':
				return new Date(now.getTime() + reminder.intervalValue * 60 * 1000);
			case 'hours':
				return new Date(now.getTime() + reminder.intervalValue * 60 * 60 * 1000);
			case 'days':
				return new Date(now.getTime() + reminder.intervalValue * 24 * 60 * 60 * 1000);
			case 'weeks':
				return new Date(now.getTime() + reminder.intervalValue * 7 * 24 * 60 * 60 * 1000);
			case 'months':
				const nextMonth = new Date(now);
				nextMonth.setMonth(nextMonth.getMonth() + reminder.intervalValue);
				return nextMonth;
			default:
				console.error(`Unknown interval type: ${reminder.interval}`);
				return null;
		}
	}

	function calculateScheduledExecution(reminder: Reminder, now: Date): Date | null {
		if (!reminder.interval || !reminder.intervalValue) {
			console.warn(`Scheduled reminder ${reminder.id} missing interval or intervalValue`);
			return null;
		}

		// Handle BeginAt logic - scheduled mode always requires beginAt
		if (!reminder.beginAt || !reminder.beginAtTime) {
			console.warn(`Scheduled reminder ${reminder.id} missing beginAt or beginAtTime`);
			return null;
		}

		const beginAtDate = new Date(reminder.beginAt);
		const [hours, minutes] = reminder.beginAtTime.split(':').map(Number);
		beginAtDate.setHours(hours, minutes, 0, 0);
		
		// If we haven't reached the begin time yet, return it
		if (now < beginAtDate) {
			return beginAtDate;
		}

		// Calculate next execution based on interval from begin time
		switch (reminder.interval) {
			case 'minutes':
				const minutesMs = reminder.intervalValue * 60 * 1000;
				const timeSinceBeginMinutes = now.getTime() - beginAtDate.getTime();
				const intervalsPassed = Math.floor(timeSinceBeginMinutes / minutesMs);
				return new Date(beginAtDate.getTime() + (intervalsPassed + 1) * minutesMs);
			case 'hours':
				const hoursMs = reminder.intervalValue * 60 * 60 * 1000;
				const timeSinceBeginHours = now.getTime() - beginAtDate.getTime();
				const hoursIntervalsPassed = Math.floor(timeSinceBeginHours / hoursMs);
				return new Date(beginAtDate.getTime() + (hoursIntervalsPassed + 1) * hoursMs);
			case 'days':
				const daysMs = reminder.intervalValue * 24 * 60 * 60 * 1000;
				const timeSinceBeginDays = now.getTime() - beginAtDate.getTime();
				const daysIntervalsPassed = Math.floor(timeSinceBeginDays / daysMs);
				return new Date(beginAtDate.getTime() + (daysIntervalsPassed + 1) * daysMs);
			case 'weeks':
				const weeksMs = reminder.intervalValue * 7 * 24 * 60 * 60 * 1000;
				const timeSinceBeginWeeks = now.getTime() - beginAtDate.getTime();
				const weeksIntervalsPassed = Math.floor(timeSinceBeginWeeks / weeksMs);
				return new Date(beginAtDate.getTime() + (weeksIntervalsPassed + 1) * weeksMs);
			case 'months':
				const nextMonth = new Date(beginAtDate);
				const monthsSinceBegin = (now.getFullYear() - beginAtDate.getFullYear()) * 12 + (now.getMonth() - beginAtDate.getMonth());
				const monthsToAdd = Math.ceil((monthsSinceBegin + 1) / reminder.intervalValue) * reminder.intervalValue;
				nextMonth.setMonth(beginAtDate.getMonth() + monthsToAdd);
				return nextMonth;
			default:
				console.error(`Unknown interval type: ${reminder.interval}`);
				return null;
		}
	}

	function calculateAppointmentExecution(reminder: Reminder, now: Date): Date | null {
		if (!reminder.appointmentDate || !reminder.appointmentTime) {
			console.warn(`Appointment reminder ${reminder.id} missing date or time`);
			return null;
		}

		const appointmentDate = new Date(reminder.appointmentDate);
		const [hours, minutes] = reminder.appointmentTime.split(':').map(Number);
		appointmentDate.setHours(hours, minutes, 0, 0);

		const recurrence = reminder.recurrence || 'once';

		if (recurrence === 'once') {
			if (appointmentDate <= now) {
				console.warn(`One-time appointment ${reminder.id} scheduled for past date: ${appointmentDate}`);
				return null;
			}
			return appointmentDate;
		}

		// Handle recurring appointments
		return calculateNextRecurringAppointment(reminder, now, appointmentDate);
	}

	function calculateNextRecurringAppointment(reminder: Reminder, now: Date, baseDate: Date): Date | null {
		const recurrence = reminder.recurrence;
		const [hours, minutes] = reminder.appointmentTime!.split(':').map(Number);
		
		let nextDate = new Date(now);
		nextDate.setHours(hours, minutes, 0, 0);

		// If today's time has passed, start from tomorrow
		if (nextDate <= now) {
			nextDate.setDate(nextDate.getDate() + 1);
		}

		switch (recurrence) {
			case 'daily':
				return nextDate;
			case 'weekly':
				const targetDay = baseDate.getDay();
				while (nextDate.getDay() !== targetDay) {
					nextDate.setDate(nextDate.getDate() + 1);
				}
				return nextDate;
			case 'monthly':
				const targetDate = baseDate.getDate();
				nextDate.setDate(targetDate);
				if (nextDate <= now) {
					nextDate.setMonth(nextDate.getMonth() + 1);
					nextDate.setDate(targetDate);
				}
				return nextDate;
			case 'custom':
				if (!reminder.weekdays || reminder.weekdays.length === 0) {
					console.warn(`Custom recurrence reminder ${reminder.id} has no weekdays set`);
					return null;
				}
				
				// Find next occurrence on selected weekdays
				for (let i = 0; i < 7; i++) {
					if (reminder.weekdays.includes(nextDate.getDay())) {
						return nextDate;
					}
					nextDate.setDate(nextDate.getDate() + 1);
				}
				return null;
			default:
				console.error(`Unknown recurrence type: ${recurrence}`);
				return null;
		}
	}

	function calculateLegacyExecution(reminder: Reminder, now: Date): Date | null {
		// Fallback for old reminders without mode property
		if (!reminder.interval) {
			return null;
		}

		switch (reminder.interval) {
			case 'minutes':
				return new Date(now.getTime() + (reminder.intervalValue || 1) * 60 * 1000);
			case 'hours':
				return new Date(now.getTime() + (reminder.intervalValue || 1) * 60 * 60 * 1000);
			case 'days':
				return new Date(now.getTime() + (reminder.intervalValue || 1) * 24 * 60 * 60 * 1000);
			case 'weeks':
				return new Date(now.getTime() + (reminder.intervalValue || 1) * 7 * 24 * 60 * 60 * 1000);
			case 'months':
				const nextMonth = new Date(now);
				nextMonth.setMonth(nextMonth.getMonth() + (reminder.intervalValue || 1));
				return nextMonth;
			case 'specific':
				if (!reminder.specificDate) {
					return null;
				}
				const specificDate = new Date(reminder.specificDate);
				if (specificDate <= now) {
					return null;
				}
				return specificDate;
			default:
				return null;
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
			// Verwende die neue Funktion, die automatisch die Einstellungen berücksichtigt
			await invoke('send_notification_with_settings', {
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
			}
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
		const mode = reminder.mode || 'interval';
		
		switch (mode) {
			case 'interval':
				if (reminder.beginAt) {
					const beginDate = new Date(reminder.beginAt);
					const dateStr = beginDate.toLocaleDateString('de-DE');
					const timeStr = reminder.beginAtTime || beginDate.toLocaleTimeString('de-DE', { hour: '2-digit', minute: '2-digit' });
					return `Alle ${reminder.intervalValue} ${intervalLabels[reminder.interval!]} (ab ${dateStr} ${timeStr})`;
				}
				return `Alle ${reminder.intervalValue} ${intervalLabels[reminder.interval!]}`;
				
			case 'scheduled':
				if (reminder.interval && reminder.intervalValue && reminder.beginAt && reminder.beginAtTime) {
					const beginDate = new Date(reminder.beginAt);
					const dateStr = beginDate.toLocaleDateString('de-DE');
					const timeStr = reminder.beginAtTime;
					return `Alle ${reminder.intervalValue} ${intervalLabels[reminder.interval]} ab ${dateStr} um ${timeStr}`;
				}
				return 'Geplante Erinnerung';
				
			case 'appointment':
				if (reminder.appointmentDate && reminder.appointmentTime) {
					const date = new Date(reminder.appointmentDate);
					const dateStr = date.toLocaleDateString('de-DE');
					const timeStr = reminder.appointmentTime;
					
					const recurrence = reminder.recurrence || 'once';
					switch (recurrence) {
						case 'once':
							return `Termin am ${dateStr} um ${timeStr}`;
						case 'daily':
							return `Täglich um ${timeStr}`;
						case 'weekly':
							const dayName = date.toLocaleDateString('de-DE', { weekday: 'long' });
							return `Jeden ${dayName} um ${timeStr}`;
						case 'monthly':
							const day = date.getDate();
							return `Jeden ${day}. des Monats um ${timeStr}`;
						case 'custom':
							if (reminder.weekdays && reminder.weekdays.length > 0) {
								const dayNames = ['Sonntag', 'Montag', 'Dienstag', 'Mittwoch', 'Donnerstag', 'Freitag', 'Samstag'];
								const selectedDays = reminder.weekdays.map(day => dayNames[day]).join(', ');
								return `Jeden ${selectedDays} um ${timeStr}`;
							}
							return `Benutzerdefiniert um ${timeStr}`;
						default:
							return `Termin am ${dateStr} um ${timeStr}`;
					}
				}
				return 'Termin-Erinnerung';
				
			default:
				// Fallback für alte Erinnerungen
				if (reminder.interval === 'specific' && reminder.specificDate) {
					const date = new Date(reminder.specificDate);
					const dateStr = date.toLocaleDateString('de-DE');
					const timeStr = date.toLocaleTimeString('de-DE', { hour: '2-digit', minute: '2-digit' });
					return `Am ${dateStr} um ${timeStr}`;
				}
				return `Alle ${reminder.intervalValue} ${intervalLabels[reminder.interval!]}`;
		}
	}

	function getTimeUntilNextReminder(reminder: Reminder): string {
			timeUpdateTrigger;
			
			if (!reminder.active) {
				return '';
			}
			
			const timerInfo = reminderTimers.get(reminder.id);
			if (!timerInfo) {
				return '';
			}

			const now = new Date();
			const timeUntil = timerInfo.nextExecution.getTime() - now.getTime();

			if (timeUntil <= 0) {
				return '';
			}

			const days = Math.floor(timeUntil / (1000 * 60 * 60 * 24));
			const hours = Math.floor((timeUntil % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
			const minutes = Math.floor((timeUntil % (1000 * 60 * 60)) / (1000 * 60));
			const seconds = Math.floor((timeUntil % (1000 * 60)) / 1000);

			if (days > 0) {
				if (hours > 0) {
					return `in ${days} Tag${days > 1 ? 'en' : ''} und ${hours} Stunde${hours > 1 ? 'n' : ''}`;
				} else {
					return `in ${days} Tag${days > 1 ? 'en' : ''}`;
				}
			} else if (hours > 0) {
				if (minutes > 0) {
					return `in ${hours} Stunde${hours > 1 ? 'n' : ''} und ${minutes} Minute${minutes > 1 ? 'n' : ''}`;
				} else {
					return `in ${hours} Stunde${hours > 1 ? 'n' : ''}`;
				}
			} else if (minutes > 0) {
				return `in ${minutes} Minute${minutes > 1 ? 'n' : ''}`;
			} else if (seconds > 0) {
				return `in ${seconds} Sekunde${seconds > 1 ? 'n' : ''}`;
			} else {
				return 'in wenigen Sekunden';
			}
		}

onMount(() => {
		let isInitialLoad = true;
		
		const unsubscribe = reminders.subscribe(currentReminders => {
			if (isInitialLoad) {
				clearAllTimers();
				
				currentReminders.forEach(reminder => {
					startReminderTimer(reminder);
				});
				
				console.log(`Initialized ${currentReminders.length} reminders with timers`);
				isInitialLoad = false;
			}
		});
		
		timerCleanupInterval = setInterval(() => {
			const now = new Date();
			reminderTimers.forEach((timerInfo, id) => {
				if (timerInfo.nextExecution <= now && !timerInfo.isActive) {
					clearReminderTimer(id);
				}
			});
		}, 5 * 60 * 1000);
		
		timeUpdateInterval = setInterval(() => {
			timeUpdateTrigger++;
		}, 1000); 
		
		return () => {
			unsubscribe();
		};
	});

onDestroy(() => {
		clearAllTimers();
		if (timerCleanupInterval) {
			clearInterval(timerCleanupInterval);
		}
		if (timeUpdateInterval) {
			clearInterval(timeUpdateInterval);
		}
		console.log('ReminderApp component destroyed, all timers cleared');
	});

	// Scroll-basiertes Button-Scaling
	onMount(() => {
		const contentScroll = document.getElementById('content-scroll');
		const newReminderBtn = document.getElementById('new-reminder-btn');
		
		if (contentScroll && newReminderBtn) {
				let isHovered = false;
				
				// Basis-Styling setzen
				newReminderBtn.style.transition = 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)';
				newReminderBtn.style.transformOrigin = 'center';
				
				// Hover-Events
				newReminderBtn.addEventListener('mouseenter', () => {
					isHovered = true;
					newReminderBtn.style.transform = 'scale(1.05)';
					newReminderBtn.style.opacity = '1';
					newReminderBtn.style.boxShadow = '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)';
				});
				
				newReminderBtn.addEventListener('mouseleave', () => {
					isHovered = false;
					// Scroll-Position prüfen und entsprechend skalieren
					const scrollTop = contentScroll.scrollTop;
					if (scrollTop > 50) {
						newReminderBtn.style.transform = 'scale(0.7)';
						newReminderBtn.style.opacity = '0.8';
						newReminderBtn.style.boxShadow = '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)';
					} else {
						newReminderBtn.style.transform = 'scale(1)';
						newReminderBtn.style.opacity = '1';
						newReminderBtn.style.boxShadow = '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)';
					}
				});
				
				// Scroll-Event
				contentScroll.addEventListener('scroll', () => {
					const scrollTop = contentScroll.scrollTop;
					
					if (!isHovered) {
						if (scrollTop > 50) {
							newReminderBtn.style.transform = 'scale(0.7)';
							newReminderBtn.style.opacity = '0.8';
							newReminderBtn.style.boxShadow = '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)';
						} else {
							newReminderBtn.style.transform = 'scale(1)';
							newReminderBtn.style.opacity = '1';
							newReminderBtn.style.boxShadow = '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)';
						}
					}
				});
			}
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
		0% {
			opacity: 0;
			transform: translateY(40px) scale(0.95);
		}
		50% {
			opacity: 0.7;
			transform: translateY(15px) scale(0.98);
		}
		100% {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	@keyframes slide-out {
		0% {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
		30% {
			opacity: 0.8;
			transform: translateY(-5px) scale(0.98);
		}
		100% {
			opacity: 0;
			transform: translateY(-30px) scale(0.9);
		}
	}



	@keyframes content-fade-out {
		0% {
			opacity: 1;
			transform: translateY(0);
			filter: blur(0px);
		}
		100% {
			opacity: 0;
			transform: translateY(-10px);
			filter: blur(1px);
		}
	}

	@keyframes delete-smooth {
		0% {
			opacity: 1;
			transform: scale(1) translateX(0);
			max-height: 500px;
			max-width: 100%;
			margin-bottom: 1.5rem;
			padding: 2rem;
			border-width: 1px;
			box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
			filter: blur(0px);
			transform-origin: center top;
		}
		20% {
			opacity: 0.8;
			transform: scale(0.98) translateX(-5px);
			max-height: 400px;
			max-width: 98%;
			margin-bottom: 1.4rem;
			padding: 1.8rem;
			border-width: 0.9px;
			box-shadow: 0 5px 8px -2px rgba(0, 0, 0, 0.09), 0 3px 4px -1px rgba(0, 0, 0, 0.045);
			filter: blur(0.3px);
		}
		40% {
			opacity: 0.5;
			transform: scale(0.95) translateX(-15px);
			max-height: 250px;
			max-width: 95%;
			margin-bottom: 1.2rem;
			padding: 1.5rem;
			border-width: 0.6px;
			box-shadow: 0 2px 3px -1px rgba(0, 0, 0, 0.06), 0 1px 2px -1px rgba(0, 0, 0, 0.03);
			filter: blur(0.7px);
		}
		60% {
			opacity: 0.2;
			transform: scale(0.9) translateX(-30px);
			max-height: 100px;
			max-width: 90%;
			margin-bottom: 0.8rem;
			padding: 1rem;
			border-width: 0.2px;
			box-shadow: 0 1px 1px 0 rgba(0, 0, 0, 0.03);
			filter: blur(1.2px);
		}
		80% {
			opacity: 0.05;
			transform: scale(0.8) translateX(-45px);
			max-height: 20px;
			max-width: 80%;
			margin-bottom: 0.3rem;
			padding: 0.4rem;
			border-width: 0px;
			box-shadow: none;
			filter: blur(1.8px);
		}
		100% {
			opacity: 0;
			transform: scale(0) translateX(-50px);
			max-height: 0;
			max-width: 0;
			margin-bottom: 0;
			padding: 0;
			border-width: 0px;
			box-shadow: none;
			filter: blur(3px);
			overflow: hidden;
		}
	}

	@keyframes smooth-move-up {
		0% {
			transform: translateY(0);
			opacity: 1;
		}
		30% {
			transform: translateY(-8px);
			opacity: 0.95;
		}
		70% {
			transform: translateY(-20px);
			opacity: 0.98;
		}
		100% {
			transform: translateY(-24px);
			opacity: 1;
		}
	}



	.form-container {
		transition: all 0.6s cubic-bezier(0.16, 1, 0.3, 1);
		transform-origin: center;
		max-height: 1000px;
		opacity: 1;
		overflow: hidden;
		animation: form-slide-in 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards;
	}

	.form-container.closing {
		max-height: 0;
		opacity: 0;
		transform: translateY(-20px) scale(0.95);
		margin-bottom: 0;
		animation: none;
	}

	@keyframes form-slide-in {
		0% {
			opacity: 0;
			transform: translateY(30px) scale(0.95);
			max-height: 0;
		}
		30% {
			opacity: 0.3;
			transform: translateY(15px) scale(0.98);
			max-height: 300px;
		}
		100% {
			opacity: 1;
			transform: translateY(0) scale(1);
			max-height: 1000px;
		}
	}

	.animate-slide-up {
		animation: slide-up 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards;
		transform-origin: center bottom;
	}

	.animate-slide-out {
		animation: slide-out 0.5s cubic-bezier(0.4, 0, 0.6, 1) forwards;
		transform-origin: center top;
	}

	.animate-content-fade-out {
		animation: content-fade-out 0.2s cubic-bezier(0.4, 0, 0.6, 1) forwards;
	}

	.animate-delete-smooth {
		animation: delete-smooth 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards;
	}

	.animate-smooth-move-up {
		animation: smooth-move-up 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards;
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

	/* Calendar dropdown styling fixes - comprehensive selectors */
	:global([data-bits-calendar-month-select] select),
	:global([data-bits-calendar-year-select] select),
	:global(.calendar select),
	:global([data-calendar-month-select] select),
	:global([data-calendar-year-select] select),
	:global([data-bits-calendar] select),
	:global([data-calendar] select) {
		background-color: hsl(var(--background)) !important;
		color: hsl(var(--foreground)) !important;
		border: 1px solid hsl(var(--border)) !important;
		border-radius: 0.5rem !important;
		padding: 0.5rem 0.75rem !important;
		font-size: 0.875rem !important;
		min-height: 2.5rem !important;
		transition: all 0.2s ease-in-out !important;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05) !important;
		appearance: none !important;
		-webkit-appearance: none !important;
		-moz-appearance: none !important;
	}

	:global([data-bits-calendar-month-select] select:hover),
	:global([data-bits-calendar-year-select] select:hover),
	:global(.calendar select:hover),
	:global([data-calendar-month-select] select:hover),
	:global([data-calendar-year-select] select:hover),
	:global([data-bits-calendar] select:hover),
	:global([data-calendar] select:hover) {
		border-color: hsl(var(--ring) / 0.5) !important;
		box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1) !important;
	}

	:global([data-bits-calendar-month-select] select:focus),
	:global([data-bits-calendar-year-select] select:focus),
	:global(.calendar select:focus),
	:global([data-calendar-month-select] select:focus),
	:global([data-calendar-year-select] select:focus),
	:global([data-bits-calendar] select:focus),
	:global([data-calendar] select:focus) {
		border-color: hsl(var(--ring)) !important;
		box-shadow: 0 0 0 3px hsl(var(--ring) / 0.2) !important;
		outline: none !important;
	}

	:global([data-bits-calendar-month-select] option),
	:global([data-bits-calendar-year-select] option),
	:global(.calendar option),
	:global([data-calendar-month-select] option),
	:global([data-calendar-year-select] option),
	:global([data-bits-calendar] option),
	:global([data-calendar] option) {
		background-color: hsl(var(--background)) !important;
		color: hsl(var(--foreground)) !important;
		padding: 0.5rem !important;
	}

	:global([data-bits-calendar-month-select] option:hover),
	:global([data-bits-calendar-year-select] option:hover),
	:global(.calendar option:hover),
	:global([data-calendar-month-select] option:hover),
	:global([data-calendar-year-select] option:hover),
	:global([data-bits-calendar] option:hover),
	:global([data-calendar] option:hover) {
		background-color: hsl(var(--accent)) !important;
		color: hsl(var(--accent-foreground)) !important;
	}


	:global([data-bits-calendar-root] select),
	:global([data-calendar-root] select) {
		background-color: hsl(var(--background)) !important;
		color: hsl(var(--foreground)) !important;
		border: 1px solid hsl(var(--border)) !important;
		border-radius: 0.5rem !important;
		padding: 0.5rem 0.75rem !important;
		font-size: 0.875rem !important;
		min-height: 2.5rem !important;
		transition: all 0.2s ease-in-out !important;
		box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05) !important;
		appearance: none !important;
		-webkit-appearance: none !important;
		-moz-appearance: none !important;
	}

	:global([data-bits-calendar-root]) {
		border-radius: 0.75rem !important;
		border: 1px solid hsl(var(--border)) !important;
		background-color: hsl(var(--card)) !important;
		box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1) !important;
	}

	:global([data-bits-calendar-header]) {
		padding: 1rem !important;
		border-bottom: 1px solid hsl(var(--border)) !important;
	}

	:global([data-bits-calendar-caption]) {
		display: flex !important;
		align-items: center !important;
		justify-content: space-between !important;
		gap: 0.5rem !important;
	}




</style>

{#if showSettings}
	<SettingsComponent on:close={closeSettings} />
{:else}
<div class="flex flex-col h-screen overflow-hidden">
	<TitleBar title="Re:Mind" icon="R" useLogoIcon={true} />

  <div id="content-scroll" class="flex-1 overflow-y-auto bg-background relative {showSettings ? 'animate-slide-out-left' : 'animate-slide-in-left'}">
	<div class="p-6">
		<div class="max-w-4xl mx-auto">
<!-- Fixed New Reminder Button -->
		<div class="fixed top-20 left-1/2 transform -translate-x-1/2 z-40">
			<Button
				id="new-reminder-btn"
				class="bg-primary hover:bg-primary/90 text-primary-foreground px-8 py-4 rounded-2xl shadow-lg border-0 text-subheading"
				style="transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); transform-origin: center;"
				onclick={() => {
					dialogMode = 'create';
					showDialog = true;
					editingReminder = null;
				}}
			>
				<Plus class="w-5 h-5 mr-3" />
				Neue Erinnerung
			</Button>
		</div>

		<!-- Spacer for fixed button -->
		<div class="mb-8 h-16"></div>

		<!-- Reminder Dialog -->
		<ReminderDialog
			bind:open={showDialog}
			mode={dialogMode}
			initialData={editingReminder}
			onsave={handleDialogSave}
			onclose={() => {
				showDialog = false;
				editingReminder = null;
			}}
		/>

	<div class="grid gap-6 transition-all duration-800" style="transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94);">
		{#if $reminders.length === 0}
			<Card class="bg-card border border-border shadow-lg rounded-3xl overflow-hidden animate-slide-up-staggered">
				<Content class="text-center py-16">
					<h3 class="text-2xl text-heading text-card-foreground mb-3">Noch keine Erinnerungen</h3>
					<p class="text-muted-foreground text-lg text-caption">Erstelle deine erste Erinnerung</p>
				</Content>
			</Card>
		{:else}
			{#each $reminders as reminder, index (reminder.id)}
			<Card id={`reminder-${reminder.id}`} class="bg-card border border-border shadow-lg hover:shadow-xl rounded-3xl overflow-hidden transition-all duration-800 group hover:scale-[1.02] animate-slide-up-staggered" style="animation-delay: {index * 0.1}s; will-change: transform, height; transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94);">
				<Content class="reminder-content p-8 transition-all duration-800" style="transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94);">
					<div class="flex items-center justify-between gap-4">
						<div class="flex items-center gap-6 flex-1 min-w-0">
							<div 
								class="w-6 h-6 rounded-full flex-shrink-0 shadow-md transition-all duration-300 group-hover:scale-110 {getColorStyle(reminder.color)}"
								style="{getCustomColorStyle(reminder.color)}"
							></div>
							<div class="reminder-text flex-1 min-w-0">
								<h3 class="text-xl text-subheading text-card-foreground mb-3 truncate">{reminder.title || reminder.name || 'Unbenannte Erinnerung'}</h3>
								<div class="reminder-info space-y-2">
									<div class="flex items-center gap-2 text-sm text-body">
										{#if reminder.interval === 'specific'}
											<CalendarIcon class="w-4 h-4 flex-shrink-0" />
										{:else}
											<Clock class="w-4 h-4 flex-shrink-0" />
										{/if}
										<span class="truncate">{formatReminderInfo(reminder)}</span>
									</div>
									
									<div class="overflow-hidden transition-all duration-800" style="max-height: {getTimeUntilNextReminder(reminder) && reminder.active ? '80px' : '0px'}; padding-top: {getTimeUntilNextReminder(reminder) && reminder.active ? '8px' : '0px'}; margin-top: {getTimeUntilNextReminder(reminder) && reminder.active ? '8px' : '0px'}; transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94);">
										<div class="flex items-start gap-2 text-sm text-muted-foreground pt-2 transition-opacity duration-800" style="opacity: {getTimeUntilNextReminder(reminder) && reminder.active ? '1' : '0'}; transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94); transition-delay: {getTimeUntilNextReminder(reminder) && reminder.active ? '0ms' : '200ms'};">
											<div class="w-2 h-2 bg-emerald-500 rounded-full flex-shrink-0 mt-2 animate-pulse"></div>
											<span class="leading-relaxed break-words">
												<span class="font-medium text-emerald-600">Nächste Erinnerung</span><br>
												<span class="text-caption">{getTimeUntilNextReminder(reminder) || ''}</span>
											</span>
										</div>
									</div>
									
									{#if reminder.lastNotified}
										<div class="flex items-center gap-2 text-sm text-muted-foreground">
											<div class="w-2 h-2 bg-gray-400 rounded-full flex-shrink-0"></div>
											<span class="text-caption truncate">Zuletzt: {new Date(reminder.lastNotified).toLocaleString('de-DE')}</span>
										</div>
									{/if}
								</div>
							</div>
						</div>
							<div class="flex items-center gap-3">
								<div class="opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-2">
									<Button 
										onclick={() => startEditReminder(reminder)}
										variant="outline"
										class="transition-all duration-300 border-2 border-border text-muted-foreground hover:bg-accent hover:border-primary rounded-xl p-3"
									>
										<Edit class="w-4 h-4" />
									</Button>
									<Button 
										onclick={() => deleteReminder(reminder.id)}
										variant="outline"
										class="transition-all duration-300 border-2 border-destructive/30 text-destructive hover:bg-destructive/10 hover:border-destructive rounded-xl p-3"
									>
										<Trash2 class="w-4 h-4" />
									</Button>
								</div>
								
								<div class="relative">
									{#if reminder.active}
										<div class="absolute -bottom-1 left-1/2 transform -translate-x-1/2 w-12 h-2 bg-emerald-400/30 rounded-full blur-sm"></div>
									{/if}
									<Button 
										onclick={() => toggleReminderActive(reminder)}
										variant="outline"
										class="relative transition-all duration-500 border-2 {reminder.active ? 'border-emerald-400/50 bg-gradient-to-r from-emerald-50 to-green-50 text-emerald-600 hover:border-emerald-500' : 'border-slate-300/50 bg-gradient-to-r from-slate-50 to-gray-50 text-slate-400 hover:border-slate-400'} rounded-2xl p-3 group/toggle"
										title={reminder.active ? 'Aktiv - Klicken zum Deaktivieren' : 'Inaktiv - Klicken zum Aktivieren'}
									>
										<div class="relative w-8 h-4 transition-all duration-500">
					
											<div class="absolute inset-0 rounded-full transition-all duration-500 {reminder.active ? 'bg-gradient-to-r from-emerald-400 to-green-500 shadow-inner' : 'bg-gradient-to-r from-slate-300 to-gray-400 shadow-inner'}"></div>
										
											<div class="absolute top-0.5 w-3 h-3 bg-white rounded-full shadow-lg transition-all duration-500 transform {reminder.active ? 'translate-x-4 shadow-emerald-300/50' : 'translate-x-0.5 shadow-slate-400/50'} group-hover/toggle:scale-110">
										
												<div class="absolute inset-0.5 rounded-full transition-all duration-500 {reminder.active ? 'bg-gradient-to-br from-emerald-200 to-green-300' : 'bg-gradient-to-br from-slate-200 to-gray-300'}"></div>
											</div>
										</div>
									</Button>
								</div>
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