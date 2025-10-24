import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';

// Types
export type ReminderInterval = 'minutes' | 'hours' | 'days' | 'weeks' | 'months' | 'specific';
export type ReminderColor = 'blue' | 'green' | 'purple' | 'red' | 'orange' | 'pink' | string;

export interface Reminder {
	id: string;
	name: string;
	interval: ReminderInterval;
	intervalValue: number;
	specificDate?: string;
	specificTime?: string;
	color: ReminderColor;
	createdAt: string;
	lastNotified?: string;
	active: boolean;
	nextExecution?: string | null;
}

export interface AppSettings {
	language: string;
	autostartEnabled: boolean;
	theme?: string | null;
	notificationSound: boolean;
	[key: string]: unknown;
}

export interface TimerStatus {
	reminderId: string;
	reminderName: string;
	nextExecution: string | null;
	isScheduled: boolean;
}

// Global Stores
export const reminders: Writable<Reminder[]> = writable([]);
export const settings: Writable<AppSettings> = writable({
	language: 'en',
	autostartEnabled: false,
	theme: null,
	notificationSound: true
});
export const isLoading: Writable<boolean> = writable(true);
export const loadingError: Writable<string | null> = writable(null);

export function updateReminder(updatedReminder: Reminder) {
	reminders.update((currentReminders) =>
		currentReminders.map((r) => (r.id === updatedReminder.id ? updatedReminder : r))
	);
}

export function addReminder(newReminder: Reminder) {
	reminders.update((currentReminders) => [...currentReminders, newReminder]);
}

export function deleteReminderFromStore(reminderId: string) {
	reminders.update((currentReminders) => currentReminders.filter((r) => r.id !== reminderId));
}

export function updateSetting(key: string, value: unknown) {
	settings.update((currentSettings) => ({
		...currentSettings,
		[key]: value
	}));
}
