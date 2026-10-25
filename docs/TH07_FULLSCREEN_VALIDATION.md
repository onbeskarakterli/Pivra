# PIV-18 / TH-07 — Fullscreen — TESTING

25.10.2026 · dev · başlangıç `0571986fc956309e37a35e4c0a42221d5cce5c42`.
TH-06 kullanıcı tarafından kabul edildi; TH-07 native GNOME kabulü ayrı ve açık.
PIV-18 TESTING; TH-03'ün bağımsız önceki kabul durumu değiştirilmedi.

## TH-06 kapanış kanıtı

Kullanıcı TH-07 talimatında TH-06'nın native Arch/GNOME/Wayland ortamında kabul
edildiğini bildirdi. Kabul edilen son değişiklikler:

- `3dc589df0f8a5aba7011af21166a84f8e5c5ec5a`: açık şeritte klavye seçimini takip.
- `0571986fc956309e37a35e4c0a42221d5cce5c42`: açık panel ve düğmenin bitişmesi.

TH-06 **DONE — kullanıcı kabulü,25.10.2026**. Bu beyan yeni Codex native testi
olarak sunulmaz. Önceki TH-06 NOT RUN/FAIL kayıtları tarihsel olarak korunur.

## Mimari ve giriş

`src/fullscreen.rs` yalnız pencereye ait fullscreen isteği ve chrome yönetir;
`window.rs::build` bunu gerçek ImageView Picture'ına kurar. Aynı ApplicationWindow,
aynı widget ağacı ve aynı ImageView kullanılır. Yeni pencere/decoder/ayar eklenmedi.

- Yerleşik `GtkWindow::set_fullscreened()` fullscreen/unfullscreen talebini iletir.
  `notify::fullscreened`/`is_fullscreen()` doğrulanmış GTK durumunu izler.
- Pencere içindeki `fullscreen` GAction'ın bool state'i gerçek GTK durumudur.
  Pencereye özel pending intent hızlı F/Esc isteklerini birleştirir; gecikmiş
  configure son isteği geçersiz kılarsa istek tekrar iletilir. Global değişken,
  sleep/fade veya fullscreen diye sahte maximization yoktur.
- Window capture key controller `F`/`Shift+F` ile aç/kapa, `Esc` ile çıkış sağlar.
  Ctrl/Alt/Super/Meta kombinasyonlarına müdahale etmez. Entry/TextView'de F yazımı
  korunur. Normal modda Esc propagate eder; pending giriş Esc ile iptal edilebilir.
- Primary-button GestureClick yalnız Picture üzerindedir. Double press fullscreen
  action'ını etkinleştirir. Mevcut photo GestureDrag ile gruplama, drag'in click'i
  yutmasını önler; çift tıklamada aktif drag reset edilir, transform reset edilmez.
  Thumbnail/panel/toggle/header/menu Picture'ın sibling/üst öğeleridir; bu
  Picture-only gesture'a event bubble etmezler. Tek tıklama toggle etmez.
- Gerçek fullscreen bildiriminde HeaderBar ve refresh-banner widget'ı gizlenir;
  normal dönüşte görünür olur. Banner'ın reveal/uyarı state'i silinmez. Header
  altındaki menü ve pencere kontrol düğmeleri de unmapped olur. GTK fullscreen
  dekorasyon yönetimi kullanılır, decorated/global WM ayarları değiştirilmez.

## Görüntü ve thumbnail korunması

Fullscreen kodunun ImageView load/cancel/cache/preload çağrısı yoktur. Mevcut
paintable/texture, selected catalog, resampling ve transform aynı nesnelerdir.
Normal boyut/konum restorasyonu GTK/compositor'a bırakılır; Wayland'da global
pencere koordinatı uydurulmaz. Normal boyut otomatik testte kontrol edilir.

Mevcut Transform::geometry viewport değişince merkez farkını uygular ve pan'i
sınırlar; mutlak zoom ölçeğini reset etmez. Sınırda pan zorunlu clamp olabilir.
Non-clamped zoom/pan round trip'i tam GSK piksel eşitliğiyle sınanır.
Fit/Fill/Actual ve nearest tercihi/paintable kimliği kontrol edilir. F/Esc için
foreground load sayısı ve preload.started artmaz. Bu ölçüm sürekli mevcut fotoğraf
üzerindeki fullscreen döngüsü içindir; fotoğraf gezinmesinin normal işleri hariçtir.

TH-06 şeridi ayrı non-measuring overlay olarak kalır: alt kenar/tam genişlik,
%25 panel/opaque resimler, tema çerçevesi, açıkken bitişik düğme, kapalı küçük
bottom trigger, T/hover/hide kuralları ve açık şeridin klavye seçimini takip etmesi.
Gizli şeritte Sol/Sağ onu açmaz. Kalıcı Enabled tercihi TH-03'teki gibi paylaşılır,
görünürlük ve fullscreen pencerelere aittir. TH-03 bağımsız kabul kapısı korunur.

