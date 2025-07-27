mod commands;
use commands::default::{read, write};
use commands::notifications::{send_notification, request_permission};
use commands::reminders::{save_reminders, load_reminders, delete_reminder, add_reminder, update_reminder, update_reminder_last_notified};
use commands::tray::{show_window, hide_window, quit_app, setup_system_tray, handle_window_event};
use tauri::Manager;

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            
            // Setup system tray
            setup_system_tray(&app.handle()).expect("Failed to setup system tray");
            
            Ok(())
        })
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Wenn eine zweite Instanz gestartet wird, zeige das Hauptfenster
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .on_window_event(|window, event| {
            handle_window_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            read, write, 
            send_notification, request_permission,
            save_reminders, load_reminders, delete_reminder, add_reminder, update_reminder, update_reminder_last_notified,
            show_window, hide_window, quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
