#!/usr/bin/env python3
"""Point the imported engine app at the Mana voice — and nothing else.

Deliberately minimal, mirroring exactly what upstream does per model
(sed a few values into TtsEngine.kt, like their own build-apk-tts-engine
script does for e.g. vits-piper-fa_IR-amir-medium):
  1. file dependency on the official sherpa-onnx AAR (replaces the NDK build;
     invisible to the user)
  2. engine preset: modelDir / modelName / dataDir / lang
  3. Persian sample sentence (upstream has no "fas" case; without it the
     system voice tester reports LANG_NOT_SUPPORTED for our voice)

UI, settings, theme, package name and app label stay EXACTLY as upstream.
"""
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent / "android"

MODEL_DIR = "vits-piper-fa_IR-mana-medium"
MODEL_NAME = "fa_IR-mana-medium.onnx"
DATA_DIR = f"{MODEL_DIR}/espeak-ng-data"
LANG = "fas"  # ISO 639-3 for Persian
AAR = "sherpa-onnx-1.13.8.aar"


def sub(path: pathlib.Path, old: str, new: str) -> None:
    s = path.read_text(encoding="utf-8")
    if old not in s:
        raise SystemExit(f"pattern missing in {path}: {old!r}")
    path.write_text(s.replace(old, new), encoding="utf-8")


def main() -> int:
    kt = ROOT / "app/src/main/java/com/k2fsa/sherpa/onnx/tts/engine"
    assert kt.is_dir(), "run scripts/import_android_app.py first"

    g = ROOT / "app/build.gradle.kts"
    sub(
        g,
        '    testImplementation("junit:junit:4.13.2")',
        f'    implementation(files("../libs/{AAR}"))\n    testImplementation("junit:junit:4.13.2")',
    )

    te = kt / "TtsEngine.kt"
    sub(te, "modelDir = null", f'modelDir = "{MODEL_DIR}"')
    sub(te, "modelName = null", f'modelName = "{MODEL_NAME}"')
    sub(te, "dataDir = null", f'dataDir = "{DATA_DIR}"')
    sub(te, "lang = null", f'lang = "{LANG}"')

    gs = kt / "GetSampleText.kt"
    sub(
        gs,
        '        "est" -> {',
        '        "fas" -> {\n'
        '            text = "سلام! موتور تبدیل متن به گفتار مانا آماده است."\n'
        "        }\n\n"
        '        "est" -> {',
    )

    print("[+] model-only patch done:")
    print(f"    model={MODEL_DIR}/{MODEL_NAME} lang={LANG} (+fas sample, +AAR dep)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
