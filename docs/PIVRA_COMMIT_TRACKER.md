# Pivra — Planlı Commit ve İlerleme Takibi

**Kaynak:** [Özellik Şartnamesi](PIVRA_FEATURE_SPEC_TR.md) · [Yol Haritası](PIVRA_DEVELOPMENT_ROADMAP.md)  
**Branch:** `dev` · **Durum:** PIV-01 **DONE (21.10.2026)**; PIV-02–60 TODO, başlatılmadı. Güncel kapanış kanıtı son bölümde; 20.10.2026 kayıtları tarihçedir. Burada listelenen `PIV-xx` kimlikleri gerçek Git commit SHA'sı **değildir**.

## Kullanım kuralları

- Bir sıradaki iş için ön koşullar sağlanınca `TODO → IN_PROGRESS → CODED → TESTING → DONE`.
- Hata veya engel varsa `BLOCKED`. Kod yazılmış olması tek başına `DONE` değildir.
- İş tamamlandıktan sonra gerçek commit SHA, test sonucu (komut, PASS/FAIL, manuel test) ve tarih ilerleme sütunlarına eklenir.
- Bir iş bölünürse alt işler `PIV-xx.a`, `PIV-xx.b` şeklinde kaydedilir; ID'ler yeniden numaralanmaz.
- Kapsam/araç değişiklikleri şartname ve yol haritasıyla tutarlı olmalı. Push ile tetiklenen CI yok; varsa Actions yalnız manuel tetiklenir.
- Faz kapıları geçilmeden sonraki aşama tamamlandı işaretlenmez. Edit moduna fotoğraf/video görüntüleme akışı doğrulanmadan başlanmaz.

## Faz 0 — Rust altyapısı ve teknik zemin

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-01 | `chore: initialize Rust Cargo app` | Cargo derlenir; temel dizin ve lisans/README korunur | **DONE — 21.10.2026** | `b246ee7`, `27315e6`, `90d495b`; doğrulama/lock/README: `2950193ca038b91c29ed97a7c46d773502d68dd4` |
| PIV-02 | `feat: create GTK4 libadwaita application shell` | GNOME'da boş/sade ana pencere açılır; düzgün kapanır | TODO | — |
| PIV-03 | `refactor: define app media and settings boundaries` | Mantıksal modüller ayrılır; birim testleri çalışır | TODO | — |
| PIV-04 | `test: establish fixtures and local verification commands` | Deterministik örnek medya/fixture kuralları ve cargo test komutları | TODO | — |
| PIV-05 | `docs: record dependency capability probes` | GTK/Glycin/GStreamer sürüm ve yetenekleri, risk kararı yazılır | TODO | — |

## Faz 1 — Fotoğraf görüntüleme çekirdeği

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-06 | `feat: accept image path from Nautilus` | Fotoğrafa çift tıklama Pivra açar; videonun varsayılan uygulaması değişmez | TODO | — |
| PIV-07 | `feat: scan neighboring media in folder` | Fotoğraf/video adaylarını tek klasörde listeler, izin hataları yönetilir | TODO | — |
| PIV-08 | `feat: apply natural filename ordering` | 1,2,10 ve karma medya testleri geçer | TODO | — |
| PIV-09 | `feat: render JPEG PNG with orientation` | Görsel merkezde fit açılır; EXIF orientation doğru | TODO | — |
| PIV-10 | `feat: navigate previous and next media` | Sol/sağ; ilk/son sınırları; tek ve boş klasör | TODO | — |
| PIV-11 | `feat: watch folder changes and external modifications` | Ekle/sil/ad değiştir listeyi yeniler; açık dosyada elle Yenile isteği | TODO | — |
| PIV-12 | `test: harden media failures and window isolation` | Bozuk medya gezinmeyi bozmaz; pencereler bağımsız | TODO | — |