Thumbnail provider/generation,64 texture/16 MiB, eviction/rehydration, secure
Glycin/Bwrap, latest request cancellation, ana fotoğraf cache/preload, siyah kareyi
önleyen texture retention veya resampling implementasyonları değiştirilmedi.
Legacy UI probe'un4px aralık beklentisi kullanıcının kabul ettiği0571986'daki
**0px bitişik** düzene uyarlandı; ürün şerit CSS/politikası değiştirilmedi.

## Test katmanları ve gözlenen sonuçlar

`thumbnail_overlay_probe --fullscreen-regression` aynı üretim fullscreen modülünü
ve ayrıca gerçek `window.rs::build` akışını kullanır. Varsayılan decoder sınırı
sentetik/kontrollüdür; `--native-thumbnails` gerçek secure Glycin'e yönlendirir.
Native komut önce image_probe kapısını çalıştırır; gate sonrası gerçek görüntü
başarısızlığı hard assertion'dır, başarılı skip veya sentetik fallback değildir.

- GTK/compositor: fullscreen bool yanında **GdkToplevelState::FULLSCREEN**, gerçek
  surface/monitor alanı, header map/visibility ve photo/window allocation eşitliği
  beklenir. Cihaz ölçekleri aynı koordinat uzayında karşılaştırılır.
- F/Esc/double-click, single normal Esc, text-entry typing, hızlı9/10 istek,
  same tree/paintable, restore size, iki pencere, fullscreen'de diğerini kapatma.
- Fit/Fill/Actual; manual zoom/pan ve nearest ile normal→full→normal tam-frame
  piksel eşitliği; mode/texture/load/preload identity.
- Gerçek üretim penceresinde25 öğeli fixture kataloğu: T/hidden arrows,
  seçili19'un görünür aralığa alınması, thumbnail18'e iki tıklamayla seçim,
  toggle/thumbnail pick path'inin fotoğrafı kapsamaması, panel/düğme bounds.
- Pending yeni medya isteğinde geçiş, broken medya hatası, selected dosyayı
  silen gerçek GIO monitor olayı, boş fotoğraf state'inde giriş/çıkış ve
  fullscreen'de yeni pending istekle close. Bu active secure decoder zamanlaması
  veya decoder process-stop kanıtı değildir; mevcut lifecycle korunur.
- X11/Xfwm testinde XTest ile gerçek GTK event routing: F/Esc, fotoğrafın gerçek
  double press sayımı ve thumbnail/toggle/header double-click negatifleri.
  Bu fiziksel GNOME donanım girişi değildir. Wayland klavye/fare kontrolleri GTK
  controller sinyalleri ve gerçek pick/hiyerarşi üzerinden test edilir; compositor
  fullscreen onayı gerçektir, fiziksel Wayland input iddiası yoktur.

İlk geliştirme kontrollerinde GTK observe_controllers list modelinin GObject
item tipi düzeltilmiştir. Testler, decoder/scan-ready ile WM'nin ilk normal frame
ack'ini ayırır. İlk normal frame tamamlanmadan configure/fullscreen ölçülmez.
XTest pencere bulma `--all --pid --name` ile yapılır; xdotool'un varsayılan OR
araması başka TEST penceresini seçtiği için ilk input denemeleri FAIL olmuş,
exact pencere araması ve GTK focus onayıyla düzeltilmiştir. Assertion kaldırılmadı.
HiDPI testi zorlanmış GDK_SCALE yerine Sway output2560×2048/scale2 kullanır ve
uygulamanın gerçek scale_factor2 olduğunu da kontrol eder.

### Codex sonuç matrisi — 25.10.2026

| Kontrol | Sonuç |
| --- | --- |
| fmt/check/all-target Clippy -D warnings/test/build | PASS;88 Rust testi,0 failed; main binary + examples build |
| Sway fullscreen1x | PASS, gerçek headless compositor; sentetik pikseller |
| Sway koyu tema/gerçek output scale2 | PASS, gerçek compositor; sentetik pikseller |
| Xvfb+mevcut Xfwm4 fullscreen | PASS, gerçek X11 WM; sentetik pikseller |
| X11 XTest keyboard/photo/thumbnail/toggle/header routing | PASS |
| TH-06 retention81/120 + katalog120 + legacy overlay/visibility | PASS, X11 kontrollü |
| TH-06.2 async transition/cancel/error + PIV-17 preload | PASS, X11 kontrollü |
| run_thumbnail_system_probe.sh | NOT RUN/exit77: Codex GNOME factory development helper yok |
| run_image_transition_probe.sh, izole Sway içinde | BLOCKED/exit77: secure loader ilk frame kapısında process exit1 |
| run_fullscreen_probe.sh native/gerçek Glycin | NOT RUN: ortak secure gate engelli, tekrar denenmedi |
| Fiziksel Arch GNOME Wayland kabulü | NOT RUN — kullanıcı bekleniyor |

