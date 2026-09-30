#!/usr/bin/env python3
"""Import the upstream SherpaOnnxTtsEngine app and rebrand it for Mana.

WHAT THIS DOES (and nothing more):
  1. Copies the upstream app skeleton (minus binaries) into android/.
  2. Renames the app package com.k2fsa.sherpa.onnx.tts.engine -> com.mana.tts.
  3. Points the engine at the Mana voice (modelDir/modelName/dataDir/lang).
  4. Adds a Persian sample sentence for the system voice tester.
  5. Depends on the official sherpa-onnx AAR (native .so for all ABIs)
     instead of building native code (no NDK needed).

Upstream: https://github.com/k2-fsa/sherpa-onnx/tree/master/android/SherpaOnnxTtsEngine
(Apache-2.0; see licenses/ENGINES.md for attribution)
"""
import pathlib
import shutil
import sys
import urllib.request

UPSTREAM = "https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/master/android/SherpaOnnxTtsEngine"
ROOT = pathlib.Path(__file__).resolve().parent.parent / "android"

OLD_PKG = "com.k2fsa.sherpa.onnx.tts.engine"
NEW_PKG = "com.mana.tts"
OLD_DIR = "app/src/main/java/com/k2fsa/sherpa/onnx/tts/engine"
NEW_DIR = "app/src/main/java/com/mana/tts"

# text files copied verbatim, then rebranded by string replacement
TEXT_FILES = [
    "build.gradle.kts",
    "settings.gradle.kts",
    "gradle.properties",
    "gradle/wrapper/gradle-wrapper.properties",
    "gradlew",
    "gradlew.bat",
    "app/build.gradle.kts",
    "app/proguard-rules.pro",
    "app/src/main/AndroidManifest.xml",
    "app/src/main/res/values/colors.xml",
    "app/src/main/res/values/strings.xml",
    "app/src/main/res/values/themes.xml",
    "app/src/main/res/values/ic_launcher_background.xml",
    "app/src/main/res/xml/backup_rules.xml",
    "app/src/main/res/xml/data_extraction_rules.xml",
    "app/src/main/res/xml/file_provider_paths.xml",
    "app/src/main/res/xml/tts_engine.xml",
    "app/src/main/res/drawable-v24/ic_launcher_foreground.xml",
    "app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml",
    "app/src/main/res/mipmap-anydpi-v26/ic_launcher_round.xml",
]

ENGINE_KT = f"{OLD_DIR}/CheckVoiceData.kt {OLD_DIR}/GetSampleText.kt {OLD_DIR}/InstallVoiceData.kt {OLD_DIR}/MainActivity.kt {OLD_DIR}/PreferencesHelper.kt {OLD_DIR}/TtsEngine.kt {OLD_DIR}/TtsService.kt {OLD_DIR}/TtsViewModel.kt {OLD_DIR}/ui/theme/Color.kt {OLD_DIR}/ui/theme/Theme.kt {OLD_DIR}/ui/theme/Type.kt".split()
# NOTE: upstream Tts.kt is a symlink to sherpa-onnx/kotlin-api/Tts.kt, NOT real
# code (fetching it returns the 65-byte link text). The real helper is fetched
# separately as HELPER_DST below; never add Tts.kt to this list.

# binary blobs copied as-is (Apache-2.0 upstream assets)
BIN_FILES = [
    "gradle/wrapper/gradle-wrapper.jar",
    "app/src/main/res/mipmap-hdpi/ic_launcher.webp",
    "app/src/main/res/mipmap-hdpi/ic_launcher_round.webp",
    "app/src/main/res/mipmap-mdpi/ic_launcher.webp",
    "app/src/main/res/mipmap-mdpi/ic_launcher_round.webp",
    "app/src/main/res/mipmap-xhdpi/ic_launcher.webp",
    "app/src/main/res/mipmap-xhdpi/ic_launcher_round.webp",
    "app/src/main/res/mipmap-xxhdpi/ic_launcher.webp",
    "app/src/main/res/mipmap-xxhdpi/ic_launcher_round.webp",
    "app/src/main/res/mipmap-xxxhdpi/ic_launcher.webp",
    "app/src/main/res/mipmap-xxxhdpi/ic_launcher_round.webp",
]

# the symlinked helper (real content lives in sherpa-onnx/kotlin-api/Tts.kt)
HELPER_URL = "https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/master/sherpa-onnx/kotlin-api/Tts.kt"
HELPER_DST = "app/src/main/java/com/k2fsa/sherpa/onnx/Tts.kt"


def fetch(url: str) -> bytes:
    import time
    import urllib.error

    last: Exception | None = None
    for attempt in range(5):
        try:
            with urllib.request.urlopen(url, timeout=120) as r:
                return r.read()
        except (urllib.error.URLError, TimeoutError, ConnectionError) as exc:
            last = exc
            time.sleep(2 * (attempt + 1))
    raise SystemExit(f"download failed after retries: {url} ({last})")


def main() -> int:
    if ROOT.exists():
        print(f"[i] removing previous {ROOT}")
        shutil.rmtree(ROOT)

    for rel in TEXT_FILES + BIN_FILES:
        data = fetch(f"{UPSTREAM}/{rel}")
        dst = ROOT / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(data)
        print(f"[+] {rel} ({len(data) // 1024} KB)")

    # helper with its ORIGINAL package (kept intentionally, see docstring)
    dst = ROOT / HELPER_DST
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(fetch(HELPER_URL))
    print(f"[+] {HELPER_DST} (upstream helper, package untouched)")

    # engine sources stay at their upstream paths (nothing is renamed anywhere)
    for rel in ENGINE_KT:
        data = fetch(f"{UPSTREAM}/{rel}")
        dst = ROOT / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(data)

    # test stubs, kept at their upstream paths (nothing is renamed anywhere)
    for rel in [
        "app/src/test/java/com/k2fsa/sherpa/onnx/tts/engine/ExampleUnitTest.kt",
        "app/src/androidTest/java/com/k2fsa/sherpa/onnx/tts/engine/ExampleInstrumentedTest.kt",
    ]:
        try:
            data = fetch(f"{UPSTREAM}/{rel}")
        except Exception as exc:
            print(f"[!] skip {rel}: {exc}")
            continue
        dst = ROOT / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(data)

    # empty asset/jniLibs placeholders (CI fills them)
    (ROOT / "app/src/main/assets/.gitkeep").parent.mkdir(parents=True, exist_ok=True)
    (ROOT / "app/src/main/assets/.gitkeep").write_text("")
    for arch in ("arm64-v8a", "armeabi-v7a", "x86", "x86_64"):
        p = ROOT / f"app/src/main/jniLibs/{arch}/.gitkeep"
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text("")

    print("[+] import done. Now run scripts/rebrand_android.py")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
