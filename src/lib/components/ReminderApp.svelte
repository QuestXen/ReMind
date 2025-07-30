<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import { Card, Content } from '$lib/components/ui/card/index';
	import { Input } from '$lib/components/ui/input/index';
	import * as Select from '$lib/components/ui/select/index';
	import { Calendar } from '$lib/components/ui/calendar/index';
	import * as Popover from '$lib/components/ui/popover/index';
	import { getLocalTimeZone, today, parseDate } from '@internationalized/date';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import {
		Plus,
		Bell,
		Trash2,
		Calendar as CalendarIcon,
		Clock,
		Edit,
		Settings,
		Palette
	} from '@lucide/svelte';
	import { onMount, onDestroy } from 'svelte';

	import ColorPicker from 'svelte-awesome-color-picker';
	import SettingsComponent from './Settings.svelte';
	import TitleBar from './TitleBar.svelte';
	import {
		reminders,
		addReminder,
		updateReminder as updateReminderStore,
		deleteReminderFromStore,
		settings
	} from '$lib/stores';
	import type { Reminder, ReminderInterval, ReminderColor } from '$lib/stores';
	import * as m from '../../paraglide/messages.js';

	function getCurrentLocale(): string {
		return $settings.language === 'de' ? 'de-DE' : 'en-US';
	}

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

	let backendTimerStatus = $state<
		Array<{
			reminderId: string;
			reminderName: string;
			nextExecution: string | null;
			isScheduled: boolean;
		}>
	>([]);

	let statusUpdateInterval: NodeJS.Timeout | null = null;
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
		{ value: 'minutes', label: m.minutes() },
		{ value: 'hours', label: m.hours() },
		{ value: 'days', label: m.days() },
		{ value: 'weeks', label: m.weeks() },
		{ value: 'months', label: m.days() },
		{ value: 'specific', label: m.specific_date() }
	];

	const intervalLabels = {
		minutes: m.minutes_lowercase(),
		hours: m.hours_lowercase(),
		days: m.days_lowercase(),
		weeks: m.weeks_lowercase(),
		months: m.months_lowercase(),
		specific: m.specific_date()
	};

	let newReminderCalendarValue = $state(today(getLocalTimeZone()));
	let editReminderCalendarValue = $state(today(getLocalTimeZone()));

	let newReminderPopoverOpen = $state(false);
	let editReminderPopoverOpen = $state(false);

	const newReminderTriggerContent = $derived(
		intervalOptions.find((option) => option.value === newReminder.interval)?.label ??
			m.select_interval()
	);

	const editReminderTriggerContent = $derived(
		editingReminder
			? (intervalOptions.find((option) => option.value === editingReminder!.interval)?.label ??
					m.select_interval())
			: m.select_interval()
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

			await updateBackendTimerStatus();
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
			// Create a local date/time and convert to UTC for storage
			const localDateTime = new Date(`${dateStr}T${newReminder.specificTime}:00`);
			newReminder.specificDate = localDateTime.toISOString();
		}
	});

	$effect(() => {
		if (editReminderCalendarValue && editingReminder && editingReminder.interval === 'specific') {
			const dateStr = editReminderCalendarValue.toString();
			const timeStr = editingReminder.specificTime || '12:00';
			// Create a local date/time and convert to UTC for storage
			const localDateTime = new Date(`${dateStr}T${timeStr}:00`);
			editingReminder.specificDate = localDateTime.toISOString();
		}
	});

	function startEditReminder(reminder: Reminder) {
		editingReminder = { ...reminder };

		if (reminder.interval === 'specific' && reminder.specificDate) {
			try {
				// Convert UTC time back to local time for editing
				const utcDate = new Date(reminder.specificDate);
				const year = utcDate.getFullYear();
				const month = String(utcDate.getMonth() + 1).padStart(2, '0');
				const day = String(utcDate.getDate()).padStart(2, '0');
				const hours = String(utcDate.getHours()).padStart(2, '0');
				const minutes = String(utcDate.getMinutes()).padStart(2, '0');

				editReminderCalendarValue = parseDate(`${year}-${month}-${day}`);
				if (editingReminder) {
					editingReminder.specificTime = `${hours}:${minutes}`;
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
		try {
			await invoke('update_reminder', { reminder: editingReminder });

			updateReminderStore(editingReminder);
			await updateBackendTimerStatus();

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
						const scrollTop = elementTop - containerHeight / 2 + elementHeight / 2;
						contentScroll.scrollTop = Math.max(0, scrollTop);
					}
				}
			}, 400);

			console.log(`Updated reminder: ${updatedReminderName}`);
		} catch (error) {
			console.error('Failed to update reminder:', error);
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
					await updateBackendTimerStatus();
					console.log(m.deleted_reminder_with_id({ id }));
				} catch (error) {
					console.error(m.failed_to_delete_reminder(), error);
					if (reminderElement) {
						reminderElement.classList.remove(randomAnimation);
					}
				}
			}, 800);
		} else {
			try {
				await invoke('delete_reminder', { reminderId: id });
				deleteReminderFromStore(id);
				await updateBackendTimerStatus();
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
				console.log(m.activated_reminder({ name: updatedReminder.name }));
			} else {
				console.log(m.deactivated_reminder({ name: updatedReminder.name }));
			}
			await updateBackendTimerStatus();
		} catch (error) {
			console.error(m.failed_to_toggle_reminder(), error);
		}
	}

	async function updateBackendTimerStatus() {
		try {
			const status = await invoke('get_timer_status');
			backendTimerStatus = status as Array<{
				reminderId: string;
				reminderName: string;
				nextExecution: string | null;
				isScheduled: boolean;
			}>;

			await checkAndUpdateExpiredReminders();
		} catch (error) {
			console.error('Failed to get timer status from backend:', error);
		}
	}

	async function checkAndUpdateExpiredReminders() {
		try {
			const updatedReminders = await invoke('load_reminders');
			const currentReminders = $reminders;

			for (const backendReminder of updatedReminders as Reminder[]) {
				const frontendReminder = currentReminders.find((r) => r.id === backendReminder.id);

				if (frontendReminder && frontendReminder.active && !backendReminder.active) {
					updateReminderStore(backendReminder);
					console.log(`Automatically deactivated expired reminder: ${backendReminder.name}`);
				}
			}
		} catch (error) {
			console.error('Failed to check expired reminders:', error);
		}
	}

	function getTimerStatusForReminder(reminderId: string) {
		return backendTimerStatus.find((status) => status.reminderId === reminderId);
	}

	function getTimerStatus() {
		const status = {
			activeTimers: backendTimerStatus.length,
			timers: backendTimerStatus.map((timerStatus) => {
				return {
					id: timerStatus.reminderId,
					name: timerStatus.reminderName,
					nextExecution: timerStatus.nextExecution
						? new Date(timerStatus.nextExecution).toLocaleString(getCurrentLocale())
						: 'N/A',
					timeUntilExecution: timerStatus.nextExecution
						? Math.max(0, new Date(timerStatus.nextExecution).getTime() - Date.now())
						: 0,
					isScheduled: timerStatus.isScheduled
				};
			})
		};
		console.table(status.timers);
		return status;
	}

	interface DebugWindow extends Window {
		getTimerStatus?: () => {
			activeTimers: number;
			timers: Array<{
				id: string;
				name: string;
				nextExecution: string;
				timeUntilExecution: number;
				isScheduled: boolean;
			}>;
		};
		updateBackendTimerStatus?: () => Promise<void>;
	}

	if (typeof window !== 'undefined' && import.meta.env.DEV) {
		(window as DebugWindow).getTimerStatus = getTimerStatus;
		(window as DebugWindow).updateBackendTimerStatus = updateBackendTimerStatus;
		console.log('Debug functions available: getTimerStatus(), updateBackendTimerStatus()');
	}

	function formatReminderInfo(reminder: Reminder): string {
		if (reminder.interval === 'specific' && reminder.specificDate) {
			const date = new Date(reminder.specificDate);
			const dateStr = date.toLocaleDateString(getCurrentLocale());
			const timeStr = date.toLocaleTimeString(getCurrentLocale(), {
				hour: '2-digit',
				minute: '2-digit'
			});
			return `${dateStr} ${timeStr}`;
		}
		return `${m.every()} ${reminder.intervalValue} ${intervalLabels[reminder.interval]}`;
	}

	function getTimeUntilNextReminder(reminder: Reminder): string {
		void timeUpdateTrigger;

		if (!reminder.active) {
			return '';
		}

		const timerStatus = getTimerStatusForReminder(reminder.id);
		if (!timerStatus || !timerStatus.nextExecution) {
			return '';
		}

		const now = new Date();
		const nextExecution = new Date(timerStatus.nextExecution);
		const timeUntil = nextExecution.getTime() - now.getTime();

		if (timeUntil <= 0) {
			return '';
		}

		const days = Math.floor(timeUntil / (1000 * 60 * 60 * 24));
		const hours = Math.floor((timeUntil % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
		const minutes = Math.floor((timeUntil % (1000 * 60 * 60)) / (1000 * 60));
		const seconds = Math.floor((timeUntil % (1000 * 60)) / 1000);

		if (days > 0) {
			if (hours > 0) {
				return `${m.in_time()} ${days} ${days > 1 ? m.days_plural() : m.day()} ${m.and()} ${hours} ${hours > 1 ? m.hours_plural() : m.hour()}`;
			} else {
				return `${m.in_time()} ${days} ${days > 1 ? m.days_plural() : m.day()}`;
			}
		} else if (hours > 0) {
			if (minutes > 0) {
				return `${m.in_time()} ${hours} ${hours > 1 ? m.hours_plural() : m.hour()} ${m.and()} ${minutes} ${minutes > 1 ? m.minutes_plural() : m.minute()}`;
			} else {
				return `${m.in_time()} ${hours} ${hours > 1 ? m.hours_plural() : m.hour()}`;
			}
		} else if (minutes > 0) {
			return `${m.in_time()} ${minutes} ${minutes > 1 ? m.minutes_plural() : m.minute()}`;
		} else if (seconds > 0) {
			return `${m.in_time()} ${seconds} ${seconds > 1 ? m.seconds_plural() : m.second()}`;
		} else {
			return m.few_seconds();
		}
	}

	onMount(() => {
		let isInitialLoad = true;
		let unlisten: (() => void) | null = null;

		if (localStorage.getItem('keepSettingsOpen') === 'true') {
			showSettings = true;
			localStorage.removeItem('keepSettingsOpen');
		}

		const unsubscribe = reminders.subscribe(async (currentReminders) => {
			if (isInitialLoad) {
				console.log(`Initialized ${currentReminders.length} reminders`);
				isInitialLoad = false;
				await updateBackendTimerStatus();
			}
		});

		(async () => {
			unlisten = await listen('reminder-deactivated', async (event) => {
				const reminderId = event.payload as string;
				console.log(`Received reminder-deactivated event for: ${reminderId}`);
				await checkAndUpdateExpiredReminders();
			});
		})();

		statusUpdateInterval = setInterval(() => {
			updateBackendTimerStatus();
		}, 5 * 1000); // WIchtigeee

		timeUpdateInterval = setInterval(() => {
			timeUpdateTrigger++;
		}, 1000);

		return () => {
			unsubscribe();
			if (unlisten) {
				unlisten();
			}
		};
	});

	onDestroy(() => {
		if (statusUpdateInterval) {
			clearInterval(statusUpdateInterval);
		}
		if (timeUpdateInterval) {
			clearInterval(timeUpdateInterval);
		}
		console.log('ReminderApp component destroyed, all timers cleared');
	});
</script>

{#if showSettings}
	<SettingsComponent on:close={closeSettings} />
{:else}
	<div class="flex h-screen flex-col overflow-hidden">
		<TitleBar title="Re:Mind" icon="R" useLogoIcon={true} />

		<div
			id="content-scroll"
			class="bg-background relative flex-1 overflow-y-auto {showSettings
				? 'animate-slide-out-left'
				: 'animate-slide-in-left'}"
		>
			<div class="p-6">
				<div class="mx-auto max-w-4xl">
					<div class="mb-12 flex justify-center">
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
							class="bg-primary hover:bg-primary/90 text-primary-foreground text-subheading rounded-2xl border-0 px-8 py-4 shadow-lg transition-all duration-300 hover:scale-105 hover:shadow-xl"
						>
							<Plus class="mr-3 h-5 w-5" />
							{m.new_reminder()}
						</Button>
					</div>

					{#if showCreateForm || showEditForm}
						<div class="form-container">
							<Card
								id="create-form"
								class="bg-card border-border animate-slide-up mb-12 overflow-hidden rounded-3xl border shadow-xl"
							>
								<Content class="space-y-8 p-8">
									<h2 class="text-heading text-card-foreground mb-6 text-3xl">
										{showEditForm ? m.edit_reminder() : m.new_reminder()}
									</h2>
									<div class="transition-all duration-300">
										<label
											for="reminder-name"
											class="text-subheading text-card-foreground mb-3 block text-sm"
											>{m.reminder_name()}</label
										>
										{#if showEditForm}
											<Input
												id="reminder-name"
												bind:value={editingReminder!.name}
												autocomplete="off"
												placeholder={m.reminder_name_placeholder()}
												class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-12 w-full rounded-xl px-4 transition-all duration-300"
											/>
										{:else}
											<Input
												id="reminder-name"
												bind:value={newReminder.name}
												autocomplete="off"
												placeholder={m.reminder_name_placeholder()}
												class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-12 w-full rounded-xl px-4 transition-all duration-300"
											/>
										{/if}
									</div>

									<div class="grid grid-cols-1 gap-8 md:grid-cols-2">
										<div class="transition-all duration-300">
											<label
												for="reminder-interval"
												class="text-subheading text-card-foreground mb-3 block text-sm"
												>{m.interval()}</label
											>
											{#if showEditForm && editingReminder}
												<Select.Root
													type="single"
													name="editReminderInterval"
													bind:value={editingReminder.interval}
												>
													<Select.Trigger
														class="bg-input border-border text-foreground focus:border-ring focus:ring-ring/20 hover:border-ring/50 flex h-12 w-full items-center rounded-xl border px-4 transition-all duration-300 focus:ring-4"
													>
														{editReminderTriggerContent}
													</Select.Trigger>
													<Select.Content>
														<Select.Group>
															<Select.Label>{m.select_interval()}</Select.Label>
															{#each intervalOptions as option (option.value)}
																<Select.Item value={option.value} label={option.label}>
																	{option.label}
																</Select.Item>
															{/each}
														</Select.Group>
													</Select.Content>
												</Select.Root>
											{:else}
												<Select.Root
													type="single"
													name="newReminderInterval"
													bind:value={newReminder.interval}
												>
													<Select.Trigger
														class="bg-input border-border text-foreground focus:border-ring focus:ring-ring/20 hover:border-ring/50 flex h-12 w-full items-center rounded-xl border px-4 transition-all duration-300 focus:ring-4"
													>
														{newReminderTriggerContent}
													</Select.Trigger>
													<Select.Content>
														<Select.Group>
															<Select.Label>{m.select_interval()}</Select.Label>
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
													<label
														for="reminder-interval-value"
														class="text-subheading text-card-foreground mb-3 block text-sm"
														>{m.amount()}</label
													>
													<Input
														id="reminder-interval-value"
														type="number"
														autocomplete="off"
														bind:value={editingReminder.intervalValue}
														min="1"
														class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-10 w-full rounded-xl px-4 transition-all duration-300"
													/>
												</div>
											{:else}
												<div class="space-y-4 transition-all duration-300">
													<div>
														<label
															for="reminder-specific-date"
															class="text-subheading text-card-foreground mb-3 block text-sm"
															>{m.date()}</label
														>
														<Popover.Root bind:open={editReminderPopoverOpen}>
															<Popover.Trigger
																class="border-border hover:bg-accent bg-input border-border text-foreground focus:border-ring focus:ring-ring/20 hover:border-ring/50 flex h-10 w-full items-center justify-start rounded-xl border px-4 text-left font-normal transition-all duration-300 focus:ring-4"
															>
																<CalendarIcon class="mr-2 h-4 w-4" />
																{editReminderCalendarValue
																	? editReminderCalendarValue
																			.toDate(getLocalTimeZone())
																			.toLocaleDateString(getCurrentLocale())
																	: m.date()}
															</Popover.Trigger>
															<Popover.Content class="w-auto p-0" align="start">
																<Calendar
																	type="single"
																	bind:value={editReminderCalendarValue}
																	class="rounded-md border-0"
																	locale={getCurrentLocale()}
																	minValue={today(getLocalTimeZone())}
																	onValueChange={() => (editReminderPopoverOpen = false)}
																/>
															</Popover.Content>
														</Popover.Root>
													</div>
													<div>
														<label
															for="reminder-specific-time"
															class="text-subheading text-card-foreground mb-3 block text-sm"
															>{m.time()}</label
														>
														<Popover.Root>
															<Popover.Trigger
																class="border-border hover:bg-accent bg-input border-border text-foreground focus:border-ring focus:ring-ring/20 hover:border-ring/50 flex h-10 w-full items-center justify-start rounded-xl border px-4 text-left font-normal transition-all duration-300 focus:ring-4"
															>
																<Clock class="mr-2 h-4 w-4" />
																{editingReminder?.specificTime || m.select_time()}
															</Popover.Trigger>
															<Popover.Content
																class="bg-card border-border w-64 rounded-xl p-4 shadow-xl"
															>
																<div class="grid grid-cols-2 gap-4">
																	<div>
																		<p class="mb-2 text-sm font-medium capitalize">{m.hour()}</p>
																		<div
																			class="border-border max-h-32 overflow-y-auto rounded-md border"
																		>
																			{#each Array.from({ length: 24 }, (_, i) => i
																					.toString()
																					.padStart(2, '0')) as hour (hour)}
																				<button
																					type="button"
																					class="hover:bg-muted text-foreground w-full py-1"
																					onclick={() => {
																						if (editingReminder) {
																							const [, m] = (
																								editingReminder.specificTime || '00:00'
																							).split(':');
																							editingReminder.specificTime = `${hour}:${m}`;
																						}
																					}}
																				>
																					{hour}
																				</button>
																			{/each}
																		</div>
																	</div>
																	<div>
																		<p class="mb-2 text-sm font-medium capitalize">{m.minute()}</p>
																		<div
																			class="border-border max-h-32 overflow-y-auto rounded-md border"
																		>
																			{#each Array.from({ length: 60 }, (_, i) => i
																					.toString()
																					.padStart(2, '0')) as minute (minute)}
																				<button
																					type="button"
																					class="hover:bg-muted text-foreground w-full py-1"
																					onclick={() => {
																						if (editingReminder) {
																							const [h] = (
																								editingReminder.specificTime || '00:00'
																							).split(':');
																							editingReminder.specificTime = `${h}:${minute}`;
																						}
																					}}
																				>
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
													class="border-border focus:border-ring focus:ring-ring bg-input focus:bg-background text-body text-foreground h-10 w-full rounded-xl px-4 transition-all duration-300"
												/>
											</div>
										{:else}
											<div class="space-y-4 transition-all duration-300">
												<div>
													<label
														for="new-reminder-specific-date"
														class="text-subheading text-card-foreground mb-3 block text-sm"
														>{m.date()}</label
													>
													<Popover.Root bind:open={newReminderPopoverOpen}>
														<Popover.Trigger
															class="border-border hover:bg-accent bg-input border-border text-foreground focus:border-ring focus:ring-ring/20 hover:border-ring/50 flex h-10 w-full items-center justify-start rounded-xl border px-4 text-left font-normal transition-all duration-300 focus:ring-4"
														>
															<CalendarIcon class="mr-2 h-4 w-4" />
															{newReminderCalendarValue
																? newReminderCalendarValue
																		.toDate(getLocalTimeZone())
																		.toLocaleDateString(getCurrentLocale())
																: m.date()}
														</Popover.Trigger>
														<Popover.Content class="w-auto p-0" align="start">
															<Calendar
																type="single"
																bind:value={newReminderCalendarValue}
																class="rounded-md border-0"
																locale={getCurrentLocale()}
																minValue={today(getLocalTimeZone())}
																onValueChange={() => (newReminderPopoverOpen = false)}
															/>
														</Popover.Content>
													</Popover.Root>
												</div>
												<div>
													<label
														for="new-reminder-specific-time"
														class="text-subheading text-card-foreground mb-3 block text-sm"
														>{m.time()}</label
													>
													<Popover.Root>
														<Popover.Trigger
															class="border-border hover:bg-accent bg-input border-border text-foreground focus:border-ring focus:ring-ring/20 hover:border-ring/50 flex h-10 w-full items-center justify-start rounded-xl border px-4 text-left font-normal transition-all duration-300 focus:ring-4"
														>
															<Clock class="mr-2 h-4 w-4" />
															{newReminder.specificTime || m.select_time()}
														</Popover.Trigger>
														<Popover.Content
															class="bg-card border-border w-64 rounded-xl p-4 shadow-xl"
														>
															<div class="grid grid-cols-2 gap-4">
																<div>
																	<p class="mb-2 text-sm font-medium capitalize">{m.hour()}</p>
																	<div
																		class="border-border max-h-32 overflow-y-auto rounded-md border"
																	>
																		{#each Array.from({ length: 24 }, (_, i) => i
																				.toString()
																				.padStart(2, '0')) as hour (hour)}
																			<button
																				type="button"
																				class="hover:bg-muted text-foreground w-full py-1"
																				onclick={() => {
																					const [, m] = (newReminder.specificTime || '00:00').split(
																						':'
																					);
																					newReminder.specificTime = `${hour}:${m}`;
																				}}
																			>
																				{hour}
																			</button>
																		{/each}
																	</div>
																</div>
																<div>
																	<p class="mb-2 text-sm font-medium capitalize">{m.minute()}</p>
																	<div
																		class="border-border max-h-32 overflow-y-auto rounded-md border"
																	>
																		{#each Array.from({ length: 60 }, (_, i) => i
																				.toString()
																				.padStart(2, '0')) as minute (minute)}
																			<button
																				type="button"
																				class="hover:bg-muted text-foreground w-full py-1"
																				onclick={() => {
																					const [h] = (newReminder.specificTime || '00:00').split(
																						':'
																					);
																					newReminder.specificTime = `${h}:${minute}`;
																				}}
																			>
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
										{/if}
									</div>

									<fieldset class="transition-all duration-300">
										<legend class="text-subheading text-card-foreground mb-4 block text-sm"
											>{m.color()}</legend
										>
										<div class="flex items-center gap-4">
											{#each Object.entries(colorClasses) as [color, classes] (color)}
												{#if showEditForm}
													<button
														type="button"
														onclick={() => {
															if (editingReminder) {
																editingReminder.color = color as ReminderColor;
																showEditColorPicker = false;
															}
														}}
														aria-label="{m.color()} {color}"
														class="h-10 w-10 rounded-full {classes} {editingReminder?.color ===
														color
															? 'ring-primary scale-110 shadow-lg ring-4'
															: 'ring-border ring-2'} transition-all duration-300 hover:scale-105 hover:shadow-md"
													></button>
												{:else}
													<button
														type="button"
														onclick={() => {
															newReminder.color = color as ReminderColor;
															showColorPicker = false;
														}}
														aria-label="{m.color()} {color}"
														class="h-10 w-10 rounded-full {classes} {newReminder.color === color
															? 'ring-primary scale-110 shadow-lg ring-4'
															: 'ring-border ring-2'} transition-all duration-300 hover:scale-105 hover:shadow-md"
													></button>
												{/if}
											{/each}

											{#if showEditForm}
												<Popover.Root bind:open={showEditColorPicker}>
													<Popover.Trigger>
														<button
															type="button"
															aria-label={m.custom_color()}
															class="flex h-10 w-10 items-center justify-center overflow-hidden rounded-full transition-all duration-300 hover:scale-105 hover:shadow-md {editingReminder?.color?.startsWith(
																'#'
															)
																? 'ring-primary scale-110 shadow-lg ring-4'
																: 'ring-border ring-2'}"
															style={editingReminder?.color?.startsWith('#')
																? `background: ${editingReminder.color};`
																: 'background: linear-gradient(45deg, #7f5af0, #50fa7b, #e8a87c, #ff6b6b);'}
														>
															{#if !editingReminder?.color?.startsWith('#')}
																<Palette class="h-4 w-4 text-white drop-shadow-sm" />
															{/if}
														</button>
													</Popover.Trigger>
													<Popover.Content
														class="bg-card border-border w-auto rounded-xl border p-4 shadow-xl"
													>
														<div class="space-y-3">
															<h4 class="text-card-foreground text-sm font-medium">
																{m.custom_color()}
															</h4>
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
																class="bg-primary hover:bg-primary/90 text-primary-foreground w-full"
															>
																{m.custom_color()}
															</Button>
														</div>
													</Popover.Content>
												</Popover.Root>
											{:else}
												<Popover.Root bind:open={showColorPicker}>
													<Popover.Trigger>
														<button
															type="button"
															aria-label={m.custom_color_choose()}
															class="flex h-10 w-10 items-center justify-center overflow-hidden rounded-full transition-all duration-300 hover:scale-105 hover:shadow-md {newReminder.color?.startsWith(
																'#'
															)
																? 'ring-primary scale-110 shadow-lg ring-4'
																: 'ring-border ring-2'}"
															style={newReminder.color?.startsWith('#')
																? `background: ${newReminder.color};`
																: 'background: linear-gradient(45deg, #7f5af0, #50fa7b, #e8a87c, #ff6b6b);'}
														>
															{#if !newReminder.color?.startsWith('#')}
																<Palette class="h-4 w-4 text-white drop-shadow-sm" />
															{/if}
														</button>
													</Popover.Trigger>
													<Popover.Content
														class="bg-card border-border w-auto rounded-xl border p-4 shadow-xl"
													>
														<div class="space-y-3">
															<h4 class="text-card-foreground text-sm font-medium">
																{m.custom_color_title()}
															</h4>
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
																class="bg-primary hover:bg-primary/90 text-primary-foreground w-full"
															>
																{m.apply_color()}
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
											class="bg-primary hover:bg-primary/90 text-primary-foreground text-subheading flex-1 rounded-xl border-0 py-4 transition-all duration-300 hover:scale-105 hover:shadow-lg"
										>
											<Bell class="mr-3 h-5 w-5" />
											{showEditForm ? m.update_reminder_button() : m.create_reminder_button()}
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
											class="border-border text-muted-foreground hover:bg-destructive/10 hover:text-destructive text-subheading hover:border-destructive flex-1 rounded-xl border-2 py-4 transition-all duration-300 hover:scale-105"
										>
											{m.cancel()}
										</Button>
									</div>
								</Content>
							</Card>
						</div>
					{/if}

					<div
						class="grid gap-6 transition-all duration-800"
						style="transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94);"
					>
						{#if $reminders.length === 0}
							<Card
								class="bg-card border-border animate-slide-up-staggered overflow-hidden rounded-3xl border shadow-lg"
							>
								<Content class="py-16 text-center">
									<h3 class="text-heading text-card-foreground mb-3 text-2xl">
										{m.no_reminders_yet()}
									</h3>
									<p class="text-muted-foreground text-caption text-lg">
										{m.create_first_reminder()}
									</p>
								</Content>
							</Card>
						{:else}
							{#each $reminders as reminder, index (reminder.id)}
								<Card
									id={`reminder-${reminder.id}`}
									class="bg-card border-border group animate-slide-up-staggered overflow-hidden rounded-3xl border shadow-lg transition-all duration-800 hover:scale-[1.02] hover:shadow-xl"
									style="animation-delay: {index *
										0.1}s; will-change: transform, height; transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94);"
								>
									<Content
										class="p-8 transition-all duration-800"
										style="transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94);"
									>
										<div class="flex items-center justify-between gap-4">
											<div class="flex min-w-0 flex-1 items-center gap-6">
												<div
													class="h-6 w-6 flex-shrink-0 rounded-full shadow-md transition-all duration-300 group-hover:scale-110 {getColorStyle(
														reminder.color
													)}"
													style={getCustomColorStyle(reminder.color)}
												></div>
												<div class="min-w-0 flex-1">
													<h3 class="text-subheading text-card-foreground mb-3 truncate text-xl">
														{reminder.name}
													</h3>
													<div class="space-y-2">
														<div class="text-body flex items-center gap-2 text-sm">
															{#if reminder.interval === 'specific'}
																<CalendarIcon class="h-4 w-4 flex-shrink-0" />
															{:else}
																<Clock class="h-4 w-4 flex-shrink-0" />
															{/if}
															<span class="truncate">{formatReminderInfo(reminder)}</span>
														</div>

														<div
															class="overflow-hidden transition-all duration-800"
															style="max-height: {getTimeUntilNextReminder(reminder) &&
															reminder.active
																? '80px'
																: '0px'}; padding-top: {getTimeUntilNextReminder(reminder) &&
															reminder.active
																? '8px'
																: '0px'}; margin-top: {getTimeUntilNextReminder(reminder) &&
															reminder.active
																? '8px'
																: '0px'}; transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94);"
														>
															<div
																class="text-muted-foreground flex items-start gap-2 pt-2 text-sm transition-opacity duration-800"
																style="opacity: {getTimeUntilNextReminder(reminder) &&
																reminder.active
																	? '1'
																	: '0'}; transition-timing-function: cubic-bezier(0.25, 0.46, 0.45, 0.94); transition-delay: {getTimeUntilNextReminder(
																	reminder
																) && reminder.active
																	? '0ms'
																	: '200ms'};"
															>
																<div
																	class="mt-2 h-2 w-2 flex-shrink-0 animate-pulse rounded-full bg-emerald-500"
																></div>
																<span class="leading-relaxed break-words">
																	<span class="font-medium text-emerald-600"
																		>{m.next_reminder_label()}</span
																	><br />
																	<span class="text-caption"
																		>{getTimeUntilNextReminder(reminder) || ''}</span
																	>
																</span>
															</div>
														</div>

														{#if reminder.lastNotified}
															<div class="text-muted-foreground flex items-center gap-2 text-sm">
																<div class="h-2 w-2 flex-shrink-0 rounded-full bg-gray-400"></div>
																<span class="text-caption truncate"
																	>{m.last_label()}: {new Date(
																		reminder.lastNotified
																	).toLocaleString(getCurrentLocale())}</span
																>
															</div>
														{/if}
													</div>
												</div>
											</div>
											<div class="flex items-center gap-3">
												<div
													class="flex items-center gap-2 opacity-0 transition-opacity group-hover:opacity-100"
												>
													<Button
														onclick={() => startEditReminder(reminder)}
														variant="outline"
														class="border-border text-muted-foreground rounded-xl border-2 p-3 transition-all duration-300 hover:border-green-500 hover:bg-green-500/10 hover:text-green-600"
													>
														<Edit class="h-4 w-4" />
													</Button>
													<Button
														onclick={() => deleteReminder(reminder.id)}
														variant="outline"
														class="border-destructive/30 text-destructive hover:bg-destructive/10 hover:border-destructive rounded-xl border-2 p-3 transition-all duration-300"
													>
														<Trash2 class="h-4 w-4" />
													</Button>
												</div>

												<div class="relative">
													{#if reminder.active}
														<div
															class="absolute -bottom-1 left-1/2 h-2 w-12 -translate-x-1/2 transform rounded-full bg-emerald-400/30 blur-sm"
														></div>
													{/if}
													<Button
														onclick={() => toggleReminderActive(reminder)}
														variant="outline"
														class="relative border-2 transition-all duration-500 {reminder.active
															? 'border-emerald-400/50 bg-gradient-to-r from-emerald-50 to-green-50 text-emerald-600 hover:border-emerald-500'
															: 'border-slate-300/50 bg-gradient-to-r from-slate-50 to-gray-50 text-slate-400 hover:border-slate-400'} group/toggle rounded-2xl p-3"
														title={reminder.active
															? m.active_click_to_deactivate()
															: m.inactive_click_to_activate()}
													>
														<div class="relative h-4 w-8 transition-all duration-500">
															<div
																class="absolute inset-0 rounded-full transition-all duration-500 {reminder.active
																	? 'bg-gradient-to-r from-emerald-400 to-green-500 shadow-inner'
																	: 'bg-gradient-to-r from-slate-300 to-gray-400 shadow-inner'}"
															></div>

															<div
																class="absolute top-0.5 h-3 w-3 transform rounded-full bg-white shadow-lg transition-all duration-500 {reminder.active
																	? 'translate-x-4 shadow-emerald-300/50'
																	: 'translate-x-0.5 shadow-slate-400/50'} group-hover/toggle:scale-110"
															>
																<div
																	class="absolute inset-0.5 rounded-full transition-all duration-500 {reminder.active
																		? 'bg-gradient-to-br from-emerald-200 to-green-300'
																		: 'bg-gradient-to-br from-slate-200 to-gray-300'}"
																></div>
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
{/if}

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

	html,
	body {
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
