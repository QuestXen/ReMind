use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use tokio::sync::{Mutex, oneshot};
use tokio::time::{sleep, Duration as TokioDuration};
use tauri::{AppHandle, Manager, Emitter, Listener};
use crate::commands::app_data::{Reminder, load_app_data, save_app_data, update_reminder_last_notified};
use crate::commands::notifications::send_notification_with_settings;
use log::{error, info};

#[derive(Clone)]
pub struct TimerManager {
    app: AppHandle,
    timers: Arc<Mutex<HashMap<String, oneshot::Sender<()>>>>,
}

impl TimerManager {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            timers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self) {
        let app_data = load_app_data(&self.app).unwrap_or_default();
        for reminder in app_data.reminders {
            if reminder.active {
                self.schedule_reminder(reminder).await;
            }
        }
        
        // Listen for reschedule events
        let timer_manager = self.clone();
        self.app.listen("reschedule-reminder", move |event| {
            let timer_manager = timer_manager.clone();
            let reminder_id = event.payload().to_string();
            let reminder_id = reminder_id.trim_matches('"').to_string();
            tauri::async_runtime::spawn(async move {
                let app_data = load_app_data(&timer_manager.app).unwrap_or_default();
                if let Some(reminder) = app_data.reminders.iter().find(|r| r.id == reminder_id && r.active) {
                    timer_manager.schedule_reminder(reminder.clone()).await;
                }
            });
        });
        
