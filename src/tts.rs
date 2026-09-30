use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum TtsError {
    Io(std::io::Error),
    PiperFailed(String),
    EmptyOutput(PathBuf),
}

impl std::fmt::Display for TtsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TtsError::Io(e) => write!(f, "خطای سیستمی: {e}"),
            TtsError::PiperFailed(stderr) => write!(f, "موتور piper شکست خورد: {stderr}"),
            TtsError::EmptyOutput(p) => write!(f, "خروجی صوتی ساخته نشد: {}", p.display()),
        }
    }
}

impl std::error::Error for TtsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TtsError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TtsError {
    fn from(e: std::io::Error) -> Self {
        TtsError::Io(e)
    }
}

/// Path to the piper binary. Override with `MANA_PIPER_BIN`.
pub fn piper_bin() -> PathBuf {
    std::env::var_os("MANA_PIPER_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tools/piper/piper"))
}

/// Path to the Mana ONNX model. Override with `MANA_MODEL`.
pub fn model_path() -> PathBuf {
    std::env::var_os("MANA_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("models/fa_IR-mana-medium.onnx"))
}

/// Synthesize Persian `text` into a WAV file at `out_wav`.
///
/// `speed` is the speaking rate (1.0 = normal); it maps to piper's
/// `--length-scale` as `1 / speed` and is clamped to 0.5..=2.0.
pub fn synthesize(text: &str, speed: f32, out_wav: &Path) -> Result<(), TtsError> {
    synthesize_with(piper_bin().as_path(), model_path().as_path(), text, speed, out_wav)
}

/// Same as [`synthesize`] but with explicit engine paths (used by the bundled app).
pub fn synthesize_with(
    piper: &Path,
    model: &Path,
    text: &str,
    speed: f32,
    out_wav: &Path,
) -> Result<(), TtsError> {
    let speed = speed.clamp(0.5, 2.0);
    let length_scale = 1.0 / speed;

    let (mut cmd, stdin_bytes) = engine_command(piper, model, out_wav, length_scale, text)?;
    let mut child = cmd
        .stdin(if stdin_bytes.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(bytes) = stdin_bytes {
        child
            .stdin
            .as_mut()
            .expect("stdin piped")
            .write_all(&bytes)?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(TtsError::PiperFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    match std::fs::metadata(out_wav) {
        Ok(meta) if meta.len() > 1000 => Ok(()),
        _ => Err(TtsError::EmptyOutput(out_wav.to_path_buf())),
    }
}

/// Build the engine command.
///
/// Standard usage per backend, exactly as documented upstream:
/// 1. Native `piper` binary (Linux): text on **stdin**
///    (`echo ... | piper -m model -f out`). The C++ side reads raw bytes,
///    so there is no locale-decoding hazard.
/// 2. `python -m piper` (Windows/macOS, `pip install piper-tts`): text as a
///    **argv** argument (`piper -m model -- 'text'`). argv avoids the
///    locale-dependent `sys.stdin` decoding entirely, and the wheel carries
///    its own espeak-ng data.
fn engine_command(
    piper: &Path,
    model: &Path,
    out_wav: &Path,
    length_scale: f32,
    text: &str,
) -> Result<(Command, Option<Vec<u8>>), TtsError> {
    if piper.is_file() {
        // Pass the phonemizer data explicitly. Without this piper falls back to a
        // compiled-in system path (/usr/share/espeak-ng-data) and produces NO audio
        // at all on a machine that has no system-wide espeak-ng-data.
        let espeak_data = piper
            .parent()
            .map(|d| d.join("espeak-ng-data"))
            .filter(|p| p.is_dir())
            .unwrap_or_else(|| PathBuf::from("espeak-ng-data"));

        let mut cmd = Command::new(piper);
        cmd.arg("-m")
            .arg(model)
            .arg("-f")
            .arg(out_wav)
            .arg("--length-scale")
            .arg(length_scale.to_string())
            .arg("--espeak_data")
            .arg(&espeak_data);
        return Ok((cmd, Some(text.as_bytes().to_vec())));
    }

    // Native binary absent (typical on Windows/macOS): use the official
    // piper-tts Python package (README documents `pip install piper-tts`).
    let python = python_interpreter().ok_or_else(|| {
        TtsError::PiperFailed(
            "موتور piper یافت نشد؛ ابتدا نصب کنید: pip install piper-tts".to_string(),
        )
    })?;
    let mut cmd = Command::new(python);
    cmd.arg("-m")
        .arg("piper")
        .arg("-m")
        .arg(model)
        .arg("--length-scale")
        .arg(length_scale.to_string())
        .arg("-f")
        .arg(out_wav)
        .arg("--")
        .arg(text);
    Ok((cmd, None))
}

/// Pick a Python interpreter that exists **and can actually import piper**.
/// `MANA_PYTHON` overrides the search.
///
/// Checking the import (not just the file) matters: on Windows runners there
/// are several interpreters (`py` launcher vs `python`), and only the one the
/// package was pip-installed into can run the engine. The result is cached
/// for the process lifetime.
fn python_interpreter() -> Option<PathBuf> {
    use std::sync::OnceLock;
    static CACHED: OnceLock<Option<PathBuf>> = OnceLock::new();
    CACHED
        .get_or_init(|| {
            // Explicit override wins if it can run the engine.
            if let Some(custom) = std::env::var_os("MANA_PYTHON") {
                let resolved = if custom.as_os_str().is_empty() {
                    None
                } else {
                    let p = PathBuf::from(&custom);
                    if p.is_absolute() {
                        Some(p)
                    } else {
                        first_on_path(&p)
                    }
                };
                match resolved {
                    Some(full) if can_import_piper(&full) => return Some(full),
                    _ => return None,
                }
            }
            #[cfg(windows)]
            let names = ["python", "python3", "py"];
            #[cfg(not(windows))]
            let names = ["python3", "python"];
            // Every interpreter on PATH, in PATH order; the first one that
            // can actually import piper wins. A bare-name lookup alone is not
            // enough: one PATH entry may hold an interpreter without the
            // package while a later one has it.
            std::env::var_os("PATH").and_then(|paths| {
                std::env::split_paths(&paths).find_map(|dir| {
                    names.iter().find_map(|n| {
                        let base = dir.join(n);
                        with_platform_ext(&base)
                            .filter(|f| f.is_file())
                            .filter(|f| can_import_piper(f))
                    })
                })
            })
        })
        .clone()
}

/// First absolute match of a bare name on PATH, if any.
fn first_on_path(p: &Path) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|d| d.join(p))
            .find(|f| f.is_file())
    })
}