Yeni paket/decoder/CI/Flatpak kurulmadı. Rust/GTK SDK ve Bwrap güvenliği korunur.
Codex secure giriş kapısı bir kez denendi; bilinen ortam kısıtı için tekrar döngüsü,
NotSandboxed, namespace/host ayar değişikliği veya fallback eklenmedi.

## Tek native Arch GNOME komutu

```bash
bash tests/run_fullscreen_probe.sh
```

Özel geçici TEST medya/config/cache kullanır; kullanıcı fotoğrafları/ayarları
değişmez. Önce secure pixel gate, sonra otomatik fullscreen/üretim penceresi
kontrolleri, ardından iki gerçek üretim TEST penceresi açılır. Test bitene kadar
geçici kaynaklar tutulur; iki pencere kapatılınca temizlenir.

Fiziksel kabul listesi:

- F ve ana fotoğraf double-click ile aç/kapa; Esc ile çık. Header/menüler/kontrol
  düğmeleri tam ekranda yok, fotoğraf oranı ve tüm monitör alanı doğru.
- Thumbnail/toggle/header çift tıklaması fullscreen tetiklemez. Header'ın normal
  maximize davranışı olabilir; yeni fullscreen kısayolu değildir.
- T/hover/auto-hide, şerit alt kenarı ve bitişik düğme; Sol/Sağ gizli şeridi açmaz,
  açık şerit selected öğeyi takip eder; thumbnail click fullscreen'den çıkmaz.
- F6/F7/F8, zoom/pan ve resampling'i değiştirip round trip yap; yeniden decode,
  siyah ara kare veya reset olmamalı. Büyük viewport pan'i sınırına clamp edebilir.
- Normal boyut/konumun GNOME'un izin verdiği ölçüde geri gelmesi; iki pencerenin
  fullscreen/selection/visibility bağımsızlığı ve fullscreen close.
- Gerçek monitor/DPI/çoklu monitör ve GNOME focus/physical çift tıklama gözlemi.

Bir kontrol başarısızsa stdout/stderr ve test edilen commit SHA kaydedilmelidir.
Aktif decoder temporal overlap/secure process cleanup, fiziksel GNOME, çoklu
monitör/fractional-DPI ve normal global pozisyon henüz otomatik doğrulanmadı.
TH-07 native kullanıcı kabulü olmadan **DONE değildir**. PIV-18 TESTING.


## Commit ve nihai doğrulama kaydı

- `089775a6d53b2341fbda9b103e3172f0db7dd4ca`: per-window native fullscreen,
  F/Esc/Picture double-click, gerçek state ile header/banner görünürlüğü.
- `78d45ad4649a115ee3fa606c9d6116f7258bc9c1`: gerçek compositor/GDK ve XTest
  fullscreen probe, native script, legacy flush-toggle assertion güncellemesi.

Nihai Codex komutları mevcut SDK ile offline/-j2 çalıştırıldı; tüm zorunlu
fmt/check/test/Clippy komutları PASS. `cargo test --locked`:40 lib+24 main+24
integration =88, failed0. Main build ve examples build PASS; shell syntax ve
Python compile PASS. Ek production-window/GIO late-result/close/survivor ve
navigation race testleri de X11'de PASS. Bunlar kontrollü error worker testleri,
gerçek secure decoder process cleanup olarak sunulmaz.

Sway1x ve koyu/output2x ile gerçek fullscreen state/monitor alanı; Xvfb+Xfwm4
ile state ve XTest gerçek GTK routing PASS.2x koşusunda scale_factor=2 ayrıca
assert edildi. Normal pencere boyutu, image tree/paintable, mode/resampling ve
non-clamped manual zoom/pan pikselleri korundu; fullscreen için ek foreground
veya photo preload işi başlamadı. Mevcut thumbnail pipeline'ının viewport
büyümesinde ihtiyaç duyduğu normal cache miss/rehydration işleri bu main-photo
ölçümüyle karıştırılmaz; thumbnail pipeline sıfırlanmaz veya durdurulmaz.

Native full script fiziksel GNOME'da henüz çalıştırılmadı. Ortak secure gate
BLOCKED olduğu için Codex'te tekrar edilmedi. **TH-07 TESTING; PIV-18 TESTING.**
Faz0 bağımsız temiz native kurulum ve DESKTOP-06 bu görevle kapatılmadı.
