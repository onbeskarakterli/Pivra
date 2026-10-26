# PIV-RELEASE-01 — Pivra v0.1.0 final yayın denetimi

Tarih: 26.10.2026 · Europe/Istanbul · Dal: dev.

## 1. Yönetici özeti

**Son yayın kararı: NO-GO.** Bu karar mevcut uygulamanın kullanıcı bilgisayarında
çalışmadığı anlamına gelmez. Güncel kaynakta yayın doğrulama kapıları kırmızı ve
planlanan dağıtımın somut ön koşulları eksiktir. Yeni bir üretim çökmesi, veri
kaybı veya sandbox bypass'ı bu çalışmada kanıtlanmadı.

**Beş yayın engeli** sayıldı: B1 katalog GUI kapısı; B2 aynı wheel sözleşmesi
uyuşmazlığına bağlı üç GUI probe'u; B3 zorunlu fmt kontrolü; B4 açık lisans/yayın
kararı; B5 eksik denetlenebilir kurulum/paketleme teslimatı. Bunlar beş ayrı
üretim çökmesi değildir. B4/B5 dağıtım hazırlığı engelleridir.

Olumlu sonuçlar: 186 Rust testi, 37 thumbnail Python testi, 33 observer/report
Python testi; check/build/Clippy; fixture bütünlüğü; kontrollü Fit/Fill/Actual,
resampling, preload, navigation race, fullscreen ve 81/120 retention PASS.
2× HiDPI'da 59 texture / 16.682.368 bayt ölçüldü; 64/16 MiB üst sınırları korundu.

Gerçek GNOME Shell/Wayland, güncel secure JPEG/PNG/EXIF decode, aktif loader
kapanışı, 500/1000 gerçek thumbnail, temiz Arch kurulum ve gerçek Nautilus kabulü
bu ortamda doğrulanamadı. Bunlara PASS verilmedi. Önceki kullanıcı Arch sonuçları
tarihsel kullanıcı kanıtıdır; bu denetimin bağımsız native koşusu değildir.

## 2. Denetlenen commit ve çalışma ortamı

- Talimattaki tarihsel referans: 7adb4b071ac26e514efd5ce48b562b4d89e8a128.
- **Fiilen denetlenen güncel kaynak: bd9c669af96d1ff7fd154b750bd08b3201f979e0.**
- Yerel temiz dev, 6efd599'dan normal fetch + pull --ff-only ile güncellendi.
- Son iki commit: 52fe9fc katalog geometri kontrolü; bd9c669 fullscreen CSS kontrolü.
  İkisi de ilk sırada yeniden çalıştırıldı; eski hataların sonuçları kopyalanmadı.
- Debian GNU/Linux 13.6 Codex; Rust/Cargo 1.99.1; GTK 4.18.6;
  libadwaita 1.7.6; sabitlenmiş Rust Glycin 3.0.8.
- Mevcut SDK etkinleştirmesi: /workspace/pivra-environment/piv09/image-activate.sh.
  Yeni SDK/paket kurulmadı; sistem veya güvenlik ayarı değiştirilmedi.
- Grafik: Xvfb/X11 ve özel headless Sway 1.10.1; Wayland koşularında XWayland
  kapalı ve DISPLAY yok. Kontrollü renderer Cairo. Fiziksel GNOME değildir.
- Grafik testlerinde özel XDG config/cache/runtime dizinleri kullanıldı.
  Kullanıcının gerçek medya/ayar/cache dosyaları kullanılmadı veya temizlenmedi.
- Kaynak, test, Cargo manifest/lock, main, tag ve release değiştirilmedi.
  Bu görevde commit/push kapsamı yalnız bu rapordur. build çıktıları teslimat değildir.

## 3. RC-03–RC-10 sonuç tablosu

Aşağıdaki birincil durum, bölümün tamamına aittir; alt testlerin PASS olması
bütün bölümün native kabulü değildir. Sekiz bölüm: **5 FAIL, 3 NOT RUN**.
RISK bulguları aşağıda ayrıca açıklanır.

