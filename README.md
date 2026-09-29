# ManaTTS — آفلاین Persian Text-to-Speech

دسکتاپ‌اپ آفلاین تبدیل متن فارسی به گفتار با صدای مانا (`MahtaFetrat/Mana-Persian-Piper`)،
رابط فارسی راست‌چین با تم دارک/لایت، و API محلی سازگار با OpenAI برای ابزارها و ایجنت‌ها.

## دانلود آماده (پیشنهادی)

از تب **Releases** گیت‌هاب، فایل مخصوص سیستم‌عاملت را بگیر:

| سیستم | فایل |
|---|---|
| لینوکس | `ManaTTS_*.AppImage` (اجرا مستقیم) یا `.deb` |
| ویندوز | نصب‌کننده `.exe` |
| مک | `.dmg` |

## پیش‌نیازها

- **لینوکس:** هیچ‌چیز. موتور به‌صورت باینری همراه اپ است.
- **ویندوز / مک:** پایتون ۳٫۱۰+ و یک دستور (موتور از پکیج رسمی استفاده می‌کند):
  ```bash
  pip install piper-tts
  ```
  بدون این پکیج، اپ باز می‌شود ولی تولید صدا خطا می‌دهد.

## اجرای محلی (توسعه)

```bash
# 1. مدل (یک‌بار، ~61MB)
mkdir -p models
curl -L https://huggingface.co/MahtaFetrat/Mana-Persian-Piper/resolve/main/fa_IR-mana-medium.onnx -o models/fa_IR-mana-medium.onnx
curl -L https://huggingface.co/MahtaFetrat/Mana-Persian-Piper/resolve/main/fa_IR-mana-medium.onnx.json -o models/fa_IR-mana-medium.onnx.json

# 2. موتور نیتیو لینوکس (یک‌بار، ~25MB؛ روی ویندوز/مک لازم نیست)
mkdir -p tools
curl -L https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_amd64.tar.gz -o tools/piper_amd64.tar.gz
tar -xzf tools/piper_amd64.tar.gz -C tools/

# 3. اجرا
./scripts/dev.sh        # رابط گرافیکی
cargo run -- "سلام دنیا" # خط فرمان (پخش مستقیم)
```

## API محلی برای ابزارها و ایجنت‌ها

با باز شدن اپ، سرویس روی `http://127.0.0.1:7788` بالا می‌آید
(فقط همین دستگاه؛ اگر پورت اشغال بود خودکار عوض می‌شود و آدرس واقعی در UI هست).

```bash
curl -X POST http://127.0.0.1:7788/v1/audio/speech \
  -H 'Content-Type: application/json' \
  -d '{"input": "سلام دنیا", "voice": "mana", "speed": 1.0, "response_format": "wav"}' \
  --output ~/Documents/ManaTTS/speech.wav
mpv ~/Documents/ManaTTS/speech.wav
```

- `speed`: ‏۰٫۲۵ تا ۴٫۰ (۱٫۰ = عادی)
- `GET /health` وضعیت سرویس را می‌دهد؛ `GET /` نمونه درخواست را.
- احراز هویت لازم نیست. اگر خواستی: `MANA_API_TOKEN=...` را ست کن و هدر
  `Authorization: Bearer ...` بفرست.
- اگر خواستی پورت دیگری باشد: `MANA_API_PORT=8080`.

## تست

```bash
cargo test -p mana-tts      # موتور + پخش
cargo test -p mana-tts-app  # مسیرها، موج صدا، منطق API
python3 scripts/phase0_spike.py  # مرجع صوتی (نیاز به .venv با piper-tts)
```

## لایسنس‌ها

| جزء | لایسنس | نکته تجاری |
|---|---|---|
| مدل مانا (وزن‌ها) | MIT | آزاد، بدون شرط |
| دیتاست Mana-TTS | CC0-1.0 | آزاد، بدون شرط |
| موتور piper + espeak-ng | **GPL-3.0** | استفاده تجاری آزاد است، ولی هنگام **توزیع** باید سورس موتور را هم بدهی — پوشه `licenses/` همین کار را می‌کند |
| Tauri / axum / rodio / onnxruntime | MIT / Apache-2.0 | آزاد |

## ساختار

```
mana-tts/
├── src/                 # فرانت‌اند آفلاین (HTML/CSS/JS، بدون باندلر)
├── src-tauri/           # بک‌اند Tauri: کامندها + سرور API
├── src/tts.rs src/audio.rs  # هسته: موتور + پخش
├── models/              # مدل (در گیت نیست؛ CI دانلود می‌کند)
├── tools/piper/         # موتور نیتیو لینوکس (در گیت نیست)
├── scripts/             # dev.sh، phase0_spike.py
└── .github/workflows/   # بیلد لینوکس/ویندوز/مک
```
