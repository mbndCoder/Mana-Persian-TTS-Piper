# مانـا — تبدیل متن فارسی به گفتار، آفلاین

<div dir="rtl">

**نسخه ۰٫۱٫۰** — اپ دسکتاپ و اندروید کاملاً آفلاین برای تبدیل متن فارسی به
گفتار با صدای مانا، به‌همراه **API محلی سازگار با OpenAI** برای ایجنت‌ها و ابزارها.

رابط فارسی و راست‌چین • تم دارک/لایت • بدون اینترنت • بدون حساب • بدون ارسال
صدا به هیچ‌جا

</div>

---

## 📥 دانلود

<div dir="rtl">

از تب **[Releases](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/releases)**،
ریلیز مناسب سیستم خود را بگیر:

| ریلیز | سیستم‌عامل | فایل | پیش‌نیاز |
|---|---|---|---|
| **[ManaTTS Desktop](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/releases/tag/desktop-latest)** | لینوکس | `*_amd64.AppImage` | ❌ هیچ‌چیز |
| ↑ همان ریلیز | لینوکس | `*_amd64.deb` | ❌ هیچ‌چیز |
| ↑ همان ریلیز | ویندوز | `*_x64-setup.exe` | 🐍 `pip install piper-tts` |
| ↑ همان ریلیز | مک | `*_x64.dmg` | 🐍 `pip install piper-tts` |
| **[ManaTTS TTS Engine](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/releases/tag/android-latest)** | اندروید | `ManaTTS-tts-engine.apk` | ❌ هیچ‌چیز |

> بعد از هر بیلد موفق، همین ریلیزها به‌روز می‌شوند — همیشه تازه‌ترین نسخه را
> از همین چهار لینک بگیر.

</div>

---

## 🐧 نصب و اجرا روی لینوکس

<div dir="rtl">

**بدون هیچ پیش‌نیازی** — فقط فایل را دانلود و اجرا کن:

```bash
chmod +x ManaTTS_0.1.0_amd64.AppImage
./ManaTTS_0.1.0_amd64.AppImage
```

اگر AppImage باز نشد (اوبونتو ۲۲٫۰۴ به بعد مشکل FUSE دارند):

```bash
sudo apt install libfuse2
```

یا نسخه `.deb` را نصب کن که به FUSE نیاز ندارد:

```bash
sudo apt install ./ManaTTS_0.1.0_amd64.deb
```

**دیدن صدا:** داخل اپ متن بنویس و «تولید و پخش صدا» را بزن، یا از API
(پایین) استفاده کن و فایل را با `mpv` پخش کن.

</div>

---

## 🪟 نصب و اجرا روی ویندوز

<div dir="rtl">

**پیش‌نیاز:** پایتون ۳٫۱۰ یا بالاتر.

اگر پایتون نداری، در **PowerShell** این را بزن:

```powershell
winget install Python.Python.3.12
```

بعد **موتور صوتی** را نصب کن (یک‌بار، حدود ۴۰ مگ):

```powershell
pip install piper-tts
```

مطمئن شو نصب درست بوده:

```powershell
python -m piper --help
```

بعد فایل `ManaTTS_*_x64-setup.exe` را اجرا کن.

اگر چند نسخه پایتون داری و موتور نصب نشد، مسیر درست را به اپ بده:

```powershell
$env:MANA_PYTHON = "C:\Users\<نام‌کاربری>\AppData\Local\Programs\Python\Python312\python.exe"
```

</div>

---

## 🍎 نصب و اجرا روی مک

<div dir="rtl">

**پیش‌نیاز:** پایتون ۳٫۱۰ یا بالاتر.