## Faz 2 — Zoom, pan, ağır medya ve kaynak yönetimi

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-13 | `feat: implement fit fill and actual size` | Sığdır/doldur/%100; oran bozulmaz, yeniden boyutta güncellenir | TODO | — |
| PIV-14 | `feat: implement cursor anchored zoom and pan` | %10–3200, imleç ankrajı, sürükleyerek kaydırma | TODO | — |
| PIV-15 | `feat: add image resampling choices` | Yumuşak/piksel keskinliği, büyütme-küçültme regresyonu | TODO | — |
| PIV-16 | `feat: add cancellable async image loading` | Hızlı gezinmede eski decode sonucu yeni görselin üzerine yazılmaz | TODO | — |
| PIV-17 | `perf: preload neighbors with bounded memory` | Öncelik mevcut medya; RAM baskısında önbellek azalır | TODO | — |
| PIV-18 | `perf: preview and full resolution image pipeline` | RAW/TIFF önizleme, yükleme göstergesi ve viewport sürekliliği | TODO | — |

## Faz 3 — Video çekirdeği ve karma gezinme

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-19 | `feat: embed GStreamer video surface` | GTK4 penceresinde video gösterilir; Wayland kontrolü | TODO | — |
| PIV-20 | `feat: unify image and video navigation` | Fotoğraf→video→fotoğraf geçişi; yeni medya fit | TODO | — |
| PIV-21 | `feat: manage video start stop and end state` | Açılışta autoplay, ayrılınca ses/decoder durur, sonda son kare | TODO | — |
| PIV-22 | `feat: implement playback sound and seek controls` | Space, Ctrl+ok ±5 sn, yukarı/aşağı ses, timeline çalışır | TODO | — |
| PIV-23 | `feat: persist volume and window local positions` | Ses/mute kalıcı; video kaldığı yer aynı pencerede, kapanınca sıfır | TODO | — |
| PIV-24 | `test: validate mixed media lifecycle and fallbacks` | 1080p/4K akış, codec hataları ve tekrar geçişte kaynak temizliği | TODO | — |

## Faz 4 — Video yeteneklerini tamamlama

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-25 | `feat: implement frame accurate stepping` | Virgül/nokta ileri/geri kare; VFR örnekleriyle sınır raporu | TODO | — |
| PIV-26 | `feat: preview frames on timeline hover` | Önizleme karesi/süre; hover pozisyonu değiştirmez | TODO | — |
| PIV-27 | `feat: support embedded and sidecar subtitles` | SRT/ASS, otomatik eşleştirme, mümkünse ASS stili | TODO | — |
| PIV-28 | `feat: select audio and subtitle tracks` | Çoklu parça, değiştirmede kaynak sızıntısı yok | TODO | — |
| PIV-29 | `feat: add subtitle presentation settings` | Font/boyut/renk/çerçeve/arka plan, orijinale dönüş | TODO | — |
| PIV-30 | `feat: implement video zoom and source frame capture` | Pan/zoom video içinde korunur; PNG/JPEG/WebP kaynak çözünürlükte kare | TODO | — |
| PIV-31 | `feat: auto hide playback controls and power inhibit` | 2 sn idle gizleme; paused görünür; oynatma/slaytta ekran açık | TODO | — |

## Faz 5 — Tam görüntüleyici ve etkileşimler

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-32 | `feat: add overlay thumbnail strip` | Ana viewport değişmez; yatay scroll ve aktif dosya ortalama | TODO | — |
| PIV-33 | `feat: implement in memory thumbnail caching` | Kalıcı Pivra cache yok; RAM baskısında tahliye | TODO | — |
| PIV-34 | `feat: add selection mode and media type filters` | Fotoğraf/Video/Tümü, Shift aralığı, tümünü/temizle | TODO | — |
| PIV-35 | `feat: show minimap only when media overflows` | Harita görünürlük kuralı ve pan çerçevesi fotoğraf/videoda | TODO | — |
| PIV-36 | `feat: add optional metadata information panel` | Panel açılınca viewport daralır; EXIF/video alanları | TODO | — |
| PIV-37 | `feat: implement fullscreen and looping slideshow` | F11/F5 ayrı; foto süreleri, tam video, 300 ms crossfade, Space/Esc | TODO | — |
| PIV-38 | `feat: add customizable input and touch gestures` | Klavye/fare çakışma denetimi; pinch/pan/medya swipe testi | TODO | — |
| PIV-39 | `feat: finish GNOME viewer integrations` | Tema/HiDPI, multi monitor, geçmiş tercihi, F2, Trash+Undo, karşılaştırma ve wallpaper | TODO | — |

