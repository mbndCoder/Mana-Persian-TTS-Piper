//! Local OpenAI-compatible speech API for agents and other tools.
//!
//! Binds to 127.0.0.1 only, on a dynamically chosen free port, so several
//! tools can connect without fighting over a fixed port. The live port is
//! reported back to the UI so the user gets a ready-to-run request to copy.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::State as AxumState;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

pub const MODEL_NAME: &str = "fa-IR-mana-medium";

// ---------------------------------------------------------------- app commands

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiInfo {
    pub running: bool,
    pub port: Option<u16>,
    pub url: String,
    pub curl: String,
}

impl ApiInfo {
    pub fn stopped() -> Self {
        Self {
            running: false,
            port: None,
            url: String::new(),
            curl: String::new(),
        }
    }

    pub fn for_port(p: u16) -> Self {
        let url = format!("http://127.0.0.1:{p}/v1/audio/speech");
        Self {
            running: true,
            port: Some(p),
            url: url.clone(),
            curl: format!(
                "curl -X POST {url} \\\n  -H 'Content-Type: application/json' \\\n  -d '{{\"input\": \"سلام دنیا\", \"voice\": \"mana\", \"speed\": 1.0, \"response_format\": \"wav\"}}' \\\n  --output speech.wav"
            ),
        }
    }
}

// ------------------------------------------------------------------- the server

#[derive(Clone)]
struct ApiState {
    piper: PathBuf,
    model: PathBuf,
}

#[derive(Deserialize)]
struct SpeechRequest {
    input: String,
    #[serde(default)]
    #[allow(dead_code)]
    voice: Option<String>,
    #[serde(default)]
    speed: Option<f32>,
    #[serde(default)]
    response_format: Option<String>,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    model: &'static str,
    voice: &'static str,
    sample_rate: u32,
    speed_range: [f32; 2],
}

async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "ManaTTS",
        "openai_compatible": "/v1/audio/speech",
        "health": "/health",
        "speed_range": [0.25, 4.0],
        "example": {
            "method": "POST",
            "path": "/v1/audio/speech",
            "body": {
                "input": "سلام دنیا",
                "voice": "mana",
                "speed": 1.0,
                "response_format": "wav"
            }
        }
    }))
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        model: MODEL_NAME,
        voice: "mana",
        sample_rate: 22050,
        speed_range: [0.25, 4.0],
    })
}

async fn speech(AxumState(st): AxumState<ApiState>, Json(req): Json<SpeechRequest>) -> Response {
    let text = req.input.trim().to_string();
    if text.is_empty() {
        return (StatusCode::BAD_REQUEST, "input must not be empty").into_response();
    }
    if text.chars().count() > 20_000 {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            "input too long (max 20000 chars)",
        )
            .into_response();
    }
    // The engine only produces WAV; say so instead of returning a wrong container.
    if let Some(fmt) = &req.response_format {
        let f = fmt.to_ascii_lowercase();
        if f != "wav" && f != "wave" && f != "pcm" {
            return (
                StatusCode::NOT_IMPLEMENTED,
                format!("response_format '{fmt}' is not supported; use 'wav'"),
            )
                .into_response();
        }
    }

    let speed = req.speed.unwrap_or(1.0).clamp(0.25, 4.0);
    let piper = st.piper.clone();
    let model = st.model.clone();

    // piper runs as a child process: keep it off the async runtime threads.
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut out = std::env::temp_dir();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        out.push(format!("manatts-api-{}-{stamp}.wav", std::process::id()));
        mana_tts::tts::synthesize_with(&piper, &model, &text, speed, &out)
            .map_err(|e| e.to_string())
            .and_then(|_| std::fs::read(&out).map_err(|e| e.to_string()))
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string()));

    match result {
        Ok(bytes) if !bytes.is_empty() => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "audio/wav".to_string()),
                (header::CONTENT_LENGTH, bytes.len().to_string()),
                (header::CACHE_CONTROL, "no-store".to_string()),
            ],
            bytes,
        )
            .into_response(),
        Ok(_) => (StatusCode::INTERNAL_SERVER_ERROR, "empty audio produced").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

/// A running server plus the channel that stops it.
pub struct RunningServer {
    pub port: u16,
    shutdown: Option<oneshot::Sender<()>>,
    flag: Arc<AtomicBool>,
}

impl RunningServer {
    pub fn stop(mut self) {
        self.flag.store(false, Ordering::SeqCst);
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

/// Bind a free localhost port and serve until stopped.
pub async fn start(piper: PathBuf, model: PathBuf) -> Result<RunningServer, String> {
    let listener = tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .map_err(|e| format!("cannot bind localhost: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let app = Router::new()
        .route("/v1/audio/speech", post(speech))
        .route("/health", get(health))
        .route("/", get(root))
        .with_state(ApiState { piper, model });

    let (tx, rx) = oneshot::channel();
    let flag = Arc::new(AtomicBool::new(true));

    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await;
    });

    Ok(RunningServer {
        port,
        shutdown: Some(tx),
        flag,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_info_exposes_url_and_curl() {
        let info = ApiInfo::for_port(8123);
        assert!(info.running);
        assert_eq!(info.url, "http://127.0.0.1:8123/v1/audio/speech");
        assert!(info.curl.contains("/v1/audio/speech"));
        assert!(info.curl.contains("\"speed\""));
        assert!(info.curl.contains("--output speech.wav"));
    }

    #[test]
    fn stopped_api_has_no_url() {
        let info = ApiInfo::stopped();
        assert!(!info.running);
        assert!(info.url.is_empty());
    }
}
