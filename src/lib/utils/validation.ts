import type { Reminder, ReminderColor, ReminderInterval, TimerStatus, AppSettings } from '$lib/stores';

const VALID_INTERVALS: ReminderInterval[] = [
	'minutes',
	'hours',
	'days',
	'weeks',
	'months',
	'specific'
];

function isString(value: unknown): value is string {
	return typeof value === 'string';
}

function isBoolean(value: unknown): value is boolean {
	return typeof value === 'boolean';
}

function sanitizeOptionalString(value: unknown): string | undefined {
	if (!isString(value)) {
		return undefined;
	}

	const trimmed = value.trim();
	return trimmed.length > 0 ? trimmed : undefined;
}

function sanitizeNullableString(value: unknown): string | null {
	if (value === null) {
		return null;
	}

	if (!isString(value)) {
		return null;
	}

	const trimmed = value.trim();
	return trimmed.length > 0 ? trimmed : null;
}

function sanitizeReminderColor(value: unknown): ReminderColor {
	if (isString(value) && value.trim().length > 0) {
		return value as ReminderColor;
	}
	return 'blue';
}

function isReminderInterval(value: unknown): value is ReminderInterval {
	return isString(value) && (VALID_INTERVALS as readonly string[]).includes(value);
}

function sanitizeNextExecution(value: unknown): string | null | undefined {
	if (value === undefined) {
		return undefined;
	}
	if (value === null) {
		return null;
	}
	if (!isString(value)) {
		return null;
	}

	const trimmed = value.trim();
	return trimmed.length > 0 ? trimmed : null;
}

export function sanitizeIntervalValue(value: unknown): number {
	const numericValue =
		typeof value === 'number' ? value : isString(value) ? Number.parseFloat(value) : Number.NaN;

	if (!Number.isFinite(numericValue) || numericValue <= 0) {
		return 1;
	}

	// Keep two decimal places to avoid overly precise intervals while
	// allowing fine-grained reminders.
	return Math.round(numericValue * 100) / 100;
}

export function sanitizeReminder(raw: unknown): Reminder | null {
	if (typeof raw !== 'object' || raw === null) {
		return null;
	}

	const record = raw as Record<string, unknown>;

	if (
		!isString(record.id) ||
		!isString(record.name) ||
		!isReminderInterval(record.interval) ||
		!isBoolean(record.active) ||
		!isString(record.createdAt)
	) {
		return null;
	}

	const name = record.name.trim();
	if (name.length === 0) {
		return null;
	}

	const intervalValue = sanitizeIntervalValue(record.intervalValue);

	return {
		id: record.id,
		name,
		interval: record.interval,
		intervalValue,
		specificDate: sanitizeOptionalString(record.specificDate),
		specificTime: sanitizeOptionalString(record.specificTime),
		color: sanitizeReminderColor(record.color),
		createdAt: record.createdAt,
		lastNotified: sanitizeOptionalString(record.lastNotified),
		active: record.active,
		nextExecution: sanitizeNextExecution(record.nextExecution)
	};
}

export function sanitizeReminderList(value: unknown): Reminder[] {
        if (!Array.isArray(value)) {
                return [];
        }

	const seenIds = new Set<string>();
	const reminders: Reminder[] = [];

	for (const entry of value) {
		const reminder = sanitizeReminder(entry);
		if (!reminder || seenIds.has(reminder.id)) {
			continue;
		}

		seenIds.add(reminder.id);
		reminders.push(reminder);
	}

	return reminders;
}

export function sanitizeAppSettings(value: unknown): AppSettings {
        const defaults: AppSettings = {
                language: 'en',
                autostartEnabled: false,
                theme: null,
                notificationSound: true
        };

        if (typeof value !== 'object' || value === null) {
                return { ...defaults };
        }

        const record = value as Record<string, unknown>;

        const rawLanguage = typeof record.language === 'string' ? record.language.trim() : '';
        const normalizedLanguage = rawLanguage === 'de' ? 'de' : 'en';

        const autostartEnabled =
                typeof record.autostartEnabled === 'boolean'
                        ? record.autostartEnabled
                        : typeof record.autostart_enabled === 'boolean'
                                ? record.autostart_enabled
                                : defaults.autostartEnabled;

        const theme = typeof record.theme === 'string' && record.theme.trim().length > 0 ? record.theme : null;

        const notificationSound =
                typeof record.notificationSound === 'boolean'
                        ? record.notificationSound
                        : record.notification_sound !== false;

        return {
                ...record,
                language: normalizedLanguage,
                autostartEnabled,
                theme,
                notificationSound
        } as AppSettings;
}

function sanitizeTimerStatus(raw: unknown): TimerStatus | null {
	if (typeof raw !== 'object' || raw === null) {
		return null;
	}

	const record = raw as Record<string, unknown>;

	if (!isString(record.reminderId) || !isBoolean(record.isScheduled)) {
		return null;
	}

	const name = isString(record.reminderName) ? record.reminderName.trim() : '';

	return {
		reminderId: record.reminderId,
		reminderName: name.length > 0 ? name : record.reminderId,
		nextExecution: sanitizeNullableString(record.nextExecution),
		isScheduled: record.isScheduled
	};
}

export function sanitizeTimerStatusList(value: unknown): TimerStatus[] {
	if (!Array.isArray(value)) {
		return [];
	}

	const seenIds = new Set<string>();
	const result: TimerStatus[] = [];

	for (const entry of value) {
		const status = sanitizeTimerStatus(entry);
		if (!status || seenIds.has(status.reminderId)) {
			continue;
		}

		seenIds.add(status.reminderId);
		result.push(status);
	}

	return result;
}
