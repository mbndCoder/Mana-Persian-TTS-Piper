#!/usr/bin/env python3
"""Prepare the Mana voice for the Android TTS engine app.

Mirrors the official sherpa-onnx layout for piper models
(e.g. vits-piper-fa_IR-amir-medium):
    assets/vits-piper-fa_IR-mana-medium/
        fa_IR-mana-medium.onnx   (with sherpa VITS metadata injected)
        tokens.txt               (sorted by id, from the piper config)
        espeak-ng-data/          (phonemizer data)

The stock fa_IR-mana-medium.onnx carries NO onnx metadata, which sherpa's
VITS loader requires (sample_rate, n_speakers, language, comment=piper).
Without injection the engine refuses to load the model at all.

Usage: python3 scripts/android_model.py [--check-only]
"""
import json
import shutil
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODEL_DIR = ROOT / "models"
ASSETS = (
    ROOT / "android/app/src/main/assets/vits-piper-fa_IR-mana-medium"
)

SRC_ONNX = MODEL_DIR / "fa_IR-mana-medium.onnx"
SRC_JSON = MODEL_DIR / "fa_IR-mana-medium.onnx.json"
SRC_ESPEAK_TAR = MODEL_DIR / "espeak-ng-data.tar.bz2"

META = {
    "model_type": "vits",
    "comment": "piper",
    "language": "fa",
    "voice": "fa",
    "sample_rate": "22050",
    "n_speakers": "1",
    "add_blank": "0",
}


def build() -> None:
    try:
        import onnx
    except ImportError:
        sys.exit("[X] pip install onnx")

    for f in (SRC_ONNX, SRC_JSON):
        if not f.exists():
            sys.exit(f"[X] missing {f} (see README: download the model first)")

    if ASSETS.exists():
        shutil.rmtree(ASSETS)
    (ASSETS / "espeak-ng-data").mkdir(parents=True)

    # 1. model + metadata (voice MUST be 'fa': sherpa passes it to espeak-ng)
    print("[*] injecting onnx metadata...")
    shutil.copyfile(SRC_ONNX, ASSETS / "fa_IR-mana-medium.onnx")
    m = onnx.load(str(ASSETS / "fa_IR-mana-medium.onnx"))
    have = {p.key for p in m.metadata_props}
    for k, v in META.items():
        if k not in have:
            e = m.metadata_props.add()
            e.key, e.value = k, v
    onnx.save(m, str(ASSETS / "fa_IR-mana-medium.onnx"))

    # 2. tokens.txt, ascending by id (sherpa looks ids up by position)
    cfg = json.loads(SRC_JSON.read_text(encoding="utf-8"))
    pmap = cfg.get("phoneme_id_map", {})
    toks = sorted(
        ((t, int(v[0] if isinstance(v, list) else v)) for t, v in pmap.items()),
        key=lambda x: x[1],
    )
    with open(ASSETS / "tokens.txt", "w", encoding="utf-8") as f:
        for t, i in toks:
            f.write(f"{t} {i}\n")
    print(f"[+] tokens.txt ({len(toks)} phonemes)")

    # 3. espeak-ng-data (native bundle if present, else the sherpa tarball)
    bundled = ROOT / "tools/piper/espeak-ng-data"
    if bundled.is_dir() and (bundled / "lang").exists():
        shutil.copytree(bundled, ASSETS / "espeak-ng-data", dirs_exist_ok=True)
        print("[+] espeak-ng-data (from tools/piper)")
    elif SRC_ESPEAK_TAR.exists():
        with tarfile.open(SRC_ESPEAK_TAR, "r:bz2") as tar:
            tar.extractall(path=ASSETS, filter="fully_trusted")
        print("[+] espeak-ng-data (from tarball)")
    else:
        sys.exit("[X] no espeak-ng-data found")

    fa = ASSETS / "espeak-ng-data/lang/ira/fa"
    if not fa.exists():
        sys.exit("[X] fa voice missing from espeak-ng-data")
    print("[+] fa espeak voice present")


def check() -> int:
    problems = []
    onnx_f = ASSETS / "fa_IR-mana-medium.onnx"
    if not (onnx_f.exists() and onnx_f.stat().st_size > 50_000_000):
        problems.append("model missing or truncated")
    else:
        try:
            import onnx

            meta = {
                p.key: p.value
                for p in onnx.load(str(onnx_f), load_external_data=False).metadata_props
            }
            for k, v in META.items():
                if meta.get(k) != v:
                    problems.append(f"metadata {k}: want {v}, got {meta.get(k)}")
        except ImportError:
            problems.append("onnx package missing for verification")
    for f in ("tokens.txt", "espeak-ng-data/lang/ira/fa"):
        if not (ASSETS / f).exists():
            problems.append(f"missing {f}")
    if problems:
        print("[X] android assets incomplete:")
        for p in problems:
            print(f"    - {p}")
        return 1
    size_mb = sum(
        f.stat().st_size for f in ASSETS.rglob("*") if f.is_file()
    ) / 1024 / 1024
    print(f"[+] android assets ok ({size_mb:.0f} MB)")
    return 0


if __name__ == "__main__":
    if "--check-only" in sys.argv:
        raise SystemExit(check())
    build()
    raise SystemExit(check())
