# مانـا — تبدیل متن فارسی به گفتار، آفلاین

<div dir="rtl">

اپ دسکتاپ **کاملاً آفلاین** برای تبدیل متن فارسی به گفتار با صدای مانا، به‌همراه
**API محلی سازگار با OpenAI** برای استفاده ایجنت‌ها و ابزارها.

رابط کاربری فارسی و راست‌چین با تم دارک/لایت — بدون اینترنت، بدون حساب کاربری، بدون
ارسال صدای شما به هیچ‌جا.

</div>

---

## چه چیزی دارد

| قابلیت | توضیح |
|---|---|
| **تولید و پخش صدا** | متن فارسی → گفتار طبیعی، پخش داخل خود اپ |
| **کنترل سرعت** | اسلایدر ۰٫۵× تا ۲٫۰× (در API تا ۰٫۲۵× تا ۴×) |
| **نمایش موج صدا** | ویژوالایزر زنده همراه پیشرفت پخش |
| **ذخیره خروجی** | انتخاب پوشه دلخواه یا پوشه پیش‌فرض، خروجی WAV |
| **API محلی OpenAI** | `POST /v1/audio/speech` + `GET /health` روی `127.0.0.1` |
| **پوشه نمونه جمله** | جمله‌های فارسی آماده برای تست سریع |
| **تم دارک / لایت** | با ذخیره انتخاب کاربر |
| **آفلاین کامل** | فونت وزیرمتن و موتور صوتی به‌صورت محلی باندل شده‌اند |

---

## دانلود آماده

از تب **Releases** همین ریپو، فایل سیستم‌عامل خود را بگیر:

| سیستم‌عامل | فایل | پیش‌نیاز |
|---|---|---|
| لینوکس | `ManaTTS_*.AppImage` یا `.deb` | ❌ هیچ‌چیز |
| ویندوز | `*.exe` | ❌ هیچ‌چیز |
| مک | `*.dmg` | ❌ هیچ‌چیز |

موتور صوتی به‌صورت یک باینری مستقل داخل خود بسته قرار دارد؛ **نه پایتون
لازم است، نه هیچ بسته سیستمی.** فقط دانلود کن، اجرا کن، تمام.

### اجرا با یک دستور

**لینوکس:**
```bash
chmod +x ManaTTS_*.AppImage && ./ManaTTS_*.AppImage
```

**ویندوز (PowerShell):**
```powershell
Invoke-WebRequest <لینک-فایل> -OutFile ManaTTS.exe; .\ManaTTS.exe
```

**مک:**
```bash
hdiutil attach ManaTTS.dmg && cp -R ManaTTS.app /Applications/ && open /Applications/ManaTTS.app
```

---

## API محلی — برای ایجنت‌ها و ابزارها

با باز شدن اپ، سرویس **خودکار** روی `http://127.0.0.1:7788` بالا می‌آید.
فقط روی همین دستگاه گوش می‌دهد و **هیچ اتصالی به اینترنت ندارد**.
(اگر پورت ۷۷۸۸ اشغال باشد، خودکار یک پورت آزاد دیگر انتخاب می‌کند و
آدرس واقعی در پنل «اتصال API» داخل اپ نمایش داده می‌شود.)

### نمونه درخواست

```bash
curl -X POST http://127.0.0.1:7788/v1/audio/speech \
  -H 'Content-Type: application/json' \
  -d '{"input": "سلام دنیا", "voice": "mana", "speed": 1.0, "response_format": "wav"}' \
  --output ~/Documents/ManaTTS/speech.wav

mpv ~/Documents/ManaTTS/speech.wav
```

> توجه: `curl` صدا پخش نمی‌کند، فقط فایل WAV می‌نویسد. برای شنیدن، دستور
> پخش‌کننده را جدا اجرا کنید (مثل `mpv` بالا).

### پارامترها

| پارامتر | نوع | توضیح |
|---|---|---|
| `input` | string | متن فارسی (اجباری، تا ۲۰٬۰۰۰ نویسه) |
| `voice` | string | `"mana"` (پیش‌فرض) |
| `speed` | number | **۰٫۲۵ تا ۴٫۰** — سرعت گفتار (پیش‌فرض ۱٫۰) |
| `response_format` | string | فقط `"wav"` |

### مسیرها

| مسیر | توضیح |
|---|---|
| `POST /v1/audio/speech` | تولید صدا (OpenAI-compatible) |
| `GET /health` | وضعیت سرویس و مدل |
| `GET /` | نمونه درخواست به‌صورت JSON |

### کدهای خطا

| کد | معنی |
|---|---|
| `400` | `input` خالی یا JSON نامعتبر |
| `413` | متن بیش از ۲۰٬۰۰۰ نویسه |
| `501` | `response_format` پشتیبانی‌نشده (فقط `wav`) |
| `500` | خطای موتور تبدیل متن به گفتار |

### تنظیمات اختیاری (متغیر محیطی)

| متغیر | کاربرد |
|---|---|
| `MANA_API_PORT` | تغییر پورت پیش‌فرض (پیش‌فرض `7788`) |
| `MANA_API_TOKEN` | فعال‌سازی احراز هویت با هدر `Authorization: Bearer ...` |
| `MANA_PIPER_BIN` | مسیر سفارشی باینری موتور |
| `MANA_MODEL` | مسیر سفارشی مدل |

احراز هویت **به‌صورت پیش‌فرض خاموش است** تا ابزارهای محلی راحت وصل شوند.

---

## ساخت از سورس

<details dir="rtl">
<summary><b>نصب دستی (برای توسعه)</b></summary>

### ۱. مدل (یک‌بار، حدود ۶۱ مگابایت)