| Kontrol | Birincil durum | Kanıt / kapsam sınırı |
| --- | --- | --- |
| RC-03 büyük klasör | NOT RUN | 500/1000 sentetik bozuk dosyada gerçek uygulamanın katalog/hata yolu, hızlı seçim ve kapanışı PASS; gerçek thumbnail/ilk görüntü zamanı ve loader yükü doğrulanmadı. |
| RC-04 thumbnail/cache | FAIL | Güncel katalog probe'u click/scroll assertion'ında exit101. 120 tekil kontrollü başarı/0 iptal ve 81/120 retention PASS; native recovery/pool kabulü açık. |
| RC-05 GNOME/Wayland | NOT RUN | Headless Sway fullscreen ve HiDPI retention PASS; fiziksel GNOME, jitter, ilk fiziksel tıklama ve pin/resize kullanıcı kabulü bu çalışmada yapılmadı. |
| RC-06 medya | NOT RUN | Gerçek secure JPEG/PNG/EXIF gate exit77. Kaynak destek matrisi incelendi; desteklenmeyen video/formatlar hata sayılmadı. Sentetik mod/filtre pikselleri PASS. |
| RC-07 etkileşim | FAIL | Zoom-pan, async ve pending-window probe'larında wheel sözleşmesi uyumsuzluğu tekrarlandı. Diğer kontrollü gezinme/preload/fullscreen kapıları PASS. |
| RC-08 kurulum/dağıtım | FAIL | Depoda dağıtım recipe/kurulum sözleşmesi ve uygulama ikonu yok; helper dahil temiz kurulum denetlenemedi. Source desktop doğrulaması ve izole GIO launch PASS. |
| RC-09 güvenlik/bütünlük | FAIL | fmt exit1. Rust/Python/check/build/Clippy PASS; kontrollü cache/source/race testleri PASS. Güncel güvenlik açığı veri tabanı ve native sandbox/child temizliği NOT RUN. |
| RC-10 yayın | FAIL | Paket/CLI sürümü tutarlı; lisans ve dağıtım teslimatı açık. main geçişi/yayın onayı verilmedi. |

## 4. Ayrıntılı test sonuçları

### Temel komutlar

Aynı etkinleştirilmiş SDK'da aşağıdaki komutlar --offline eklenerek çalıştırıldı.
Bu ek bayrak paket güncellemesi yapılmasını engeller; lock değiştirilmedi.

| Komut | Sonuç |
| --- | --- |
| cargo test --locked --offline -j2 | PASS: 124 lib + 27 bin + 35 integration = 186; 0 başarısız. Örneklerin ortak testleri ayrıca benzersiz test sayısına eklenmedi. |
| cargo check --locked --offline | PASS |
| cargo build --locked --offline --bin pivra --example thumbnail_overlay_probe --example thumbnail_provider_probe | PASS |
| cargo build --locked --offline --examples -j2 | PASS |
| cargo clippy --locked --offline | PASS / exit0, uyarılı: kullanılmayan load_stats ve aynı dallı pin ikon ifadesi. -D warnings veya tüm hedefler için PASS iddiası yok. |
| cargo fmt --check | FAIL / exit1; image_view, thumbnail_overlay, thumbnail_strip, settings, window ve bazı örneklerde biçim farkları. Davranışsal hata sayılmadı; otomatik format uygulanmadı. |
| python3 -B -m unittest discover -s tests/thumbnail -p 'test_*.py' -v | PASS: 37; C API/compiler stubları gerçek GNOME üretimi değildir. |
| python3 -B -m unittest discover -s tests -p 'test_*.py' -v | PASS: 33 observer/native-report unit testi. |
| tests/fixtures/generate.py --check | PASS: 4 deterministik fixture kaydı |
| tests/fixtures/images/generate.py --check | PASS: 18 image asset; hash kontrolü decode değildir. |
| desktop-file-validate data/io.github.onbeskarakterli.Pivra.desktop | PASS |
| target/debug/pivra --version | PASS: Pivra 0.1.0 |

### Güncel GUI/probe matrisi