## Faz 6 — Formatlar, kalite, test ve native paketleme

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-40 | `feat: expand still image and animation decoders` | GIF/APNG/animated WebP, TIFF/SVG/BMP/ICO vb. fixture testi | TODO | — |
| PIV-41 | `feat: support RAW HEIC AVIF JXL and PSD adapters` | Format bazlı başarı/eksik yetenek raporu, kontrollü fallback | TODO | — |
| PIV-42 | `feat: support conditional multipage and layer views` | TIFF sayfaları, ICO ölçüleri, PSD layer imkânı | TODO | — |
| PIV-43 | `feat: integrate ICC and HDR fallback paths` | ICC/SDR/HDR davranışı, renk profil ve ekrana göre rapor | TODO | — |
| PIV-44 | `test: profile memory responsiveness and robustness` | Büyük klasör, 4K, bozuk/eksik dosya, uzun süre tekrar, regresyon | TODO | — |
| PIV-45 | `build: package Arch native PKGBUILD` | Arch kurulum/güncelleme/kaldırma, desktop MIME ilişkileri | TODO | — |
| PIV-46 | `build: package Debian and RPM native releases` | DEB/RPM hedef matrisinde kurulabilir; sistem kütüphaneleri | TODO | — |
| PIV-47 | `docs: publish viewer video test release checklist` | Destek matrisi, manuel test kanıtları, görüntüleme/video test sürümü kararı | TODO | — |

## Faz 7 — Son faz: yalnız Edit modunda düzenleme ve toplu dönüştürme

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-48 | `feat: create isolated non destructive Edit mode` | Edit ile açılır; normal görüntüleme temiz; dirty-state uyarısı | TODO | — |
| PIV-49 | `feat: implement crop rotate flip and horizon` | Kırpma oranları, otomatik köşe kırpma, EXIF orientation | TODO | — |
| PIV-50 | `feat: resize image by pixels or target size` | Oran korunur; hedef KB için kalite/çözünürlük dengesi | TODO | — |
| PIV-51 | `feat: add exposure color and detail adjustments` | Temel ayarlar + 6 ışık/renk + sharpen/denoise/clarity | TODO | — |
| PIV-52 | `feat: implement auto enhance and red eye correction` | %0–100 etki, otomatik ve manuel müdahale | TODO | — |
| PIV-53 | `feat: add Edit before after split and undo` | Sol orijinal/sağ düzenlenmiş, sürüklenebilir ayırıcı, Ctrl+Z | TODO | — |
| PIV-54 | `feat: safely save images and metadata` | Atomik kaydetme, Farklı Kaydet, EXIF/ICC, mümkünse kayıpsız işlem | TODO | — |
| PIV-55 | `feat: batch convert selected images in background` | Format/kalite/boyut/çıktı klasörü; görüntüleme bloklanmaz | TODO | — |
| PIV-56 | `feat: add pause cancel collisions and notifications` | Mevcut dosya korunur, (1) adlandırma, aktif iş bitince pause, GNOME bildirimleri | TODO | — |
| PIV-57 | `test: validate edit safety and batch lifecycle` | Orijinal bütünlüğü; iptal/kapanış, dirty-state, hata yönetimi | TODO | — |

## Faz 8 — Son bütünleştirme ve yayın kontrolü

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-58 | `test: run complete regression and compatibility matrix` | Otomatik+manuel gerçek cihaz testleri, known issues listesi | TODO | — |
| PIV-59 | `docs: finalize user docs and package requirements` | Yerel paket, kurulum, dosya formatları ve kısayol belgeleri | TODO | — |
| PIV-60 | `chore: prepare verified release candidate` | Kritik hata yok; sürüm kararı ve `main` aktarımı için ayrıca kullanıcı onayı | TODO | — |

