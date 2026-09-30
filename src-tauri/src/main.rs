// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;

use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager, State};

/// Resolved engine paths, shared with all commands.
pub struct EnginePaths {
    piper: PathBuf,
    model: PathBuf,
}

/// Locate the engine inside a bundle root.
///
/// Real layouts vary:
/// * `<root>/tools/piper/piper` — flat bundle
/// * `<root>/_up_/tools/piper/piper` — `_up_` is how tauri-bundler encodes a
///   `..` resource path, so this is what AppImage/deb actually ship
/// * `<root>/<AppName>/_up_/tools/piper/piper` — one level deeper again,
///   because `resource_dir()` resolves to `/usr/lib/<cargo package name>`
///   while the bundler writes to `/usr/lib/<productName>`
///
/// So search `root`, its parent chain, and one level down at each step; and at
/// every candidate also look inside a `_up_` subdirectory.
fn find_in(root: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut dirs = vec![root.to_path_buf()];
    let mut up = root;
    for _ in 0..3 {
        if let Some(parent) = up.parent() {
            dirs.push(parent.to_path_buf());
            up = parent;
        } else {
            break;
        }
    }
    // one level down at every candidate level (AppImage `_up_`, lib dirs, …)
    let mut candidates = dirs.clone();
    for d in &dirs {
        if let Ok(entries) = std::fs::read_dir(d) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    candidates.push(p);
                }
            }
        }
    }
    // …and the `_up_` encoding of a `..` resource path at every candidate,
    // which is one level too deep for the plain child search above.
    let ups: Vec<PathBuf> = candidates.iter().map(|d| d.join("_up_")).collect();
    candidates.extend(ups);

    candidates.iter().find_map(|d| find_shallow(d))
}

/// Look for the engine directly under `dir` (no recursion).
fn find_shallow(dir: &Path) -> Option<(PathBuf, PathBuf)> {
    const PIPER_REL: [&str; 3] = ["tools/piper/piper", "piper/piper", "piper"];
    const MODEL_REL: [&str; 2] = ["models/fa_IR-mana-medium.onnx", "fa_IR-mana-medium.onnx"];
    let piper = PIPER_REL.iter().map(|p| dir.join(p)).find(|p| p.is_file())?;
    let model = MODEL_REL
        .iter()
        .map(|p| piper.parent().map(|d| d.join(p)))
        .chain(MODEL_REL.iter().map(|p| Some(dir.join(p))))
        .flatten()
        .find(|p| p.is_file())?;
    Some((piper, model))
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
        if let Some((piper, model)) = find_in(&res) {
            return EnginePaths { piper, model };
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

/// Open the native folder picker.
///
/// Uses the callback form on purpose: the blocking_* dialog methods deadlock
/// the event loop when called from the main thread, which is where a sync
/// Tauri command runs. This is the documented cross-platform pattern.
#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .set_title("انتخاب پوشه خروجی")
        .pick_folder(move |folder| {
            let _ = tx.send(folder);
        });

    // Blocks this async worker (not the main thread) until the user answers.
    let picked = tauri::async_runtime::spawn_blocking(move || rx.recv().ok())
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "دیالوگ پوشه بسته شد".to_string())?;

    Ok(picked
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned()))
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

// ------------------------------------------------------------- API commands
//
// The Tauri command macros must live in the crate root, so the thin wrappers
// are here while the server itself lives in `api`.

struct ApiServer(std::sync::Mutex<Option<api::RunningServer>>);

fn api_out_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .document_dir()
        .map(|d| d.join("ManaTTS"))
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn api_snapshot(app: &tauri::AppHandle, server: &ApiServer) -> api::ApiInfo {
    let dir = api_out_dir(app);
    server
        .0
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|s| s.port))
        .map(|p| api::ApiInfo::for_port(p, &dir))
        .unwrap_or_else(api::ApiInfo::stopped)
}

