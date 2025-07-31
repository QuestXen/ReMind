mod commands;
use commands::app_data::{
    add_reminder, delete_reminder, load_reminders, save_reminders, update_reminder,
    update_reminder_preserve_timer, update_reminder_last_notified, load_settings, save_settings, update_setting, get_setting,
};
use commands::default::{read, write};
use commands::notifications::{
    request_permission, send_notification, send_notification_with_sound, 
    send_reminder_notification, test_notification_sound, send_notification_with_settings,
    test_notification_with_settings
};
use commands::system_info::get_system_info;
use commands::tray::{handle_window_event, hide_window, quit_app, setup_system_tray, show_window, check_update_from_tray, update_tray_menu};
use commands::updater::{check_for_updates, install_update, check_and_install_update};
use commands::timer::{TimerManager, get_timer_status};
use tauri::Manager;

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            setup_system_tray(&app.handle()).expect("Failed to setup system tray");

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.eval(
                    r#"
                    document.addEventListener('DOMContentLoaded', function() {
                        // Disable right-click context menu
                        document.addEventListener('contextmenu', function(e) {
                            e.preventDefault();
                            return false;
                        });
                        
                        // Disable common keyboard shortcuts
                        document.addEventListener('keydown', function(e) {
                            // Disable F12 (Developer Tools)
                            if (e.key === 'F12') {
                                e.preventDefault();
                                return false;
                            }
                            // Disable Ctrl+Shift+I (Developer Tools)
                            if (e.ctrlKey && e.shiftKey && e.key === 'I') {
                                e.preventDefault();
                                return false;
                            }
                            // Disable Ctrl+U (View Source)
                            if (e.ctrlKey && e.key === 'u') {
                                e.preventDefault();
                                return false;
                            }
                            // Disable Ctrl+Shift+C (Inspect Element)
                            if (e.ctrlKey && e.shiftKey && e.key === 'C') {
                                e.preventDefault();
                                return false;
                            }
                            // Disable F5 and Ctrl+R (Refresh)
                            if (e.key === 'F5' || (e.ctrlKey && e.key === 'r')) {
                                e.preventDefault();
                                return false;
                            }
                            // Disable Ctrl+P (Print)
                            if (e.ctrlKey && e.key === 'p') {
                                e.preventDefault();
                                return false;
                            }
                            // Disable Ctrl+S (Save)
                            if (e.ctrlKey && e.key === 's') {
                                e.preventDefault();
                                return false;
                            }
                        });
                    });
                    "#,
                );
            }

            let timer_manager = TimerManager::new(app.handle().clone());
            app.manage(timer_manager.clone());
            
            // Start timer manager in a proper async context
            let timer_manager_clone = timer_manager.clone();
            tauri::async_runtime::spawn(async move {
                timer_manager_clone.start().await;
            });
            Ok(())
        })
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
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
            send_notification_with_sound,
            send_reminder_notification,
            test_notification_sound,
            send_notification_with_settings,
            test_notification_with_settings,
            request_permission,
            save_reminders,
            load_reminders,
            delete_reminder,
            add_reminder,
            update_reminder,
            update_reminder_preserve_timer,
            update_reminder_last_notified,
            load_settings,
            save_settings,
            update_setting,
            get_setting,
            get_system_info,
            show_window,
            hide_window,
            quit_app,
            check_update_from_tray,
            update_tray_menu,
            check_for_updates,
            install_update,
            check_and_install_update,
            get_timer_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
