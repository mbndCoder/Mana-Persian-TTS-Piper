# مانـا — تبدیل متن فارسی به گفتار، آفلاین

<div dir="rtl">

**نسخه ۰٫۱٫۰** — اپ دسکتاپ کاملاً آفلاین برای تبدیل متن فارسی به گفتار با صدای
مانا، به‌همراه **API محلی سازگار با OpenAI** برای ایجنت‌ها و ابزارها.

رابط فارسی و راست‌چین • تم دارک/لایت • بدون اینترنت • بدون حساب • بدون ارسال
صدا به هیچ‌جا

</div>

---

## 📥 دانلود و اجرا

<div dir="rtl">

از تب **[Releases](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/releases)**
همین ریپو، فایل سیستم‌عامل خود را بگیر:

| سیستم‌عامل | فایل | پیش‌نیاز |
|---|---|---|
| لینوکس | `ManaTTS_*_amd64.AppImage` | ⚠️ فقط `libfuse2` در اوبونتو ۲۲٫۰۴+ (پایین) |
| لینوکس | `ManaTTS_*_amd64.deb` | ❌ هیچ‌چیز (پیشنهادی دبیان/اوبونتو) |
| ویندوز | `ManaTTS_*_x64-setup.exe` | 🐍 پایتون ۳٫۱۰+ و `pip install piper-tts` |
| مک | `ManaTTS_*_x64.dmg` | 🐍 پایتون ۳٫۱۰+ و `pip install piper-tts` |

### لینوکس

```bash
chmod +x ManaTTS_*_amd64.AppImage
./ManaTTS_*_amd64.AppImage
```

> اگر AppImage با خطای FUSE باز نشد (رایج در اوبونتو ۲۲٫۰۴ به بعد):
> ```bash
> sudo apt install libfuse2
> ```
> یا به‌جایش همان نسخه `.deb` را نصب کن که به FUSE نیاز ندارد.

### ویندوز (PowerShell — قدم‌به‌قدم)

```powershell
# ۱. پایتون (اگر نداری)
winget install Python.Python.3.12

# ۲. موتور صوتی (یک‌بار)
pip install piper-tts

# ۳. اجرای اپ
.\ManaTTS_*_x64-setup.exe
```

### مک (ترمینال — قدم‌به‌قدم)

```bash
# ۱. پایتون (اگر نداری)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install python@3.12

# ۲. موتور صوتی (یک‌بار)
pip install piper-tts

# ۳. اجرای اپ — بعد از باز کردن dmg، آیکون را به Applications بکش
```

### گوش دادن به نمونه

بعد از تولید صدا در اپ، یا با API (پایین)، فایل WAV را با هر پخش‌کننده‌ای
بشنو، مثلاً:

```bash
mpv ~/Documents/ManaTTS/speech.wav
```

</div>

---

## ✨ قابلیت‌ها

<div dir="rtl">

| قابلیت | توضیح |
|---|---|
| **تولید و پخش صدا** | متن فارسی ← گفتار طبیعی، پخش داخل خود اپ |
| **کنترل سرعت** | اسلایدر ۰٫۵× تا ۲٫۰× در رابط؛ ۰٫۲۵× تا ۴٫۰× در API |
| **موج صدا** | ویژوالایزر زنده همراه پیشرفت پخش |
| **ذخیره خروجی** | انتخاب پوشه با دیالوگ سیستمی، یا پوشه پیش‌فرض اسناد |
| **API محلی** | سازگار با OpenAI، خودکار با باز شدن اپ روشن می‌شود |
| **جمله‌های نمونه** | سه جمله فارسی آماده برای تست سریع |
| **تم دارک / لایت** | انتخاب در سربرگ، ذخیره خودکار |
| **آفلاین کامل** | فونت وزیرمتن، مدل و موتور — همه داخل بسته |

</div>

---

## 🔌 API محلی — برای ایجنت‌ها و ابزارها

<div dir="rtl">

با باز شدن اپ، سرویس **خودکار** روی `http://127.0.0.1:7788` بالا می‌آید.
فقط روی همین دستگاه گوش می‌دهد و **هیچ اتصالی به اینترنت ندارد**.
اگر پورت ۷۷۸۸ اشغال باشد، خودکار یک پورت آزاد انتخاب می‌کند و آدرس واقعی
در پنل «اتصال API» داخل اپ (همراه دستور آماده کپی) نمایش داده می‌شود.