#[tauri::command]
fn api_status(app: tauri::AppHandle, server: State<'_, ApiServer>) -> api::ApiInfo {
    api_snapshot(&app, &server)
}

#[tauri::command]
async fn api_start(
    app: tauri::AppHandle,
    server: State<'_, ApiServer>,
) -> Result<api::ApiInfo, String> {
    if server.0.lock().map(|g| g.is_some()).unwrap_or(true) {
        return Ok(api_snapshot(&app, &server));
    }
    let paths = app.state::<EnginePaths>();
    let running = api::start(paths.piper.clone(), paths.model.clone()).await?;
    let dir = api_out_dir(&app);
    let info = api::ApiInfo::for_port(running.port, &dir);
    if let Ok(mut g) = server.0.lock() {
        *g = Some(running);
    }
    let _ = app.emit("api-port", info.port);
    let _ = std::fs::create_dir_all(&dir);
    Ok(info)
}

#[tauri::command]
fn api_stop(app: tauri::AppHandle, server: State<'_, ApiServer>) -> api::ApiInfo {
    if let Ok(mut g) = server.0.lock() {
        if let Some(s) = g.take() {
            s.stop();
        }
    }
    let _ = app.emit("api-port", serde_json::Value::Null);
    api::ApiInfo::stopped()
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
            app.manage(ApiServer(std::sync::Mutex::new(None)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            synthesize,
            play,
            save_audio,
            default_dir,
            pick_folder,
            api_status,
            api_start,
            api_stop
        ])
        .run(tauri::generate_context!())
        .expect("error while running ManaTTS");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Serializes the find_in tests: they all scan the shared temp-dir parent,
    /// so parallel runs see each other's fixture directories.
    static FIND_IN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_root(prefix: &str) -> std::path::PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
    }

    /// Take the find_in lock without caring whether an earlier test panicked.
    /// A poisoned lock made every other test report `PoisonError` instead of
    /// its own real failure, hiding the actual bug.
    fn lock_find_in() -> std::sync::MutexGuard<'static, ()> {
        FIND_IN_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Regression: the real bundle layout is <root>/tools/piper/piper and
    /// <root>/models/*.onnx, but the resolver used to look for <root>/piper/piper,
    /// so the packaged app could never find its engine.
    #[test]
    fn find_in_matches_the_real_bundle_layout() {
        let _guard = lock_find_in();
        let root = unique_root("manatts-layout");
        let piper_dir = root.join("tools/piper");
        let model_dir = root.join("models");
        std::fs::create_dir_all(&piper_dir).unwrap();
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(piper_dir.join("piper"), b"x").unwrap();
        std::fs::write(model_dir.join("fa_IR-mana-medium.onnx"), b"x").unwrap();

        let (piper, model) = find_in(&root).expect("engine must be found in bundle layout");
        assert_eq!(piper, piper_dir.join("piper"));
        assert_eq!(model, model_dir.join("fa_IR-mana-medium.onnx"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn find_in_matches_the_appimage_up_layout() {
        let _guard = lock_find_in();
        // Real AppImage layout: <resource_dir>/_up_/tools/piper/piper
        let root = unique_root("manatts-up");
        let up = root.join("_up_");
        let piper_dir = up.join("tools/piper");
        let model_dir = up.join("models");
        std::fs::create_dir_all(&piper_dir).unwrap();
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(piper_dir.join("piper"), b"x").unwrap();
        std::fs::write(model_dir.join("fa_IR-mana-medium.onnx"), b"x").unwrap();

        // resource_dir() in an AppImage points next to the binary, i.e. at a
        // sibling level like <mount>/usr/bin; simulate by searching from a
        // directory that only contains an unrelated entry.
        let probe = root.join("bin");
        std::fs::create_dir_all(&probe).unwrap();
        std::fs::write(probe.join("mana-tts-app"), b"x").unwrap();

        let (piper, model) = find_in(&probe).expect("engine must be found via parent+child search");
        assert_eq!(piper, piper_dir.join("piper"));
        assert_eq!(model, model_dir.join("fa_IR-mana-medium.onnx"));
        std::fs::remove_dir_all(&root).ok();
    }

    /// Regression: this is the layout the published .deb and AppImage
    /// actually have.
    ///
    /// `tauri-utils::resource_dir` resolves the Linux resource directory as
    /// `<exe_dir>/../lib/<cargo package name>` = `/usr/lib/mana-tts-app`, and
    /// falls back to the literal `/usr/lib/<package name>` off AppImage. The
    /// bundler, however, writes the files to `/usr/lib/<productName>` — so the
    /// path Tauri hands back does not exist. The files live one level deeper:
    /// `/usr/lib/ManaTTS/_up_/{tools,models}`.
    ///
    /// Guards the naming assumption behind the bug above.
    ///
    /// tauri-utils derives the Linux resource directory from the **Cargo
    /// package name** while tauri-bundler writes resources under the
    /// **productName**. Those two differ here, which is exactly why
    /// `resource_dir()` pointed somewhere empty. `find_in` copes with it by
    /// probing `_up_`, so this test only records the divergence and fails if
    /// someone renames the product without thinking about the resolver.
    #[test]
    fn product_name_and_cargo_package_name_are_recorded() {
        let cfg = include_str!("../tauri.conf.json");
        let product = cfg
            .split("\"productName\"")
            .nth(1)
            .and_then(|s| s.split('"').nth(1))
            .expect("productName in tauri.conf.json");

        // From Cargo.toml: name = "mana-tts-app"
        assert_eq!(
            product, "ManaTTS",
            "productName changed; re-check that find_in still covers where \
             tauri-bundler writes resources vs what resource_dir() returns"
        );
    }

    /// The old resolver only descended one level from the resource dir, so it
    /// stopped at `/usr/lib/ManaTTS` and never saw the `_up_` component.
    #[test]
    fn find_in_finds_the_packaged_deb_layout() {
        let _guard = lock_find_in();
        let root = unique_root("manatts-deb");
        // Resources exactly where tauri-bundler puts them.
        let app_dir = root.join("usr/lib/ManaTTS/_up_");
        let piper_dir = app_dir.join("tools/piper");
        std::fs::create_dir_all(&piper_dir).unwrap();
        std::fs::create_dir_all(app_dir.join("models")).unwrap();
        std::fs::write(piper_dir.join("piper"), b"x").unwrap();
        std::fs::write(
            app_dir.join("models/fa_IR-mana-medium.onnx"),
            b"x",
        )
        .unwrap();
        // The executable dir that resource_dir() actually starts from.
        std::fs::create_dir_all(root.join("usr/bin")).unwrap();

        // Tauri returns /usr/lib/<package name>, which does not exist.
        let probe = root.join("usr/lib/mana-tts-app");
        assert!(
            !probe.exists(),
            "probe models Tauri's non-existent resource dir"
        );

        let (piper, model) =
            find_in(&probe).expect("engine must be found in the packaged deb layout");
        assert_eq!(piper, piper_dir.join("piper"));
        assert_eq!(model, app_dir.join("models/fa_IR-mana-medium.onnx"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn find_in_also_accepts_a_flat_layout() {
        let _guard = lock_find_in();
        let root = unique_root("manatts-flat");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("piper"), b"x").unwrap();
        std::fs::write(root.join("fa_IR-mana-medium.onnx"), b"x").unwrap();
        assert!(find_in(&root).is_some(), "flat layout must be supported");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn find_in_returns_none_when_engine_is_absent() {
        let _guard = lock_find_in();
        let root = unique_root("manatts-empty");
        std::fs::create_dir_all(&root).unwrap();
        assert!(find_in(&root).is_none());
        std::fs::remove_dir_all(&root).ok();
    }

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