Her probe özel config/cache ile çalıştırıldı. Kontrollü loader hiçbir zaman
native decoder olarak sunulmadı. Beklemeler sınırlıydı; takılı probe bırakılmadı.

| Probe / argüman | Ortam | Sonuç |
| --- | --- | --- |
| thumbnail_overlay_probe --catalog-regression | Sway/Wayland | FAIL / child101, runner1; thumbnail_catalog.rs:352, thumbnail click reset scroll. Öncesinde120 benzersiz başarı/0 iptal. |
| thumbnail_overlay_probe --fullscreen-regression | Sway/Wayland | PASS: compositor/GDK fullscreen onayı, F/Esc/double-click sinyalleri, header/viewport/normal boyut, texture/transform sürekliliği ve bağımsız pencereler. native=false, scale1. Fiziksel pointer routing/active secure timing değil. |
| thumbnail_overlay_probe --retention-regression | X11 | PASS:81/120, görünür/seçili pikseller, scroll-return, stabil decode, resize/hide/iki pencere. |
| aynı retention; wayland_smoke --output-scale=2 | Sway/Wayland | PASS: gerçek2× output üzerinde adet/bayt sınırları ve yeniden hazırlama. |
| view_modes_probe --synthetic | X11 | PASS:48 GSK Fit/Fill/Actual snapshot; yaklaşık2.659s |
| view_modes_probe --synthetic --resampling | X11 | PASS: Smooth/Nearest, clipping ve piksel kontrolleri; yaklaşık0.902s |
| view_modes_probe --synthetic --zoom-pan | X11 | FAIL101, zoom_pan_cases.rs:189,32× zoom assertion'ı |
| async_cancel_probe --controlled | X11 | FAIL101, async_cancel_probe.rs:240; actual != manual |
| preload_probe --controlled | X11 | PASS; cache hit/miss/LRU, priority/promotion ve close/drain; yaklaşık0.218s |
| window_failure_race_probe | X11 | FAIL101, window_failure_race_probe.rs:167; pending scroll handled=true |
| navigation_race_probe | X11 | PASS; ters tamamlanma, stale callback, video durumu, token/join/pencere izolasyonu |
| image_probe, mevcut Wayland runner içinde | Sway/Wayland | NOT RUN77; Bwrap loader early exit1. Güvenlik değiştirilmedi, decode için tekrar döngüsü yapılmadı. |
| thumbnail_overlay_probe --require-generation-helper | display gerektirmez | NOT RUN77; bu SDK'da API kontrolünden geçmiş native GNOME sidecar yok. |
| desktop_smoke.py target/debug/pivra --expect-image-error | X11 | PASS yalnız URI/desktop launch ve kapanış; gerçek piksel BLOCKED. |

Üç wheel-related FAIL, mevcut kaynaktan yeniden derlenen örneklerle bir kez daha
aynı satırlarda doğrulandı. cargo rustc --locked --offline --example <probe> --
-C debuginfo=1 yalnız debug bilgisini değiştirerek derlemeyi zorladı; program
kaynağı, assertion, decoder veya çalışma modu değiştirilmedi. Bu tekrar yeni
bağımsız native test sayısı değildir.

GIO desktop testi özel PATH/XDG ile Unicode, boşluk, #/% ve percent-encoded URI'yi
kaynak Exec=pivra %u üzerinden açtı; başlık, normal WM close ve process yok oluşu
geçti. Detached process exit code gözlenmedi. Nautilus kurulumu, varsayılan MIME
değişimi ve kurulu paket testi değildir.

### Sürüm, kimlik ve yayın metadata denetimi