        info!("TimerManager started with all active reminders scheduled.");
    }

    pub async fn schedule_reminder(&self, mut reminder: Reminder) {
        let now = Utc::now();
        let next_execution = match Self::calculate_next_execution(&reminder, now) {
            Some(next) => next,
            None => {
                info!("Cannot calculate next execution for reminder {}", reminder.id);
                return;
            }
        };

        reminder.next_execution = Some(next_execution.to_rfc3339());
        Self::save_reminder(&self.app, &reminder);

        if next_execution <= now {
            Self::execute_reminder(&self.app, &reminder).await;
            if reminder.interval != "specific" {
                // For recurring reminders, calculate the next execution time and save it
                let mut updated_reminder = reminder.clone();
                if let Some(next_exec) = Self::calculate_next_execution(&reminder, Utc::now()) {
                    updated_reminder.next_execution = Some(next_exec.to_rfc3339());
                    Self::save_reminder(&self.app, &updated_reminder);
                    // Don't recursively call schedule_reminder, let the main timer loop handle it
                }
            } else {
                // Deactivate one-time reminders
                let mut updated_reminder = reminder.clone();
                updated_reminder.active = false;
                Self::save_reminder(&self.app, &updated_reminder);
            }
            return;
        }

        let duration = (next_execution - now).to_std().unwrap_or(std::time::Duration::from_secs(0));
        let (tx, rx) = oneshot::channel();
        {
            let mut timers = self.timers.lock().await;
            timers.insert(reminder.id.clone(), tx);
        }



        let app = self.app.clone();
        let timers = self.timers.clone();
        let reminder_id = reminder.id.clone();
        let reminder_name = reminder.name.clone();

        info!("Scheduled reminder '{}' for {}", reminder_name, next_execution.format("%Y-%m-%d %H:%M:%S UTC"));

        tokio::spawn(async move {
            tokio::select! {
                _ = sleep(TokioDuration::from(duration)) => {
                    TimerManager::execute_reminder(&app, &reminder).await;
                    if reminder.interval != "specific" {
                        // For recurring reminders, schedule the next execution
                        let mut updated_reminder = reminder.clone();
                        if let Some(next_exec) = TimerManager::calculate_next_execution(&reminder, Utc::now()) {
                            updated_reminder.next_execution = Some(next_exec.to_rfc3339());
                            TimerManager::save_reminder(&app, &updated_reminder);
                            
                            info!("Next execution scheduled for: {}", updated_reminder.next_execution.as_ref().unwrap_or(&"Unknown".to_string()));
                            
                            // Emit event to reschedule the reminder
                            if let Err(e) = app.emit("reschedule-reminder", &updated_reminder.id) {
                                error!("Failed to emit reschedule-reminder event: {}", e);
                            }
                        }
                    } else {
                        let mut updated_reminder = reminder.clone();
                        updated_reminder.active = false;
                        TimerManager::save_reminder(&app, &updated_reminder);
                    }
                }
                _ = rx => {
                    info!("Timer for reminder {} cancelled.", reminder_id);
                }
            }
            let mut timers = timers.lock().await;
            timers.remove(&reminder_id);
        });
        

    }

    pub async fn cancel_reminder(&self, reminder_id: &str) {
        let mut timers = self.timers.lock().await;
        if let Some(tx) = timers.remove(reminder_id) {
            let _ = tx.send(());
        }
    }

    pub fn calculate_next_execution(reminder: &Reminder, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match reminder.interval.as_str() {
            "minutes" => {
                let total_seconds = (reminder.interval_value * 60.0) as i64;
                Some(now + Duration::seconds(total_seconds))
            },
            "hours" => {
                let total_seconds = (reminder.interval_value * 3600.0) as i64;
                Some(now + Duration::seconds(total_seconds))
            },
            "days" => {
                let total_seconds = (reminder.interval_value * 86400.0) as i64;
                Some(now + Duration::seconds(total_seconds))
            },
            "weeks" => {
                let total_seconds = (reminder.interval_value * 604800.0) as i64;
                Some(now + Duration::seconds(total_seconds))
            },
            "months" => {
                let total_days = (reminder.interval_value * 28.0) as i64;
                Some(now + Duration::days(total_days))
            }
            "specific" => {
                reminder.specific_date.as_ref().and_then(|d| {
                    DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&Utc))
                }).filter(|dt| *dt >= now)
            }
            _ => None,
        }
    }

    async fn execute_reminder(app: &AppHandle, reminder: &Reminder) {
        if let Err(e) = send_notification_with_settings(app.clone(), "ReMind".to_string(), format!("Reminder: {}", reminder.name)) {
            error!("Failed to send notification for {}: {}", reminder.name, e);
        }
        let timestamp = Utc::now().to_rfc3339();
        if let Err(e) = update_reminder_last_notified(app.clone(), reminder.id.clone(), timestamp) {
            error!("Failed to update last_notified for {}: {}", reminder.name, e);
        }
        
        if reminder.interval == "specific" {
            let mut app_data = load_app_data(app).unwrap_or_default();
            if let Some(existing) = app_data.reminders.iter_mut().find(|r| r.id == reminder.id) {
                existing.active = false;
                info!("Deactivated specific reminder: {}", reminder.name);
                
                if let Err(e) = app.emit("reminder-deactivated", &existing.id) {
                    error!("Failed to emit reminder-deactivated event: {}", e);
                }
            }
            if let Err(e) = save_app_data(app, &app_data) {
                error!("Failed to deactivate reminder {}: {}", reminder.name, e);
            }
         } else {
            // For recurring reminders, emit an event to notify frontend of execution
            if let Err(e) = app.emit("reminder-executed", &reminder.id) {
                error!("Failed to emit reminder-executed event: {}", e);
            }
        }
    }

    fn save_reminder(app: &AppHandle, reminder: &Reminder) {
        let mut app_data = load_app_data(app).unwrap_or_default();
        if let Some(existing) = app_data.reminders.iter_mut().find(|r| r.id == reminder.id) {
            *existing = reminder.clone();
        }
        if let Err(e) = save_app_data(app, &app_data) {
            error!("Failed to save reminder {}: {}", reminder.name, e);
        }
    }

    pub async fn get_timer_status(&self) -> Vec<TimerStatus> {
        let timers = self.timers.lock().await;
        let app_data = load_app_data(&self.app).unwrap_or_default();
        
        app_data.reminders.iter().filter_map(|reminder| {
            if !reminder.active {
                return None;
            }
            
            let next_execution = reminder.next_execution.as_ref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));
                
            Some(TimerStatus {
                reminder_id: reminder.id.clone(),
                reminder_name: reminder.name.clone(),
                next_execution: next_execution.map(|dt| dt.to_rfc3339()),
                is_scheduled: timers.contains_key(&reminder.id),
            })
        }).collect()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerStatus {
    pub reminder_id: String,
    pub reminder_name: String,
    pub next_execution: Option<String>,
    pub is_scheduled: bool,
}

#[tauri::command]
pub async fn get_timer_status(app: AppHandle) -> Result<Vec<TimerStatus>, String> {
    if let Some(timer_manager) = app.try_state::<TimerManager>() {
        Ok(timer_manager.get_timer_status().await)
    } else {
        Err("TimerManager not available".to_string())
    }
}