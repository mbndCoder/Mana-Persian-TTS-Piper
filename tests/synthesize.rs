use std::path::PathBuf;

fn unique_wav(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("mana-tts-{name}-{}-{}.wav", std::process::id(), name.len()));
    p
}

#[test]
fn synthesize_produces_valid_persian_wav() {
    let out = unique_wav("test");
    mana_tts::tts::synthesize("سلام دنیا، موتور تبدیل متن به گفتار مانا آماده است.", 1.0, &out)
        .expect("synthesis must succeed");

    let reader = hound::WavReader::open(&out).expect("output must be a valid wav");
    let spec = reader.spec();
    assert_eq!(spec.channels, 1, "must be mono");
    assert_eq!(spec.sample_rate, 22050, "must be 22050 Hz");
    assert_eq!(spec.bits_per_sample, 16);

    let frames = reader.len() / spec.channels as u32;
    assert!(
        frames > spec.sample_rate,
        "must contain more than 1s of audio, got {frames} frames"
    );
    std::fs::remove_file(&out).ok();
}

/// The engine is now a self-contained binary for every platform, so the app
/// must not depend on Python at all. This asserts the packaged engine is
/// present and speaks, which is what ships to users.
///
/// Linux-only: the native binary is only ever downloaded/bundled on Linux.
/// Windows and macOS intentionally use the documented piper-tts package
/// instead (see `python_package_synthesizes_with_argv_text`).
#[cfg(target_os = "linux")]
#[test]
fn bundled_engine_synthesizes_without_python() {
    let engine = if cfg!(windows) {
        std::path::Path::new("tools/piper/piper.exe")
    } else {
        std::path::Path::new("tools/piper/piper")
    };
    assert!(
        engine.is_file(),
        "bundled engine missing at {} — build it with scripts/build_engine.sh",
        engine.display()
    );

    let out = unique_wav("bundled");
    mana_tts::tts::synthesize_with(
        engine,
        std::path::Path::new("models/fa_IR-mana-medium.onnx"),
        "سلام دنیا، موتور مستقل بدون پایتون کار می‌کند.",
        1.0,
        &out,
    )
    .expect("bundled engine must synthesize");

    let reader = hound::WavReader::open(&out).expect("output must be a valid wav");
    let spec = reader.spec();
    assert_eq!(spec.sample_rate, 22050);
    let frames = reader.len() / spec.channels as u32;
    assert!(frames > spec.sample_rate, "must exceed 1s, got {frames} frames");
    std::fs::remove_file(&out).ok();
}

/// The documented Windows/macOS path: official piper-tts package, text passed
/// as argv (never stdin, which Python decodes with the ambient locale).
#[test]
fn python_package_synthesizes_with_argv_text() {
    let out = unique_wav("pyargv");
    mana_tts::tts::synthesize_with(
        std::path::Path::new("tools/piper/does-not-exist"),
        std::path::Path::new("models/fa_IR-mana-medium.onnx"),
        "سلام دنیا، تست مسیر رسمی پایتون.",
        1.0,
        &out,
    )
    .expect("piper-tts package must synthesize (pip install piper-tts)");

    let reader = hound::WavReader::open(&out).expect("output must be a valid wav");
    let spec = reader.spec();
    assert_eq!(spec.sample_rate, 22050);
    let frames = reader.len() / spec.channels as u32;
    assert!(frames > spec.sample_rate, "must exceed 1s, got {frames} frames");
    std::fs::remove_file(&out).ok();
}
