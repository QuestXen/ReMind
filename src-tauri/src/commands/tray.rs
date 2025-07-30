use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent, Emitter};
use crate::commands::updater::check_and_install_update;
use crate::commands::app_data::get_setting;

#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn update_tray_menu(app: AppHandle) -> Result<(), String> {
    let _ = app.remove_tray_by_id("main");
    
    setup_system_tray(&app).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn check_update_from_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();

        let _ = window.emit("update-check-started", ());
    }
    
    // Check and install update
    match check_and_install_update(app.clone()).await {
        Ok(update_info) => {
            if let Some(window) = app.get_webview_window("main") {
                if update_info.available {
                    let _ = window.emit("update-installing", update_info);
                } else {
                    let _ = window.emit("update-not-available", ());
                }
            }
            Ok(())
        }
        Err(e) => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("update-error", e.clone());
            }
            Err(e)
        }
    }
}

fn get_localized_text(app: &AppHandle, key: &str) -> String {
    let language = get_setting(app.clone(), "language".to_string())
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "en".to_string());
    
    match (key, language.as_str()) {
        ("show", "de") => "Anzeigen".to_string(),
        ("show", _) => "Show".to_string(),
        ("update", "de") => "Nach Updates suchen".to_string(),
        ("update", _) => "Check for Updates".to_string(),
        ("quit", "de") => "Beenden".to_string(),
        ("quit", _) => "Quit".to_string(),
        _ => key.to_string(),
    }
}

pub fn setup_system_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_text = get_localized_text(app, "show");
    let update_text = get_localized_text(app, "update");
    let quit_text = get_localized_text(app, "quit");
    
    let show_item = MenuItem::with_id(app, "show", &show_text, true, None::<&str>)?;
    let update_item = MenuItem::with_id(app, "update", &update_text, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", &quit_text, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &update_item, &quit_item])?;

    let _tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(move |app, event| {
            let app_clone = app.clone();
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "update" => {
                    tauri::async_runtime::spawn(async move {
                        let _ = check_update_from_tray(app_clone).await;
                    });
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) -> bool {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = window.hide();
            true
        }
        _ => false,
    }
}
