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
    pub specific_time: Option<String>,
    pub color: String,
    pub created_at: String,
    pub last_notified: Option<String>,
    pub active: bool,
}

// Legacy reminder structure for migration support
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyReminder {
    pub id: String,
    pub name: String,
    pub interval: String,
    pub interval_value: u32,
    pub specific_date: Option<String>,
    pub specific_time: Option<String>,
    pub color: String,
    pub created_at: String,
    pub last_notified: Option<String>,
    // active field is optional for migration
    pub active: Option<bool>,
}

// Convert legacy reminder to current reminder with default values
impl From<LegacyReminder> for Reminder {
    fn from(legacy: LegacyReminder) -> Self {
        Self {
            id: legacy.id,
            name: legacy.name,
            interval: legacy.interval,
            interval_value: legacy.interval_value,
            specific_date: legacy.specific_date,
            specific_time: legacy.specific_time,
            color: legacy.color,
            created_at: legacy.created_at,
            last_notified: legacy.last_notified,
            // Default to true for existing reminders
            active: legacy.active.unwrap_or(true),
        }
    }
}

use serde_json::{Map, Value};

pub type AppSettings = Map<String, Value>;

fn default_settings() -> AppSettings {
    let mut map = Map::new();
    map.insert("autostartEnabled".to_string(), Value::Bool(false));
    map.insert("theme".to_string(), Value::Null);
    map.insert("notificationSound".to_string(), Value::Bool(true));
    map
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub reminders: Vec<Reminder>,
    pub settings: AppSettings,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            reminders: Vec::new(),
            settings: default_settings(),
        }
    }
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

    let json_data = fs::read_to_string(&file_path)?;
    
    // Try to load as current format first
    if let Ok(app_data) = serde_json::from_str::<AppData>(&json_data) {
        return Ok(app_data);
    }
    
    // If that fails, try migration from legacy format
    match migrate_app_data(&json_data) {
        Ok(migrated_data) => {
            // Save the migrated data back to file
            save_app_data(app, &migrated_data)?;
            println!("Successfully migrated app data to current format");
            Ok(migrated_data)
        }
        Err(e) => {
            println!("Failed to migrate app data: {}", e);
            // If migration fails, return default data
            Ok(AppData::default())
        }
    }
}

// Migration function to handle legacy data formats
fn migrate_app_data(json_data: &str) -> Result<AppData, Error> {
    // Parse as Value first to handle partial structures
    let mut data: serde_json::Value = serde_json::from_str(json_data)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    // Migrate reminders if they exist
    if let Some(reminders_value) = data.get_mut("reminders") {
        if let Some(reminders_array) = reminders_value.as_array_mut() {
            for reminder_value in reminders_array {
                // Add missing fields with default values
                if let Some(reminder_obj) = reminder_value.as_object_mut() {
                    // Add 'active' field if missing (default to true)
                    if !reminder_obj.contains_key("active") {
                        reminder_obj.insert("active".to_string(), serde_json::Value::Bool(true));
                    }
                    
                    // Add other future fields here as needed
                    // Example: if !reminder_obj.contains_key("newField") { ... }
                }
            }
        }
    }
    
    // Ensure settings exist
    if !data.as_object().unwrap().contains_key("settings") {
        data.as_object_mut().unwrap().insert("settings".to_string(), serde_json::to_value(default_settings()).unwrap());
    }
    
    // Parse the migrated data as AppData
    let app_data: AppData = serde_json::from_value(data)
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
    let mut app_data = load_app_data(&app).unwrap_or_default();
    app_data.settings = settings;
    save_app_data(&app, &app_data)?;
    Ok(())
}

#[tauri::command]
pub fn load_settings(app: AppHandle) -> AppSettings {
    let app_data = load_app_data(&app).unwrap_or_default();
    app_data.settings
}

#[tauri::command]
pub fn update_setting(app: AppHandle, key: String, value: Value) -> Result<(), Error> {
    let mut app_data = load_app_data(&app).unwrap_or_default();
    app_data.settings.insert(key.clone(), value.clone());
    save_app_data(&app, &app_data)?;
    Ok(())
}

#[tauri::command]
pub fn get_setting(app: AppHandle, key: String) -> Option<Value> {
    let app_data = load_app_data(&app).unwrap_or_default();
    app_data.settings.get(&key).cloned()
}