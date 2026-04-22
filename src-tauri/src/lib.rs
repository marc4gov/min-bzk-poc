mod commands;
mod errors;
mod services;

use services::{ConfigService, StorageService};
use std::sync::Arc;
use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize tracing
            tracing_subscriber::fmt::init();

            // Get app data directory
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");

            // Use a blocking runtime for async initialization
            let runtime = tokio::runtime::Runtime::new()
                .expect("Failed to create runtime");

            // Initialize storage service
            let db_path = app_data_dir.join("localassistant.db");
            let storage = runtime.block_on(StorageService::new(&db_path))
                .expect("Failed to initialize storage service");

            // Initialize config service
            let config_dir = app_data_dir.join("config");
            let config = runtime.block_on(ConfigService::new(&config_dir))
                .expect("Failed to initialize config service");

            app.manage(Arc::new(storage));
            app.manage(Arc::new(config));

            tracing::info!("LocalAssistant initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::create_conversation,
            commands::get_conversations,
            commands::get_conversation,
            commands::add_message,
            commands::delete_conversation,
            commands::get_templates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
