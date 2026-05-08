pub mod config;
pub mod document;
pub mod hotkey;
pub mod inference;
pub mod storage;

pub use config::{ConfigService, AppConfig};
pub use document::{extract_text_from_file, ExtractedDocument, DocumentError};
pub use inference::{DefaultEngine, InferenceEngine, GenerationParams, create_engine};
pub use storage::{StorageService, ChatHistory, ChatMessage, Document, Template};
