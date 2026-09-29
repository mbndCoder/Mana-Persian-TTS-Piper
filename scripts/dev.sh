#!/usr/bin/env bash
# Start ManaTTS in dev mode with explicit engine paths (dev-only launcher).
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_BUILD_JOBS=2
export MANA_PIPER_BIN="$PWD/tools/piper/piper"
export MANA_MODEL="$PWD/models/fa_IR-mana-medium.onnx"
exec cargo tauri dev
