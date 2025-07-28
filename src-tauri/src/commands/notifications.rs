use tauri_plugin_notification::NotificationExt;
use std::process::Command;
use super::app_data::get_setting;

#[tauri::command]
pub fn send_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .sound("default".to_string())
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn send_notification_with_sound(app: tauri::AppHandle, title: String, body: String, play_sound: bool) -> Result<(), String> {
    let mut builder = app.notification().builder().title(title).body(body);
    
    if play_sound {
        builder = builder.sound("default".to_string());
        
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            
            let _ = Command::new("powershell")
                .args([
                    "-WindowStyle", "Hidden",
                    "-c",
                    "[System.Media.SystemSounds]::Notification.Play()"
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
            
            let _ = Command::new("rundll32")
                .args(["user32.dll,MessageBeep", "0"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }
    }
    
    builder.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn send_reminder_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    // Check if notification sound is enabled in settings
    let sound_enabled = get_setting(app.clone(), "notificationSound".to_string())
        .and_then(|v| v.as_bool())
        .unwrap_or(true); // Default to true if setting not found

    let mut builder = app.notification()
        .builder()
        .title(title)
        .body(body);

    if sound_enabled {
        builder = builder.sound("default".to_string());
        
        // Ensure Windows notification sound plays
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            
            // Play system notification sound using multiple methods
            let _ = Command::new("powershell")
                .args([
                    "-WindowStyle", "Hidden",
                    "-c",
                    "[System.Media.SystemSounds]::Notification.Play()"
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
            
            let _ = Command::new("rundll32")
                .args(["user32.dll,MessageBeep", "0"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }
    }

    builder.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn send_notification_with_settings(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    let sound_enabled = get_setting(app.clone(), "notificationSound".to_string())
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    send_notification_with_sound(app, title, body, sound_enabled)
}

#[tauri::command]
pub fn request_permission(app: tauri::AppHandle) -> Result<String, String> {
    let permission = app
        .notification()
        .request_permission()
        .map_err(|e| e.to_string())?;

    Ok(format!("{:?}", permission))
}

#[tauri::command]
pub fn test_notification_sound(app: tauri::AppHandle) -> Result<(), String> {
    // Test function to verify notification sound works
    send_notification_with_sound(
        app,
        "Test Benachrichtigung".to_string(),
        "Dies ist ein Test der Benachrichtigung mit Sound.".to_string(),
        true
    )
}

#[tauri::command]
pub fn test_notification_with_settings(app: tauri::AppHandle) -> Result<(), String> {
    // Test function that respects user settings
    send_notification_with_settings(
        app,
        "Test Benachrichtigung".to_string(),
        "Dies ist ein Test der Benachrichtigung mit Benutzereinstellungen.".to_string()
    )
}