## Faz özeti

| Faz | ID aralığı | Planlı commit | Tamamlanan |
|---|---|---:|---:|
| 0 — Temel | 01–05 | 5 | 1 |
| 1 — Fotoğraf | 06–12 | 7 | 0 |
| 2 — Zoom/performans | 13–18 | 6 | 0 |
| 3 — Video temel | 19–24 | 6 | 0 |
| 4 — Video gelişmiş | 25–31 | 7 | 0 |
| 5 — Arayüz | 32–39 | 8 | 0 |
| 6 — Uyumluluk/paket | 40–47 | 8 | 0 |
| 7 — Edit (en son özellik geliştirmesi) | 48–57 | 10 | 0 |
| 8 — Son doğrulama | 58–60 | 3 | 0 |
| **TOPLAM** | **PIV-01–60** | **60** | **1** |

> PIV-01 tamamlandı; PIV-02–60 gelecekteki işlerdir. Görev kimlikleri commit SHA'sı değildir. İlk planlama ve önceki engel kayıtları aşağıda tarihçe olarak korunur.

## PIV-01 ayrıntılı ilerleme — 20.10.2026

| Alt görev | Durum | Kanıt / not |
|---|---|---|
| PIV-01.a Depo/ortam denetimi | DONE | `dev` incelendi; Cargo.toml, main.rs, .gitignore ve LICENSE başlangıçta yok; Git erişimi çalışıyor. Bu çalışma ortamında rustc/cargo komutları yok. |
| PIV-01.b Cargo iskeleti | CODED | `Cargo.toml`, `src/main.rs` eklendi. Rust 2024, üçüncü taraf bağımlılık yok. Derleme henüz doğrulanamadı. |
| PIV-01.c Dizin düzeni | CODED | `src/` uygulama girişi, mevcut `docs/` korunuyor; gereksiz boş dizin/crate eklenmedi. |
| PIV-01.d Git/metaveri | PARTIAL | `.gitignore` ve README güncellendi. **Lisans seçilmedi**; sonradan kullanıcı onayı gerekir. |
| PIV-01.e Başlangıç davranışı | CODED | `--help`, `--version`, tanımsız parametre için kontrollü hata; GUI PIV-02'de. Üç Rust unit test yazıldı, çalıştırılamadı. |
| PIV-01.f Otomatik doğrulama | BLOCKED | `rustc: command not found`, `cargo: command not found`. Bu nedenle fmt/check/test/clippy PASS olarak işaretlenmedi. |
| PIV-01.g Commit ve takip | PARTIAL | Kod commitleri: `b246ee7`, `27315e6`, `90d495b`; README: `4bb5ba9`. Test kanıtı ve nihai kapanış bekliyor. |

**Bekleyen doğrulama komutları:** `cargo fmt --check`, `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo run -- --help`, `cargo run -- --version`. Bunlar yerel veya Work içindeki uygun Rust ortamında çalıştırılabilir; kullanıcının bilgisayarına erişimi şu anda yok. PIV-01 **DONE değil**. Özellikler, GTK penceresi veya medya fonksiyonları henüz uygulanmadı.

## PIV-01 Work denetimi — 20.10.2026

Bu kayıt yukarıdaki ilk iskelet çalışmasının ortam ve doğrulama durumunu günceller; önceki kayıt tarihçe olarak korunur.

