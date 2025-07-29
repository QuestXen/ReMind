use super::errors::Error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const CURRENT_DATA_VERSION: u32 = 2;

// Version history:
// v1: Initial format without version field and active field in reminders
// v2: Added version field, active field in reminders, improved settings structure

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
    map.insert("language".to_string(), Value::String("en".to_string()));
    map
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    #[serde(default = "default_version")]
    pub version: u32,
    pub reminders: Vec<Reminder>,
    pub settings: AppSettings,
}

fn default_version() -> u32 {
    CURRENT_DATA_VERSION
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            version: CURRENT_DATA_VERSION,
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

    // Parse as Value first to check version
    let data_value: serde_json::Value = serde_json::from_str(&json_data)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // Check version and migrate if necessary
    let version = data_value.get("version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32; // Default to v1 if no version field

    if version == CURRENT_DATA_VERSION {
        // Current version, try direct deserialization
        match serde_json::from_str::<AppData>(&json_data) {
            Ok(app_data) => Ok(app_data),
            Err(_) => {
                // Even current version might have issues, try migration
                migrate_and_save(app, &json_data, version)
            }
        }
    } else if version < CURRENT_DATA_VERSION {
        // Older version, migrate
        println!("Migrating app data from version {} to version {}", version, CURRENT_DATA_VERSION);
        migrate_and_save(app, &json_data, version)
    } else {
        // Future version, this shouldn't happen but handle gracefully
        println!("Warning: Data version {} is newer than supported version {}. Using default data.", version, CURRENT_DATA_VERSION);
        Ok(AppData::default())
    }
}

fn migrate_and_save(app: &AppHandle, json_data: &str, from_version: u32) -> Result<AppData, Error> {
    match migrate_app_data(json_data, from_version) {
        Ok(migrated_data) => {
            // Create backup before saving migrated data
            create_backup(app, json_data)?;
            
            // Save the migrated data back to file
            save_app_data(app, &migrated_data)?;
            println!("Successfully migrated app data from version {} to version {}", from_version, CURRENT_DATA_VERSION);
            Ok(migrated_data)
        }
        Err(e) => {
            println!("Failed to migrate app data: {}", e);
            // If migration fails, create backup and return default data
            let _ = create_backup(app, json_data);
            Ok(AppData::default())
        }
    }
}

fn create_backup(app: &AppHandle, json_data: &str) -> Result<(), Error> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = app_data_dir.join(format!("app_data_backup_{}.json", timestamp));
    
    fs::write(backup_path, json_data)?;
    println!("Created backup of app data before migration");
    Ok(())
}

// Migration function to handle different data format versions
fn migrate_app_data(json_data: &str, from_version: u32) -> Result<AppData, Error> {
    // Parse as Value first to handle partial structures
    let mut data: serde_json::Value = serde_json::from_str(json_data)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // Apply migrations step by step from the current version to the target version
    let mut current_version = from_version;
    
    while current_version < CURRENT_DATA_VERSION {
        match current_version {
            1 => {
                // Migration from v1 to v2
                migrate_v1_to_v2(&mut data)?;
                current_version = 2;
            }
            // Add future migrations here:
            // 2 => {
            //     migrate_v2_to_v3(&mut data)?;
            //     current_version = 3;
            // }
            _ => {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Unknown migration path from version {}", current_version)
                )));
            }
        }
    }

    // Set the current version
    if let Some(obj) = data.as_object_mut() {
        obj.insert("version".to_string(), serde_json::Value::Number(serde_json::Number::from(CURRENT_DATA_VERSION)));
    }

    // Parse the migrated data as AppData
    let app_data: AppData = serde_json::from_value(data)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(app_data)
}

// Migration from v1 to v2: Add version field, active field to reminders, ensure settings structure
fn migrate_v1_to_v2(data: &mut serde_json::Value) -> Result<(), Error> {
    let obj = data.as_object_mut().ok_or_else(|| {
        Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Invalid JSON structure"))
    })?;

    // Migrate reminders if they exist
    if let Some(reminders_value) = obj.get_mut("reminders") {
        if let Some(reminders_array) = reminders_value.as_array_mut() {
            for reminder_value in reminders_array {
                if let Some(reminder_obj) = reminder_value.as_object_mut() {
                    // Add 'active' field if missing (default to true for existing reminders)
                    if !reminder_obj.contains_key("active") {
                        reminder_obj.insert("active".to_string(), serde_json::Value::Bool(true));
                    }
                    
                    // Ensure all required fields exist with defaults
                    if !reminder_obj.contains_key("lastNotified") {
                        reminder_obj.insert("lastNotified".to_string(), serde_json::Value::Null);
                    }
                }
            }
        }
    } else {
        // No reminders exist, create empty array
        obj.insert("reminders".to_string(), serde_json::Value::Array(vec![]));
    }

    // Ensure settings exist with proper structure
    if !obj.contains_key("settings") {
        obj.insert(
            "settings".to_string(),
            serde_json::to_value(default_settings()).unwrap(),
        );
    } else {
        // Migrate existing settings to ensure all required fields exist
        if let Some(settings_obj) = obj.get_mut("settings").and_then(|s| s.as_object_mut()) {
            // Add missing settings with defaults
            if !settings_obj.contains_key("autostartEnabled") {
                settings_obj.insert("autostartEnabled".to_string(), serde_json::Value::Bool(false));
            }
            if !settings_obj.contains_key("theme") {
                settings_obj.insert("theme".to_string(), serde_json::Value::Null);
            }
            if !settings_obj.contains_key("notificationSound") {
                settings_obj.insert("notificationSound".to_string(), serde_json::Value::Bool(true));
            }
            if !settings_obj.contains_key("language") {
                settings_obj.insert("language".to_string(), serde_json::Value::String("en".to_string()));
            }
        }
    }

    Ok(())
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