```bash
mkdir -p models
curl -L https://huggingface.co/MahtaFetrat/Mana-Persian-Piper/resolve/main/fa_IR-mana-medium.onnx \
  -o models/fa_IR-mana-medium.onnx
curl -L https://huggingface.co/MahtaFetrat/Mana-Persian-Piper/resolve/main/fa_IR-mana-medium.onnx.json \
  -o models/fa_IR-mana-medium.onnx.json
```

### ۲. موتور صوتی (باینری مستقل)

```bash
pip install pyinstaller piper-tts
./scripts/build_engine.sh    # خروجی: tools/piper/piper (+ piper.exe روی ویندوز)
```

### ۳. اجرا

```bash
./scripts/dev.sh            # رابط گرافیکی (Tauri)
cargo run -- "سلام دنیا"     # خط فرمان، همراه پخش
```

</details>

### تست‌ها

```bash
cargo test -p mana-tts      # موتور تبدیل + پخش
cargo test -p mana-tts-app  # مسیریابی، موج صدا، منطق API
```

---

## ساختار پروژه

```text
mana-tts/
├── src/                      # فرانت‌اند آفلاین (بدون باندلر و بدون مرحله build)
│   ├── index.html
│   ├── style.css
│   ├── app.js
│   └── fonts/Vazirmatn-*.ttf
├── src-tauri/                # بک‌اند Tauri (Rust)
│   ├── src/main.rs           # کامندها + مسیریابی موتور
│   ├── src/api.rs            # سرور HTTP سازگار با OpenAI
│   └── tauri.conf.json
├── src/tts.rs                # قرارداد موتور (باینری نیتیو / fallback پایتون)
├── src/audio.rs              # پخش با rodio
├── models/                   # مدل (در گیت نیست؛ CI دانلود می‌کند)
├── tools/piper/              # موتور نیتیو لینوکس (در گیت نیست)
├── scripts/                  # dev.sh، phase0_spike.py
├── licenses/                 # متن GPL-3.0 و منابع سورس موتور
└── .github/workflows/        # بیلد خودکار لینوکس / ویندوز / مک
```

### معماری

هسته اپ از سیستم‌عامل مستقل است. موتور صوتی یک **باینری مستقل واحد** است که با
`scripts/build_engine.sh` (از طریق PyInstaller) برای هر سه سیستم‌عامل ساخته
می‌شود و یک قرارداد ثابت دارد:

```
piper -m <مدل> -f <خروجی.wav> --length-scale <x> --espeak_data <مسیر>
متن روی ورودی استاندارد ← WAV روی فایل خروجی
```

نتیجه: کاربر روی هر سیستمی فقط فایل را دانلود و اجرا می‌کند — بدون پایتون،
بدون نصب بسته سیستمی، بدون اینترنت.

---

## لایسنس

| جزء | لایسنس | کاربرد تجاری |
|---|---|---|
| [مدل مانا](https://huggingface.co/MahtaFetrat/Mana-Persian-Piper) | MIT | ✅ آزاد |
| [دیتاست Mana-TTS](https://huggingface.co/datasets/MahtaFetrat/Mana-TTS) | CC0-1.0 | ✅ آزاد |
| موتور [piper](https://github.com/OHF-Voice/piper1-gpl) + [espeak-ng](https://github.com/espeak-ng/espeak-ng) | **GPL-3.0** | ⚠️ آزاد، با شرط توزیع سورس |
| [onnxruntime](https://github.com/microsoft/onnxruntime) | MIT | ✅ آزاد |
| [Tauri](https://github.com/tauri-apps/tauri) / [axum](https://github.com/tokio-rs/axum) / [rodio](https://github.com/RustAudio/rodio) | MIT / Apache-2.0 | ✅ آزاد |
| فونت [Vazirmatn](https://github.com/rastikerdar/vazirmatn) | SIL OFL | ✅ آزاد |

**خلاصه:** استفاده تجاری **مجاز است**. تنها شرط، هنگام توزیع اپ، ارائه سورس
موتورهای GPL است که در پوشه [`licenses/`](licenses/) انجام شده است.

---

## منابع

- **مدل اصلی (Hugging Face):** [MahtaFetrat/Mana-Persian-Piper](https://huggingface.co/MahtaFetrat/Mana-Persian-Piper) — [مدل کارت](https://huggingface.co/MahtaFetrat/Mana-Persian-Piper#model-files)
- **دیتاست آموزشی:** [MahtaFetrat/Mana-TTS](https://huggingface.co/datasets/MahtaFetrat/Mana-TTS)
- **مقاله مدل:** [Beyond Unified Models: A Service-Oriented Approach to Low-Latency, Context-Aware Phonemization for Real-Time TTS](https://arxiv.org/abs/2512.08006) (arXiv:2512.08006)
- **موتور piper:** [OHF-Voice/piper1-gpl](https://github.com/OHF-Voice/piper1-gpl) — [مستندات خط فرمان](https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/CLI.md) — [مستندات API](https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/API_HTTP.md)
- **باینری نیتیو لینوکس:** [rhasspy/piper v1.2.0](https://github.com/rhasspy/piper/releases/tag/v1.2.0)
- **espeak-ng:** [espeak-ng/espeak-ng](https://github.com/espeak-ng/espeak-ng)
- **onnxruntime:** [microsoft/onnxruntime](https://github.com/microsoft/onnxruntime)
- **چارچوب دسکتاپ:** [tauri-apps/tauri](https://github.com/tauri-apps/tauri)
- **فونت وزیرمتن:** [rastikerdar/vazirmatn](https://github.com/rastikerdar/vazirmatn)
