# PIV-18 / TH-06 — TESTING

25.10.2026 · dev · başlangıç `1b11074e6ae6b00cdd79c0c6dae985cbb6ea6dce`.
TH-04/TH-05 kullanıcı Arch GNOME kabulüyle DONE; kapanış commit'i
`0b50b7a8a0e514b2ea7743b7ff81c983b7a12acc`. PIV-18 TESTING; TH-07 TODO.
Kullanıcı native kabulü olmadan TH-06 DONE değildir.

## TH-06.1 — Alt kenara bağlı şerit / üst orta düğme

Panel ve düğme bağımsız, non-measuring/clip edilen GtkOverlay çocukları olarak
kaldı. Panelin alt/start/end margin'i0; alt köşeleri düz, arka planı%25. Panel
viewport'un alt kenarına ve iki yanına ulaşır. Panel görünürken küçük düğmenin
bottom margin'i panelin gerçek GTK natural height isteği+4 olur; düğme üst ortada
aşağı okla durur. Panel kapalıyken düğme4px bottom margin ve yukarı oktadır.
Enabled/Available/Visible, T, hide gecikmesi, scroll ve loader mantığı değişmedi.

GSK testleri açık/kapalı Fit/Fill/Actual ve zoom/pan piksellerinin aynı kaldığını,
Picture/Overlay allocation eşitliğini, panel bottom/right/left sınırlarını ve
düğmenin panelin üstünde olduğunu assert eder. Düğme satırında düğme dışındaki
bütün pikseller Picture ile aynıdır; tam genişlikte ayrı bar eklenmez. Açıkken
pick-through fotoğraf üzerindeki düğme satırında; kapalıyken alt kenarda sınanır.
Açık panelin kendisi tekerlek/fare olaylarını almaya devam eder.

TH-06.1 native fiziksel GNOME görsel kabulü NOT RUN. Tek komut:

```bash
bash tests/run_thumbnail_system_probe.sh
```

Gerçek120 JPEG/PNG/secure cache ve iki üretim fixture penceresinde panelin alt
kenarını, üst orta düğmeyi, T/hover/scroll, seçili frame, resize ve viewport'un
aynen korunmasını incele. Gerçek kullanıcının medya/config/cache dosyaları
silinmez; test özel geçici dizinleri kullanır.


TH-06.1 Codex PASS: fmt/check/test (86 Rust:62 unit+24 integration), all-target
Clippy -D warnings; X11 synthetic layout/pixel testi, Sway/Wayland aynı geometri,
X11 koyu tema+scale2 altında120 kayıtlı TH-05 gallery/production-window regresyonu.
Bunlar kontrollü GTK/GIO pikselleridir, yeni native GNOME kabulü sayılmaz.
İlk testte eski açık-panel pick koordinatı artık gerçek alt scrollbar'a denk
 geldi; fotoğraf pick-through testi üstteki düğme satırına taşındı. Panel içi
scrollbar'ın olay alması doğru davranıştır, etkileşim kapatılmadı.

## TH-06.2 — Hazır görüntüyü tek texture geçişine kadar koruma

Kök neden `ImageView::load_request()` başında `prepare("", false)` çağrısının
Picture paintable/texture/transform ve cache pin'ini temizlemesiydi. Yeni dosya
henüz çözülmeden boş alan oluşuyordu. Düzeltme öncesi kontrollü regresyon gerçek
GTK paintable üzerinde `pending selection cleared the last ready image`
assertion'ıyla FAIL verdi; varsayıma dayalı süre/gecikme eklenmedi.

İstek ve gösterim ayrıldı: FullLoading/istenen path yeni dosyayı temsil ederken
mevcut paintable, texture, zoom/pan ve cache pin'i korunur. Decode veya geçerli
cache sonucu hazır olduğunda aynı main-context callback'i içinde transform reset,
texture ve seçili Fit/Fill/Actual sunumu değiştirilir; arada await/frame bırakılmaz.
Per-image manual transform yeni resme aktarılmaz, resampling tercihi korunur.
ActualPaintable yeniden kullanımında transform yanında texture kimliği de
zorunludur; aksi halde eski texture'ın yeni seçim için çizilmesi mümkün olurdu.

Hata Phase::Failed olarak kalır; dosya adı ve varsa “Previous photo is still
displayed.” açıklaması gösterilir. Başarılı açılış gibi sunulmaz. Yeni istek hata
metnini gizler, Loading yazısı eklemez. Video/medya kaldırma gibi açık mesaj
geçişlerinin temizleme davranışı korunur. Son başarılı tek foreground görüntü
korunur; geçmiş görüntü listesi veya yeni decoder eklenmedi. Mevcut cache/LRU,
büyük görüntü sınırı, 3-worker/latest pending admission, prefetch priority,
cancellable/token ve weak-reference kapanış kontrolleri korunur. Secure Glycin,
Bwrap, image_loader/preload/cache implementasyonları değiştirilmedi.

