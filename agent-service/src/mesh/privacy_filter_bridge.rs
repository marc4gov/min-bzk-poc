//! HTTP-brug naar een sidecar die het HF-model **[openai/privacy-filter]** draait
//! (<https://huggingface.co/openai/privacy-filter>).
//!
//! Zet `PII_PRIVACY_FILTER_URL` (bv. `http://127.0.0.1:8091`) — zie `./scripts/privacy_filter_server.py`.
//!
//! Dit is géén volledige compliance-garantie; zie ook de HF-modelkaart bij **Bias, Risks, and Limitations**.

use serde::{Deserialize, Serialize};

/// Basis-URL, bv. `http://127.0.0.1:8091`
pub const ENV_PII_PRIVACY_SERVICE: &str = "PII_PRIVACY_FILTER_URL";

const DEFAULT_SCRUB_TIMEOUT_SECS: u64 = 120;

#[derive(Debug, Serialize)]
pub struct PrivacyMaskRequest<'a> {
    pub content: &'a str,
    /// Korte labels zoals `"Email"`, `"PhoneNumber"` vanuit [`crate::mesh::expert::PIICategory`].
    pub categories: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PrivacyMaskResponse {
    pub masked: String,
    #[serde(default)]
    pub entities: Vec<PrivacyEntityHit>,
}

#[derive(Debug, Deserialize)]
pub struct PrivacyEntityHit {
    pub entity_group: String,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub word: Option<String>,
}

pub fn service_base_url() -> Option<String> {
    let v = std::env::var(ENV_PII_PRIVACY_SERVICE).ok()?;
    let v = v.trim().to_owned();
    if v.is_empty() {
        return None;
    }
    Some(v.trim_end_matches('/').to_string())
}

pub fn privacy_model_scrub_enabled() -> bool {
    service_base_url().is_some()
}

fn timeout_secs() -> std::time::Duration {
    let s = std::env::var("PII_PRIVACY_FILTER_TIMEOUT_SECS")
        .ok()
        .and_then(|x| x.parse::<u64>().ok())
        .unwrap_or(DEFAULT_SCRUB_TIMEOUT_SECS);
    std::time::Duration::from_secs(s.max(5))
}

pub async fn mask_via_service(req: PrivacyMaskRequest<'_>) -> Result<PrivacyMaskResponse, String> {
    let base = service_base_url()
        .ok_or_else(|| format!("zet {} op de service-URL", ENV_PII_PRIVACY_SERVICE))?;
    let url = format!("{base}/mask");

    let client = reqwest::Client::builder()
        .timeout(timeout_secs())
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    if !status.is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(format!(
            "privacy-filter service HTTP {} — {}",
            status,
            txt.chars().take(400).collect::<String>()
        ));
    }

    resp.json::<PrivacyMaskResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_request_shape() {
        let req = PrivacyMaskRequest {
            content: "hallo alice@test.nl",
            categories: vec!["Email".to_string()],
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("alice"));
    }
}