- İncelenen depo/branch: `onbeskarakterli/Pivra`, `dev`.
- Başlangıç HEAD: `d6db4aa92792e39bfbb2ffaa4c12b3b45ff2f9b3`.
- Korunan `main` başlangıç HEAD: `5d55ee599df4617bd51c09b1c7a008b8d0ece480`; bu görevde yazılmadı.
- Dört bağlayıcı belge, Cargo.toml, src/main.rs, .gitignore ve tam depo ağacı okundu. Ağaçta AGENTS.md, LICENSE, Cargo.lock veya workflow yok.
- Rust/Cargo sürümleri: **BELİRLENEMEDİ**. İki komut çalıştırma girişimi shell başlamadan `No usable execution environment is available` hatası verdi. Bu oturumda `command not found` sonucu alınmadı; toolchain'in eksik olduğu sonucuna varılamaz. Kurulum da çalıştırılamadı.
- Kaynak kodu, bağımlılıklar ve mimari değiştirilmedi. Yeni kod commit'i oluşturulmadı. CI ve PIV-02 kodu eklenmedi.
- Statik inceleme Cargo geçerliliği, derleme veya runtime PASS kanıtı değildir.

| Alt görev | Güncel durum | Kanıt / kalan iş |
|---|---|---|
| PIV-01.a | BLOCKED | Depo, dev HEAD ve dosya ağacı doğrulandı; Rust/Cargo ortamı ve sürümleri çalıştırma servisi nedeniyle doğrulanamadı. |
| PIV-01.b | CODED / doğrulama BLOCKED | Manifestte isim, sürüm, Rust 2024, repository ve publish=false var; üçüncü taraf bağımlılık yok. Cargo ile manifest ve kaynak doğrulaması bekliyor. |
| PIV-01.c | DONE (statik denetim) | src/main.rs ve mevcut docs yeterli; gereksiz crate/dizin veya erken GTK/medya katmanı yok. |
| PIV-01.d | DONE (statik denetim) | target/editor/OS ignore kuralları incelendi; Cargo.lock ignore edilmiyor. README güncellendi. Lisans dosyası yok; seçim açık konu olarak bırakıldı. |
| PIV-01.e | BLOCKED | Kaynakta --help/-h, --version/-V, argümansız mesaj ve invalid için stderr + exit(2) var. Üç unit test mevcut; CLI çıktıları ve çıkış kodları çalıştırılmadı. |
| PIV-01.f | BLOCKED | Aşağıdaki sekiz komut çalıştırılamadı; hiçbirine PASS/FAIL atanmadı. |
| PIV-01.g | PARTIAL | README belge commit'i ve bu denetim kaydı dev'e işlendi; test kanıtı ve nihai kapanış bekliyor. |

| Doğrulama komutu | Gerçek sonuç | Beklenti |
|---|---|---|
| `cargo fmt --check` | BLOCKED — çalıştırılmadı | Çıkış 0 |
| `cargo check` | BLOCKED — çalıştırılmadı | Manifest/kaynak doğrulaması, çıkış 0 |
| `cargo test` | BLOCKED — çalıştırılmadı | Mevcut üç birim testi geçer |
| `cargo clippy --all-targets -- -D warnings` | BLOCKED — çalıştırılmadı | Uyarısız, çıkış 0 |
| `cargo run -- --help` | BLOCKED — çalıştırılmadı | Yardım stdout, çıkış 0 |
| `cargo run -- --version` | BLOCKED — çalıştırılmadı | Pivra 0.1.0 stdout, çıkış 0 |
| `cargo run --` | BLOCKED — çalıştırılmadı | GUI henüz yok mesajı stdout, çıkış 0 |
| `cargo run -- --unknown` | BLOCKED — çalıştırılmadı | Kontrollü hata stderr, sıfırdan farklı çıkış (kaynakta 2) |

### Commit kanıtı ve test durumunun ayrımı

| Commit | Kapsam | Test durumu |
|---|---|---|
| `b246ee7`, `27315e6`, `90d495b`, `4bb5ba9` | Önceki Cargo, Rust, ignore ve README commitleri | Önceki kayıtta test edilmedi; bu oturumda da BLOCKED |
| `d6db4aa92792e39bfbb2ffaa4c12b3b45ff2f9b3` | Önceki takip kaydı / bu denetimin başlangıç HEAD'i | Belge; test başarısı anlamına gelmez |
| `9f13bc811be0244fbd7c72cd9ff1f5bbb2cb83e6` | `docs(PIV-01): record Work audit execution blocker` — README ve eksik iki CLI kontrolünün yönergesi | Belge; Cargo/CLI kontrolleri BLOCKED |