| Alan | Sonuç |
| --- | --- |
| Cargo paketi / sürüm | PASS: Cargo.toml package=pivra, version=0.1.0; CLI aynı sürümü verir. |
| Uygulama kimliği | PASS: src/app.rs APP_ID=io.github.onbeskarakterli.Pivra, GLib ID unit testi geçer; desktop dosya taban adı aynı. |
| Desktop Exec/MIME | PASS: pivra %u; yalnız image/jpeg,image/png; DBusActivatable=false. Video varsayılanı ilan edilmez. |
| Cargo publish=false | Bilinçli crates.io engeli; kaynak/AUR dağıtımını engelleyen hata değildir. |
| Lisans / icon / kurulum recipe | Eksikler B4/B5 altında; uygulama asset'i ile fixture PNG'leri karıştırılmadı. |
| Tag stratejisi | Önceki kabul edilmiş hedef: onaylı main commit'inden v0.1.0, tek AUR pivra paketi; dev otomatik yayınlanmaz. Yeni karar/merge/tag verilmedi. |
| Yayın dosyaları | Yeni paket, release binary veya checksum teslimatı oluşturulmadı. Temiz release-profile/kurulu artifact doğrulaması NOT RUN. |

### Medya ve etkileşim sınırları

| Alan | Kaynaktan doğrulanan güncel davranış | Yeni gerçek native sonuç |
| --- | --- | --- |
| JPEG/PNG | input.rs:59 jpg/jpeg/png; image_loader.rs:119 içerik MIME JPEG/PNG kontrolü, EXIF dönüşümü, Bwrap zorunlu | NOT RUN; kullanıcı önceki Arch başarısı tarihsel |
| WebP/GIF/BMP/TIFF | discovery.rs:30 katalog adayı olabilir; açılış/üretim görüntüleyici desteği anlamına gelmez | Desteklenmiyor; FAIL olarak sayılmadı |
| APNG | PNG yolunda tek next_frame çağrısı; animasyon zamanlayıcısı yok | APNG ilk frame'i ayrıca doğrulanmadı; animasyon desteklenmiyor |
| Animasyonlu WebP/GIF | Uygulama açılış desteği ve playback yok | Desteklenmiyor |
| Video | window.rs:202 açık unavailable-playback durumu; gezinme öğesi atlanmıyor | Oynatma, pause, loop ve son-kare davranışı uygulanmamış; hata sayılmadı |
| Fit/Fill/%100 | Sentetik gerçek GSK snapshot PASS; %100 GTK mantıksal birimidir | Gerçek dosya tekrarı NOT RUN |
| Zoom/pan | Üretimde wheel varsayılanı gezinme; Zoom/Z seçeneği veya BUTTON1 modifier zoom yolunu açar | Eski varsayıma bağlı probe'lar FAIL; fiziksel native kabul tekrarlanmadı |
| Hatalı yollar/yenile/stale click | Mevcut Rust input/catalog/epoch-path testleri PASS; kontrollü navigation-race PASS | Bütün filesystem/GNOME zamanlamaları kapsandı iddiası yok |

## 5. Performans ölçümleri

### 500/1000 katalog — yalnız hata yolu

Yeni bd9c669 çalışması; gerçek uygulama, Xvfb, mevcut X11 yardımcılarıyla tuş/WM
olayları. Dosyalar .png uzantılı **sentetik metin**, geçerli fotoğraf değildir.
Bu seçim native decoder engelini aşmak için sahte decoder kullanmaz; gerçek
kontrollü hata yolunu ölçer. Her boyutta iki ayrı süreç/pencere açıldı. İlk/orta/
son seçim, diğer pencerenin başlığının korunması, kapanış ve kaynak byte
bütünlüğü PASS; iki süreç de exit0, panic/critical yok.

| Ölçüm |500 |1000 |
| --- | ---: | ---: |
| Başlatma → tamamlanmış katalog başlığı |0.210s |0.213s |
| Katalog sonrası ana süreç RSS |89.919.488 B |89.772.032 B |
| Kapanış öncesi RSS |97.529.856 B |103.469.056 B |
| Periyodik sampler en yüksek RSS |97.525.760 B |103.469.056 B |
| Son örnekte birikimli parent CPU |0.69s |1.78s |
| Hızlı tuş dizisiyle ortaya seçim |0.079s |0.285s |
| Sona seçim |0.077s |0.272s |
| Başa dönüş |0.168s |0.676s |
| Normal kapatma → süreç bitişi |0.016s |0.016s |
| İlk gerçek fotoğraf/thumbnail zamanı |NOT RUN |NOT RUN |
| Gerçek başarılı thumbnail decode/iptal/tekrar sayısı |NOT MEASURED |NOT MEASURED |

