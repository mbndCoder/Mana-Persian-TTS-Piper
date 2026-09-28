use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rodio::{Decoder, OutputStream, Sink};

#[derive(Debug)]
pub enum AudioError {
    Stream(String),
    Decode(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::Stream(e) => write!(f, "دستگاه پخش صدا در دسترس نیست: {e}"),
            AudioError::Decode(e) => write!(f, "فایل صوتی خراب است: {e}"),
        }
    }
}

impl std::error::Error for AudioError {}

/// Play a WAV file on the default output device, blocking until finished.
pub fn play(wav: &Path) -> Result<(), AudioError> {
    let (_stream, handle) =
        OutputStream::try_default().map_err(|e| AudioError::Stream(e.to_string()))?;
    let sink = Sink::try_new(&handle).map_err(|e| AudioError::Stream(e.to_string()))?;

    let file = BufReader::new(File::open(wav).map_err(|e| AudioError::Decode(e.to_string()))?);
    let source = Decoder::new(file).map_err(|e| AudioError::Decode(e.to_string()))?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

/// Decode a WAV file with rodio (no hardware needed) and count samples.
/// Used to verify rodio integration headlessly.
pub fn decoded_samples(wav: &Path) -> Result<u64, AudioError> {
    let file = BufReader::new(File::open(wav).map_err(|e| AudioError::Decode(e.to_string()))?);
    let source = Decoder::new(file).map_err(|e| AudioError::Decode(e.to_string()))?;
    Ok(source.count() as u64)
}
