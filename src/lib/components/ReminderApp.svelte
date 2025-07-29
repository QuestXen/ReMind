<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import { Card, Header, Title, Content } from '$lib/components/ui/card/index';
	import { Input } from '$lib/components/ui/input/index';
	import * as Select from '$lib/components/ui/select/index';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index';
	import { Calendar } from '$lib/components/ui/calendar/index';
	import * as Popover from '$lib/components/ui/popover/index';
	import { getLocalTimeZone, today, parseDate } from '@internationalized/date';
	import { invoke } from '@tauri-apps/api/core';
	import { Plus, Bell, Trash2, Calendar as CalendarIcon, Clock, Edit, Settings, Palette } from '@lucide/svelte';
	import { onMount, onDestroy } from 'svelte';
	
	import ColorPicker from 'svelte-awesome-color-picker';
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
		specificTime: '12:00',
		color: 'blue' as ReminderColor
	});

	let showColorPicker = $state(false);
	let showEditColorPicker = $state(false);
	let customColor = $state('#7f5af0'); 
	let editCustomColor = $state('#7f5af0');

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

	const intervalOptions = [
		{ value: 'minutes', label: 'Minuten' },
		{ value: 'hours', label: 'Stunden' },
		{ value: 'days', label: 'Tage' },
		{ value: 'weeks', label: 'Wochen' },
		{ value: 'months', label: 'Monate' },
		{ value: 'specific', label: 'Bestimmtes Datum' }
	];

	const timeOptions = (() => {
		const options = [];
		for (let hour = 0; hour < 24; hour++) {
			for (let minute = 0; minute < 60; minute += 15) {
				const timeStr = `${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`;
				options.push({ value: timeStr, label: timeStr });
			}
		}
		return options;
	})();

	const intervalLabels = {
		minutes: 'Minuten',
		hours: 'Stunden',
		days: 'Tage',
		weeks: 'Wochen',
		months: 'Monate',
		specific: 'Bestimmtes Datum'
	};

	let newReminderCalendarValue = $state(today(getLocalTimeZone()));
	let editReminderCalendarValue = $state(today(getLocalTimeZone()));
	
	let newReminderPopoverOpen = $state(false);
	let editReminderPopoverOpen = $state(false);

	const newReminderTriggerContent = $derived(
		intervalOptions.find((option) => option.value === newReminder.interval)?.label ?? 'Intervall wählen'
	);

	const editReminderTriggerContent = $derived(
		editingReminder ? intervalOptions.find((option) => option.value === editingReminder!.interval)?.label ?? 'Intervall wählen' : 'Intervall wählen'
	);

	async function createReminder() {
		if (!newReminder.name.trim()) return;
		
		const reminder: Reminder = {
			id: crypto.randomUUID(),
			name: newReminder.name,
			interval: newReminder.interval,
			intervalValue: newReminder.intervalValue,
			specificDate: newReminder.interval === 'specific' ? newReminder.specificDate : undefined,
			specificTime: newReminder.interval === 'specific' ? newReminder.specificTime : undefined,
			color: newReminder.color,
			createdAt: new Date().toISOString(),
			active: true
		};
		
		try {
			await invoke('add_reminder', { reminder });
			addReminder(reminder);
			resetForm();
			showCreateForm = false;
			setTimeout(() => {
				const newReminderElement = document.getElementById(`reminder-${reminder.id}`);
				if (newReminderElement) {
					newReminderElement.scrollIntoView({ 
						behavior: 'smooth', 
						block: 'center',
						inline: 'nearest'
					});
				}
			}, 500);
			
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
			specificTime: '12:00',
			color: 'blue'
		};
		newReminderCalendarValue = today(getLocalTimeZone());
		newReminderPopoverOpen = false;
		showColorPicker = false;
		customColor = '#3b82f6';
	}
	$effect(() => {
		if (newReminderCalendarValue && newReminder.interval === 'specific') {
			const dateStr = newReminderCalendarValue.toString();
			newReminder.specificDate = `${dateStr}T${newReminder.specificTime}:00`;
		}
	});

	$effect(() => {
		if (editReminderCalendarValue && editingReminder && editingReminder.interval === 'specific') {
			const dateStr = editReminderCalendarValue.toString();
			const timeStr = editingReminder.specificTime || '12:00';
			editingReminder.specificDate = `${dateStr}T${timeStr}:00`;
		}
	});

	function startEditReminder(reminder: Reminder) {
		editingReminder = { ...reminder };
		
		if (reminder.interval === 'specific' && reminder.specificDate) {
			try {
				const [datePart, timePart] = reminder.specificDate.split('T');
				editReminderCalendarValue = parseDate(datePart);
				if (timePart && editingReminder) {
					editingReminder.specificTime = timePart.substring(0, 5);
				}
			} catch {
				editReminderCalendarValue = today(getLocalTimeZone());
				if (editingReminder) {
					editingReminder.specificTime = '12:00';
				}
			}
		} else {
			editReminderCalendarValue = today(getLocalTimeZone());
			if (editingReminder) {
				editingReminder.specificTime = '12:00';
			}
		}

		showEditForm = true;
		showCreateForm = false;
		editReminderPopoverOpen = false;
		
		setTimeout(() => {
			const formElement = document.getElementById('create-form');
			if (formElement) {
				const contentScroll = document.getElementById('content-scroll');
				if (contentScroll) {
					contentScroll.style.scrollBehavior = 'smooth';
					const elementTop = formElement.offsetTop;
					contentScroll.scrollTop = Math.max(0, elementTop - 20);
				}
			}
		}, 200);
	}

	async function updateReminder() {
		if (!editingReminder || !editingReminder.name.trim()) return;
		clearReminderTimer(editingReminder.id);
		
		try {
			await invoke('update_reminder', { reminder: editingReminder });
			
			updateReminderStore(editingReminder);
			startReminderTimer(editingReminder);
			
			const updatedReminderId = editingReminder.id;
			const updatedReminderName = editingReminder.name;
			showEditForm = false;
			editingReminder = null;
			setTimeout(() => {
				const updatedReminderElement = document.getElementById(`reminder-${updatedReminderId}`);
				if (updatedReminderElement) {
					const contentScroll = document.getElementById('content-scroll');
					if (contentScroll) {
						contentScroll.style.scrollBehavior = 'smooth';
						const elementTop = updatedReminderElement.offsetTop;
						const elementHeight = updatedReminderElement.offsetHeight;
						const containerHeight = contentScroll.clientHeight;
						const scrollTop = elementTop - (containerHeight / 2) + (elementHeight / 2);
						contentScroll.scrollTop = Math.max(0, scrollTop);
					}
				}
			}, 400);
			
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
		showEditColorPicker = false;
		editCustomColor = '#3b82f6';
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
			const animationStyles = ['animate-delete-scale-rotate', 'animate-delete-slide-out'];
			const randomAnimation = animationStyles[Math.floor(Math.random() * animationStyles.length)];
			
			reminderElement.classList.add(randomAnimation);
			const allReminders = document.querySelectorAll('[id^="reminder-"]');
			let foundTarget = false;
			allReminders.forEach((element) => {
				if (foundTarget && element !== reminderElement) {
					element.classList.add('animate-smooth-move-up');
				setTimeout(() => {
					element.classList.remove('animate-smooth-move-up');
				}, 800);
				}
				if (element === reminderElement) {
					foundTarget = true;
				}
			});
			
			setTimeout(async () => {
				try {
					await invoke('delete_reminder', { reminderId: id });
					deleteReminderFromStore(id);
					console.log(`Deleted reminder with ID: ${id}`);
				} catch (error) {
					console.error('Failed to delete reminder:', error);
					if (reminderElement) {
						reminderElement.classList.remove(randomAnimation);
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
		if (reminder.interval === 'specific' && reminder.specificDate) {
			const date = new Date(reminder.specificDate);
			const dateStr = date.toLocaleDateString('de-DE');
			const timeStr = date.toLocaleTimeString('de-DE', { hour: '2-digit', minute: '2-digit' });
			return `Am ${dateStr} um ${timeStr}`;
		}
	return `Alle ${reminder.intervalValue} ${intervalLabels[reminder.interval]}`;
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



	@keyframes delete-scale-rotate {
		0% {
			opacity: 1;
			transform: scale(1) rotate(0deg);
			height: auto;
			margin-bottom: 1.5rem;
		}
		25% {
			opacity: 0.8;
			transform: scale(1.08) rotate(3deg);
			height: auto;
			margin-bottom: 1.5rem;
		}
		50% {
			opacity: 0.5;
			transform: scale(0.92) rotate(-2deg);
			height: auto;
			margin-bottom: 1.2rem;
		}
		75% {
			opacity: 0.2;
			transform: scale(0.7) rotate(1deg);
			height: 50%;
			margin-bottom: 0.8rem;
		}
		100% {
			opacity: 0;
			transform: scale(0.4) rotate(-3deg);
			height: 0;
			margin-bottom: 0;
			padding-top: 0;
			padding-bottom: 0;
			overflow: hidden;
		}
	}

	@keyframes delete-slide-out {
		0% {
			opacity: 1;
			transform: translateX(0) scale(1) rotateY(0deg);
			height: auto;
			margin-bottom: 1.5rem;
			filter: blur(0px);
		}
		20% {
			opacity: 0.9;
			transform: translateX(-5px) scale(1.02) rotateY(-2deg);
			height: auto;
			margin-bottom: 1.5rem;
			filter: blur(0px);
		}
		40% {
			opacity: 0.6;
			transform: translateX(-20px) scale(0.95) rotateY(-5deg);
			height: auto;
			margin-bottom: 1.2rem;
			filter: blur(1px);
		}
		70% {
			opacity: 0.3;
			transform: translateX(-60px) scale(0.8) rotateY(-10deg);
			height: 60%;
			margin-bottom: 0.8rem;
			filter: blur(2px);
		}
		100% {
			opacity: 0;
			transform: translateX(-120px) scale(0.6) rotateY(-15deg);
			height: 0;
			margin-bottom: 0;
			padding-top: 0;
			padding-bottom: 0;
			filter: blur(3px);
			overflow: hidden;
		}
	}

	@keyframes smooth-move-up {
		0% {
			transform: translateY(0);
			opacity: 1;
		}
		20% {
			transform: translateY(-5px);
			opacity: 0.95;
		}
		60% {
			transform: translateY(-15px);
			opacity: 0.9;
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

	.animate-delete-scale-rotate {
		animation: delete-scale-rotate 0.8s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards;
	}

	.animate-delete-slide-out {
		animation: delete-slide-out 0.7s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards;
	}

	.animate-smooth-move-up {
		animation: smooth-move-up 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards;
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
		<div class="flex justify-center mb-12">
			<Button 
				onclick={() => {
			showCreateForm = true;
			setTimeout(() => {
				const createFormElement = document.getElementById('create-form');
				if (createFormElement) {
					const contentScroll = document.getElementById('content-scroll');
					if (contentScroll) {
						contentScroll.style.scrollBehavior = 'smooth';
						const elementTop = createFormElement.offsetTop;
						contentScroll.scrollTop = Math.max(0, elementTop - 20);
					}
				}
			}, 200);
		}}
				class="bg-primary hover:bg-primary/90 text-primary-foreground px-8 py-4 rounded-2xl shadow-lg transition-all duration-300 hover:scale-105 hover:shadow-xl border-0 text-subheading"
			>
				<Plus class="w-5 h-5 mr-3" />
				Neue Erinnerung
			</Button>
		</div>

	{#if showCreateForm || showEditForm}
		<div class="form-container">
			<Card 
				id="create-form" 
				class="mb-12 bg-card border border-border shadow-xl rounded-3xl overflow-hidden animate-slide-up"
			>
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
							<Select.Root type="single" name="editReminderInterval" bind:value={editingReminder.interval}>
								<Select.Trigger class="w-full h-12 px-4 bg-input border border-border rounded-xl text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 flex items-center">
									{editReminderTriggerContent}
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
						{:else}
							<Select.Root type="single" name="newReminderInterval" bind:value={newReminder.interval}>
								<Select.Trigger class="w-full h-12 px-4 bg-input border border-border rounded-xl text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 flex items-center">
									{newReminderTriggerContent}
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
									class="w-full h-10 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
								/>
							</div>
						{:else}
							<div class="transition-all duration-300 space-y-4">
								<div>
									<label for="reminder-specific-date" class="block text-sm text-subheading text-card-foreground mb-3">Datum wählen</label>
									<Popover.Root bind:open={editReminderPopoverOpen}>
									<Popover.Trigger class="w-full h-10 justify-start text-left font-normal border-border hover:bg-accent bg-input border border-border rounded-xl px-4 text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 flex items-center">
										<CalendarIcon class="mr-2 h-4 w-4" />
										{editReminderCalendarValue ? editReminderCalendarValue.toDate(getLocalTimeZone()).toLocaleDateString('de-DE') : 'Datum auswählen'}
									</Popover.Trigger>
										<Popover.Content class="w-auto p-0" align="start">
											<Calendar 
												type="single" 
												bind:value={editReminderCalendarValue}
												class="rounded-md border-0"
												captionLayout="dropdown"
												locale="de-DE"
												minValue={today(getLocalTimeZone())}
												onValueChange={() => editReminderPopoverOpen = false}
											/>
										</Popover.Content>
									</Popover.Root>
								</div>
								<div>
									<label for="reminder-specific-time" class="block text-sm text-subheading text-card-foreground mb-3">Uhrzeit</label>
									<DropdownMenu.Root>
										<DropdownMenu.Trigger class="w-full h-10 justify-start text-left font-normal border-border hover:bg-accent bg-input border border-border rounded-xl px-4 text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 flex items-center">
											<Clock class="mr-2 h-4 w-4" />
											{editingReminder?.specificTime || 'Uhrzeit wählen'}
										</DropdownMenu.Trigger>
										<DropdownMenu.Content class="w-48 max-h-60 overflow-y-auto">
											<DropdownMenu.Group>
												<DropdownMenu.Label>Uhrzeit auswählen</DropdownMenu.Label>
												{#each timeOptions as timeOption (timeOption.value)}
													<DropdownMenu.Item onclick={() => { if (editingReminder) editingReminder.specificTime = timeOption.value; }}>
														{timeOption.label}
													</DropdownMenu.Item>
												{/each}
											</DropdownMenu.Group>
										</DropdownMenu.Content>
									</DropdownMenu.Root>
								</div>
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
									class="w-full h-10 border-border focus:border-ring focus:ring-ring rounded-xl px-4 transition-all duration-300 bg-input focus:bg-background text-body text-foreground"
								/>
							</div>
						{:else}
						<div class="transition-all duration-300 space-y-4">
							<div>
								<label for="new-reminder-specific-date" class="block text-sm text-subheading text-card-foreground mb-3">Datum wählen</label>
								<Popover.Root bind:open={newReminderPopoverOpen}>
								<Popover.Trigger class="w-full h-10 justify-start text-left font-normal border-border hover:bg-accent bg-input border border-border rounded-xl px-4 text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 flex items-center">
									<CalendarIcon class="mr-2 h-4 w-4" />
									{newReminderCalendarValue ? newReminderCalendarValue.toDate(getLocalTimeZone()).toLocaleDateString('de-DE') : 'Datum auswählen'}
								</Popover.Trigger>
									<Popover.Content class="w-auto p-0" align="start">
										<Calendar 
											type="single" 
											bind:value={newReminderCalendarValue}
											class="rounded-md border-0"
											captionLayout="dropdown"
											locale="de-DE"
											minValue={today(getLocalTimeZone())}
											onValueChange={() => newReminderPopoverOpen = false}
										/>
									</Popover.Content>
								</Popover.Root>
							</div>
							<div>
								<label for="new-reminder-specific-time" class="block text-sm text-subheading text-card-foreground mb-3">Uhrzeit</label>
								<DropdownMenu.Root>
									<DropdownMenu.Trigger class="w-full h-10 justify-start text-left font-normal border-border hover:bg-accent bg-input border border-border rounded-xl px-4 text-foreground focus:border-ring focus:ring-4 focus:ring-ring/20 transition-all duration-300 hover:border-ring/50 flex items-center">
										<Clock class="mr-2 h-4 w-4" />
										{newReminder.specificTime || 'Uhrzeit wählen'}
									</DropdownMenu.Trigger>
									<DropdownMenu.Content class="w-48 max-h-60 overflow-y-auto">
										<DropdownMenu.Group>
											<DropdownMenu.Label>Uhrzeit auswählen</DropdownMenu.Label>
											{#each timeOptions as timeOption (timeOption.value)}
												<DropdownMenu.Item onclick={() => newReminder.specificTime = timeOption.value}>
													{timeOption.label}
												</DropdownMenu.Item>
											{/each}
										</DropdownMenu.Group>
									</DropdownMenu.Content>
								</DropdownMenu.Root>
							</div>
						</div>
						{/if}
					{/if}
				</div>

				<fieldset class="transition-all duration-300">
					<legend class="block text-sm text-subheading text-card-foreground mb-4">Akzentfarbe wählen</legend>
					<div class="flex gap-4 items-center">
						{#each Object.entries(colorClasses) as [color, classes]}
							{#if showEditForm}
								<button
									type="button"
									onclick={() => { 
										if (editingReminder) {
											editingReminder.color = color as ReminderColor;
											showEditColorPicker = false;
										}
									}}
									aria-label="Farbe {color} auswählen"
									class="w-10 h-10 rounded-full {classes} {editingReminder?.color === color ? 'ring-4 ring-primary scale-110 shadow-lg' : 'ring-2 ring-border'} transition-all duration-300 hover:scale-105 hover:shadow-md"
								></button>
							{:else}
								<button
									type="button"
									onclick={() => {
										newReminder.color = color as ReminderColor;
										showColorPicker = false;
									}}
									aria-label="Farbe {color} auswählen"
									class="w-10 h-10 rounded-full {classes} {newReminder.color === color ? 'ring-4 ring-primary scale-110 shadow-lg' : 'ring-2 ring-border'} transition-all duration-300 hover:scale-105 hover:shadow-md"
								></button>
							{/if}
						{/each}
						
							{#if showEditForm}
								<Popover.Root bind:open={showEditColorPicker}>
									<Popover.Trigger>
										<button
											type="button"
											aria-label="Benutzerdefinierte Farbe wählen"
											class="w-10 h-10 rounded-full transition-all duration-300 hover:scale-105 hover:shadow-md overflow-hidden flex items-center justify-center {editingReminder?.color?.startsWith('#') ? 'ring-4 ring-primary scale-110 shadow-lg' : 'ring-2 ring-border'}"
											style="{editingReminder?.color?.startsWith('#') ? `background: ${editingReminder.color};` : 'background: linear-gradient(45deg, #7f5af0, #50fa7b, #e8a87c, #ff6b6b);'}"
										>
											{#if !editingReminder?.color?.startsWith('#')}
												<Palette class="w-4 h-4 text-white drop-shadow-sm" />
											{/if}
										</button>
									</Popover.Trigger>
									<Popover.Content class="w-auto p-4 bg-card border border-border rounded-xl shadow-xl">
										<div class="space-y-3">
											<h4 class="text-sm font-medium text-card-foreground">Benutzerdefinierte Farbe</h4>
											<ColorPicker 
												bind:hex={editCustomColor}
												position="responsive"
												isDialog={false}
												--picker-height="120px"
												--picker-width="200px"
												--slider-width="20px"
												--cp-bg-color="hsl(var(--card))"
												--cp-border-color="hsl(var(--border))"
												--cp-text-color="hsl(var(--card-foreground))"
												--cp-input-color="hsl(var(--input))"
												--cp-button-hover-color="hsl(var(--accent))"
											/>
											<Button 
												onclick={() => {
													if (editingReminder) {
														editingReminder.color = editCustomColor;
														showEditColorPicker = false;
													}
												}}
												class="w-full bg-primary hover:bg-primary/90 text-primary-foreground"
											>
												Farbe übernehmen
											</Button>
										</div>
									</Popover.Content>
								</Popover.Root>
							{:else}
								<Popover.Root bind:open={showColorPicker}>
									<Popover.Trigger>
										<button
											type="button"
											aria-label="Benutzerdefinierte Farbe wählen"
											class="w-10 h-10 rounded-full transition-all duration-300 hover:scale-105 hover:shadow-md overflow-hidden flex items-center justify-center {newReminder.color?.startsWith('#') ? 'ring-4 ring-primary scale-110 shadow-lg' : 'ring-2 ring-border'}"
											style="{newReminder.color?.startsWith('#') ? `background: ${newReminder.color};` : 'background: linear-gradient(45deg, #7f5af0, #50fa7b, #e8a87c, #ff6b6b);'}"
										>
											{#if !newReminder.color?.startsWith('#')}
											<Palette class="w-4 h-4 text-white drop-shadow-sm" />
										{/if}
										</button>
									</Popover.Trigger>
									<Popover.Content class="w-auto p-4 bg-card border border-border rounded-xl shadow-xl">
										<div class="space-y-3">
											<h4 class="text-sm font-medium text-card-foreground">Benutzerdefinierte Farbe</h4>
											<ColorPicker 
												bind:hex={customColor}
												position="responsive"
												isDialog={false}
												--picker-height="120px"
												--picker-width="200px"
												--slider-width="20px"
												--cp-bg-color="hsl(var(--card))"
												--cp-border-color="hsl(var(--border))"
												--cp-text-color="hsl(var(--card-foreground))"
												--cp-input-color="hsl(var(--input))"
												--cp-button-hover-color="hsl(var(--accent))"
											/>
											<Button 
												onclick={() => {
													newReminder.color = customColor;
													showColorPicker = false;
												}}
												class="w-full bg-primary hover:bg-primary/90 text-primary-foreground"
											>
												Farbe übernehmen
											</Button>
										</div>
									</Popover.Content>
								</Popover.Root>
							{/if}
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
			const formContainer = document.querySelector('.form-container');
			if (formContainer) {
				formContainer.classList.add('closing');
			}
			
			setTimeout(() => {
				showCreateForm = false;
				showEditForm = false;
				resetForm();
				cancelEdit();
			}, 600);
			
			setTimeout(() => {
				const contentScroll = document.getElementById('content-scroll');
				if (contentScroll) {
					contentScroll.style.scrollBehavior = 'smooth';
					contentScroll.scrollTop = 0;
				}
			}, 700);
		}}
						variant="outline"
						class="flex-1 border-2 border-border text-muted-foreground hover:bg-destructive/10 hover:text-destructive py-4 rounded-xl text-subheading transition-all duration-300 hover:scale-105 hover:border-destructive"
					>
						Abbrechen
					</Button>
				</div>
			</Content>
			</Card>
		</div>
	{/if}

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
				<Content class="p-8 transition-all duration-800" style="transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94);">
					<div class="flex items-center justify-between gap-4">
						<div class="flex items-center gap-6 flex-1 min-w-0">
							<div 
								class="w-6 h-6 rounded-full flex-shrink-0 shadow-md transition-all duration-300 group-hover:scale-110 {getColorStyle(reminder.color)}"
								style="{getCustomColorStyle(reminder.color)}"
							></div>
							<div class="flex-1 min-w-0">
								<h3 class="text-xl text-subheading text-card-foreground mb-3 truncate">{reminder.name}</h3>
								<div class="space-y-2">
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