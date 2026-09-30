#!/usr/bin/env python3
"""Rebrand the imported engine app for Mana (idempotent).

  - package com.k2fsa.sherpa.onnx.tts.engine -> com.mana.tts (code + manifest
    + fileprovider authority + theme + project name)
  - app label -> Persian name
  - engine points at the Mana voice (official Example-2 pattern for piper)
  - Persian sample sentence for the system voice tester
  - file dependency on the official sherpa-onnx AAR (no NDK build)
"""
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent / "android"

MODEL_DIR = "vits-piper-fa_IR-mana-medium"
MODEL_NAME = "fa_IR-mana-medium.onnx"
DATA_DIR = f"{MODEL_DIR}/espeak-ng-data"
LANG = "fas"  # ISO 639-3 for Persian
APP_LABEL = "موتور گفتار مانا"
AAR = "sherpa-onnx-1.13.8.aar"


def sub(path: pathlib.Path, old: str, new: str) -> None:
    s = path.read_text(encoding="utf-8")
    if old not in s:
        # e.g. PreferencesHelper.kt ships without a package line upstream
        print(f"[!] pattern missing in {path.name}, skipping: {old!r}")
        return
    path.write_text(s.replace(old, new), encoding="utf-8")


def main() -> int:
    kt = ROOT / "app/src/main/java/com/mana/tts"
    assert kt.is_dir(), "run scripts/import_android_app.py first"

    # 1. package rename in every kotlin file (+ prefs namespace).
    # NOTE: test stubs already live at the new package (see import script).
    for f in list(kt.rglob("*.kt")):
        sub(f, "package com.k2fsa.sherpa.onnx.tts.engine", "package com.mana.tts")
    for f in [
        ROOT / "app/src/test/java/com/mana/tts/ExampleUnitTest.kt",
        ROOT / "app/src/androidTest/java/com/mana/tts/ExampleInstrumentedTest.kt",
    ]:
        if f.exists():
            sub(f, "package com.k2fsa.sherpa.onnx.tts.engine", "package com.mana.tts")
    sub(
        kt / "PreferencesHelper.kt",
        '"com.k2fsa.sherpa.onnx.tts.engine"',
        '"com.mana.tts"',
    )

    # 2. manifest: package, provider authority, theme, label stays via strings
    mf = ROOT / "app/src/main/AndroidManifest.xml"
    sub(mf, 'package="com.k2fsa.sherpa.onnx.tts.engine"', 'package="com.mana.tts"')
    sub(
        mf,
        "com.k2fsa.sherpa.onnx.tts.engine.fileprovider",
        "com.mana.tts.fileprovider",
    )
    sub(mf, "@style/Theme.SherpaOnnxTtsEngine", "@style/Theme.ManaTts")

    # 3. theme rename
    th = ROOT / "app/src/main/res/values/themes.xml"
    sub(th, "Theme.SherpaOnnxTtsEngine", "Theme.ManaTts")

    # 4. app label (Persian). The TTS service label comes from the same string.
    st = ROOT / "app/src/main/res/values/strings.xml"
    s = st.read_text(encoding="utf-8")
    s = s.replace(
        "<string name=\"app_name\">TTS Engine: Next-gen Kaldi</string>",
        f"<string name=\"app_name\">{APP_LABEL}</string>",
    )
    st.write_text(s, encoding="utf-8")

    # 5. gradle: namespace, application id, version; AAR file dependency.
    #    jniLibs stay empty: the AAR already carries .so for all four ABIs.
    g = ROOT / "app/build.gradle.kts"
    sub(g, 'namespace = "com.k2fsa.sherpa.onnx.tts.engine"', 'namespace = "com.mana.tts"')
    sub(
        g,
        'applicationId = "com.k2fsa.sherpa.onnx.tts.engine"',
        'applicationId = "com.mana.tts"',
    )
    sub(g, 'versionName = "1.13.8"', 'versionName = "0.1.0"')
    sub(g, "versionCode = 20260910", "versionCode = 1")
    sub(
        g,
        '    testImplementation("junit:junit:4.13.2")',
        f'    implementation(files("../libs/{AAR}"))\n    testImplementation("junit:junit:4.13.2")',
    )
    sg = ROOT / "settings.gradle.kts"
    sub(sg, 'rootProject.name = "SherpaOnnxTtsEngine"', 'rootProject.name = "ManaTtsEngine"')

    # 6. engine values: the official piper pattern (see Example 2 in TtsEngine.kt)
    te = kt / "TtsEngine.kt"
    sub(te, "modelDir = null", f'modelDir = "{MODEL_DIR}"')
    sub(te, "modelName = null", f'modelName = "{MODEL_NAME}"')
    sub(te, "dataDir = null", f'dataDir = "{DATA_DIR}"')
    sub(te, "lang = null", f'lang = "{LANG}"')

    # 7. Persian sample sentence (upstream has no "fas" case; without it the
    #    system voice tester reports LANG_NOT_SUPPORTED)
    gs = kt / "GetSampleText.kt"
    sub(
        gs,
        '        "est" -> {',
        '        "fas" -> {\n'
        '            text = "سلام! موتور تبدیل متن به گفتار مانا آماده است."\n'
        "        }\n\n"
        '        "est" -> {',
    )

    # 8. settings activity reference must follow the package rename
    te_xml = ROOT / "app/src/main/res/xml/tts_engine.xml"
    if te_xml.exists():
        sub(
            te_xml,
            "com.k2fsa.sherpa.onnx.tts.engine.MainActivity",
            "com.mana.tts.MainActivity",
        )

    print("[+] rebrand done:")
    print(f"    package=com.mana.tts model={MODEL_DIR}/{MODEL_NAME} lang={LANG}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
