mod commands;
use commands::app_data::{
    add_reminder, delete_reminder, load_reminders, save_reminders, update_reminder,
    update_reminder_last_notified, load_settings, save_settings, update_autostart_setting, get_autostart_setting,
};
use commands::default::{read, write};
use commands::notifications::{request_permission, send_notification};
use commands::system_info::get_system_info;
use commands::tray::{handle_window_event, hide_window, quit_app, setup_system_tray, show_window};
use tauri::Manager;

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None
        ))
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
            read,
            write,
            send_notification,
            request_permission,
            save_reminders,
            load_reminders,
            delete_reminder,
            add_reminder,
            update_reminder,
            update_reminder_last_notified,
            load_settings,
            save_settings,
            update_autostart_setting,
            get_autostart_setting,
            get_system_info,
            show_window,
            hide_window,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
