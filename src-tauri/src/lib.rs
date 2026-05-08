mod bridges;
mod commands;
pub mod errors;
pub mod services;

use services::{StorageService, ConfigService};
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
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize tracing with limited output (only errors and warnings)
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::WARN)
                .init();

            // Use project directory for data (development mode)
            let project_dir = std::env::current_dir()
                .expect("Failed to get current dir");
            std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

            let config_dir = project_dir.join("config");
            std::fs::create_dir_all(&config_dir).expect("Failed to create config dir");

            // Initialize services using block_on
            let db_path = project_dir.join("localassistant.db");
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
            commands::chat::delete_all_histories,
            commands::document::extract_text,
            commands::document::get_documents,
            commands::document::upload_document_for_improvement,
            commands::config::get_config,
            commands::config::update_config,
            commands::config::get_templates,
            commands::model::get_available_models,
            commands::model::get_downloaded_models,
            commands::model::download_model,
            commands::model::delete_model,
            commands::agent::mesh_demo,
            commands::agent::mesh_document,
            commands::agent::mesh_improve_document,
            commands::agent::agent_chat,
            commands::agent::agent_health,
            commands::agent::list_agents,
            commands::agent::ollama_models,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
