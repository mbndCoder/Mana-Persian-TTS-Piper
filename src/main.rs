use mana_tts::tts;
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let text = args.next().unwrap_or_else(|| {
        eprintln!("استفاده: mana-tts \"متن فارسی\" [فایل-خروجی]");
        std::process::exit(2);
    });
    let out: PathBuf = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("output/cli.wav"));

    match tts::synthesize(&text, &out) {
        Ok(()) => println!("ذخیره شد: {}", out.display()),
        Err(e) => {
            eprintln!("خطا: {e}");
            std::process::exit(1);
        }
    }
}
