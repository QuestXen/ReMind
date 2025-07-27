use super::errors::Error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub name: String,
    pub interval: String,
    pub interval_value: u32,
    pub specific_date: Option<String>,
    pub color: String,
    pub created_at: String,
    pub last_notified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub autostart_enabled: bool,
    pub theme: Option<String>,
    pub notification_sound: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autostart_enabled: false,
            theme: None,
            notification_sound: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub reminders: Vec<Reminder>,
    pub settings: AppSettings,
}

fn get_app_data_file_path(app: &AppHandle) -> Result<PathBuf, Error> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // Create directory if it doesn't exist
    fs::create_dir_all(&app_data_dir)?;

    Ok(app_data_dir.join("app_data.json"))
}

fn load_app_data(app: &AppHandle) -> Result<AppData, Error> {
    let file_path = get_app_data_file_path(app)?;

    if !file_path.exists() {
        return Ok(AppData::default());
    }

    let json_data = fs::read_to_string(file_path)?;
    let app_data: AppData = serde_json::from_str(&json_data)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(app_data)
}

fn save_app_data(app: &AppHandle, app_data: &AppData) -> Result<(), Error> {
    let file_path = get_app_data_file_path(app)?;
    let json_data = serde_json::to_string_pretty(app_data)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    fs::write(file_path, json_data)?;
    Ok(())
}

// Reminder commands
#[tauri::command]
pub fn save_reminders(app: AppHandle, reminders: Vec<Reminder>) -> Result<(), Error> {
    let mut app_data = load_app_data(&app).unwrap_or_default();
    app_data.reminders = reminders;
    save_app_data(&app, &app_data)?;
    Ok(())
}

#[tauri::command]
pub fn load_reminders(app: AppHandle) -> Result<Vec<Reminder>, Error> {
    let app_data = load_app_data(&app).unwrap_or_default();
    Ok(app_data.reminders)
}

#[tauri::command]
pub fn delete_reminder(app: AppHandle, reminder_id: String) -> Result<(), Error> {
    let mut app_data = load_app_data(&app).unwrap_or_default();
    app_data.reminders.retain(|r| r.id != reminder_id);
    save_app_data(&app, &app_data)?;
    Ok(())
}

#[tauri::command]
pub fn add_reminder(app: AppHandle, reminder: Reminder) -> Result<(), Error> {
    let mut app_data = load_app_data(&app).unwrap_or_default();
    app_data.reminders.push(reminder);
    save_app_data(&app, &app_data)?;
    Ok(())
}

#[tauri::command]
pub fn update_reminder(app: AppHandle, reminder: Reminder) -> Result<(), Error> {
    let mut app_data = load_app_data(&app).unwrap_or_default();

    if let Some(existing_reminder) = app_data.reminders.iter_mut().find(|r| r.id == reminder.id) {
        *existing_reminder = reminder;
        save_app_data(&app, &app_data)?;
    }

    Ok(())
}

#[tauri::command]
pub fn update_reminder_last_notified(
    app: AppHandle,
    reminder_id: String,
    timestamp: String,
) -> Result<(), Error> {
    let mut app_data = load_app_data(&app).unwrap_or_default();

    if let Some(reminder) = app_data.reminders.iter_mut().find(|r| r.id == reminder_id) {
        reminder.last_notified = Some(timestamp);
        save_app_data(&app, &app_data)?;
    }

    Ok(())
}

// Settings commands
#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), Error> {
    println!("[Backend] Saving settings: {:?}", settings);
    let mut app_data = load_app_data(&app).unwrap_or_default();
    app_data.settings = settings.clone();
    save_app_data(&app, &app_data)?;
    println!("[Backend] Settings saved successfully");
    Ok(())
}

#[tauri::command]
pub fn load_settings(app: AppHandle) -> Result<AppSettings, Error> {
    println!("[Backend] Loading settings...");
    let app_data = load_app_data(&app).unwrap_or_default();
    println!("[Backend] Loaded settings: {:?}", app_data.settings);
    Ok(app_data.settings)
}

#[tauri::command]
pub fn update_autostart_setting(app: AppHandle, enabled: bool) -> Result<(), Error> {
    println!("[Backend] Updating autostart setting to: {}", enabled);
    let mut app_data = load_app_data(&app).unwrap_or_default();
    app_data.settings.autostart_enabled = enabled;
    save_app_data(&app, &app_data)?;
    println!("[Backend] Autostart setting updated successfully");
    Ok(())
}

#[tauri::command]
pub fn get_autostart_setting(app: AppHandle) -> Result<bool, Error> {
    println!("[Backend] Getting autostart setting...");
    let app_data = load_app_data(&app).unwrap_or_default();
    let enabled = app_data.settings.autostart_enabled;
    println!("[Backend] Autostart setting: {}", enabled);
    Ok(enabled)
}

#[tauri::command]
pub fn update_setting(app: AppHandle, key: String, value: serde_json::Value) -> Result<(), Error> {
    let mut app_data = load_app_data(&app).unwrap_or_default();
    
    match key.as_str() {
        "autostart_enabled" => {
            if let Some(bool_val) = value.as_bool() {
                app_data.settings.autostart_enabled = bool_val;
            }
        }
        "theme" => {
            app_data.settings.theme = value.as_str().map(|s| s.to_string());
        }
        "notification_sound" => {
            if let Some(bool_val) = value.as_bool() {
                app_data.settings.notification_sound = bool_val;
            }
        }
        _ => return Err(Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unknown setting key")))
    }
    
    save_app_data(&app, &app_data)?;
    Ok(())
}