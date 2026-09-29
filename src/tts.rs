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

    let mut child = Command::new(piper)
        .arg("-m")
        .arg(model)
        .arg("-f")
        .arg(out_wav)
        .arg("--length-scale")
        .arg(length_scale.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    child
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(text.as_bytes())?;

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
