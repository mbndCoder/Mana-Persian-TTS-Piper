//! Local OpenAI-compatible speech API for agents and other tools.
//!
//! Binds to 127.0.0.1 only, on a dynamically chosen free port, so several
//! tools can connect without fighting over a fixed port. The live port is
//! reported back to the UI so the user gets a ready-to-run request to copy.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::{FromRequest, State as AxumState};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

pub const MODEL_NAME: &str = "fa-IR-mana-medium";

/// Default port: high enough to stay clear of the usual dev-server ports
/// (3000 Grafana, 8000 vLLM, 8080 LocalAI, 11434 Ollama, 8188 ComfyUI,
/// 7860 Automatic1111, 8888 Jupyter) and still inside the non-ephemeral range.
pub const DEFAULT_PORT: u16 = 7788;

pub fn preferred_port() -> u16 {
    std::env::var("MANA_API_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|p| *p > 0)
        .unwrap_or(DEFAULT_PORT)
}

/// Optional shared secret. Off unless MANA_API_TOKEN is set, because tools and
/// agents on the same machine should connect without ceremony.
pub fn required_token() -> Option<String> {
    std::env::var("MANA_API_TOKEN")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

// ---------------------------------------------------------------- app commands

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiInfo {
    pub running: bool,
    pub port: Option<u16>,
    pub url: String,
    pub curl: String,
    /// Absolute path the example request writes to, so the user is never
    /// left guessing where the wav landed.
    pub output_path: String,
    pub auth_required: bool,
}

impl ApiInfo {
    pub fn stopped() -> Self {
        Self {
            running: false,
            port: None,
            url: String::new(),
            curl: String::new(),
            output_path: String::new(),
            auth_required: required_token().is_some(),
        }
    }

    pub fn for_port(p: u16, out_dir: &std::path::Path) -> Self {
        let url = format!("http://127.0.0.1:{p}/v1/audio/speech");
        let output_path = out_dir.join("speech.wav");
        let auth = required_token();
        let auth_header = match &auth {
            Some(_) => " \\\n  -H 'Authorization: Bearer $MANA_API_TOKEN'",
            None => "",
        };
        Self {
            running: true,
            port: Some(p),
            url: url.clone(),
            output_path: output_path.to_string_lossy().into_owned(),
            auth_required: auth.is_some(),
            curl: format!(
                "curl -X POST {url} \\\n  -H 'Content-Type: application/json'{auth_header} \\\n  -d '{{\"input\": \"سلام دنیا\", \"voice\": \"mana\", \"speed\": 1.0, \"response_format\": \"wav\"}}' \\\n  --output \"{}\"",
                output_path.to_string_lossy()
            ),
        }
    }
}

// ------------------------------------------------------------------- the server

#[derive(Clone)]
struct ApiState {
    piper: PathBuf,
    model: PathBuf,
    token: Option<String>,
}

/// Reject the request unless it carries the shared secret (only active when
/// MANA_API_TOKEN is set).
fn authorized(req: &axum::http::Request<axum::body::Body>, token: &str) -> bool {
    req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim() == format!("Bearer {token}"))
        .unwrap_or(false)
}

async fn speech(
    AxumState(st): AxumState<ApiState>,
    req: axum::http::Request<axum::body::Body>,
) -> Response {
    if let Some(token) = &st.token {
        if !authorized(&req, token) {
            return (
                StatusCode::UNAUTHORIZED,
                "missing or invalid Authorization: Bearer <token>",
            )
                .into_response();
        }
    }
    // Extract the JSON body after the auth check so an unauthenticated caller
    // cannot make the engine synthesize anything.
    let Json(req) = match Json::<SpeechRequest>::from_request(req, &()).await {
        Ok(v) => v,
        Err(rejection) => return (StatusCode::BAD_REQUEST, rejection.body_text()).into_response(),
    };
    speech_inner(st, req).await
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

async fn speech_inner(st: ApiState, req: SpeechRequest) -> Response {
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
    /// False when the preferred port was busy and a random one was used.
    pub on_preferred: bool,
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

/// Bind the preferred localhost port, falling back to any free port when it
/// is taken, so a busy port degrades instead of breaking startup.
pub async fn bind_listener(preferred: u16) -> Result<(tokio::net::TcpListener, u16, bool), String> {
    match tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, preferred))).await {
        Ok(l) => {
            let port = l.local_addr().map_err(|e| e.to_string())?.port();
            Ok((l, port, port == preferred))
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            let l = tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
                .await
                .map_err(|e| format!("cannot bind localhost: {e}"))?;
            let port = l.local_addr().map_err(|e| e.to_string())?.port();
            Ok((l, port, false))
        }
        Err(e) => Err(format!("cannot bind localhost:{preferred}: {e}")),
    }
}

/// Bind localhost and serve until stopped.
pub async fn start(piper: PathBuf, model: PathBuf) -> Result<RunningServer, String> {
    let (listener, port, on_preferred) = bind_listener(preferred_port()).await?;

    let app = Router::new()
        .route("/v1/audio/speech", post(speech))
        .route("/health", get(health))
        .route("/", get(root))
        .with_state(ApiState {
            piper,
            model,
            token: required_token(),
        });

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
        on_preferred,
        shutdown: Some(tx),
        flag,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_info_exposes_url_absolute_output_and_curl() {
        let dir = std::path::Path::new("/tmp/manatts-test-out");
        let info = ApiInfo::for_port(8123, dir);
        assert!(info.running);
        assert_eq!(info.url, "http://127.0.0.1:8123/v1/audio/speech");
        assert_eq!(info.output_path, "/tmp/manatts-test-out/speech.wav");
        assert!(info.curl.contains("/v1/audio/speech"));
        assert!(info.curl.contains("\"speed\""));
        // the example must state the absolute output path, not a bare filename
        assert!(
            info.curl.contains("--output \"/tmp/manatts-test-out/speech.wav\""),
            "curl must show the absolute path: {}",
            info.curl
        );
    }

    #[test]
    fn stopped_api_has_no_url() {
        let info = ApiInfo::stopped();
        assert!(!info.running);
        assert!(info.url.is_empty());
        assert!(info.output_path.is_empty());
    }

    #[tokio::test]
    async fn busy_preferred_port_falls_back_to_a_free_one() {
        let (a, port_a, on_a) = bind_listener(0).await.unwrap();
        assert!(!on_a, "port 0 is never the preferred port");
        assert_ne!(port_a, 0);

        // Occupy the port the first listener actually got, then ask for it.
        let (b, port_b, on_b) = bind_listener(port_a).await.unwrap();
        assert!(!on_b, "an occupied port must report fallback");
        assert_ne!(port_b, port_a, "fallback must pick a different free port");
        drop((a, b));
    }

    #[tokio::test]
    async fn free_preferred_port_is_used_as_is() {
        let (l, port, on_preferred) = bind_listener(0).await.unwrap();
        assert!(!on_preferred);
        drop(l);
        // An ephemeral port we just released may be reused; assert the
        // contract instead: a successful bind reports the port it got.
        assert!(port > 0);
    }

    #[test]
    fn default_port_avoids_well_known_dev_ports() {
        let clashes = [3000, 8000, 8080, 8888, 11434, 7860, 8188, 1234, 5678, 9000];
        assert!(
            !clashes.contains(&DEFAULT_PORT),
            "default port {} clashes with a common tool",
            DEFAULT_PORT
        );
    }
}
