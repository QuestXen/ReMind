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

fn get_reminders_file_path(app: &AppHandle) -> Result<PathBuf, Error> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    // Create directory if it doesn't exist
    fs::create_dir_all(&app_data_dir)?;
    
    Ok(app_data_dir.join("reminders.json"))
}

#[tauri::command]
pub fn save_reminders(app: AppHandle, reminders: Vec<Reminder>) -> Result<(), Error> {
    let file_path = get_reminders_file_path(&app)?;
    let json_data = serde_json::to_string_pretty(&reminders)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    fs::write(file_path, json_data)?;
    Ok(())
}

#[tauri::command]
pub fn load_reminders(app: AppHandle) -> Result<Vec<Reminder>, Error> {
    let file_path = get_reminders_file_path(&app)?;
    
    if !file_path.exists() {
        return Ok(Vec::new());
    }
    
    let json_data = fs::read_to_string(file_path)?;
    let reminders: Vec<Reminder> = serde_json::from_str(&json_data)
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    Ok(reminders)
}

#[tauri::command]
pub fn delete_reminder(app: AppHandle, reminder_id: String) -> Result<(), Error> {
    let mut reminders = load_reminders(app.clone())?;
    reminders.retain(|r| r.id != reminder_id);
    save_reminders(app, reminders)?;
    Ok(())
}

#[tauri::command]
pub fn add_reminder(app: AppHandle, reminder: Reminder) -> Result<(), Error> {
    let mut reminders = load_reminders(app.clone())?;
    reminders.push(reminder);
    save_reminders(app, reminders)?;
    Ok(())
}

#[tauri::command]
pub fn update_reminder(app: AppHandle, reminder: Reminder) -> Result<(), Error> {
    let mut reminders = load_reminders(app.clone())?;
    
    if let Some(existing_reminder) = reminders.iter_mut().find(|r| r.id == reminder.id) {
        *existing_reminder = reminder;
        save_reminders(app, reminders)?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn update_reminder_last_notified(app: AppHandle, reminder_id: String, timestamp: String) -> Result<(), Error> {
    let mut reminders = load_reminders(app.clone())?;
    
    if let Some(reminder) = reminders.iter_mut().find(|r| r.id == reminder_id) {
        reminder.last_notified = Some(timestamp);
        save_reminders(app, reminders)?;
    }
    
    Ok(())
}