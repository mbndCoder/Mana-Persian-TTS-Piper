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

#[test]
fn python_fallback_synthesizes_when_native_binary_is_absent() {
    let out = unique_wav("fallback");
    mana_tts::tts::synthesize_with(
        std::path::Path::new("tools/piper/does-not-exist"),
        std::path::Path::new("models/fa_IR-mana-medium.onnx"),
        "سلام دنیا، تست مسیر جایگزین پایتون.",
        1.0,
        &out,
    )
    .expect("python fallback must synthesize");

    let reader = hound::WavReader::open(&out).expect("output must be a valid wav");
    let spec = reader.spec();
    assert_eq!(spec.sample_rate, 22050);
    let frames = reader.len() / spec.channels as u32;
    assert!(frames > spec.sample_rate, "must exceed 1s, got {frames} frames");
    std::fs::remove_file(&out).ok();
}
