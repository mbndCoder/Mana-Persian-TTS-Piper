use std::path::PathBuf;

fn unique_wav(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("mana-tts-{name}-{}-{}.wav", std::process::id(), name.len()));
    p
}

#[test]
fn rodio_decodes_engine_output() {
    let out = unique_wav("playback");
    mana_tts::tts::synthesize("سلام دنیا، موتور تبدیل متن به گفتار مانا آماده است.", 1.0, &out)
        .expect("synthesis must succeed");

    let samples = mana_tts::audio::decoded_samples(&out).expect("rodio must decode it");
    assert!(
        samples > 22050,
        "must decode more than 1s of audio, got {samples} samples"
    );
    std::fs::remove_file(&out).ok();
}
