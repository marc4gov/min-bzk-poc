//! Live broadcast van mesh-stappen naar WebSocket-clients (`AgentEvent::Mesh`).

use tokio::sync::broadcast;

/// Stuurt een mesh-event naar gekoppelde clients; bij ontbrekende zender wordt niets gedaan.
pub fn emit_mesh(
    tx: &Option<broadcast::Sender<crate::AgentEvent>>,
    actor: &str,
    phase: &str,
    detail: &str,
) {
    emit_mesh_with_model(tx, actor, phase, detail, None);
}

/// Stuurt een mesh-event met model-informatie.
pub fn emit_mesh_with_model(
    tx: &Option<broadcast::Sender<crate::AgentEvent>>,
    actor: &str,
    phase: &str,
    detail: &str,
    model: Option<&str>,
) {
    let Some(sender) = tx else {
        return;
    };
    let _ = sender.send(crate::AgentEvent::Mesh {
        actor: actor.to_string(),
        phase: phase.to_string(),
        detail: clamp_detail(detail, 480),
        model: model.map(|m| m.to_string()),
        timestamp: chrono::Utc::now().timestamp(),
    });
}

fn clamp_detail(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    let take = max_chars.saturating_sub(1);
    s.chars().take(take).collect::<String>() + "…"
}