/// On Windows a bare name never matches (`python.exe`); try PATHEXT-ish
/// suffixes. Elsewhere the name is used as-is.
#[cfg(windows)]
fn with_platform_ext(p: &PathBuf) -> Option<PathBuf> {
    ["", ".exe", ".bat", ".cmd"]
        .iter()
        .map(|ext| {
            let mut q = p.as_os_str().to_owned();
            q.push(ext);
            PathBuf::from(q)
        })
        .find(|f| f.is_file())
}

#[cfg(not(windows))]
fn with_platform_ext(p: &PathBuf) -> Option<PathBuf> {
    Some(p.clone())
}

/// True when `<python> -c "import piper"` exits 0 (fast; cached by caller).
fn can_import_piper(python: &Path) -> bool {
    std::process::Command::new(python)
        .arg("-c")
        .arg("import piper")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_paths_are_checked_directly() {
        // A shell exists on every Unix runner; the Windows system shell
        // plays the same role there.
        #[cfg(not(windows))]
        assert!(first_on_path(Path::new("sh")).is_some());
        #[cfg(windows)]
        assert!(first_on_path(Path::new("cmd")).is_some());
        assert!(first_on_path(Path::new("definitely-not-a-real-binary-xyz")).is_none());
    }

    #[test]
    fn interpreter_without_piper_is_rejected() {
        // /bin/sh exists but cannot `import piper`; the probe must say no.
        // (On Windows sh is absent, so the test asserts the negative only.)
        #[cfg(not(windows))]
        assert!(!can_import_piper(Path::new("/bin/sh")));
        assert!(!can_import_piper(Path::new("/nonexistent-xyz-123")));
    }
}
