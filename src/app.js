/* ManaTTS frontend — vanilla JS, no build step, works fully offline. */
(function () {
  'use strict';

  var FA_DIGITS = ['۰', '۱', '۲', '۳', '۴', '۵', '۶', '۷', '۸', '۹'];
  function faNum(x) {
    return String(x).replace(/[0-9]/g, function (d) { return FA_DIGITS[+d]; });
  }

  var $ = function (id) { return document.getElementById(id); };
  var textEl = $('text'), speedEl = $('speed'), speedVal = $('speedVal'),
      speakBtn = $('speakBtn'), saveBtn = $('saveBtn'), wave = $('wave'),
      statusEl = $('status'), durationEl = $('duration'),
      folderEl = $('folder'), charCount = $('charCount');

  var invoke = null;
  if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
    invoke = window.__TAURI__.core.invoke;
  }

  var state = { wavBase64: null, peaks: [], duration: 0, busy: false };
  var audioCtx = null, currentSrc = null;

  /* ---- theme ---- */
  function setTheme(t) {
    document.documentElement.setAttribute('data-theme', t);
    try { localStorage.setItem('manatts-theme', t); } catch (e) {}
  }
  var savedTheme = null;
  try { savedTheme = localStorage.getItem('manatts-theme'); } catch (e) {}
  setTheme(savedTheme === 'light' ? 'light' : 'dark');
  $('themeBtn').addEventListener('click', function () {
    var cur = document.documentElement.getAttribute('data-theme');
    setTheme(cur === 'dark' ? 'light' : 'dark');
    drawWave(0);
  });

  /* ---- settings persist ---- */
  try {
    var s = JSON.parse(localStorage.getItem('manatts-settings') || '{}');
    if (s.speed) { speedEl.value = s.speed; }
    if (s.folder) { folderEl.value = s.folder; }
  } catch (e) {}
  function persistSettings() {
    try {
      localStorage.setItem('manatts-settings', JSON.stringify({
        speed: parseFloat(speedEl.value), folder: folderEl.value
      }));
    } catch (e) {}
  }

  function fmtSpeed(v) { return faNum(v.toFixed(2).replace('.', '٫')) + '×'; }
  function refreshSpeed() { speedVal.textContent = fmtSpeed(parseFloat(speedEl.value)); }
  speedEl.addEventListener('input', function () { refreshSpeed(); persistSettings(); });
  refreshSpeed();
  folderEl.addEventListener('input', persistSettings);

  textEl.addEventListener('input', function () {
    charCount.textContent = faNum(textEl.value.length) + ' / ' + faNum(10000);
  });

  var chips = document.querySelectorAll('.chip');
  for (var i = 0; i < chips.length; i++) {
    (function (c) {
      c.addEventListener('click', function () {
        textEl.value = c.getAttribute('data-sample');
        textEl.dispatchEvent(new Event('input'));
        textEl.focus();
      });
    })(chips[i]);
  }
  $('clearBtn').addEventListener('click', function () {
    textEl.value = '';
    textEl.dispatchEvent(new Event('input'));
    textEl.focus();
  });

  function setStatus(msg, isErr) {
    statusEl.textContent = msg || '';
    statusEl.className = 'status' + (isErr ? ' error' : '');
  }

  /* ---- waveform ---- */
  function waveColor() {
    return getComputedStyle(document.documentElement).getPropertyValue('--accent').trim() || '#22c55e';
  }
  function drawWave(progress) {
    var dpr = window.devicePixelRatio || 1;
    var w = wave.clientWidth, h = wave.clientHeight || 72;
    wave.width = w * dpr; wave.height = h * dpr;
    var ctx = wave.getContext('2d');
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, w, h);
    var peaks = state.peaks;
    if (!peaks.length) {
      ctx.fillStyle = '#808080';
      ctx.font = '13px Vazirmatn, sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText('موج صدا اینجا نمایش داده می‌شود', w / 2, h / 2);
      return;
    }
    var n = peaks.length, bw = w / n, mid = h / 2;
    for (var i = 0; i < n; i++) {
      var ph = Math.max(2, peaks[i] * (h - 12));
      ctx.fillStyle = (i / n <= progress) ? waveColor() : '#64748b';
      ctx.fillRect(i * bw, mid - ph / 2, Math.max(1, bw - 1), ph);
    }
  }
  window.addEventListener('resize', function () { drawWave(0); });
  drawWave(0);

  function b64ToBytes(b64) {
    var bin = atob(b64), len = bin.length, out = new Uint8Array(len);
    for (var i = 0; i < len; i++) { out[i] = bin.charCodeAt(i); }
    return out;
  }

  function stopAudio() {
    if (currentSrc) { try { currentSrc.stop(); } catch (e) {} currentSrc = null; }
  }

  function decodeWav(bytes) {
    // WebKitGTK may only support the callback form; support both.
    return new Promise(function (resolve, reject) {
      var ab = bytes.buffer.slice(0);
      var maybePromise = audioCtx.decodeAudioData(ab, resolve, reject);
      if (maybePromise && typeof maybePromise.then === 'function') {
        maybePromise.then(resolve, reject);
      }
    });
  }

  /* Playback goes through Rust/rodio, not WebAudio: WebKit's audio backend
     never starts on this platform, so every clip is silently dropped. The
     waveform is animated on a wall clock for the same reason. */
  function animateWhilePlaying(durationSec) {
    stopAudio();
    var start = performance.now();
    currentSrc = { stop: function () { clearInterval(currentSrc._t); } };
    currentSrc._t = setInterval(function () {
      var p = (performance.now() - start) / 1000 / durationSec;
      if (p >= 1) { stopAudio(); drawWave(0); return; }
      drawWave(p);
    }, 80);
  }

  /* ---- actions ---- */
  speakBtn.addEventListener('click', function () {
    if (!invoke) { setStatus('خارج از محیط برنامه اجرا شده‌اید', true); return; }
    var text = textEl.value.trim();
    if (!text) { setStatus('اول متنی بنویسید', true); textEl.focus(); return; }
    if (state.busy) { return; }
    state.busy = true;
    speakBtn.disabled = true;
    setStatus('در حال تولید صدا…');
    invoke('synthesize', { text: text, speed: parseFloat(speedEl.value) })
      .then(function (res) {
        state.wavBase64 = res.wavBase64;
        state.peaks = res.peaks;
        state.duration = res.durationSecs;
        state.outPath = res.outPath;
        durationEl.textContent = 'مدت: ' + faNum(res.durationSecs.toFixed(1)) + ' ثانیه';
        saveBtn.disabled = false;
        setStatus('در حال پخش…');
        drawWave(0);
        animateWhilePlaying(res.durationSecs);
        return invoke('play', { wavPath: res.outPath });
      })
      .then(function () { setStatus('تمام شد'); })
      .catch(function (err) { setStatus(String(err), true); })
      .then(function () {
        state.busy = false;
        speakBtn.disabled = false;
      });
  });

  /* ---------------- local API ---------------- */
  var apiRunning = false;

  function renderApi(info) {
    apiRunning = !!info.running;
    $('apiDot').className = 'dot' + (apiRunning ? ' on' : '');
    $('apiState').textContent = apiRunning
      ? 'روشن — پورت ' + faNum(info.port)
      : 'خاموش';
    $('apiToggle').textContent = apiRunning ? 'خاموش کردن' : 'روشن کردن';
    $('apiUrl').value = apiRunning ? info.url : '';
    $('apiCurl').textContent = apiRunning ? info.curl : '—';
    $('apiOut').textContent = apiRunning ? 'فایل خروجی این دستور: ' + info.outputPath : '';
    $('apiAuth').textContent = apiRunning && info.authRequired
      ? 'این سرویس توکن می‌خواهد (متغیر محیطی MANA_API_TOKEN).'
      : '';
  }

  function refreshApi() {
    if (!invoke) { return; }
    invoke('api_status').then(renderApi).catch(function () {});
  }

  $('apiToggle').addEventListener('click', function () {
    if (!invoke) { return; }
    var btn = $('apiToggle');
    btn.disabled = true;
    var call = apiRunning ? invoke('api_stop') : invoke('api_start');
    call.then(renderApi)
        .catch(function (err) { setStatus('خطای API: ' + err, true); })
        .then(function () { btn.disabled = false; });
  });

  $('copyUrl').addEventListener('click', function () {
    var url = $('apiUrl').value;
    if (!url) { return; }
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(url)
        .then(function () { setStatus('آدرس کپی شد'); })
        .catch(function () { $('apiUrl').select(); setStatus('برای کپی: Ctrl+C', true); });
    } else {
      $('apiUrl').select();
      setStatus('برای کپی: Ctrl+C', true);
    }
  });

  if (invoke) {
    // Agents and tools need the endpoint without opening the app first.
    invoke('api_start')
      .then(renderApi)
      .catch(function (err) { setStatus('شروع API ناموفق بود: ' + err, true); });
  }

  $('pickFolderBtn').addEventListener('click', function () {
    if (!invoke) { setStatus('خارج از محیط برنامه اجرا شده‌اید', true); return; }
    var btn = $('pickFolderBtn');
    btn.disabled = true;
    setStatus('در انتظار انتخاب پوشه…');
    invoke('pick_folder')
      .then(function (d) {
        if (d) { folderEl.value = d; persistSettings(); setStatus('پوشه انتخاب شد'); }
        else { setStatus('انتخاب پوشه لغو شد'); }
      })
      .catch(function (err) { setStatus('انتخاب پوشه ناموفق: ' + err, true); })
      .then(function () { btn.disabled = false; });
  });

  $('defaultFolderBtn').addEventListener('click', function () {
    if (!invoke) { return; }
    invoke('default_dir').then(function (d) {
      folderEl.value = d;
      persistSettings();
    }).catch(function (err) { setStatus(String(err), true); });
  });

  saveBtn.addEventListener('click', function () {
    if (!invoke || !state.wavBase64) { return; }
    setStatus('در حال ذخیره…');
    var name = 'mana-' + Date.now();
    invoke('save_audio', { wavBase64: state.wavBase64, dir: folderEl.value.trim(), filename: name })
      .then(function (p) { setStatus('ذخیره شد: ' + p); })
      .catch(function (err) { setStatus(String(err), true); });
  });

  if (invoke) {
    invoke('default_dir').then(function (d) {
      if (!folderEl.value) { folderEl.value = d; }
    }).catch(function () {});
  }
})();
