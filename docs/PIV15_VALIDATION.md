# PIV-15 — Smooth / Nearest resampling

24.10.2026 · `dev` · **DONE**, gerçek Arch/GNOME kabulü tamamlandı.
Başlangıç: `28505e4d377759f1550b20d8b199841ce6aa80dd`.
Güncel origin/dev fast-forward ile alındı. PIV-01–14 DONE ve kullanıcı kaynaklı
native kanıtlar korunur; bu görevdeki yeni native kabul yerine kullanılmaz.

## Kullanıcı davranışı ve tercih ömrü

Mevcut Fit/Fill/%100 menüsüne küçük **Resampling** bölümü eklendi:
**Smooth** varsayılan, **Nearest** alternatif. Stateful `win.image-resampling`
action seçili kaliteyi menüde işaretler. Mode action, F6/F7/F8, Zoom göstergesi ve
Sol/Sağ gezinmesi değişmedi; büyük toolbar veya yeni kısayol eklenmedi.

Tercih **pencere başınadır**, yeni medya/Yenile/bozuk görüntüden toparlanmada
korunur. Yeni pencere Smooth başlar. Uygulama yeniden başlatıldığında kalıcı
kaydetme eklenmedi: mevcut Settings yalnız in-memory varsayılanlardır;
GSettings veya yeni ayar dosyası bu görevin kapsamına alınmadı. Kullanıcı
metinleri `viewing.rs` içindeki Resampling modelinde merkezidir; PIV-39 tr/en
i18n planı korunur.

Filtre değişimi geometry/Transform/reset nesline dokunmaz. Yakınlaştırılmış
ve aktif drag durumundaki aynı kaynak texture ile çalışır; zoom/pan, merkez,
clip ve mod korunur. Manuel adapter varsa aynı paintable invalidate edilir;
Smooth preset texture ile Nearest adapter arasında geçişte paintable doğrudan
yeni **Some** değerine atanır, görüntü boşaltılmaz. Yeni decode veya uygulama
kaynaklı büyük CPU bitmap/texture kopyası yoktur. Geç worker callback'i güncel
paylaşılan filtre tercihini kullanır; eski seçimi geri getirmez.

## GTK/GSK teknik kanıtı

Mevcut GTK 4.18.6 SDK GIR belgeleri ve gtk4-rs/gsk4 0.10.3 kaynakları incelendi:

- `Gtk.Snapshot.append_scaled_texture`, **GTK 4.10** itibarıyla filtreyi açıkça
  seçer. Projenin mevcut GTK minimumu 4.16 / Cargo `v4_16` özelliği bunu kapsar;
  yeni bağımlılık veya ABI/sürüm hack'i gerekmiyor.
- `append_texture` belgesi, ölçekleme gerektiğinde **linear** filtre kullandığını
  açıkça belirtir. Smooth Fit/Fill'in mevcut GtkPicture/texture yolu korunur.
- Actual/manual ve Nearest preset sunumları mevcut paintable üzerinden
  `append_scaled_texture` kullanır: **Smooth → GSK ScalingFilter::Linear**,
  **Nearest → GSK ScalingFilter::Nearest**. Geometri hesapları aynı kalır.
- GSK belgesi filtrelerin gerçek uygulamasını render pipeline'a bırakır.
  CPU testleri Cairo renderer içindir; GPU renk uzayı/yuvarlama/hardware
  performansı bunlardan PASS sayılmaz. Native komut kullanıcının renderer'ını
  zorla Cairo'ya çevirmez. Filtre desteklenmezse sessiz başka algoritma seçen
  uygulama fallback'i yoktur; piksel testleri başarısızlığı görünür kılar.

Smooth burada GTK'nın **linear/bilinear** seçimidir; Lanczos veya trilinear/
mipmap kalite seviyesi eklenmedi. Native GPU render-node yolu korunur;
uygulama Python/CPU resizer kullanmaz. Render pipeline'ın kendi iç bellek
politikasına yönelik sıfır allocation veya donanım performansı garantisi yoktur.

## Gerçek test sonuçları