20ms örneklemeli /proc parent RSS/CPU; CPU yüzde değildir, loader/GPU/diğer
pencereyi kapsamaz. Sampler ve ayrıca alınan kapanış örneği farklı anlar olduğu
icin500 satırında4KiB fark vardır. Bunlar tek koşu, yerel küçük sentetik dosya
ölçümleridir; genel hız/ölçeklenebilirlik sonucu çıkarılmaz. Hızlı wheel/drag
olayları da gönderildi, fakat hit hedefi ve thumbnail pikselleri bu büyük
katalog koşusunda sertifikalandırılmadı. İlk/son sınırları ve worker limitleri
ayrıca Rust kontrollü testleriyle desteklenir; bu500/1000 native worker peak'i değildir.

### Texture retention

| Kontrollü fixture kataloğu | Ortam | En yüksek gözlenen texture / bayt |
| --- | --- | --- |
|81 ve120 | X11 scale1 |64 /4.524.032 |
|81 ve120 | Sway scale2 |59 /16.682.368 |

16MiB=16.777.216 B. HiDPI'da bayt limiti adet limitinden önce çalıştı. Görünür ve
seçili öğe korunması, evicted öğeye dönüş ve stabil konumda decode döngüsü
olmaması PASS. started görünür admission sayacıdır; hidden işi de içeren toplam
başarılı decode gibi yorumlanmadı. Örneğin120 retention'da görünür started114'ten
121'e çıktı; scroll-return sırasında yeniden hazırlama beklentisi test edildi.
Bu sayıları gereksiz yedi decode diye sınıflandırmak yanlış olur.

### Worker/kaynak politikası

Kaynak ve kontrollü unit kanıtı: thumbnail_manager Permit küresel en fazla2 iş,
aynı path dedup; foreground slot en fazla3 admitted job + latest pending;
thumbnail large-job en fazla1 ve ağırlıklı2GiB accounting, foreground32MP sınırı
ayrı. Main photo cache pencere başına128MiB/3 entry/64MiB entry; thumbnail display
cache64/16MiB. Bunlar toplam RSS/VRAM veya cgroup hard cap değildir. Bu denetimde
native decoder overlap, CPU yüzdesi, loader RSS peak'i ve aktif büyük decode
iptali ölçülmedi.

## 6. Kritik ve yüksek öncelikli bulgular

### B1 — Yüksek / FAIL: katalog kabul kapısı yeni bir eski-geometri varsayımında duruyor

- Konum: examples/support/thumbnail_catalog.rs:346–354;
  src/thumbnail_strip.rs:802 civarı seçimi görünür alana taşıyan align yolu.
- Tekrar: mevcut Sway runner'a thumbnail_overlay_probe --catalog-regression verin.
- Kanıt:120 benzersiz kontrollü cache başarısı/0 iptal; ardından
  thumbnail_catalog.rs:352, thumbnail click reset scroll, child101/runner1.
- Beklenti: görünür hücreye tıklama scroll'u değiştirmesin.
- Gerçek: test hâlâ100*104 offset kullanıp103 indeksine programatik tıklıyor.
  Güncel nominal pitch112+8=120 olduğundan seçtiği hücre viewport dışında kalıyor;
  üretim align yolu onu görünür alana taşımayı amaçlıyor. Son geometri düzeltmesi
  önceki sıra assertion'ını geçirdi; bu sonraki varsayımı düzeltmedi.
- Etki: bütün katalog/native kabul paketi tamamlanamıyor. Fiziksel kullanıcı
  tıklamasının bozuk olduğu bu testten çıkarılamaz.
- Minimum öneri: allocation'dan ölçülmüş, gerçekten görünür hücreyi tıklama
  önkoşulu yapın; stable-cell ve scroll assertion'larını koruyun. Üretim
  auto-alignment'ını sırf bu test için kaldırmayın. Bu görevde uygulanmadı.