چرا پورت ۷۷۸۸؟ با ابزارهای رایج تداخل ندارد (۳۰۰۰ گرافانا، ۸۰۰۰ vLLM،
۸۰۸۰ LocalAI، ۸۸۸۸ ژوپیتر، ۱۱۴۳۴ Ollama، ۷۸۶۰ Automatic1111، ۸۱۸۸ ComfyUI).

### نمونه درخواست

```bash
curl -X POST http://127.0.0.1:7788/v1/audio/speech \
  -H 'Content-Type: application/json' \
  -d '{"input": "سلام دنیا", "voice": "mana", "speed": 1.0, "response_format": "wav"}' \
  --output ~/Documents/ManaTTS/speech.wav

mpv ~/Documents/ManaTTS/speech.wav
```

> `curl` صدا **پخش** نمی‌کند، فقط فایل می‌نویسد. برای شنیدن، پخش‌کننده را
> جدا اجرا کن (مثل `mpv` بالا).

### پارامترها

| پارامتر | نوع | پیش‌فرض | توضیح |
|---|---|---|---|
| `input` | string | — | متن فارسی، اجباری، حداکثر ۲۰٬۰۰۰ نویسه |
| `voice` | string | `"mana"` | فعلاً فقط همین صدا |
| `speed` | number | `1.0` | سرعت گفتار، بین ۰٫۲۵ تا ۴٫۰ |
| `response_format` | string | `"wav"` | فقط `wav` پشتیبانی می‌شود |

### مسیرها

| مسیر | خروجی |
|---|---|
| `POST /v1/audio/speech` | فایل صوتی `audio/wav` |
| `GET /health` | `{"status":"ok","model":"fa-IR-mana-medium",...}` |
| `GET /` | نمونه درخواست به‌صورت JSON |

### کدهای خطا

| کد | معنی |
|---|---|
| `400` | `input` خالی یا JSON نامعتبر |
| `413` | متن بیش از ۲۰٬۰۰۰ نویسه |
| `501` | فرمت درخواستی غیر از `wav` |
| `500` | خطای موتور تبدیل |

### تنظیمات (متغیر محیطی، همه اختیاری)

| متغیر | پیش‌فرض | کاربرد |
|---|---|---|
| `MANA_API_PORT` | `7788` | پورت ثابت دلخواه |
| `MANA_API_TOKEN` | (خاموش) | با ست کردن، هدر `Authorization: Bearer ...` لازم می‌شود |
| `MANA_PIPER_BIN` | موتور باندل‌شده | مسیر سفارشی باینری موتور |
| `MANA_MODEL` | مدل باندل‌شده | مسیر سفارشی فایل `onnx` |
| `MANA_PYTHON` | جست‌وجوی خودکار | مفسر پایتون برای مسیر fallback |

احراز هویت عمداً پیش‌فرض **خاموش** است تا ابزارهای روی همان دستگاه بدون
تشریفات وصل شوند؛ سرور اصلاً از بیرون شبکه قابل دسترس نیست.

</div>

---

## 🛠 ساخت از سورس (توسعه‌دهندگان)

<div dir="rtl">

<details>
<summary><b>پیش‌نیازهای توسعه</b></summary>

- Rust پایدار (`rustup` یا بسته توزیع) + پیش‌نیازهای Tauri لینوکس:
  `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev`
- پایتون ۳٫۱۰+ با `pip install piper-tts soundfile` (برای اسکریپت مرجع)

</details>

### ۱. مدل (یک‌بار، حدود ۶۱ مگابایت — در گیت نیست)

```bash
mkdir -p models
curl -L https://huggingface.co/MahtaFetrat/Mana-Persian-Piper/resolve/main/fa_IR-mana-medium.onnx \
  -o models/fa_IR-mana-medium.onnx
curl -L https://huggingface.co/MahtaFetrat/Mana-Persian-Piper/resolve/main/fa_IR-mana-medium.onnx.json \
  -o models/fa_IR-mana-medium.onnx.json
```

### ۲. موتور صوتی لینوکس (یک‌بار — در گیت نیست)

```bash
mkdir -p tools
curl -L https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_amd64.tar.gz \
  -o tools/piper_amd64.tar.gz
tar -xzf tools/piper_amd64.tar.gz -C tools/
```

### ۳. اجرا و تست

