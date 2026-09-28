"""Phase 0 oracle: synthesize the reference WAV with the official piper engine.

Engine of record is the prebuilt piper binary (tools/piper/piper),
NOT sherpa-onnx: the Mana ONNX carries no metadata, so sherpa's VITS
loader (which requires sample_rate/n_speakers/language) cannot read it.

Regenerates output/oracle.wav and asserts: valid RIFF, mono 16-bit,
22050 Hz, longer than 1s, non-silent (RMS check).
"""
import math
import struct
import subprocess
import sys
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODEL = ROOT / "models" / "fa_IR-mana-medium.onnx"
CONFIG = ROOT / "models" / "fa_IR-mana-medium.onnx.json"
PIPER = ROOT / "tools" / "piper" / "piper"
ORACLE = ROOT / "output" / "oracle.wav"

TEXT = "سلام به همگی! تست اولیه سیستم تبدیل متن به گفتار مانا با موفقیت انجام شد."


def fail(msg: str) -> None:
    print(f"[X] {msg}")
    sys.exit(1)


size_mb = MODEL.stat().st_size / 1024 / 1024 if MODEL.exists() else 0
print(f"[i] model: {MODEL} ({size_mb:.1f} MB)")
if size_mb < 50:
    fail("model missing or truncated (< 50 MB), re-download it")
if not CONFIG.exists():
    fail("model config json missing")
if not PIPER.exists():
    fail("piper binary missing at tools/piper/piper")

ORACLE.parent.mkdir(exist_ok=True)
proc = subprocess.run(
    [str(PIPER), "-m", str(MODEL), "-f", str(ORACLE)],
    input=TEXT.encode("utf-8"),
    capture_output=True,
)
if proc.returncode != 0:
    fail(f"piper failed: {proc.stderr.decode(errors='replace').strip()}")
print("[+] piper synthesis ok")

with wave.open(str(ORACLE), "rb") as w:
    n, rate, ch, width = w.getnframes(), w.getframerate(), w.getnchannels(), w.getsampwidth()
    samples = struct.unpack("<" + "h" * n, w.readframes(n))

assert ch == 1, f"must be mono, got {ch}"
assert rate == 22050, f"must be 22050 Hz, got {rate}"
assert width == 2, "must be 16-bit"
assert n > rate, f"must exceed 1s, got {n / rate:.2f}s"
rms = math.sqrt(sum(s * s for s in samples) / n)
assert rms > 100, f"output is silent (rms={rms:.0f})"

print(f"[+] oracle ok: {ORACLE} ({n / rate:.1f}s, rms={rms:.0f})")