Bu takip güncellemesini içeren belge commit'inin kendi SHA'sı, öz referans oluşturmak yerine bu dosyanın Git geçmişinden alınır. Kodda değişiklik yoktur; test durumu belge commit'lerinden bağımsızdır.

**Belge tutarsızlığı:** Şartname ve yol haritasında “henüz kod yazılmadı/kaynak kodu değiştirilmeyecek”, takip başlığında ise tüm 60 işin gelecekte olduğu yönünde tarihsel ifadeler var. PIV-01 iskeleti artık mevcut. Bu görev talimatı yalnız PIV-01 denetimini yetkilendirir; kabul edilen ürün kararları ve diğer görevlerin kapsamı değiştirilmedi.

**GNOME GUI:** Gerçek grafik oturumu testi yapılmadı; PIV-01 kapanış koşulu değil. PIV-02'de ayrıca doğrulanacak.

**Kapanış:** PIV-01 tamamlandı mı? **HAYIR — BLOCKED.** Önce kullanılabilir bir Rust çalıştırma ortamında tüm kontroller ve CLI çıkış kodları doğrulanmalı; üretilen Cargo.lock ve varsa gerekli minimum düzeltmeler ayrıca incelenip dev'e kaydedilmeli. Lisans kararı ayrı açık konu olarak kalabilir. **PIV-02'ye teknik geçiş kapısı henüz geçilmedi; PIV-02 başlatılmadı.**


## PIV-01 kapanışı — 21.10.2026

**Güncel sonuç: DONE.** Önceki BLOCKED kayıtları tarihçedir; kullanılabilir Rust ortamında yapılan bu doğrulama onları günceller. Kullanıcı yalnız PIV-01 kapanışını yetkilendirdi; PIV-02 başlatılmadı. Faz 0'ın bütünü tamamlanmış değildir.

### Kapsam, ortam ve inceleme

- Dal: `dev`; doğrulanan kaynak başlangıcı `86a54f7094639d6a75a0c8798d8b48e7a65e8f12`.
- README, şartname, yol haritası, takip tablosu, Cargo.toml, src/main.rs ve ignore kuralları okundu; AGENTS.md yok.
- rustc `1.99.1 (58cf8f9c3 2026-10-13)`; Cargo `1.99.1 (5f94df478 2026-08-27)`; rustfmt `1.10.0-stable`; Clippy `0.1.99`.
- Gerçek check/build/test/Clippy ve CLI çalıştırmaları başlangıçta boş `/tmp/pivra-piv01-validation` dizinini `CARGO_TARGET_DIR` olarak kullandı. Harici crate yok; komutlar çevrimdışı ve kilitli çalıştı. Cargo.lock v4 yalnız `pivra 0.1.0` içeriyor; incelendi ve dev'e kaydedildi.
- Kaynak kodu ve manifest değiştirilmedi. Mevcut argüman ayrıştırma, kontrollü exit(2), stdout/stderr ayrımı incelendi ve çalıştırıldı. Bu CLI dosya/medya yazmıyor, decoder veya arka plan işi başlatmıyor; medya/disk/bellek regresyon testleri sonraki özelliklerin kapsamındadır.
- Lisans kararı açık; LICENSE veya manifest lisansı uydurulmadı. Önceki kapanış tanımı bu kararı ayrı açık konu olarak bırakmaya izin veriyor.
- Gerçek GNOME/GUI manuel testi **yapılmadı ve PASS sayılmadı**; CLI bootstrap için gerekli değil, PIV-02 ve sonrasının işi. GTK, medya motoru ve CI eklenmedi.
- `main` uzak uç SHA'sı okuma ile doğrulandı: `5d55ee599df4617bd51c09b1c7a008b8d0ece480`; bu görev main'e yazmadı ve hiçbir dala push yapmadı.

### Gerçek doğrulama sonuçları