اگر Homebrew نداری:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install python@3.12
```

بعد موتور صوتی:

```bash
pip install piper-tts
python3 -m piper --help   # بررسی نصب
```

فایل `ManaTTS_*_x64.dmg` را باز کن و `ManaTTS.app` را به پوشه
**Applications** بکش، بعد بازش کن.

</div>

---

## 🤖 نصب روی اندروید

<div dir="rtl">

### ۱. دانلود و نصب

از ریلیز **[ManaTTS TTS Engine (Android)](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/releases/tag/android-latest)**
فایل `ManaTTS-tts-engine.apk` را دانلود کن، روی گوشی بازش کن و اگر پیام
«نصب از منابع ناشناس» دیدی اجازه بده و نصب را ادامه بده.

> حجم APK حدود ۱۲۸ مگ است: مدل فارسی ۶۱ مگابایتی به‌علاوهٔ موتور صوتی برای
> هر چهار معماری گوشی. بعد از نصب **برای همیشه آفلاین** کار می‌کند.
>
> حداقل نسخهٔ اندروید: **Android 5.0 (API 21)**

### ۲. انتخاب موتور مانا در تنظیمات (مهم — بدون این مرحله صدا نمی‌آید)

نصب کردن کافی نیست. برای اینکه گوشی از صدای مانا استفاده کند، باید آن را
**دستی انتخاب کنی** — دقیقاً مثل هر موتور گفتار دیگری:

**۱.** برو به **تنظیمات** گوشی
**۲.** **دسترسی‌پذیری** (Accessibility)
**۳.** **خروجی متن به گفتار** (Text-to-speech output)
**۴.** در قسمت **موتور ترجیحی** (Preferred engine)، گزینه
    **«TTS Engine: Next-gen Kaldi»** را انتخاب کن
**۵.** دکمه ▶️ **پخش نمونه** (Play sample) را بزن

اگر صدای فارسی را شنیدی، همه‌چیز درست کار می‌کند. حالا هر برنامه‌ای که از
گفتار استفاده کند (مرورگر، کتاب‌خوان، Google Translate، پیام‌رسان‌ها) صدا را
با موتور مانا می‌خواند.

> **چرا اسمش «Next-gen Kaldi» است و «مانا» نیست؟**
> موتور اندرویدی ما همان اپ رسمی و بی‌تغییر **sherpa-onnx** است و فقط **مدل**
> عوض شده (به‌جای مدل انگلیسی، مدل فارسی مانا). نام و رابط کاربری دست‌نخورده
> مانده تا هیچ تغییری در رفتار موتور و تنظیماتش ایجاد نشود. پس در فهرست
> موتورها دنبال **«TTS Engine: Next-gen Kaldi»** بگرد — این همان موتور ماست.

> **اگر موتور مانا در فهرست نیست:** یک‌بار از تنظیمات خارج و دوباره وارد
> بخش «خروجی متن به گفتار» شو. اندروید فقط موتورهای نصب‌شده را فهرست
> می‌کند و گاهی تا ری‌استارت گوشی فهرست را تازه نمی‌کند.

> **چرا صدا نمی‌آید؟** گوشی به‌صورت پیش‌فرض موتور گوگل را انتخاب کرده.
> تا وقتی خودت موتور را عوض نکنی، موتور مانا فقط نصب است و صدا نمی‌دهد —
> این رفتار استاندارد اندروید برای همه موتورهای گفتار است.

### ۳. (اختیاری) تنظیم زبان و سرعت

در همان صفحه می‌توانی زبان را روی **فارسی** بگذاری و سرعت گفتار را تنظیم کنی.

### برگشت به موتور قبلی

اگر خواستی دوباره صدای گوگل را برگردانی، همین مسیر را برو و در «موتور
ترجیحی» گزینه **«Google Speech Services»** را انتخاب کن. حذف کردن اپ
هم کافی است.

### ۳. (اختیاری) تنظیم زبان و سرعت

در همان صفحه می‌توانی زبان را روی **فارسی** بگذاری و سرعت گفتار را
تنظیم کنی.

</div>

---

## 🔌 API محلی — برای ایجنت‌ها و ابزارها

<div dir="rtl">

با باز شدن اپ **دسکتاپ**، سرویس **خودکار** روی `http://127.0.0.1:7788`
بالا می‌آید. فقط روی همین دستگاه گوش می‌دهد و **هیچ اتصالی به اینترنت
ندارد**. اگر پورت ۷۷۸۸ اشغال باشد، خودکار یک پورت آزاد انتخاب می‌کند و
آدرس واقعی در پنل «اتصال API» داخل اپ (همراه دستور آماده کپی) نمایش داده
می‌شود.