### Codex sonuçları — 25.10.2026

- PASS: `cargo fmt --check`, `cargo check --locked`,
  `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`:
  **86 test (62 unit,24 integration),0 failed**. Mevcut SDK ile offline/-j2
  kullanıldı. Tüm examples build PASS.
- PASS: X11 kontrollü async_cancel_probe: yavaş istek boyunca aynı paintable ve
  gerçek GSK tam-frame piksel eşitliği, zoom/pan sürekliliği, A/B/C ters sonuç,
  A/B/A,1000 latest-only pending, max3 worker, reentrant callback, iki pencere,
  kapanış/join; isimli son hata sonrası önceki pikseller ve başarılı recovery.
  Actual→Actual farklı texture geçişi yeni rengin pikselleriyle doğrulandı.
- PASS: X11 kontrollü preload_probe: cache hit/miss, foreground önceliği,
  background promotion, LRU/source identity, silinen hedefte önceki fotoğrafı
  koruyan hata ve bounded worker cleanup.
- PASS: Sway/Wayland kontrollü async_cancel_probe ve120 öğeli TH-05 katalog/
  production-window testi: tamamlanan120, failed0,64 texture, bounded cache,
  stable hücre/scroll, indeks0/79/119 hizalama ve iki pencere. Gerçek Wayland
  yüzeyleri kullanıldı; synthetic pikseller native Glycin decode sayılmaz.
- PASS: X11 production-window/GIO ve navigation kontrollü hata yarışları:
  monitor/reload/silme/eski callback/iptal/kapanış/bağımsız survivor. Önceki testin
  eski Loading metni ve istek anında Fit reset beklentileri yeni sözleşmeye
  uyarlandı; worker bariyerleri ve sonuç/iptal assertion'ları korundu.
- PIV-10 native navigation probe beklentisi de uyarlanmıştır: hata sonrası
  aynı önceki paintable ve açıklama korunur; yeni hedef beklenirken rastgele
  eski paintable değil, beklenen frame boyutu beklenir. Derleme/Clippy PASS;
  bu native probe bu oturumda çalıştırılmadı (NOT RUN).
- Native secure decode ve fiziksel Arch GNOME TH-06 kabulü **NOT RUN**.
  Bilinen Codex Bwrap uid-map/read-only proc engeli tekrar denenmedi ve güvenlik
  gevşetilmedi. Yukarıdaki kontrollü testler gerçek decoder hızı/aktif decode
  çakışması veya fiziksel GNOME'da siyah kare yokluğu kanıtı değildir.

`src/thumbnail.rs` değişikliği yalnız upstream zincirinin rustfmt düzenidir;
`1b11074` malformed cache regeneration düzeltmesi aynen korunmuştur.

### Arch GNOME kabulü

```bash
bash tests/run_thumbnail_system_probe.sh
bash tests/run_image_transition_probe.sh
```

İlk komut mevcut özel TEST medya/cache/config'iyle panel alt kenarı/üst düğme ve
iki üretim penceresi incelemesini korur. İkinci komut secure JPEG/PNG önkoşulu,
gerçek async geçiş/iptal ve cache/preload probe'larını çalıştırır. İlk secure
hata exit77 ile durdurur; başarılı skip değildir. Normal uygulama loader'ı
stub ile değiştirilmez. Her iki komut da native Wayland oturumunda çalıştırılır.

Manuel: iki pencerede panel açık/kapalı, dar/geniş resize, T/hover/tekerlek,
thumbnail tıklaması ve hızlı Sol/Sağ; büyük klasörde cache/prefetch dışı hedefe
geçerken siyah kare olmaması, eski fotoğrafın hazır yeni fotoğrafa geçmesi;
Fit/Fill/Actual ve zoom/pan korunması; bozuk/silinmiş hedefte doğru dosya adı ve
önceki fotoğraf açıklaması, sonraki sağlam fotoğrafa geçiş doğrulanmalıdır.
Tamamlanınca iki TEST penceresini kapat. Bu manuel kabul henüz yapılmadı.

### Commit kanıtları

- `0b50b7a8a0e514b2ea7743b7ff81c983b7a12acc`: TH-04/TH-05 kullanıcı native kapanışı.
- `8c077954bd14a0d41b716ad71d8095cb44c5247d`: TH-06.1 panel/düğme ve GTK geometrisi.
- TH-06.2 kod/test/bu kayıt aynı sonraki commit'tedir; kesin SHA teslim raporunda
  ve `git log --oneline -- docs/TH06_VALIDATION.md` ile bulunur.

**TH-05 DONE; TH-06 TESTING; TH-07 TODO; PIV-18 TESTING.** Faz0 bağımsız temiz
kurulum/DESKTOP-06 ve diğer PIV kapıları açık kalır.