```bash
./scripts/dev.sh            # رابط گرافیکی (Tauri dev)
cargo run -- "سلام دنیا"     # خط فرمان، همراه پخش مستقیم
cargo test --workspace      # همه تست‌ها: موتور، پخش، منطق API
python3 scripts/verify_wav.py output/oracle.wav   # اعتبارسنجی فایل صوتی
```

### ساخت باندل

```bash
cargo tauri build --bundles appimage,deb   # لینوکس
```

بیلد ویندوز و مک به‌صورت خودکار در **GitHub Actions** انجام می‌شود
(`.github/workflows/build.yml`) و خروجی‌ها در **Releases** می‌نشینند.

### ساختار پروژه

```text
mana-tts/
├── src/                      # فرانت‌اند آفلاین فارسی/راست‌چین (بدون باندلر)
│   ├── index.html / style.css / app.js
│   └── fonts/Vazirmatn-*.ttf # فونت محلی، بدون نیاز به اینترنت
├── src-tauri/                # بک‌اند Tauri (Rust)
│   ├── src/main.rs           # کامندها + مسیریابی موتور + تست‌ها
│   ├── src/api.rs            # سرور HTTP سازگار با OpenAI + تست‌ها
│   └── tauri.conf.json       # فقط منابع قابل‌حمل (موتور per-OS در CI)
├── src/
│   ├── tts.rs                # قرارداد موتور: نیتیو (stdin) / پایتون (argv)
│   └── audio.rs              # پخش با rodio
├── tests/                    # تست موتور، پخش، fallback پایتون
├── scripts/                  # dev.sh، phase0_spike.py (مرجع)، verify_wav.py
├── models/                   # مدل (gitignore؛ CI دانلود می‌کند)
├── tools/piper/              # موتور نیتیو لینوکس (gitignore)
├── licenses/                 # متن GPL-3.0 + آدرس سورس موتورها (داخل باندل هم هست)
└── .github/workflows/        # بیلد خودکار هر سه سیستم‌عامل
```

### ایده معماری (دو خط)

هسته از سیستم‌عامل مستقل است. موتور همان روش **مستندشده** هر پلتفرم است،
بدون اختراع: روی لینوکس باینری رسمی piper با متن روی **stdin** (بایت خام،
مصون از locale)؛ روی ویندوز/مک پکیج رسمی `piper-tts` با متن به‌صورت
**آرگومان** (هرگز stdin، چون پایتون آن را با locale رمزگشایی می‌کند).

</div>

---

## ❓ پرسش‌های پرتکرار

<div dir="rtl">

**آیا واقعاً آفلاین است؟**
بله. اینترنت را قطع کن و امتحان کن. تنها چیزی که اپ لازم دارد داخل خودش است.

**صدا نمی‌آید؟**
اول ولوم سیستم و خروجی پیش‌فرض صدا را چک کن (روی لینوکس: `pavucontrol`).
بعد فایل WAV ذخیره‌شده را مستقیم پخش کن تا معلوم شود مشکل تولید است یا پخش.

**روی ویندوز خطای موتور می‌دهد؟**
`pip install piper-tts` را اجرا کن و مطمئن شو `python -m piper --help` جواب
می‌دهد. اگر چند پایتون داری، مسیر درست را با `MANA_PYTHON` به اپ بده.

**پورت ۷۷۸۸ اشغال است؟**
مشکلی نیست؛ اپ خودکار پورت آزاد می‌گیرد و آدرس واقعی را در پنل API نشان
می‌دهد. یا با `MANA_API_PORT` پورت ثابت بده.

**استفاده تجاری مجاز است؟**
بله — جدول لایسنس پایین را ببین.

**اندروید چی؟**
در راه است: اپ TTS انجین مبتنی بر sherpa-onnx با همین صدای مانا. جزئیات
پایین در «نقشه راه».

</div>

---

## 🗺 نقشه راه

<div dir="rtl">

