use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};

use crate::api::routes::AppState;

pub async fn websocket_handler(
    State(_state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|mut socket| async move {
        let _ = socket.send(axum::extract::ws::Message::Text(
            "WebSocket connected".to_string()
        )).await;
    })
}
