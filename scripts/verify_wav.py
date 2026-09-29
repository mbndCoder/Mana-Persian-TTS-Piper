#!/usr/bin/env python3
"""Verify a synthesized wav: valid RIFF, mono 16-bit, 22050 Hz, >1s, not silent.

Used by CI to prove the frozen engine really produced Persian speech instead of
merely exiting 0. Stdlib only, so it runs on every runner.
"""
import json
import math
import struct
import sys
import wave

EXPECTED_RATE = 22050


def main(path: str) -> int:
    try:
        handle = wave.open(path, "rb")
    except (OSError, wave.Error) as exc:
        print(json.dumps({"file": path, "ok": False, "problems": [str(exc)]}))
        print(f"[X] cannot open {path}: {exc}", file=sys.stderr)
        return 1

    with handle as w:
        channels, width, rate, frames = (
            w.getnchannels(),
            w.getsampwidth(),
            w.getframerate(),
            w.getnframes(),
        )
        raw = w.readframes(frames)

    problems = []
    if channels != 1:
        problems.append(f"channels={channels}, want 1")
    if width != 2:
        problems.append(f"sample width={width * 8}-bit, want 16")
    if rate != EXPECTED_RATE:
        problems.append(f"rate={rate}, want {EXPECTED_RATE}")
    if frames <= rate:
        problems.append(f"duration={frames / rate:.2f}s, want > 1s")

    samples = struct.unpack("<" + "h" * frames, raw)
    rms = math.sqrt(sum(s * s for s in samples) / frames) if frames else 0.0
    if rms < 100:
        problems.append(f"silent output (rms={rms:.0f})")

    report = {
        "file": path,
        "duration_s": round(frames / rate, 2) if rate else 0,
        "rate": rate,
        "rms": round(rms),
        "ok": not problems,
        "problems": problems,
    }
    print(json.dumps(report, ensure_ascii=False))
    if problems:
        for p in problems:
            print(f"[X] {p}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1] if len(sys.argv) > 1 else "out.wav"))