- [x] موتور فارسی آفلاین + تست صوتی مرجع
- [x] خط فرمان دسکتاپ با پخش مستقیم
- [x] رابط گرافیکی فارسی (دارک/لایت، موج صدا، سرعت، ذخیره)
- [x] API محلی سازگار با OpenAI (سرعت، توکن اختیاری، پورت هوشمند)
- [x] بیلد خودکار هر سه دسکتاپ در CI + انتشار Release
- [ ] **اندروید:** اپ TTS انجین (system Text-to-Speech) با صدای مانا —
  بر پایه [`SherpaOnnxTtsEngine`](https://github.com/k2-fsa/sherpa-onnx/tree/master/android/SherpaOnnxTtsEngine)
  که دقیقاً برای مدل‌های piper ساخته شده (نمونه‌های فارسی آماده‌اش مثل
  `fa_IR-amir-medium` [اینجا](https://k2-fsa.github.io/sherpa/onnx/tts/apk.html) هستند)

</div>

---

## 📜 لایسنس

<div dir="rtl">

| جزء | لایسنس | کاربرد تجاری |
|---|---|---|
| [مدل مانا](https://huggingface.co/MahtaFetrat/Mana-Persian-Piper) | MIT | ✅ آزاد |
| [دیتاست Mana-TTS](https://huggingface.co/datasets/MahtaFetrat/Mana-TTS) | CC0-1.0 | ✅ آزاد |
| موتور [piper](https://github.com/OHF-Voice/piper1-gpl) + [espeak-ng](https://github.com/espeak-ng/espeak-ng) | **GPL-3.0** | ⚠️ آزاد، با شرط توزیع سورس |
| [onnxruntime](https://github.com/microsoft/onnxruntime) | MIT | ✅ آزاد |
| [Tauri](https://github.com/tauri-apps/tauri) / [axum](https://github.com/tokio-rs/axum) / [rodio](https://github.com/RustAudio/rodio) | MIT / Apache-2.0 | ✅ آزاد |
| فونت [Vazirmatn](https://github.com/rastikerdar/vazirmatn) | SIL OFL | ✅ آزاد |
| کد این ریپو | MIT | ✅ آزاد (فایل LICENSE را ببین) |

**خلاصه:** استفاده تجاری **مجاز است**. تنها شرط، هنگام توزیع اپ، ارائه سورس
موتورهای GPL است که در پوشه [`licenses/`](licenses/) (و داخل خود باندل)
انجام شده است.

</div>

---

## 🔗 همه منابع، دسته‌بندی‌شده

<div dir="rtl">

**مدل و داده فارسی:**
- [MahtaFetrat/Mana-Persian-Piper](https://huggingface.co/MahtaFetrat/Mana-Persian-Piper) — مدل اصلی (وزن `fa_IR-mana-medium.onnx` + کانفیگ `.onnx.json`)
- [MahtaFetrat/Mana-TTS](https://huggingface.co/datasets/MahtaFetrat/Mana-TTS) — دیتاست آموزشی
- [مقاله مدل (arXiv:2512.08006)](https://arxiv.org/abs/2512.08006)

**موتور صوتی:**
- [OHF-Voice/piper1-gpl](https://github.com/OHF-Voice/piper1-gpl) — موتور رسمی (جانشین rhasspy/piper)
- [مستندات خط فرمان piper](https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/CLI.md)
- [مستندات سرور HTTP piper](https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/API_HTTP.md)
- [rhasspy/piper v1.2.0](https://github.com/rhasspy/piper/releases/tag/v1.2.0) — باینری نیتیو لینوکس استفاده‌شده
- [espeak-ng/espeak-ng](https://github.com/espeak-ng/espeak-ng) — فونتیکالایزر
- [microsoft/onnxruntime](https://github.com/microsoft/onnxruntime) — ران‌تایم استنتاج

**اندروید (مرحله بعد):**
- [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) — فریم‌ورک چندسکویی (Apache-2.0)
- [سورس اپ TTS انجین](https://github.com/k2-fsa/sherpa-onnx/tree/master/android/SherpaOnnxTtsEngine)
- [فهرست APKهای آماده TTS (شامل مدل‌های فارسی piper)](https://k2-fsa.github.io/sherpa/onnx/tts/apk.html)
- [مستندات اندروید sherpa-onnx](https://k2-fsa.github.io/sherpa/onnx/android/index.html)

**زیرساخت اپ:**
- [tauri-apps/tauri](https://github.com/tauri-apps/tauri) — چارچوب دسکتاپ
- [tokio-rs/axum](https://github.com/tokio-rs/axum) — سرور HTTP
- [RustAudio/rodio](https://github.com/RustAudio/rodio) — پخش صدا
- [rastikerdar/vazirmatn](https://github.com/rastikerdar/vazirmatn) — فونت فارسی

</div>