> چرا ۷۷۸۸؟ با ابزارهای رایج تداخل ندارد: ۳۰۰۰ (گرافانا)، ۸۰۰۰ (vLLM)،
> ۸۰۸۰ (LocalAI)، ۸۸۸۸ (ژوپیتر)، ۱۱۴۳۴ (Ollama)، ۷۸۶۰ (Automatic1111)،
> ۸۱۸۸ (ComfyUI).

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

## ✨ قابلیت‌ها (دسکتاپ)

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

### قابلیت‌ها (اندروید)

| قابلیت | توضیح |
|---|---|
| **موتور گفتار سیستمی** | در تنظیمات اندروید ثبت می‌شود و با همه اپ‌ها کار می‌کند |
| **همه معماری‌ها** | arm64-v8a، armeabi-v7a، x86، x86_64 |
| **انتخاب دستی** | تنظیمات ← دسترسی‌پذیری ← خروجی متن به گفتار ← موتور ترجیحی |
| **آفلاین** | بعد از نصب هیچ اینترنتی لازم نیست |
| **رابط سیستمی** | از تنظیمات خود اندروید برای تنظیم زبان و سرعت |
| **نام در فهرست موتورها** | `TTS Engine: Next-gen Kaldi` (اپ رسمی sherpa-onnx با مدل مانا) |

</div>

---

## ❓ پرسش‌های پرتکرار

<div dir="rtl">

**آیا واقعاً آفلاین است؟**
بله. اینترنت را قطع کن و امتحان کن. همه‌چیز داخل خود بسته است.

**روی اندروید نصب شد ولی صدا نمی‌آید؟**
سه چیز را به ترتیب چک کن:
۱. آیا در تنظیمات، «موتور ترجیحی» را روی **TTS Engine: Next-gen Kaldi**
گذاشته‌ای؟ بدون این کار اصلاً صدایی پخش نمی‌شود.
۲. آیا آن گزینه اصلاً در فهرست هست؟ اگر نیست، از تنظیمات خارج و دوباره
وارد شو.
۳. دکمه «پخش نمونه» چه می‌دهد؟ اگر صدای فارسی پخش شد، موتور سالم است و
همهٔ اپ‌ها از آن استفاده می‌کنند.

**چرا در فهرست موتورهای گوشی «مانا» نمی‌بینم؟**
اسمش **TTS Engine: Next-gen Kaldi** است. موتور اندرویدی همان اپ رسمی
sherpa-onnx بی‌تغییر است و فقط مدل فارسی جایگزین شده، پس نامش همان نام
upstream ماند. این برنامهٔ دیگری نیست — همان موتور ماست.

**روی ویندوز/مک خطای موتور می‌دهد؟**
`pip install piper-tts` را اجرا کن و مطمئن شو `python -m piper --help` جواب
می‌دهد. اگر چند پایتون داری، مسیر درست را با `MANA_PYTHON` به اپ بده.

**می‌خواهم صدا را سریع‌تر/کندتر کنم؟**
دسکتاپ: اسلایدر سرعت داخل اپ، یا پارامتر `speed` در API (۰٫۲۵ تا ۴).
اندروید: از تنظیمات خود گوشی، بخش «خروجی متن به گفتار» ← «سرعت گفتار».

**دو تا موتور گفتار نصب دارم، کدام صدای فارسی می‌دهد؟**
فقط موتوری که در «موتور ترجیحی» انتخاب کرده‌ای. برای برگرداندن صدای پیش‌فرض
گوگل، همان‌جا **Google Speech Services** را انتخاب کن.

**صدا نمی‌آید ولی متن را می‌پذیرد؟**
اول ولوم سیستم و خروجی پیش‌فرض صدا را چک کن (لینوکس: `pavucontrol`).
بعد فایل WAV ذخیره‌شده را مستقیم پخش کن تا معلوم شود مشکل تولید است یا پخش.

