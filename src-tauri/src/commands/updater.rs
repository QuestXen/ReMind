use super::errors::Error;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    match app.updater() {
        Some(updater) => {
            match updater.check().await {
                Ok(update) => {
                    if let Some(update) = update {
                        Ok(UpdateInfo {
                            available: true,
                            version: Some(update.version),
                            notes: update.body,
                            pub_date: update.date.map(|d| d.to_rfc3339()),
                        })
                    } else {
                        Ok(UpdateInfo {
                            available: false,
                            version: None,
                            notes: None,
                            pub_date: None,
                        })
                    }
                }
                Err(e) => {
                    log::error!("Failed to check for updates: {}", e);
                    Err(format!("Update check failed: {}", e))
                }
            }
        }
        None => Err("Updater not available".to_string()),
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    match app.updater() {
        Some(updater) => {
            match updater.check().await {
                Ok(update) => {
                    if let Some(update) = update {
                        log::info!("Installing update version: {}", update.version);
                        
                        // Download and install update
                        match update.download_and_install(|chunk_length, content_length| {
                            let progress = chunk_length as f64 / content_length.unwrap_or(chunk_length) as f64 * 100.0;
                            log::info!("Download progress: {:.1}%", progress);
                        }, || {
                            log::info!("Download completed, installing...");
                        }).await {
                            Ok(_) => {
                                log::info!("Update installed successfully, restarting...");
                                // The app will restart automatically after installation
                                Ok(())
                            }
                            Err(e) => {
                                log::error!("Failed to install update: {}", e);
                                Err(format!("Update installation failed: {}", e))
                            }
                        }
                    } else {
                        Err("No update available".to_string())
                    }
                }
                Err(e) => {
                    log::error!("Failed to check for updates during installation: {}", e);
                    Err(format!("Update check failed: {}", e))
                }
            }
        }
        None => Err("Updater not available".to_string()),
    }
}

#[tauri::command]
pub async fn check_and_install_update(app: AppHandle) -> Result<UpdateInfo, String> {
    let update_info = check_for_updates(app.clone()).await?;
    
    if update_info.available {
        log::info!("Update available, installing automatically...");
        install_update(app).await?;
    }
    
    Ok(update_info)
}