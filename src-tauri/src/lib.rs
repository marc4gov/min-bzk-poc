mod bridges;
mod commands;
pub mod errors;
pub mod services;

use services::{StorageService, DefaultEngine, ConfigService};
use commands::{ChatState, DocumentState, ConfigState};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Initialize tracing
            tracing_subscriber::fmt::init();

            // Get config directory
            let config_dir = app.path().app_config_dir()
                .expect("Failed to get config dir");
            std::fs::create_dir_all(&config_dir).expect("Failed to create config dir");

            // Get data directory
            let data_dir = app.path().app_data_dir()
                .expect("Failed to get data dir");
            std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

            // Initialize services using block_on
            let db_path = data_dir.join("localassistant.db");
            let storage = tauri::async_runtime::block_on(async {
                StorageService::new(db_path).await
                    .expect("Failed to init storage")
            });

            let config_service = tauri::async_runtime::block_on(async {
                ConfigService::new(config_dir).await
                    .expect("Failed to init config")
            });

            let storage = Arc::new(storage);
            let config = Arc::new(Mutex::new(config_service));
            let engine = Arc::new(Mutex::new(services::create_engine()));

            // Create state structs
            let chat_state = ChatState {
                storage: storage.clone(),
                engine,
            };
            let document_state = DocumentState {
                storage,
            };
            let config_state = ConfigState {
                config,
            };

            // Store in app state for commands
            app.manage(chat_state);
            app.manage(document_state);
            app.manage(config_state);

            // Register global hotkey for window toggle (Cmd+Shift+A)
            let app_handle = app.handle().clone();
            let shortcut = Shortcut::new(
                Some(tauri_plugin_global_shortcut::Modifiers::SUPER | tauri_plugin_global_shortcut::Modifiers::SHIFT),
                tauri_plugin_global_shortcut::Code::KeyA,
            );

            if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
                if let Some(window) = app_handle.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }) {
                eprintln!("Failed to register global hotkey: {}", e);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::chat::send_message,
            commands::chat::get_histories,
            commands::chat::get_history,
            commands::chat::delete_history,
            commands::document::extract_text,
            commands::document::get_documents,
            commands::config::get_config,
            commands::config::update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