**پورت ۷۷۸۸ اشغال است؟**
مشکلی نیست؛ اپ خودکار پورت آزاد می‌گیرد و آدرس واقعی را در پنل API نشان
می‌دهد. یا با `MANA_API_PORT` پورت ثابت بده.

**استفاده تجاری مجاز است؟**
بله — جدول لایسنس پایین را ببین.

</div>

---

## 🛠 ساخت از سورس (توسعه‌دهندگان)

<details dir="rtl">
<summary><b>راهنمای کامل توسعه</b></summary>

### دسکتاپ — پیش‌نیازها

- Rust پایدار + پیش‌نیازهای Tauri لینوکس:
  `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev`
- Node.js ۲۰+
- پایتون ۳٫۱۰+ با `pip install piper-tts soundfile`

### ۱. مدل (یک‌بار، ~۶۱ مگابایت)

```bash
mkdir -p models
curl -L https://huggingface.co/MahtaFetrat/Mana-Persian-Piper/resolve/main/fa_IR-mana-medium.onnx \
  -o models/fa_IR-mana-medium.onnx
curl -L https://huggingface.co/MahtaFetrat/Mana-Persian-Piper/resolve/main/fa_IR-mana-medium.onnx.json \
  -o models/fa_IR-mana-medium.onnx.json
```

### ۲. موتور صوتی لینوکس (یک‌بار)

```bash
mkdir -p tools
curl -L https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_amd64.tar.gz \
  -o tools/piper_amd64.tar.gz
tar -xzf tools/piper_amd64.tar.gz -C tools/
```

### ۳. اجرا و تست

```bash
./scripts/dev.sh            # رابط گرافیکی
cargo run -- "سلام دنیا"     # خط فرمان با پخش
cargo test --workspace      # همه تست‌ها
```

### ساخت باندل

```bash
cargo tauri build --bundles appimage,deb   # لینوکس
```

### اندروید

```bash
# ۱. وارد کردن سورس رسمی sherpa-onnx
python3 scripts/import_android_app.py

# ۲. فقط تعویض مدل (UI/تنظیمات دست‌نخورده می‌ماند)
python3 scripts/rebrand_android.py

# ۳. ساخت assetها (تزریق متادیتا + توکن + دیتای فونتیک)
pip install onnx
python3 scripts/android_model.py

# ۴. دانلود AAR و بیلد
curl -L https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.8/sherpa-onnx-1.13.8.aar \
  -o android/libs/
cd android && ./gradlew assembleDebug
```

### ساختار پروژه

```text
mana-tts/
├── src/                      # فرانت‌اند آفلاین فارسی (بدون باندلر)
├── src-tauri/                # بک‌اند Tauri (Rust)
│   ├── src/main.rs           # کامندها + مسیریابی
│   └── src/api.rs            # سرور HTTP سازگار با OpenAI
├── src/tts.rs, src/audio.rs  # قرارداد موتور + پخش
├── android/                  # اپ TTS انجین (Kotlin، بر پایه sherpa-onnx)
├── tests/                    # تست موتور، پخش، API
├── scripts/                  # ابزارهای توسعه
├── models/, tools/piper/     # مدل و موتور (در گیت نیستند)
├── licenses/                 # GPL-3.0 + منابع سورس
└── .github/workflows/        # CI برای دسکتاپ و اندروید
```

</details>

---

## 📜 لایسنس

<div dir="rtl">

