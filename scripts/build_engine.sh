#!/usr/bin/env bash
# Build a self-contained `piper` executable for the current platform.
#
# Why: the official piper1-gpl releases ship only Python wheels, so there is
# no native binary for Windows/macOS. Freezing the same engine with PyInstaller
# gives every platform a real executable, so the app needs no Python at all.
#
# Usage:  pip install pyinstaller piper-tts && ./scripts/build_engine.sh
set -euo pipefail

cd "$(dirname "$0")/.."
OUT="tools/piper"
NAME="piper"

if [[ "${1:-}" == "--help" ]]; then sed -n '2,12p' "$0"; exit 0; fi

python3 - <<'PY'
import importlib.util as u, sys
missing = [m for m in ("PyInstaller", "piper") if u.find_spec(m) is None]
if missing:
    sys.exit("missing: " + ", ".join(missing) + "  ->  pip install pyinstaller piper-tts")
print("pyinstaller + piper-tts present")
PY

# PyInstaller entry: reuse piper's own CLI so the frozen binary keeps the exact
# same contract our Rust side speaks (-m model -f out --length-scale x, stdin).
# Entry point is piper.__main__:main (the package's declared console script).
mkdir -p build
cat > build/piper_entry.py <<'PY'
from piper.__main__ import main

if __name__ == "__main__":
    main()
PY

python3 -m PyInstaller \
  --noconfirm --clean --onefile \
  --name "$NAME" \
  --distpath "$OUT" \
  --workpath build/work \
  --specpath build \
  --collect-all piper \
  --collect-all espeakbridge \
  build/piper_entry.py

# the espeak-ng data must sit next to the executable; the Rust side passes it
# explicitly via --espeak_data, so keep the same folder name as the native build
if [[ -d "dist/_internal/piper/espeak-ng-data" ]]; then
  cp -r "dist/_internal/piper/espeak-ng-data" "$OUT/espeak-ng-data"
fi

echo "built: $OUT/$NAME"
"$OUT/$NAME" --help | head -5
