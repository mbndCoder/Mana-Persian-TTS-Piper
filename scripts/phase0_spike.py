"""Phase 0 spike: validate Mana Persian Piper model + synthesize test wav via sherpa-onnx Python API."""
import json
import sys
import tarfile
from pathlib import Path
from urllib.request import urlretrieve

BASE_URL = "https://huggingface.co/MahtaFetrat/Mana-Persian-Piper/resolve/main"
ESPEAK_URL = "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/espeak-ng-data.tar.bz2"

MODEL_DIR = Path("models")
OUTPUT_DIR = Path("output")
CONFIG_PATH = MODEL_DIR / "fa_IR-mana-medium.onnx.json"
MODEL_PATH = MODEL_DIR / "fa_IR-mana-medium.onnx"
TOKENS_PATH = MODEL_DIR / "tokens.txt"
ESPEAK_DIR = MODEL_DIR / "espeak-ng-data"
ESPEAK_TAR = MODEL_DIR / "espeak-ng-data.tar.bz2"
OUTPUT_WAV = OUTPUT_DIR / "test_python.wav"

MODEL_DIR.mkdir(exist_ok=True)
OUTPUT_DIR.mkdir(exist_ok=True)


def download(filename: str, url: str, target: Path) -> None:
    if not target.exists():
        print(f"[*] downloading {filename}...")
        urlretrieve(url, target)
        print(f"[+] done: {filename}")
    else:
        print(f"[i] exists: {filename}")


download("fa_IR-mana-medium.onnx.json", f"{BASE_URL}/fa_IR-mana-medium.onnx.json", CONFIG_PATH)
download("fa_IR-mana-medium.onnx", f"{BASE_URL}/fa_IR-mana-medium.onnx", MODEL_PATH)
download("espeak-ng-data.tar.bz2", ESPEAK_URL, ESPEAK_TAR)

# sanity: model must not be truncated (HF XET partial downloads happen)
size = MODEL_PATH.stat().st_size
print(f"[i] model size: {size / 1024 / 1024:.1f} MB")
if size < 50_000_000:
    print("[!] model file too small, likely truncated. delete and re-download.")
    sys.exit(1)

if not ESPEAK_DIR.exists():
    print("[*] extracting espeak-ng-data...")
    with tarfile.open(ESPEAK_TAR, "r:bz2") as tar:
        tar.extractall(path=MODEL_DIR, filter="fully_trusted")
    print("[+] extracted.")

fa_voice = ESPEAK_DIR / "lang" / "ira" / "fa"
if not fa_voice.exists():
    print(f"[!] fa voice missing: {fa_voice}")
    sys.exit(1)
print("[+] fa espeak voice present.")

with open(CONFIG_PATH, "r", encoding="utf-8") as f:
    config = json.load(f)

phoneme_map = config.get("phoneme_id_map", {})
print(f"[i] config sample_rate={config.get('audio', {}).get('sample_rate')}, "
      f"voice={config.get('espeak', {}).get('voice')}, tokens={len(phoneme_map)}")

normalized = []
for token, ids in phoneme_map.items():
    idx = ids[0] if isinstance(ids, list) else ids
    normalized.append((token, int(idx)))
sorted_tokens = sorted(normalized, key=lambda x: x[1])

with open(TOKENS_PATH, "w", encoding="utf-8") as f:
    for token, idx in sorted_tokens:
        f.write(f"{token} {idx}\n")
print(f"[+] wrote {TOKENS_PATH} ({len(sorted_tokens)} tokens, sorted by id).")

try:
    import sherpa_onnx
    import soundfile as sf
except ImportError:
    print("[!] missing packages. run: pip install sherpa-onnx soundfile")
    sys.exit(1)

tts_config = sherpa_onnx.OfflineTtsConfig(
    model=sherpa_onnx.OfflineTtsModelConfig(
        vits=sherpa_onnx.OfflineTtsVitsModelConfig(
            model=str(MODEL_PATH),
            tokens=str(TOKENS_PATH),
            data_dir=str(ESPEAK_DIR),
            noise_scale=0.667,
            noise_scale_w=0.8,
            length_scale=1.0,
        ),
        num_threads=2,
        debug=False,
        provider="cpu",
    )
)

print("[*] initializing TTS...")
tts = sherpa_onnx.OfflineTts(tts_config)

text = "سلام، تست اولیه سیستم تبدیل متن به گفتار مانا با موفقیت انجام شد."
print(f"[*] synthesizing: '{text}'")
audio = tts.generate(text, sid=0, speed=1.0)

if not len(audio.samples):
    print("[!] empty audio output, synthesis failed.")
    sys.exit(1)

sf.write(str(OUTPUT_WAV), audio.samples, samplerate=audio.sample_rate, subtype="PCM_16")
print(f"[+] saved: {OUTPUT_WAV}")
print(f"    sample_rate={audio.sample_rate} Hz, frames={len(audio.samples)}, "
      f"duration={len(audio.samples) / audio.sample_rate:.1f}s")