### B2 — Yüksek / FAIL: wheel sözleşmesiyle üç probe'un beklentisi uyuşmuyor

- Konumlar: src/image_view.rs:120 ve608–620; src/window.rs:325–339;
  examples/support/zoom_pan_cases.rs:108–115,189;
  examples/async_cancel_probe.rs:239–240;
  examples/window_failure_race_probe.rs:167–169.
- Kanıt: fresh derleme sonrası zoom32× assertion101; async actual != manual101;
  pending-window event-handled assertion101. Aynı X11 özel config koşulunda tekrarlandı.
- Güncel üretim davranışı: zoom_wheel=false; wheel normal gezinmeyi tüketir,
  Zoom/Z veya BUTTON1 ile zoom açılır. Standalone zoom testinin yeni altıncı
  argümanı da false iken eski testler wheel'in zoom yapmasını bekliyor.
- Pending-window probe'u event'in tüketilmesini texture üretildiğiyle eşitliyor;
  bu mantıksal olarak aynı kontrol değildir. Bu log yeni bir texture veya üretim
  veri kaybı kanıtı değildir.
- Etki: zoom/lifecycle güvenlik ağları kırmızı; unit test186 PASS bu GUI kapılarını
  kapsamaz.
- Minimum öneri: kabul edilmiş ürün wheel sözleşmesini netleştirip zoom testini
  gerçek açık zoom durumuyla çalıştırın; default-navigation ayrı korunmalı.
  Pending durumda paintable/selection/source invariants'ını doğrudan doğrulayın.
  Assertion kaldırma veya üretim varsayılanını zorla değiştirme önerilmiyor.

### B4 — Yüksek / yayın kararı açık: proje lisansı tanımlanmamış

- Konum: Cargo.toml:1–10; tests/fixtures/README.md:3–6;
  docs/PHASE2_RELEASE_READINESS.md:259. git ls-files LICENSE/COPYING sonucu boş;
  root Cargo metadata license/license_file yok.
- Etki: planlanan kamuya açık0.1.0/AUR dağıtımının lisans/yeniden dağıtım
  sözleşmesi tamamlanmamış. Bu bir runtime hatası veya hukuki görüş değildir.
- Minimum: proje sahibi yayın/lisans kararını vermeli; ilgili dosya ve metadata
  bu kararla tamamlanmalı. Bağımlılık lisansı proje lisansı yerine kullanılamaz.
  Bu denetim bir lisans seçmedi/oluşturmadı.

### B5 — Yüksek / FAIL: hedef kurulum teslimatı denetlenebilir değil

- Konum: data/io.github.onbeskarakterli.Pivra.desktop:1–9; build.rs:8–45;
  src/thumbnail.rs:301–314; docs/PHASE2_RELEASE_READINESS.md:254–265.
- Kanıt: kaynak desktop var, Icon alanı yok; repo'da uygulama ikon asset'i,
  PKGBUILD/.SRCINFO veya install/uninstall recipe yok. Optional helper OUT_DIR'de
  üretiliyor; kurulu sibling keşfi mevcut ama bunu kuran teslimat/test yok.
- Etki: yalnız pivra binary'sini kopyalamak GNOME factory helper'ını kurmaz;
  clean install/remove/upgrade ve kurulu helper bulunabilirliği doğrulanamaz.
  İkon eksiği ürün bütünleşmesi eksiğidir; çökme nedeni diye sunulmaz.
- Minimum: önceki onaylı tek AUR pivra/main-tag planı için binary+helper+desktop+
  ikon yerleşimi ve bağımlılıklar açık recipe ile tanımlanıp temiz Arch'ta
  install/remove/upgrade doğrulanmalı. Haricî AUR yayın durumu bu oturumda
  doğrulanmadı. AppStream eksikliği AUR için zorunluymuş gibi sayılmadı.

Kritik/P0 güvenlik açığı veya üretim çökmesi kanıtlanmadı. Bu, tam güvenlik
sertifikası veya gerçek native koşuların yerine geçen bir hüküm değildir.

## 7. Orta ve düşük öncelikli bulgular