Komutlar depo kökünde, stable toolchain ile çalıştırıldı. `cargo run` sonuçlarında stdout/stderr tam metni ve process çıkış kodu Python subprocess assertions ile karşılaştırıldı.

| Komut | Gerçek sonuç |
|---|---|
| `cargo +stable fmt --check` | PASS — çıkış 0 |
| `cargo +stable check --offline --locked` | PASS — çıkış 0 |
| `cargo +stable build --offline --locked` | PASS — çıkış 0 |
| `cargo +stable test --offline --locked` | PASS — 3 geçti, 0 başarısız, 0 ignored; çıkış 0 |
| `cargo +stable clippy --offline --locked --all-targets -- -D warnings` | PASS — uyarı yok, çıkış 0 |
| `cargo +stable run --offline --locked --quiet -- --help` ve `-h` | PASS — stdout: `Pivra 0.1.0`, `Usage: pivra [--help \| --version]`, gelecek GNOME işlevi açıklaması; stderr boş; çıkış 0 |
| `cargo +stable run --offline --locked --quiet -- --version` ve `-V` | PASS — stdout `Pivra 0.1.0` + newline; stderr boş; çıkış 0 |
| `cargo +stable run --offline --locked --quiet --` | PASS — stdout `Pivra: graphical application is not yet implemented` + newline; stderr boş; çıkış 0 |
| `cargo +stable run --offline --locked --quiet -- --unknown` | PASS — stdout boş, stderr `Pivra: invalid arguments; use --help` + newline; beklenen çıkış 2 |
| Aynı run komutuyla `one two` ve `--help --version` | PASS — aynı kontrollü stderr, stdout boş; beklenen çıkış 2 |

Üç birim testi: `no_arguments_starts_app`, `help_and_version_flags`, `invalid_arguments_are_rejected`. Toplam sekiz CLI senaryosu doğrulandı; test atlanmadı veya devre dışı bırakılmadı.

### Alt görev kapanışları

| Alt görev | Güncel durum | Kanıt |
|---|---|---|
| PIV-01.a | DONE | dev ve Rust araç sürümleri gerçek ortamda doğrulandı |
| PIV-01.b | DONE | Cargo manifesti ve kilidi; check/build PASS |
| PIV-01.c | DONE | src/ + docs/ düzeni korundu; gereksiz crate/dizin yok |
| PIV-01.d | DONE | Cargo.lock ve güncel README kaydedildi; lisans ayrı açık konu |
| PIV-01.e | DONE | Üç unit test ve sekiz CLI senaryosu PASS |
| PIV-01.f | DONE | fmt/check/build/test/Clippy PASS |
| PIV-01.g | DONE | Aşağıdaki gerçek commit kanıtları ve bu kapanış kaydı dev'e işlendi |

### Gerçek commit kanıtları

| Tam SHA | Kapsam |
|---|---|
| `b246ee75303ed36a2ffa88bc45f735419c3494ae` | Cargo.toml iskeleti |
| `27315e62a99db9106c0560547773a9278a88a849` | Rust CLI ve üç birim testi |
| `90d495bf9b9956ab77d29ce69ee995f2e4d959a8` | Rust ignore kuralları |
| `2950193ca038b91c29ed97a7c46d773502d68dd4` | Cargo.lock ve README; başarılı doğrulama sonuçları |

Bu son takip kaydının kendi commit SHA'sı öz referans oluşturmaz; `git log -1 --format=%H -- docs/PIVRA_COMMIT_TRACKER.md` ile alınır. Yukarıdaki `2950193` commit'i doğrulanan kaynakla aynı Cargo.toml/src içeriğini taşır; eklenen kilit üçüncü taraf bağımlılık getirmez.

**Kapanış kararı:** PIV-01'in derleme, temel yapı/README, test, CLI davranışı, kilit ve commit kanıtı koşulları sağlandı: **DONE**. Lisans seçimi açık tutulur. PIV-02 **TODO**, başlatılmadı; sonraki aşamaya otomatik geçiş yok.
