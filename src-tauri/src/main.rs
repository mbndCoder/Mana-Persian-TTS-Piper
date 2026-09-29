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
    // Normalize against the file's own full-scale, not i32::MAX: the samples
    // are 16-bit, so dividing by i32::MAX flattens every peak to ~0.
    let full_scale = ((1i64 << (spec.bits_per_sample - 1)) - 1) as f32;
    let per_bucket = (samples.len() / buckets).max(1);
    let peaks: Vec<f32> = samples
        .chunks(per_bucket)
        .map(|c| {
            (c.iter().map(|s| (*s as f32).abs()).fold(0.0, f32::max) / full_scale).min(1.0)
        })
        .collect();
    Ok((spec.sample_rate, peaks))
}

#[tauri::command]
async fn synthesize(
    text: String,
    speed: f32,
    paths: State<'_, EnginePaths>,
) -> Result<SynthResult, String> {
    // Owned so the blocking task can take it by move.
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("متن خالی است".to_string());
    }
    let mut out = std::env::temp_dir();
    out.push(format!("manatts-{}-{}.wav", std::process::id(), text.len()));

    // Guardrail, not a hard limit: piper synthesizes the whole text in one
    // pass, so this is only a sanity bound on runtime and memory.
    const MAX_CHARS: usize = 10_000;
    if text.chars().count() > MAX_CHARS {
        return Err(format!(
            "متن خیلی طولانی است (حداکثر {MAX_CHARS} نویسه)"
        ));
    }

    // Piper runs for seconds; keep it off the UI thread.
    let piper = paths.piper.clone();
    let model = paths.model.clone();
    let out_for_task = out.clone();
    let speed = speed;
    tauri::async_runtime::spawn_blocking(move || {
        mana_tts::tts::synthesize_with(&piper, &model, &text, speed, &out_for_task)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

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

/// Play a previously synthesized WAV through rodio.
///
/// Playback deliberately does NOT go through WebKit's AudioContext: on this
/// platform its audio backend never starts (the context clock stays frozen and
/// every clip is silently dropped). rodio is the path already proven to work.
#[tauri::command]
async fn play(
    app: tauri::AppHandle,
    wav_path: String,
) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    let playing = app.state::<Arc<std::sync::atomic::AtomicBool>>().inner().clone();
    if playing
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("در حال پخش صدای قبلی…".to_string());
    }

    let path = PathBuf::from(&wav_path);
    let result = tauri::async_runtime::spawn_blocking(move || {
        mana_tts::audio::play(&path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string());

    playing.store(false, Ordering::SeqCst);
    result.unwrap_or_else(|e| Err(e))
}

#[tauri::command]
fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .set_title("انتخاب پوشه خروجی")
        .blocking_pick_folder()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned())
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
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let paths = resolve_paths(app.handle());
            app.manage(paths);
            app.manage(std::sync::Arc::new(
                std::sync::atomic::AtomicBool::new(false),
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            synthesize,
            play,
            save_audio,
            default_dir,
            pick_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running ManaTTS");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: peaks were normalized by i32::MAX while samples are 16-bit,
    /// which produced an all-zero (flat) waveform.
    #[test]
    fn waveform_peaks_are_normalized_to_the_files_bit_depth() {
        let mut p = std::env::temp_dir();
        p.push(format!("manatts-peaks-{}.wav", std::process::id()));
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&p, spec).unwrap();
        for i in 0..22050 {
            let v = if i % 200 < 100 { 20000i16 } else { -20000i16 };
            w.write_sample(v).unwrap();
        }
        w.finalize().unwrap();

        let (_rate, peaks) = waveform_peaks(&p, 100).unwrap();
        let loudest = peaks.iter().cloned().fold(0.0f32, f32::max);
        assert!(
            loudest > 0.5,
            "peaks must reflect real amplitude, got max {loudest}"
        );
        assert!(peaks.iter().all(|v| (0.0..=1.0).contains(v)));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn empty_wav_is_rejected() {
        let mut p = std::env::temp_dir();
        p.push(format!("manatts-empty-{}.wav", std::process::id()));
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&p, spec).unwrap();
        w.finalize().unwrap();
        assert!(waveform_peaks(&p, 100).is_err());
        std::fs::remove_file(&p).ok();
    }
}