### B3 — Orta / FAIL: biçimlendirme kapısı kırmızı

cargo fmt --check exit1. Örnek konumlar src/image_view.rs:459,
src/thumbnail_overlay.rs:189, src/thumbnail_strip.rs:184 ve src/window.rs:70.
Aynı dosyanın örneklere path ile alınması çıktıda tekrar üretir; bütün diff
satırları ayrı hata sayılmaz. Beklenen biçim farkının olmamasıdır; gerçek sonuç
çok sayıda mevcut farktır. Davranışsal kusur değildir. Minimum: ayrı yetkiyle
format-only değişiklik ve doğrulama; bu rapor test/kod formatını düzeltmedi.
RC-09'un istenen kapısı olduğundan yayın engeli listesine dahil edildi.

### R1 — Orta / RISK: hidden/visible cache recovery asimetrisi

src/thumbnail_strip.rs:484–494 hidden yol, aynı dosyada946–958 visible yol.
CacheReader Missing/Stale/Invalid sonucu ikisinde Manager üzerinden üretilebilir.
Fakat metadata-valid Hit'in piksel decode'u bozulursa visible recover_cache
çağırabilir; hidden çağırmayıp Pending bırakabilir. Gerçek native bozuk-piksel
senaryosu bu ortamda denenmedi; bilinçli tasarım tercihi olduğuna dair yeterli
belge yok. Bu nedenle kaynak asimetrisi gerçek, kullanıcıya etkisi RISK'tir.
Önce native senaryo ölçülmeli; şimdi ortak worker refaktörü gerekçelendirilmedi.

### R2 — Orta / RISK: README tarihsel/güncel kapsam karışıklığı

README.md:2 video/edit pazarlama ifadesi, :6 PIV-18 TESTING; tracker'ın daha yeni
full-only v0.1 kabul kaydıyla tam uyumlu değil. Ayar/model bağımsızlığına ait eski
bölümler de bugünkü GTK/global/persistence durumundan farklı. Tarihsel oldukları
belirtiliyor, ama kullanıcıya kısa güncel v0.1 yetenek/kurulum matrisi eksik.
Minimum: eski kanıtları silmeden güncel başlangıç/format/kurulum bölümü netleştirilmeli.
Bu görev README'yi değiştirmedi; desteklenmeyen video/WebP özellik sayılmadı.

### R3 — Düşük/orta / RISK: sayaç ve büyük-katalog ölçüm kapsamı

src/thumbnail_strip.rs:301–309 ve473–503: warming ayrı, started görünür admission
sayacı. stats.running/started tüm hidden ve native pool yaşamını temsil etmiyor.
Mevcut katalog probe'u120'ye sabit (thumbnail_catalog.rs:198);500/1000 gerçek
thumbnail kanıtı yok. Minimum: ileride mevcut probe parametreleştirmesi/ölçüm
sözleşmesi değerlendirilebilir; bu görev yeni test dosyası veya üretim telemetrisi
oluşturmadı. PID yaşaması tek başına aktif decode/sızıntı diye sınıflandırılmadı.

## 8. Doğrulanamayan testler

- Gerçek GNOME Shell/Wayland fiziksel input, ilk tıklama/zero-drag/6px eşiği,
  jitter, üst kenar resize, pin/restart ve kayıtlı yüksekliğin bütün pencere koşulları.
  Kullanıcının önceki başarılı tıklama/sürükleme kabulü ayrı tarihsel kanıttır.
- Güncel secure JPEG/PNG/EXIF gerçek pikselleri, APNG ilk frame, gerçek decoder
  overlap, aktif large-job iptalinin native child/pool temizliği.
-500/1000 geçerli gerçek fotoğraf thumbnail yükü; ilk görüntü süresi, frame latency,
  GPU/VRAM/loader RAM ve native maksimum worker gözlemi.
- Optimize release-profile build ve temiz Cargo target/native kurulum bu çalışmada
  yapılmadı; mevcut dependency cache/SDK ile debug build PASS clean install değildir.