| جزء | لایسنس | کاربرد تجاری |
|---|---|---|
| [مدل مانا](https://huggingface.co/MahtaFetrat/Mana-Persian-Piper) | MIT | ✅ آزاد |
| [دیتاست Mana-TTS](https://huggingface.co/datasets/MahtaFetrat/Mana-TTS) | CC0-1.0 | ✅ آزاد |
| موتور [piper](https://github.com/OHF-Voice/piper1-gpl) + [espeak-ng](https://github.com/espeak-ng/espeak-ng) | **GPL-3.0** | ⚠️ آزاد، با شرط توزیع سورس |
| [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) (اندروید) | Apache-2.0 | ✅ آزاد |
| [onnxruntime](https://github.com/microsoft/onnxruntime) | MIT | ✅ آزاد |
| [Tauri](https://github.com/tauri-apps/tauri) / [axum](https://github.com/tokio-rs/axum) / [rodio](https://github.com/RustAudio/rodio) | MIT / Apache-2.0 | ✅ آزاد |
| فونت [Vazirmatn](https://github.com/rastikerdar/vazirmatn) | SIL OFL | ✅ آزاد |
| کد این ریپو | MIT | ✅ آزاد |

**خلاصه:** استفاده تجاری **مجاز است**. تنها شرط، هنگام توزیع، ارائه سورس
موتورهای GPL است که در پوشه [`licenses/`](licenses/) (و داخل خود باندل)
انجام شده است.

</div>

---

## 🔗 همه منابع، دسته‌بندی‌شده

<div dir="rtl">

**مدل و داده فارسی:**
- [MahtaFetrat/Mana-Persian-Piper](https://huggingface.co/MahtaFetrat/Mana-Persian-Piper) — مدل اصلی
- [MahtaFetrat/Mana-TTS](https://huggingface.co/datasets/MahtaFetrat/Mana-TTS) — دیتاست آموزشی
- [مقاله مدل (arXiv:2512.08006)](https://arxiv.org/abs/2512.08006)

**موتور صوتی دسکتاپ:**
- [OHF-Voice/piper1-gpl](https://github.com/OHF-Voice/piper1-gpl) — موتور رسمی
- [مستندات خط فرمان](https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/CLI.md)
- [rhasspy/piper v1.2.0](https://github.com/rhasspy/piper/releases/tag/v1.2.0) — باینری لینوکس
- [espeak-ng](https://github.com/espeak-ng/espeak-ng) — فونتیکالایزر

**موتور صوتی اندروید:**
- [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) — فریم‌ورک (Apache-2.0)
- [سورس TTS Engine](https://github.com/k2-fsa/sherpa-onnx/tree/master/android/SherpaOnnxTtsEngine)
- [فهرست APKهای فارسی piper](https://k2-fsa.github.io/sherpa/onnx/tts/apk.html)

**زیرساخت:**
- [tauri-apps/tauri](https://github.com/tauri-apps/tauri) — چارچوب دسکتاپ
- [tokio-rs/axum](https://github.com/tokio-rs/axum) — سرور HTTP
- [RustAudio/rodio](https://github.com/RustAudio/rodio) — پخش صدا
- [rastikerdar/vazirmatn](https://github.com/rastikerdar/vazirmatn) — فونت فارسی
- [microsoft/onnxruntime](https://github.com/microsoft/onnxruntime) — ران‌تایم استنتاج

**دانلودها (ریلیزها):**
- [ManaTTS Desktop — لینوکس / ویندوز / مک](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/releases/tag/desktop-latest) — AppImage، deb، exe، dmg
- [ManaTTS TTS Engine — اندروید](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/releases/tag/android-latest) — APK موتور گفتار
- [همهٔ ریلیزها](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/releases)

**ریپوی ما:**
- [mbndCoder/Mana-Persian-TTS-Piper](https://github.com/mbndCoder/Mana-Persian-TTS-Piper) — سورس کامل
- صفحهٔ اصلی پروژه: [github.com/mbndCoder/Mana-Persian-TTS-Piper](https://github.com/mbndCoder/Mana-Persian-TTS-Piper)
- گزارش مشکل: [Issues](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/issues)
- لایسنس: [MIT](https://github.com/mbndCoder/Mana-Persian-TTS-Piper/blob/master/LICENSE) — فایل کامل [GPL-3.0](https://github.com/OHF-Voice/piper1-gpl/blob/master/LICENSE)

**مستندات رسمی مرتبط:**
- [راهنمای Tauri](https://tauri.app/start/) — ساخت اپ دسکتاپ
- [مستندات piper (OHF)](https://github.com/OHF-Voice/piper1-gpl) — موتور اصلی
- [مستندات sherpa-onnx](https://k2-fsa.github.io/sherpa/onnx/) — موتور اندروید

</div>
