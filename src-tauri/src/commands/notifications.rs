use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub fn send_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn request_permission(app: tauri::AppHandle) -> Result<String, String> {
    let permission = app
        .notification()
        .request_permission()
        .map_err(|e| e.to_string())?;

    Ok(format!("{:?}", permission))
}