- Temiz Arch makinesinde kurulum ve minimum SDK kombinasyonları; gerçek Rust1.85
  build. Linux-filtered Cargo metadata'da168 resolved paket incelendi; ilan edilen
  dependency MSRV'leri1.85'i aşmadı, fakat bu gerçek1.85 derlemesi değildir.
- Gerçek kurulu Nautilus Open With/default MIME ve remove/upgrade testi.
- Güncel RustSec/OSV advisory taraması: cargo-audit bu ortamda yok; kurulmadı.
  Derleme PASS olması CVE bulunmadığı anlamına gelmez.
- GitHub Releases API listesi proxy CONNECT403 nedeniyle okunamadı. Remote tag
  sorgusu boş döndü; yerel/remote tag yaratılmadı. Haricî AUR paketi sorgulanmadı.

## 9. Yayın engelleyici sorunlar

| ID | Yayın engeli | Giderme/karar sahibi |
| --- | --- | --- |
| B1 | Katalog GUI kabul kapısı FAIL | Gerçek görünür hücre önkoşullu test düzeltmesi ve yeniden doğrulama |
| B2 | Zoom/async/pending GUI kapıları FAIL, ortak wheel sözleşmesi | Ürün sözleşmesi + buna uygun dar test hazırlığı; lifecycle assertions korunmalı |
| B3 | RC-09 fmt kapısı FAIL | Ayrı format-only değişiklik ve kontrol |
| B4 | Proje lisans/yayın kararı açık | Proje sahibi; denetçi lisans seçmez |
| B5 | Hedef kurulum/paket teslimatı ve assets eksik | Ayrı yetkili paketleme + temiz Arch kabulü |

**Sayı:5.** B2 altında üç probe FAIL'i aynı kök nedenle bir kez sayıldı.
Native NOT RUN kapıları bu beşe ilave doğrulama gereksinimidir; yeni kanıtlanmış
kod hataları gibi sayılmadı. Unsupported format/video veya AppStream eksiği ayrı
blocker olarak eklenmedi. Salt kaynak dağıtımı gibi farklı bir yayın kapsamı
kararı alınmadığından önceki AUR/main-tag hedefi kendiliğinden daraltılmadı.

## 10. Minimum düzeltme önerileri

1. Önce B1/B2 test önkoşullarını güncel ürün davranışıyla hizalayın; sıra, scroll,
   cache, stale callback, texture ve worker assertion'larını koruyun. Bunlar
   üretim görüntüleyicisini yeniden yazma gerekçesi değildir.
2. Ayrı format-only değişiklikle B3 kapısını yeşile getirin; fonksiyon değişikliğiyle
   karıştırmayın. Clippy'nin uyarıları PASS/warnings olarak ayrı değerlendirilsin.
3. Sahibin lisans/yayın ve güncel v0.1 full-only kapsamını açıklaştırmasını bekleyin;
   source/README/metadata bunu tutarlı ifade etmeli.
4. Yetkilendirilmiş ayrı dağıtım aşamasında binary/helper/desktop/ikon yerleşimi
   ve clean install/remove/upgrade kanıtı üretin; bu görev paket oluşturmadı.
5. Native ortamda kalan kapıları çalıştırın. Özellikle önceki kabul mevcutsa bile
   bu kaynak snapshot'ına ait secure decode, GNOME input, Nautilus ve pool sonucu
   ayrı kaydedilmeli. RISK gerekçesiyle speculative refaktör yapmayın.

## 11. Son yayın kararı

**NO-GO — v0.1.0 henüz yayımlanmamalı ve dev → main yayın geçişi onaylanmadı.**
Gerekçe beş doğrulanmış yayın hazırlığı engeli ve açık native/clean-install
kapılarıdır. Çalışan davranışı bozacak bir ürün müdahalesi önerilmedi. Hiçbir
özellik, sürüm, dependency, assertion, sandbox veya kullanıcı verisi bu denetimde
değiştirilmedi. Release/tag/merge/CI yapılmadı. Yalnız bu rapor dev'e kaydedilir;
raporun commit SHA'sı teslimatta bildirilir.
