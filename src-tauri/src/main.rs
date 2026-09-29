// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use tauri::{Manager, State};

/// Resolved engine paths, shared with all commands.
struct EnginePaths {
    piper: PathBuf,
    model: PathBuf,
}

fn candidate_files(dir: &Path) -> (PathBuf, PathBuf) {
    (
        dir.join("piper").join("piper"),
        dir.join("models").join("fa_IR-mana-medium.onnx"),
    )
}

/// Resolve piper binary + model:
/// 1. `MANA_PIPER_BIN` / `MANA_MODEL` env overrides (dev/testing)
/// 2. Tauri resource dir (bundled app)
/// 3. Repo-relative fallback (`tools/`, `models/` for `cargo tauri dev`)
fn resolve_paths(app: &tauri::AppHandle) -> EnginePaths {
    if let (Some(piper), Some(model)) = (
        std::env::var_os("MANA_PIPER_BIN").map(PathBuf::from),
        std::env::var_os("MANA_MODEL").map(PathBuf::from),
    ) {
        return EnginePaths { piper, model };
    }
    if let Ok(res) = app.path().resource_dir() {
        let (piper, model) = candidate_files(&res);
        if piper.exists() && model.exists() {
            return EnginePaths { piper, model };
        }
        // dev layout: resources land directly under target/... — also try parent
        if let Some(parent) = res.parent() {
            let (piper, model) = candidate_files(parent);
            if piper.exists() && model.exists() {
                return EnginePaths { piper, model };
            }
        }
    }
    EnginePaths {
        piper: mana_tts::tts::piper_bin(),
        model: mana_tts::tts::model_path(),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SynthResult {
    /// Temp WAV file path (also playable reference).
    out_path: String,
    /// Base64-encoded WAV bytes for in-app playback.
    wav_base64: String,
    sample_rate: u32,
    duration_secs: f32,
    /// Downsampled waveform peaks (0.0..=1.0) for the canvas visualizer.
    peaks: Vec<f32>,
}

fn waveform_peaks(wav: &Path, buckets: usize) -> Result<(u32, Vec<f32>), String> {
    let mut reader = hound::WavReader::open(wav).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let samples: Vec<i32> = reader
        .samples::<i32>()
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    if samples.is_empty() {
        return Err("خروجی خالی است".to_string());
    }
    let per_bucket = (samples.len() / buckets).max(1);
    let peaks = samples
        .chunks(per_bucket)
        .map(|c| {
            c.iter().map(|s| s.abs() as f32 / i32::MAX as f32).fold(0.0, f32::max)
        })
        .collect();
    Ok((spec.sample_rate, peaks))
}

#[tauri::command]
fn synthesize(
    text: String,
    speed: f32,
    paths: State<'_, EnginePaths>,
) -> Result<SynthResult, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("متن خالی است".to_string());
    }
    if text.chars().count() > 2000 {
        return Err("متن بیش از حد طولانی است (حداکثر ۲۰۰۰ نویسه)".to_string());
    }

    let mut out = std::env::temp_dir();
    out.push(format!("manatts-{}-{}.wav", std::process::id(), text.len()));

    mana_tts::tts::synthesize_with(&paths.piper, &paths.model, text, speed, &out)
        .map_err(|e| e.to_string())?;

    let bytes = std::fs::read(&out).map_err(|e| e.to_string())?;
    let (sample_rate, peaks) = waveform_peaks(&out, 400)?;
    let duration_secs = hound::WavReader::open(&out)
        .map(|r| r.duration() as f32 / r.spec().sample_rate as f32)
        .unwrap_or(0.0);

    Ok(SynthResult {
        out_path: out.to_string_lossy().into_owned(),
        wav_base64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes),
        sample_rate,
        duration_secs,
        peaks,
    })
}

#[tauri::command]
fn save_audio(wav_base64: String, dir: String, filename: String) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(wav_base64.trim())
        .map_err(|_| "داده صوتی نامعتبر است".to_string())?;
    if bytes.len() < 1000 {
        return Err("داده صوتی خراب است".to_string());
    }
    let safe_name: String = filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let name = if safe_name.ends_with(".wav") {
        safe_name
    } else {
        format!("{safe_name}.wav")
    };
    let dest = PathBuf::from(&dir).join(name);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().into_owned())
}

#[tauri::command]
fn default_dir(app: tauri::AppHandle) -> String {
    if let Ok(doc) = app.path().document_dir() {
        return doc.join("ManaTTS").to_string_lossy().into_owned();
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .to_string_lossy()
        .into_owned()
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let paths = resolve_paths(app.handle());
            app.manage(paths);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![synthesize, save_audio, default_dir])
        .run(tauri::generate_context!())
        .expect("error while running ManaTTS");
}