| Kontrol | Sonuç |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo check --locked`, `cargo build --locked` | PASS; mevcut stable/offline/-j2 helper |
| `cargo test --locked` | **64 PASS**, 40 unit + 24 integration, 0 failed/ignored |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| Tüm örneklerin locked/offline derlenmesi | PASS |
| Python aktif decoder gözlemci unit testleri | **30 PASS**, decoder sonucu değildir |
| Fixture bütünlüğü | Önceki asset hash'leri + yeni checker/manifest deterministik PASS |
| GTK/GIO/helper çağrıları | **27 scoped PASS**: X11 16, Sway 9, Weston 2 |
| Gerçek secure decoder kapıları | **10 NOT RUN / exit77** |
| Shell/Python syntax, whitespace | PASS |

### Model / fixture

Yeni Resampling unit testi varsayılan Smooth, iki canonical ID ve bilinmeyen
seçim reddini doğrular. Önceki geometri, CLI, doğal sıra, katalog, desktop,
input ve iptal testleri korunur.

Yeni özgün `tests/fixtures/resampling-checker.png`: 16×16, opaque RGBA siyah/beyaz
tek-piksel checker. Standart Python test üreticisi sabit uncompressed DEFLATE ile
üretir; harici encoder veya yeni lisans kararı yoktur. Manifest'e yeni satır
eklendi; eski PNG/model-video ve 18 image fixture byte'ları değişmedi.

SHA256: `62e7c1175a5faed50333bcfdae10eaae35fcce7613934bf6db61fba1372a491d`
(1108 bayt). `generate.py --check` hem byte hem manifest doğrulamasını yapar.

### Gerçek GTK/GSK rendering, sentetik kaynak

Mevcut view_modes_probe'ye `--synthetic --resampling` senaryosu eklendi.
Üretim `present_filtered`, paintable ve gerçek GSK renderer kullanılır; source
texture testte oluşturulur. CPU matematik referansı ile **gerçek render download
pikselleri** karşılaştırılır; yalnız model PASS'ı değildir.

Üç çağrı: Xvfb, Xvfb `GDK_SCALE=2`, headless native Sway; hepsinde Cairo renderer.
Her birinde **34 snapshot**, toplam **102** yeni filtre snapshot'ı. PIV-13/14'ün
360 sentetik snapshot'ı da korunur; toplam **462**. Bunlar gerçek Glycin decode
veya fiziksel GNOME/HiDPI kabulü değildir.

- Kesin 2×2 siyah/beyaz örnekte 2×/4×/8×, 2.5×, 1×, %10/%3200 ve Fit/Fill/Actual.
- Nearest: fully opaque iç pikseller yalnız 0 veya 255; RGB eşit, kaynak texel
  konumuna uygun seçim. Kaynak sınırındaki eşit uzaklık tie yönü backend'ler
  arasında zorlanmaz; beklenmeyen ara renk hiçbir opaque iç pikselde kabul edilmez.
- Smooth: yalnız görüntü farkı değil, Cairo'nun encoded-channel bilinear
  referansına karşı piksel değerleri (en fazla 5/255 tolerans) ve gerçekten
  üretilen ara değerler doğrulanır. GPU renk uzayı aynı sayı varsayılmaz.
- 16×16 checker 0.5× küçültmede Nearest palette ve tekrarlanan snapshot byte
  eşitliği; Smooth ara renkleri ve farklı çıktı. %10'da çok küçük görüntüde
  opaque iç piksel bulunmayabileceği için küçültme kanıtı yalnız buna dayanmaz.
- 1600×1200 degrade kaynakta preset/manuel pan sırasında Nearest→Smooth→Nearest:
  kaynak renk koordinatları ve render-node clip sınırları; aynı geometry;
  aynı filtreye dönüşte byte eşitliği. Dikey/küçük/büyük kaynakların eski mod/
  zoom/pan snapshot regresyonları da PASS.
- 1000 hızlı kalite değişiminde aynı manuel paintable; preset Smooth'a dönüşte
  önceki adapter weak referansının bırakılması. Bu GObject lifetime kontrolüdür,
  genel process RSS/GPU leak benchmark'ı değildir.

Outer fractional image/clip coverage alpha'sı filtreyle yeni kaynak rengi
üretilmesi sayılmaz; palette testi fully opaque görüntü içini ayırır. Viewport
clip sınırı tüm snapshot'larda ayrıca kontrol edilir.

Önceki 24 GUI/helper çağrısı korunur; yeni üç filtre çağrısıyla 27 olur. Kontrollü
üretim window/worker yarışında pending load sırasında Nearest seçimi, yeni
isteklerde ve eski callback'ler sonrası tercihin korunması, diğer pencerenin
Smooth kalması, 7 worker sayısının değişmemesi ve filtre değiştirerek kapanış
PASS. Bunlar gerçek decoder testi değildir. Sol/Sağ, F6/F7/F8, Yenile, monitor,
bozuk medya, zoom/pan ve cleanup regresyonları PASS.

Sway gerçek Wayland yüzeyidir (DISPLAY/XWayland yok); action/controller
sinyalleri fiziksel kullanıcı girişi değildir. Weston normal close **NOT RUN**
kalır; SIGTERM cleanup onun yerine geçmez. Hardware GPU, fiziksel HiDPI ve
FPS/latency/RSS ölçümleri bu görevde **NOT RUN / NOT MEASURED**.

### Gerçek secure decode — açık engel

Ayrı girişlerle PNG/JPEG-first image_probe, image_gui_probe, strict navigation,
strict folder watch, strict medya hatası, active_decode_close, strict view_modes,
strict zoom/pan ve yeni strict resampling denendi: **10 exit77 / NOT RUN**.
İlk secure baseline oluşmadı; bilinen Codex Bwrap/read-only proc UID-map kısıtı
sürer. Yeni native pozitif yollar yalnız derlendi, PASS sayılmadı. Önceki
kullanıcı PIV-09–14 native kanıtı geriye dönük veya bu görev için yeniden PASS
olarak kullanılmaz. Glycin/Bwrap, decoder ve sandbox seçimi değiştirilmedi;
NotSandboxed/güvenlik bypass'ı eklenmedi.

## Native Arch/GNOME kabulü

Güncel `dev` üzerinde gerçek GNOME Wayland oturumunda:

```bash
bash tests/run_resampling_probe.sh
```

HEAD yazılır, locked build yapılır, GDK_BACKEND=wayland kullanılır. Önce mevcut
image_probe gerçek JPEG/PNG/EXIF 1–8 piksel kapısı; sonra üretim pencere/renderer
resampling senaryosu çalışır. Renderer kullanıcının normal GSK backend'idir.
Native kabulde `--synthetic` eklemeyin. İlk baseline engeli exit77; başarılı
baseline sonrasındaki sağlam görüntü/piksel/assertion hataları FAIL'dir.

Hazırlanan yeni native pozitif yol **henüz NOT RUN**:

- Gerçek JPEG/PNG ve EXIF sonrası texture'larda filtre geçişi, mevcut mod/render/
  resize kontrolleri. Her fotoğrafta iki filtrenin farklı olması zorlanmaz:
  downscale/uygun texel hizasında sonuçlar meşru olarak aynı olabilir.
- Aktif drag sırasında Smooth→Nearest→Smooth: aynı manuel paintable ve mod,
  Smooth snapshot'ının byte olarak aynı dönmesi (pan/zoom korunması), paintable'ın
  hiçbir notify'da None olmaması, sonraki drag update'inin çalışması.
- Yeni exact-color PNG **üretim güvenli decoder'ından** açılır. 8× büyütmede
  Nearest palette/spatial sınırlar, Smooth'un ara değerleri ve farklı çıktısı;
  aynı texture'a dönüş, yeni fotoğrafta Nearest'in korunması, bozuk PNG'den
  sağlam görüntüye tercih kaybetmeden dönüş, iki gerçek pencerenin farklı
  tercihi ve özel kaynak kopyalarının byte bütünlüğü.
- Daha önceki iki pencere/mod ve source integrity kapıları da çalışır.

Kısa manuel kabul: küçük/pixel-art fotoğrafta büyütüp menüden iki kaliteyi
karşılaştırın; fotoğrafı sürüklerken tercihi değiştirip konumun sabit kaldığını,
yeni fotoğrafta tercihin korunduğunu ve diğer pencerenin etkilenmediğini görün.
F6/F7/F8, Sol/Sağ, resize ve Yenile akışını kısa kontrol edin.

**PIV-15 TESTING**, native SHA/sonuç bildirilmeden DONE değil. PIV-16 async
mimarisi/cache/preload/minimap/video/pinch/HDR/edit/i18n/paketleme/CI başlatılmadı.
PIV-01–14 DONE, Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus
NOT RUN ve PIV-39 tr/en planı korunur.

## Kaynak SHA'ları

| Commit | İçerik |
|---|---|
| `5fec698673ea331eb6bf43922381349e26b2cd09` | Pencereye özel resampling state/action, GSK Linear/Nearest, mevcut texture/geometry koruması ve menü bölümü; eski zoom probuna yeni tercih argümanı |
| `fd36f560203f7de6830ef3c3a256c33df53a91f9` | Gerçek piksel/CPU referans/snapshot ve preference lifecycle kapıları, exact-color PNG/manifest üretimi, tek native giriş |


## PIV-15 Arch native ve GNOME manuel kabul kapanışı — 24.10.2026 — DONE

Kullanıcı `3313113246baca8b2e19073b0ed20c64f569a50c` üzerinde `bash tests/run_resampling_probe.sh` çalıştırdı: **exit 0**, gerçek Glycin JPEG/PNG/EXIF 1–8, secure PNG checker Nearest palette ve Smooth interpolasyon, aynı decoded texture/paintable, yeni fotoğrafta tercih, iki pencere izolasyonu, kaynak bütünlüğü PASS. Beklenen bozuk PNG hata logları test verisindendi. Kullanıcı normal Pivra'da küçük checker görselinde Smooth/Nearest arasında görünür piksel farkını ve değiştirirken zoom/konumun korunduğunu **manuel PASS** olarak onayladı. Fill görüntüyü alanı dolduracak şekilde ölçeklediğinden bu davranış doğrudur; %100 hatası doğrulanmadı. Çok küçük test görsellerinde %3200 manuel zoom sınırı yetersiz hissedilebilir; mevcut PIV-14 sınırı korunur, artırma önerisi ayrı değerlendirmedir, PIV-15 kabulünü engellemez. Gerçek GPU/HiDPI eşlemesi, FPS/RSS ölçümü ve decoder overlap **NOT MEASURED**; Codex exit77 kayıtları geriye dönük PASS sayılmaz. **PIV-15 DONE, PIV-16 TODO.** Faz 0 bağımsız temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 i18n planı açık kalır.
