# Pivra — Planlı Commit ve İlerleme Takibi

**Kaynak:** [Özellik Şartnamesi](PIVRA_FEATURE_SPEC_TR.md) · [Yol Haritası](PIVRA_DEVELOPMENT_ROADMAP.md)  
**Branch:** `dev` · **Durum:** PIV-01 **DONE (21.10.2026)**; PIV-02 **DONE (21.10.2026, Xvfb GTK pencere doğrulaması)**; PIV-03 **DONE (21.10.2026)**; PIV-04 **DONE (21.10.2026; grafik test kategorileri ayrı)**; PIV-05 **DONE (21.10.2026; belgeleme/probe kapsamı)**; PIV-06–60 TODO, başlatılmadı. Faz 0 **INCOMPLETE** (temiz native kurulum yeniden üretimi NOT RUN). Güncel kapanış kanıtı son bölümde; 20.10.2026 kayıtları tarihçedir. Burada listelenen `PIV-xx` kimlikleri gerçek Git commit SHA'sı **değildir**.

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
| PIV-02 | `feat: create GTK4 libadwaita application shell` | GNOME stili boş/sade GTK pencere; açılış/kapanış Xvfb X11 üzerinde doğrulandı | **DONE — 21.10.2026** | `59bac1da79456cb39f42d2215b9ad670e1f3f900`, `9b8fa621fbfff7d26f61746d040fd7033afe8606` |
| PIV-03 | `refactor: define app media and settings boundaries` | Modül sınırları, GTK bağımsız modeller, CLI/GUI regresyonları doğrulandı | **DONE — 21.10.2026** | `7a1f867d5021a83014f8f4f72c5ef772327bc85d`, `4eaa34bf949f9cc77238198b499f45d21bdd569b` |
| PIV-04 | `test: establish fixtures and local verification commands` | Deterministik fixture/geçici dizin testleri, Rust/CLI ve ayrı X11/Wayland kanıtları | **DONE — 21.10.2026** | `42d9a35efaabef16e93a74354d2684c0f5937a7c`, `875bc07f92ceca4c6588d4ab8ef05c6b3a531d28` |
| PIV-05 | `docs: record dependency capability probes` | Sürümlü araştırma, izole probe, açık risk/ön koşullar ve Rust/X11/Sway/Weston regresyonları | **DONE — 21.10.2026** | `83e3590c1d4dbe7fd9cc89d9e352031198a3319e`, `aca1fb215fc7b23bdc5cfaa6fff3171bd11bddd5` |

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
| 0 — Temel | 01–05 | 5 | 5 |
| 1 — Fotoğraf | 06–12 | 7 | 0 |
| 2 — Zoom/performans | 13–18 | 6 | 0 |
| 3 — Video temel | 19–24 | 6 | 0 |
| 4 — Video gelişmiş | 25–31 | 7 | 0 |
| 5 — Arayüz | 32–39 | 8 | 0 |
| 6 — Uyumluluk/paket | 40–47 | 8 | 0 |
| 7 — Edit (en son özellik geliştirmesi) | 48–57 | 10 | 0 |
| 8 — Son doğrulama | 58–60 | 3 | 0 |
| **TOPLAM** | **PIV-01–60** | **60** | **5** |

> PIV-01–05 tamamlandı; PIV-06–60 gelecekteki işlerdir. Görev sayacı 5/5 olsa da Faz 0 kapısı INCOMPLETE: temiz native kurulum yeniden üretimi NOT RUN. Görev kimlikleri commit SHA'sı değildir. İlk planlama ve önceki engel kayıtları aşağıda tarihçe olarak korunur.

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


## PIV-02 kapanışı — 21.10.2026

**Güncel sonuç: DONE.** Kullanıcının PIV-02 görev metnindeki gerçek grafik pencere açılış/kapanış koşulu Xvfb X11 üzerinde sağlandı. Bu gerçek GTK pencere testidir; fiziksel GNOME/Wayland oturumu testi değildir. Önceki PIV-01 bölümleri tarihçedir. **PIV-03 başlatılmadı; Faz 0 tamamlanmış sayılmaz.**

### Başlangıç ve korunan durum

- Başlangıç yerel dev ve uzak dev: `04870b901333fe3dd0439d6bf75fbcd61d127f95`; çalışma dizini temizdi, PIV-01 DONE doğrulandı.
- Depo: `/workspace/Pivra`, `onbeskarakterli/Pivra`. Güncel şartname, yol haritası, takip tablosu, Cargo.toml ve README okundu. AGENTS.md bulunmadı.
- Uzak `main`: `5d55ee599df4617bd51c09b1c7a008b8d0ece480`; öncesinde ve uygulama push'u sonrasında aynı. Main'e yazılmadı, force-push/rebase/reset yapılmadı.
- Rust kurulumu ve önceki yapılandırma korundu; yeni Rust kurulumu yapılmadı. rustc/Cargo 1.99.1, rustfmt 1.10.0-stable, Clippy 0.1.99.

### Bağımlılıklar ve ortam

| Katman | Seçim / sürüm |
|---|---|
| Rust | Stable 1.99.1 ile doğrulandı; manifest minimumu 1.85 (2024 edition ve bağımlılık metaverisi) |
| Rust GTK bağlayıcısı | gtk4 0.10.3; Cargo adı `gtk`, `v4_6` API özelliği |
| Rust libadwaita bağlayıcısı | libadwaita 0.8.1; Cargo adı `adw`, `v1_2` API özelliği |
| Sistem GTK | Debian 13 GTK 4.18.6 |
| Sistem libadwaita | Debian 13 libadwaita 1.7.6 |
| Sistem GLib | 2.84.4 |
| Seçilen asgari API tabanı | GTK >=4.6, libadwaita >=1.2, GLib >=2.66; en düşük sürümlerle ayrı matris testi yapılmadı |

Dağıtımın sürümleri daha yüksek alt bağımlılık gereksinimleri taşıyabilir. İki doğrudan Rust bağımlılığı eklendi, geçişli bağımlılıklar Cargo.lock ile kilitlendi; medya motoru eklenmedi. Sistem geliştirme paketleri `libgtk-4-dev`, `libadwaita-1-dev` ve gerekli bağımlılıklarıdır. GUI test araçları: Xvfb, xauth, xdotool, Python 3 ve dbus-run-session.

Bu kullanıcıda yönetici paket kurulumu yok. APT'nin imza/checksum doğrulaması korunarak resmi Debian paketleri indirildi; `/workspace/pivra-environment/native` altına açıldı, mevcut sistem runtime kütüphaneleri yerel geliştirme bağlantıları üzerinden tekrar kullanıldı. Temel sistem paketleri değiştirilmedi. `gtk-activate.sh` pkg-config, linker/runtime ve yazılabilir XDG dizinlerini hazırlar; Rust için mevcut activate.sh kullanılır. `prepare-gtk.sh` bu yerel GTK hazırlığını tekrarlar. Bu yardımcılar ortam dosyalarıdır, proje dağıtım/paketleme kodu değildir.

### PIV-02.a–i durumları

| Alt görev | Durum | Kanıt |
|---|---|---|
| PIV-02.a Ortam/uyumluluk | DONE | Native sürümler pkg-config ile; gerçek yüklenen GTK/adwaita/GLib yolları ldd ile doğrulandı; Rust korunuyor |
| PIV-02.b Cargo | DONE | Uyumlu iki bağlayıcı ve Cargo.lock; check/build PASS |
| PIV-02.c Yaşam döngüsü | DONE | adw::Application activation ile pencere kurar; son pencere kapanınca normal çıkış 0 |
| PIV-02.d GNOME penceresi | DONE | AdwApplicationWindow, native HeaderBar ve boş genişleyen viewport; render görüntüsü incelendi |
| PIV-02.e Kimlik | DONE | `io.github.onbeskarakterli.Pivra`; GIO id_is_valid testi PASS; pencere başlığı Pivra. MIME/desktop dosyası ve dosya açma ileriki görevlerde |
| PIV-02.f Pencere yaşamı | DONE | Xvfb üzerinde iki bağımsız süreç/pencere, biri kapanırken diğeri açık; ikisi de WM_DELETE_WINDOW ile çıkış 0; displaysiz kontrollü çıkış 1 |
| PIV-02.g CLI regresyonu | DONE | --help/-h, --version/-V; geçersiz argümanlar stderr + exit 2; üç eski unit test değişmeden korunuyor |
| PIV-02.h Doğrulama | DONE | fmt/check/build/test/Clippy ve gerçek GTK X11 smoke PASS; atlanan Rust testi yok |
| PIV-02.i GitHub/takip | DONE | Bağımlılık ve uygulama/test commit'leri normal push ile origin/dev'e gönderildi ve uzak HEAD doğrulandı; bu kapanış kaydı ayrıca commit edilir |

### Son doğrulama sonuçları

Tüm Cargo komutları depo kökünde `source /workspace/pivra-environment/gtk-activate.sh` sonrasında stable toolchain ile çalıştırıldı. Derleme paralelliği `-j 2`; Cargo lock korunarak önceden indirilen crate'ler kullanıldı.

| Komut / kontrol | Gerçek sonuç |
|---|---|
| `cargo +stable fmt --check` | PASS — çıkış 0 |
| `cargo +stable check --locked --offline -j 2` | PASS — çıkış 0 |
| `cargo +stable build --locked --offline -j 2` | PASS — çıkış 0 |
| `cargo +stable test --locked --offline -j 2` | PASS — 4 unit + 4 integration; 0 başarısız, 0 ignored |
| `cargo +stable clippy --locked --offline --all-targets -j 2 -- -D warnings` | PASS — çıkış 0, uyarı yok |
| CLI help/alias | PASS — --help/-h için PIV-01 ile aynı tam stdout, stderr boş, exit 0 |
| CLI version/alias | PASS — --version/-V için Pivra 0.1.0 + newline, stderr boş, exit 0 |
| CLI geçersiz argümanlar | PASS — --unknown, one two, --help --version: stdout boş, stderr `Pivra: invalid arguments; use --help` + newline, exit 2 |
| Argümansız, DISPLAY/WAYLAND_DISPLAY yok, GDK_BACKEND=x11 | PASS — stdout boş; `Pivra: unable to initialize GTK; a graphical session is required:` ile başlayan kontrollü stderr, exit 1; panic yok |
| `GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 xvfb-run -a dbus-run-session -- python3 tests/gui_smoke.py` | PASS — iki ayrı 960x640 mapped GTK penceresi, başlık Pivra; normal kapatmalar exit 0; stdout/stderr boş; ilk kapanış ikincisini etkilemedi |
| Render görüntüsü | PASS — aynı Xvfb ortamında ImageMagick import ile alınan pencere görüntüsünde başlık, minimize/maximize/close kontrolleri ve boş alan incelendi; görüntü ortamda `piv02-window.png` olarak tutuluyor |
| `git diff --check` | PASS — whitespace hatası yok |

Unit testler: PIV-01'in `no_arguments_starts_app`, `help_and_version_flags`, `invalid_arguments_are_rejected` testleri ve yeni `application_id_is_valid`. Dört integration testi yedi CLI argüman senaryosunu ve tek displaysiz başlatmayı process seviyesinde stdout/stderr/exit assertions ile doğrular.

**İlk denemelerde bulunan ve giderilen sorunlar:** İlk check, GLib ExitCode için olmayan `.value()` çağrısı nedeniyle FAIL oldu; desteklenen Into dönüşümüyle düzeltildi. İlk link, çıkarılan geliştirme paketlerinin mevcut sistem runtime hedeflerine erişemeyen bağları nedeniyle FAIL oldu; yerel sysroot bağlantıları tamamlandı. İlk GUI testinde xdotool araması PID ve başlığı OR ile eşleştiriyordu; `--all` ile düzeltildi. Sonraki GUI denemesi yazılamayan varsayılan cache dizinleri nedeniyle stderr kontrolünde FAIL oldu; uygulama davranışını gizlemeden yazılabilir XDG dizinleri tanımlandı. Yukarıdaki son komut seti düzeltmelerden sonra bütünüyle PASS oldu. Debian snapshot adresinin 403 yanıtı, doğrulaması korunmuş resmi deb.debian.org deposu kullanılarak aşıldı.

### Gerçek grafik testinin kapsamı

Xvfb bir X11 grafik sunucusudur: uygulamanın gerçek GTK pencereleri map edildi ve Cairo ile render edildi. WM_DELETE_WINDOW standart kapatma protokolüdür; test başarı durumunda süreçleri kill/timeout ile kapatmaz. Pivra'ya test için otomatik kapanma bayrağı veya zamanlayıcı eklenmedi. Erişilebilirlik başlatımı yalnız minimal test oturumunda GTK_A11Y=none ile kapatıldı; uygulama erişilebilirliği veya sistem teması zorlamıyor.

**Yapılmayanlar:** Fiziksel GNOME Shell/Wayland oturumu, HiDPI/çok monitör, donanım renderer, erişilebilirlik ve Nautilus dosya ilişkisi testi. Bunlar PASS olarak işaretlenmez; bu görev metnindeki gerçek GTK grafik açılış/kapanış kriteri Xvfb ile doğrulandı. Lisans kararı açık. GTK/adwaita alt sürüm matrisi test edilmedi.

### Commit ve uzak dal kanıtı

| Tam SHA | Kapsam / uzak durum |
|---|---|
| `04870b901333fe3dd0439d6bf75fbcd61d127f95` | Başlangıç yerel/uzak dev; PIV-01 kapanış kaydı |
| `59bac1da79456cb39f42d2215b9ad670e1f3f900` | build(PIV-02): GTK/adwaita bağımlılıkları, minimum Rust ve Cargo.lock; origin/dev'e gönderildi |
| `9b8fa621fbfff7d26f61746d040fd7033afe8606` | feat(PIV-02): native pencere, CLI/GUI testleri ve README; normal push sonrası origin/dev HEAD olarak doğrulandı |

Bu son takip kaydının kendi SHA'sı `git log -1 --format=%H -- docs/PIVRA_COMMIT_TRACKER.md` ile alınır; commit kendi SHA'sını içermez. Son push'un uzak HEAD kanıtı ayrıca görev sonuç raporunda verilir.

**Değişen proje dosyaları:** Cargo.toml, Cargo.lock, src/main.rs, README.md, docs/PIVRA_COMMIT_TRACKER.md; yeni tests/cli.rs ve tests/gui_smoke.py. Ürün kararları, şartname ve yol haritası değiştirilmedi. Görsel/video çözme, katalog, zoom/pan, önizleme, edit, slayt, paketleme ve CI kapsamına girilmedi.

**Kapanış:** PIV-02 **DONE**; gerçek grafik açılış/kapanış, CLI regresyonu, tüm Rust kontrolleri ve kod push'u doğrulandı. PIV-03'e teknik ön koşul sağlandı; **PIV-03 başlatılmadı ve bu görevde ilerlenmeyecek.**


## PIV-03 kapanışı — 21.10.2026

**Güncel sonuç: DONE.** Yalnız PIV-03 uygulandı. Önceki bölümler kendi görevlerinin tarihçesidir. **PIV-04 başlatılmadı; Faz 0 bütünüyle tamamlanmadı.**

### Başlangıç, ortam ve kapsam

- Başlangıç yerel/uzak `dev`: `129a9826a75de0b5aaa4440a35df8871933fb2dc`; çalışma dizini temizdi. PIV-01 ve PIV-02 DONE kayıtları doğrulandı.
- Depo `/workspace/Pivra`; güncel şartname, yol haritası, takip tablosu ve README okundu. Mevcut main.rs, Cargo.toml, CLI regresyonları ve GTK/Xvfb smoke testi incelendi. AGENTS.md yok.
- Mevcut `/workspace/pivra-environment/gtk-activate.sh` kullanıldı. Yeni paket kurulmadı; Rust, GTK ve ortam yapılandırması sıfırlanmadı/değiştirilmedi.
- Doğrulanan sürümler: rustc/Cargo 1.99.1, native GTK 4.18.6, libadwaita 1.7.6, GLib 2.84.4. Mevcut rustfmt/Clippy kullanıldı.
- Cargo.toml, Cargo.lock ve tests/cli.rs değişmedi; yeni bağımlılık yok. Lisans kararı ve mevcut ürün kararları korundu.
- Korunan uzak main: `5d55ee599df4617bd51c09b1c7a008b8d0ece480`; kod push'u öncesi/sonrası aynı. Main'e yazılmadı, rebase/force-push/geçmiş değişikliği yapılmadı.

### PIV-03.a–h durumları

| Alt görev | Durum | Gerçek uygulama ve kanıt |
|---|---|---|
| PIV-03.a Sorumluluk analizi | DONE | Önce tek main.rs içindeki CLI parse/help, process dispatch, GTK init/activation ve widget kurma sorumlulukları belirlendi |
| PIV-03.b Modül ayrımı | DONE | main: dispatch/exit; cli: parse/help; app: GTK/kimlik/activation; window: mevcut pencere ağacı. Önceki CLI çıktıları ve pencere davranışı korundu |
| PIV-03.c Medya modeli | DONE | GUI bağımsız MediaItem(path, explicit MediaKind Image/Video); uzantıdan tür tahmini, varlık/okunabilirlik testi veya decoder yok |
| PIV-03.d Gezinme sınırları | DONE | MediaCollection: çağıranın verdiği sıralı liste ve geçerli seçili indeks; boş/tek öğe ve sınır aşımı testleri. Tarama, doğal sıralama, ileri/geri eylemi ve döngü politikası uygulanmadı |
| PIV-03.e Ayarlar | DONE | Settings yalnız bellekteki window_width/window_height varsayılanları; app üzerinden window tarafından kullanılıyor. 960x640 gerçek GUI boyutu doğrulandı; GSettings/global durum/persistence yok |
| PIV-03.f Bağımlılık yönleri | DONE | main -> cli/app, app -> window/settings, window -> settings; core'da yalnız catalog -> media. Geri yön/döngü yok; core doğrudan rustc ile GTK bağlamadan derlenip test edildi |
| PIV-03.g Testler | DONE | Format/check/build/Clippy, 9 unit + 4 integration, bağımsız core ve gerçek Xvfb pencere smoke PASS |
| PIV-03.h Commit/push/takip | DONE | İki kod commit'i normal push ile origin/dev'e gönderildi, uzak HEAD doğrulandı; gerçek SHA'lar bu kayda işlendi ve kapanış kaydı ayrıca commit edilir |

### Modül sözleşmeleri ve sınırlamalar

- `src/lib.rs`, `media`, `catalog`, `settings` çekirdeğini ve ortak APP_NAME sabitini sunar; GTK/GIO/app/window/CLI import etmez.
- MediaItem sadece çağıranın verdiği PathBuf ve MediaKind değerini taşır. Tür bilgisi dosyanın desteklendiği veya var olduğu iddiası değildir; dosya I/O yok.
- MediaCollection verilen sırayı değiştirmez; boş listede seçim None, dolu listede başlangıç 0. select(index) geçersiz indeks için false döndürüp önceki seçimi korur. Dışarıya yalnız immutable öğe slice'ı verir; indeks/listenin tutarlılığı dışarıdan bozulamaz.
- Settings mevcut pencere varsayılanlarını taşır; geleceğe yönelik kullanılmayan tema/geçmiş/decoder/ayar backend soyutlaması eklenmedi. Tema hâlâ GTK/libadwaita ve oturumdan miras alınır.
- App'in activation closure'ı yalnız Settings değerine sahiptir; Application referansını tutmaz, yeni bir referans döngüsü oluşturmaz. Pencere widget sahipliği ve kapanış davranışı PIV-02 ile aynı kaldı.
- Catalog/medya modeli henüz pencereye bağlanmadı; boş pencereye sahte medya/katalog akışı eklenmedi. Klasör keşfi, sıralama ve gerçek klavye gezinmesi sonraki görevlerin işi.

### Gerçek test sonuçları

Tüm komutlar depo kökünde, mevcut GTK activation betiği sonrasında çalıştırıldı; native kütüphaneler ve kilitli crate cache'i kullanıldı.

| Komut / kontrol | Sonuç |
|---|---|
| `cargo +stable fmt --check` | PASS — çıkış 0 |
| `cargo +stable check --offline --locked -j 2` | PASS — çıkış 0 |
| `cargo +stable build --offline --locked -j 2` | PASS — çıkış 0 |
| `cargo +stable test --offline --locked -j 2` | PASS — 5 core unit + 4 binary unit + 4 CLI integration; toplam 13 geçti, 0 başarısız, 0 ignored. Doctest sayısı 0; doğrulama kanıtı sayılmaz |
| `cargo +stable clippy --offline --locked --all-targets -j 2 -- -D warnings` | PASS — çıkış 0, uyarı yok |
| `rustc +stable --edition=2024 --test src/lib.rs -o /tmp/pivra-piv03-core-tests` ve üretilen binary | PASS — GTK/Cargo/extern olmadan aynı 5 core testi geçti; bunlar ek benzersiz test sayılmaz |
| `ldd /tmp/pivra-piv03-core-tests` | PASS — yalnız standart sistem runtime'ları; GTK, GLib, GIO veya libadwaita bağlantısı yok |
| PIV-01 CLI regresyonu | PASS — üç eski parse testi cli modülüne taşındı; değiştirilmemiş CLI integration testleri help/-h, version/-V, üç invalid argüman ve displaysiz açılışın tam stdout/stderr/exit sonuçlarını doğruladı |
| `GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 xvfb-run -a dbus-run-session -- python3 tests/gui_smoke.py` | PASS — iki ayrı gerçek GTK pencere, Pivra başlığı, tam 960x640 boyut; WM_DELETE_WINDOW ile exit 0, boş stdout/stderr; bir pencere kapanırken diğeri açık |
| Import/bağımlılık yönü incelemesi | PASS — core GUI katmanlarını import etmiyor; catalog yalnız media'ya bağlı; geri referans yok |
| `git diff --check` ve korunan manifest/lock/CLI test fark kontrolü | PASS — whitespace hatası yok; Cargo.toml, Cargo.lock, tests/cli.rs değişmedi |

Core testleri: verilen path/türün tahminsiz korunması, boş listede seçim olmaması, verilen sıranın korunarak ilk/son öğenin seçimi, sınır dışı seçimde mevcut öğenin korunması ve tek öğeli liste sınırı. Binary testler: önceki üç CLI parse testi ve application ID geçerliliği. Yeni GUI geometri assertion'ı 960x640 varsayılanlarının ayar modelinden pencereye gerçekten aktarıldığını doğrular.

Fiziksel GNOME Shell/Wayland, donanım renderer ve erişilebilirlik testi yapılmadı; PASS olarak raporlanmaz. Xvfb yalnız test süresince başlatıldı; uygulamada otomatik kapanma/test bayrağı yok. Başarılı GUI testinde süreçler normal pencere kapatma protokolüyle sonlandı. Açık PIV-03 blokajı yok; gerçek dosya/tür algılama ve kalıcı ayar gereksinimleri bu görevde uygulanmış sayılmaz.

### Commit ve push kanıtı

| Tam SHA | Kapsam / uzak durum |
|---|---|
| `129a9826a75de0b5aaa4440a35df8871933fb2dc` | Başlangıç yerel ve uzak dev |
| `7a1f867d5021a83014f8f4f72c5ef772327bc85d` | refactor(PIV-03): GUI bağımsız medya/katalog/ayar modelleri, core testleri |
| `4eaa34bf949f9cc77238198b499f45d21bdd569b` | refactor(PIV-03): CLI/app/window ayrımı, README ve güçlendirilmiş GUI boyut assertion'ı; normal push sonrası origin/dev HEAD doğrulandı |

Bu kapanış kaydının kendi SHA'sı öz referans oluşturmak yerine `git log -1 --format=%H -- docs/PIVRA_COMMIT_TRACKER.md` ile alınır; son uzak HEAD kontrolü görev sonuç raporunda verilir. Kod commit'leri origin/dev'e gönderildi; takip commit'i de aynı normal push ile gönderilir.

**Değişen dosyalar:** src/main.rs, README.md, docs/PIVRA_COMMIT_TRACKER.md, tests/gui_smoke.py; yeni src/lib.rs, src/app.rs, src/cli.rs, src/window.rs, src/media.rs, src/catalog.rs, src/settings.rs. Şartname ve yol haritası değiştirilmedi. Decoder, video oynatma, klasör tarama, zoom, Edit, Flatpak/AppImage, CI veya lisans eklenmedi.

**Kapanış:** PIV-03 **DONE**. PIV-04 ve sonrası **TODO; başlatılmadı**. Görev burada durur.


## PIV-04 kapanışı — 21.10.2026

**Güncel sonuç: DONE.** Deterministik test/fixture altyapısı ve mevcut regresyonlar doğrulandı. Grafik kategorileri aşağıda ayrı tutulur. **Fiziksel GNOME/Wayland NOT RUN; Weston normal kapanış NOT RUN.** Bunlar X11 veya Sway başarısıyla PASS sayılmadı. Kullanıcının PIV-04 koşulları bu kayıtlı kapsam sınırlarıyla sağlandı. **PIV-05 başlatılmadı; Faz 0 bütünüyle tamamlanmadı.**

### Başlangıç ve ortamın korunması

- Başlangıç yerel ve uzak `dev`: `2b425fa5b5808fff26ccf994559459be4b80a4aa`; çalışma dizini temizdi. PIV-01–03 DONE ve PIV-03 commit'leri doğrulandı.
- Depo `/workspace/Pivra`. Şartname, yol haritası, takip tablosu, README, mevcut unit/integration/CLI testleri ve Xvfb smoke okundu. AGENTS.md yok.
- Rust/Cargo 1.99.1, native GTK 4.18.6, libadwaita 1.7.6, GLib 2.84.4 korundu. Mevcut `gtk-activate.sh` kullanıldı; Rust/GTK kurulumu veya activation dosyası değiştirilmedi, yeniden kurulmadı.
- Production `src/` dosyaları değişmedi. Yalnız dev-dependency `tempfile 3.27.0` ve kilitli test bağımlılıkları eklendi; gerçek medya decoder/video motoru veya tarama işlevi yok.
- Başlangıç/uygulama push'u sonrası uzak main: `5d55ee599df4617bd51c09b1c7a008b8d0ece480`; main'e yazılmadı. Force-push/rebase/geçmiş değişikliği yapılmadı.

### PIV-04.a–h durumları

| Alt görev | Durum | Kanıt |
|---|---|---|
| PIV-04.a Test denetimi | DONE | Eski 9 unit + 4 integration korundu. Mevcut boş/tek/sınır katalog testleri gereksiz kopyalanmadı. CLI'nin yanlışlıkla gerçek Wayland socket'ine bağlanma riski özel runtime ile giderildi; GUI alt süreç timeout/cleanup ortaklaştırıldı |
| PIV-04.b Fixture stratejisi | DONE | 86-byte original PNG, 83-byte açıkça oynatılamaz video-model metni, deterministik generator ve SHA-256 manifest. JPEG/WebP/animasyon/video gereksinimleri belgeli; lisans seçilmedi |
| PIV-04.c Geçici dosyalar | DONE | tempfile özel dizinleri, Türkçe/boşluk/Unicode yollar; normal scope ve panic-unwind temizliği, 8 paralel aynı-ad dosya izolasyonu test edildi |
| PIV-04.d Model testleri | DONE | Eski boş/tek/çoklu/sıra/geçersiz indeks unit testleri PASS; yeni fixture integration testi gerçek kopyalarla Unicode path, Image/Video metadata, seçimin ve byte'ların korunmasını doğruladı |
| PIV-04.e CLI/GUI regresyonu | DONE | Help/-h/version/-V, üç invalid argüman exit 2; displaysiz X11 ve Wayland exit 1. Xvfb ve Sway'de başlık/boyut/iki bağımsız pencere/normal kapanış ve boş stdout/stderr doğrulandı |
| PIV-04.f Wayland + X11 | DONE — kapsamları ayrı | Xvfb PASS; Sway headless native Wayland yaşam döngüsü PASS; Weston headless gerçek yüzey/protokol PASS, Weston normal kapanış NOT RUN; fiziksel GNOME/Wayland NOT RUN |
| PIV-04.g Çalıştırma/belge | DONE | tests/check.sh, fixture integrity, 16 Rust testi ve tüm zorunlu Cargo kontrolleri PASS; README ve kısa yol haritası stratejisi güncellendi |
| PIV-04.h Commit/push/takip | DONE | İki test commit'i normal push ile origin/dev'e gönderildi ve uzak HEAD doğrulandı; gerçek SHA'lar bu tabloya kaydedildi. Bu kapanış belgesi ayrıca commit/push edilir |

### Fixture ve yeni test listesi

- `tests/fixtures/pixels-2x2.png`: orijinal sabit RGBA değerlerinden üretilmiş 2x2 PNG, 86 byte; dışarıdan indirilmiş içerik yok.
- `tests/fixtures/model-video.txt`: 83 byte **NOT playable media** metni; yalnız explicit Video metadata testi. Bu dosya gerçek video veya codec doğrulaması değildir.
- `tests/fixtures/generate.py`, `manifest.json`, `README.md`: sabit stored-DEFLATE PNG üretimi, SHA-256/uzunluk/provenance, proje lisansının hâlâ kararsız olduğu bilgisi; lisans dosyası/grant eklenmedi. JPEG/WebP/iki kare animasyon ve küçük gerçek video ihtiyaçları ileriki testlere belgeli.
- `tests/support/mod.rs`: test başına private tempfile dizini. `tests/models.rs` içindeki üç yeni integration testi: mixed_catalog_preserves_unicode_paths_kinds_order_and_file_contents; temporary_files_are_removed_after_normal_scope_and_unwinding; parallel_tests_can_use_the_same_filename_without_sharing_files.
- `tests/cli.rs`: mevcut dört test korundu; boş runtime ve zorlanmış backend ile headless hata artık X11 ve Wayland için ayrı alt senaryolarda doğrulanıyor.
- `tests/check.sh`: fixture + fmt/check/build/test/Clippy runner; GUI kategorilerini sessizce atlayıp başarılı saymaz. `tests/gui_support.py`: yalnız teste ait süreçler için bounded cleanup; cleanup bir normal kapatma kanıtı değildir.
- `tests/wayland_smoke.py`: Sway native Wayland yaşam döngüsü; `tests/weston_smoke.py`: ayrıca Weston 14 server-side yüzey/protokol testi. Eski `tests/gui_smoke.py` X11 kategorisi olarak korundu.

### Gerçek çalıştırma sonuçları

Mevcut GTK activation sonrasında komutlar `/workspace/Pivra` içinde çalıştırıldı; `cargo fetch --locked` ile yeni dev-dependency cache'i bir kez hazırlandı. İlk offline test girişimi henüz cache'te bulunmayan cfg-if nedeniyle FAIL oldu; kilitli fetch'ten sonra ilgili tüm kontroller PASS oldu. Bu kurulum/cache sorunu, test assertion başarısı olarak gizlenmedi.

| Komut / kontrol | Gerçek sonuç |
|---|---|
| `python3 tests/fixtures/generate.py --check` | PASS — PNG/metin/manifest beklenen byte'larla aynı |
| Geçici kopyada PNG'nin tek byte'ını bozup aynı --check'i çalıştırma | PASS negatif kontrol — bozulmuş kopya nonzero ve FAIL fixture differs ile reddedildi; repodaki fixture değiştirilmedi |
| `cargo +stable fmt --check` | PASS — exit 0 |
| `cargo +stable check --locked --offline -j 2` | PASS — exit 0 |
| `cargo +stable build --locked --offline -j 2` | PASS — exit 0 |
| `cargo +stable test --locked --offline -j 2` | PASS — **9 unit + 7 integration = 16**, 0 fail, 0 ignored; 0 doctest doğrulama kanıtı sayılmaz |
| `cargo +stable clippy --locked --offline --all-targets -j 2 -- -D warnings` | PASS — exit 0, uyarı yok |
| `PIVRA_ENV_ACTIVATE=/workspace/pivra-environment/gtk-activate.sh bash tests/check.sh` | PASS — aynı fixture ve Cargo seti gerçekten tamamlandı |
| PIV-01 CLI regresyonları | PASS — help/-h, version/-V tam stdout, boş stderr, exit 0; üç invalid argüman stdout boş, kontrollü stderr, exit 2 |
| Displaysiz CLI başlangıcı | PASS — özel socket-free runtime içinde hem GDK_BACKEND=x11 hem wayland için stdout boş, kontrollü GTK hata stderr'i, exit 1, panic yok |
| `GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 xvfb-run -a dbus-run-session -- python3 tests/gui_smoke.py` | **PASS X11/Xvfb** — iki ayrı mapped 960x640 Pivra pencere, WM_DELETE_WINDOW normal kapanışları exit 0; stdout/stderr boş ve bağımsız yaşam |
| `dbus-run-session -- python3 tests/wayland_smoke.py` | **PASS headless Wayland/Sway** — iki native xdg_shell yüzey, PID/app ID/başlık/960x640, Wayland close isteği sonrası exit 0; stdout/stderr boş ve bağımsız yaşam |
| `PIVRA_WESTON_PREFIX=/workspace/pivra-environment/weston/usr dbus-run-session -- python3 tests/weston_smoke.py` | **PASS Weston yüzey kontrolleri / normal kapanış NOT RUN** — aşağıdaki server-side kanıtlar; cleanup SIGTERM sonucu exit-0 kapanış testi sayılmaz |
| Fiziksel GNOME Shell/Wayland | **NOT RUN** — headless/X11 sonuçları bunun yerine geçmez |
| `git diff --check`, production src fark kontrolü | PASS — whitespace hatası yok; uygulama kaynakları değişmedi |

### Headless Wayland / Sway kanıtı

Başlangıçta compositor bulunmadı. Resmi Debian 13 APT paketleri, imza/checksum doğrulaması korunarak download-only ile indirildi; sway 1.10.1 ve wlroots 0.18.2 gerekli runtime'ları mevcut yerel native araç ağacına açıldı. Sistem paketleri kurulmadı/yükseltilmedi. Geçici config, wlroots headless/Pixman renderer, 1280x1024 çıkış, floating borderless pencereler ve `xwayland disable` kullanıyor.

Her test ayrı 0700 runtime/cache/config/data dizini ve D-Bus oturumunda çalışır. GDK_BACKEND=wayland zorlanır, DISPLAY kaldırılır. Başarılı sonuç yalnız PID/socket'e dayanmaz: compositor IPC ağacında her PID ve `io.github.onbeskarakterli.Pivra` için visible **xdg_shell** yüzey, Pivra başlığı ve 960x640 window_rect zorunludur. Kapatmada Sway'in con_id kill komutu xdg_toplevel.close gönderir; test app process exit 0, boş stdout/stderr, yüzeyin ağaçtan kalkması ve diğer pencerenin açık kalmasını doğrular. Test hook'u veya otomatik kapanma uygulamaya eklenmedi.

### Kullanıcının ek Weston araştırması ve kanıtı

- Sistem ve önceki native ağaçta Weston/libweston yoktu; temel Wayland client/server, Pixman ve DRM runtime'ları zaten vardı. Yönetici paket kurulumu yok; resmi Debian weston 14.0.2/libweston-14 ve gerekli runtime paketleri download-only ile alınıp **ayrı** `/workspace/pivra-environment/weston` altına açıldı.
- Rust/GTK kurulumu, mevcut activation ve sistem `/usr`/paket veritabanı değiştirilmedi. Weston'un relocated library path, WESTON_MODULE_MAP, veri ve desktop-shell/keyboard client yolları yalnız testin environment/config dosyasında tanımlı; kalıcı sistem ayarı yok.
- İlk absolute --backend path girişimi Weston tarafından reddedildi; belgelenen backend adı `headless` ve desteklenen WESTON_MODULE_MAP ile düzeltildi. Sonraki hard-coded /usr/libexec client hatası geçici weston.ini içindeki extracted client yollarıyla düzeltildi. Headless/Pixman bundan sonra başarıyla çalıştı.
- Test özel 0700 runtime, ayrı dbus-run-session, GDK_BACKEND=wayland, DISPLAY olmaması, xwayland=false ve Pixman/Cairo kullanır. Weston debug uzantısı yalnız bu izole test süresince etkin; compositor test sonunda durdurulur.
- **Gerçek yüzey kanıtı:** Weston scene-graph içinde app PID'sine ait xdg_toplevel, Pivra başlığı, headless output üzerinde mapped View ve gerçek SHM buffer. Sunucunun proto akışı aynı PID için set_title, set_window_geometry(...,960,640), ack_configure ve non-null wl_buffer attach/commit gösterir. Sadece process/socket başarısı PASS sayılmadı. Buffer'ın CSD gölgeleri nedeniyle pencere geometrisinden büyük olması hata değildir; 960x640 kontrolü xdg window geometry üzerinden yapılır.
- **Weston normal kapanış: NOT RUN.** Bu packaged headless kurulumun arayüzlerinde normal pencere kapatmayı otomatikleştiren standart kontrol IPC'si bulunmadı. Test cleanup sırasında yalnız kendi Pivra sürecine SIGTERM gönderir; bu exit-0/normal kapanış iddiası değildir. stdout/stderr'in boş olduğu ayrıca kontrol edilir. Normal native Wayland kapanışı Sway kategorisinde test edildi, Weston'a aktarılmadı.
- Fiziksel GNOME Shell/Wayland, HiDPI, touchpad, çoklu monitör, erişilebilirlik ve donanım render **NOT RUN**; gelecek doğrulamalar olarak ayrı tutulur. GTK_A11Y=none yalnız minimal test oturumu içindir.

Eksik compositor aracı ilgili scriptte BLOCKED/exit 77; startup/assertion hatası nonzero FAIL olur. Bu oturumdaki nihai Sway ve Weston surface kontrolleri gerçekten çalıştı. Test araçlarının tekrar hazırlanması ortam yardımcısı `prepare-wayland.sh` ile download-only/extract olarak doğrulandı; mevcut Rust/GTK yeniden kurulmaz.

### Commit ve uzak dal kanıtı

| Tam SHA | Kapsam / uzak durum |
|---|---|
| `2b425fa5b5808fff26ccf994559459be4b80a4aa` | Başlangıç yerel/uzak dev; PIV-03 DONE |
| `42d9a35efaabef16e93a74354d2684c0f5937a7c` | Deterministik fixture'lar, tempfile/test-only lock, private runtime CLI regresyonu, model/cleanup testleri ve core runner |
| `875bc07f92ceca4c6588d4ab8ef05c6b3a531d28` | Ayrı Sway/Weston native Wayland testleri, ortak GUI cleanup/timeout ve README/yol haritası; normal push sonrası origin/dev HEAD doğrulandı |

Bu son kapanış kaydının kendi SHA'sı öz referans yerine `git log -1 --format=%H -- docs/PIVRA_COMMIT_TRACKER.md` ile alınır. Son push ve uzak HEAD doğrulaması görev sonuç raporunda verilir.

**Değişen/eklenen dosyalar:** .gitignore, Cargo.toml/Cargo.lock, README, yol haritası/takip tablosu; tests/cli.rs, gui_smoke.py, check.sh, gui_support.py, models.rs, support/mod.rs, wayland_smoke.py, weston_smoke.py ve fixtures/{README.md,generate.py,manifest.json,model-video.txt,pixels-2x2.png}. Production src ve şartname değişmedi. Uygulama decoder, oynatıcı, tarama, zoom/pan/Edit, paketleme/Flatpak/AppImage veya CI içermiyor; lisans seçilmedi.

**Kapanış:** PIV-04 **DONE**. Weston normal kapanış ve fiziksel GNOME/Wayland sınırlamaları ayrı ve açık kayıttadır. PIV-05 **TODO; başlatılmadı**. Görev burada durur.


## PIV-05 kapanış — 21.10.2026

**PIV-05: DONE; Faz 0: INCOMPLETE.** Önceki PIV-04 sonundaki "PIV-05 başlatılmadı"
kaydı tarihçedir. Yalnız PIV-05 araştırma, probe ve belgelemesi yapıldı; PIV-06
ve sonraki özellikler başlatılmadı. Production `src/`, `Cargo.toml`, `Cargo.lock`,
mevcut fixture ve smoke testleri değiştirilmedi. Ortam sıfırlanmadı; sistem paketi,
CI, decoder/player, Flatpak/AppImage veya lisans eklenmedi.

### Başlangıç ve gerçek commit kanıtı

- Başlangıç: temiz `dev`, `origin/dev` ile eşit: `8b3947215c54dfd2515df61a50dd05c080eccf31`.
- İzole probe + envanter: `83e3590c1d4dbe7fd9cc89d9e352031198a3319e`.
- Teknik karar belgesi + README/yol haritası: `aca1fb215fc7b23bdc5cfaa6fff3171bd11bddd5`.
- Bu kapanış kaydı ayrı takip commit'idir; kendi SHA'sı kendi dosyasına yazılamaz.
  Kesin bitiş SHA'sı Git log/uzak HEAD ve teslim raporunda doğrulanır.
- `main` referansı: `5d55ee599df4617bd51c09b1c7a008b8d0ece480`; değiştirilmedi.

### PIV-05.a–h kabul durumu

| Alt görev | Durum | Kanıt ve sınır |
|---|---|---|
| a Ortam | DONE | Debian 13 x86_64; Rust/Cargo 1.99.1, GTK 4.18.6/adw 1.7.6/GLib 2.84.4; 190 pkg-config modülü ve factory snapshot'ı `docs/probes/PIV05_ENVIRONMENT.json` |
| b GTK | DONE | Sentetik texture/pixel/Paintable/Picture, GSK TextureNode, gesture/scroll nesneleri, realized surface scale=1; gerçek gesture, GPU ve fractional scale NOT RUN |
| c Glycin | DONE — araştırma | 3.0.8 wrapper hattı aday; format/loader/ICC/orientation/bellek/sandbox sürümlü kaynak değerlendirmesi. Loader/SDK eksik, sandbox probe BLOCKED; gerçek decode NOT RUN |
| d GStreamer | DONE — araştırma | 1.26.2 registry sorgusu, plugin grupları ve sürümlü sink/seek/audio/subtitle/export değerlendirmesi. Sıfır pipeline; playback NOT RUN |
| e Backend | DONE | Aşağıdaki gerçek Pivra X11/Sway/Weston regresyonları; fiziksel GNOME ve Weston normal kapanış NOT RUN |
| f Risk | DONE | Teknik belge bölümleri 8–10: byte bütçesi, iptal, codec, renk, driver, loader/ABI ve dağıtım riskleri |
| g Belge | DONE | `docs/PIVRA_TECHNICAL_CAPABILITIES.md`; çalıştırıldı/kaynak/NOT RUN/BLOCKED açıkça ayrı |
| h Kapanış | DONE | Küçük dev commit'leri, gerçek SHA/test kaydı; normal origin/dev push ve uzak HEAD eşitliği teslim sırasında doğrulanır |

### Gerçek komut sonuçları

Ortam: `source /workspace/pivra-environment/gtk-activate.sh`.

| Komut / deney | Sonuç |
|---|---|
| `bash tests/check.sh` | PASS: fixture, fmt/check/build/test/clippy |
| `cargo +stable fmt --check` | PASS |
| `cargo +stable check --locked --offline -j 2` | PASS |
| `cargo +stable build --locked --offline -j 2` | PASS |
| `cargo +stable test --locked --offline -j 2` | PASS: 9 unit + 7 integration = **16**, 0 failed/ignored; 0 doctest |
| `cargo +stable clippy --locked --offline --all-targets -j 2 -- -D warnings` | PASS |
| `python3 tests/fixtures/generate.py --check` | PASS: PNG/model-video/manifest deterministik bütünlüğü; decode iddiası yok |
| PIV-01 CLI regresyonları | PASS: mevcut integration testlerindeki help/version/no-args/hatalı argüman davranışları |
| `tests/gui_smoke.py` (Xvfb + D-Bus, GDK_BACKEND=x11, Cairo) | PASS: iki mapped 960x640 Pivra penceresi, normal WM_DELETE_WINDOW, exit 0 ve boş stdout/stderr |
| `tests/wayland_smoke.py` (D-Bus, Sway headless) | PASS: iki native xdg_shell yüzeyi, DISPLAY yok/XWayland kapalı; iki normal close, exit 0 ve boş stdout/stderr |
| `tests/weston_smoke.py` (D-Bus, Weston headless/Pixman) | PASS: native xdg_toplevel, title/geometri, ack_configure + buffer attach/commit; **normal close NOT RUN**, SIGTERM cleanup başarı sayılmaz |
| Ayrı boş Cargo target | PASS: locked/offline build 1m28s, test 1m07s, aynı 16 test. Yeni native kurulum değildir |
| Temiz target ikilisiyle X11/Sway/Weston | PASS: yukarıdaki üç kapsam ayrı ayrı tekrar doğrulandı; Weston normal close hâlâ NOT RUN |
| `bash tests/probes/run_gtk_api.sh` (Xvfb) | PASS: texture pixel round-trip, snapshot node, gesture/scroll nesneleri, scale=1. İlk realize trait derleme belirsizliği düzeltildi; nihai çalıştırma geçti |
| `bash tests/probes/resolve_media.sh` | RESOLVED ONLY: gtk/gdk 0.10.3, adw 0.8.1, glib/gio 0.21.5, glycin 3.0.8, common 1.0.3, utils 4.0.4, Gst 0.24.5. **Compile/link/runtime NOT RUN** |
| `python3 tests/probes/environment.py` | REGISTRY INSPECTED ONLY: runtime 1.26.2; 0 pipeline, medya decode edilmedi |
| Probe rustfmt, bash syntax, Python AST, JSON parse ve `git diff --check` | PASS |

Temiz target: `/workspace/pivra-environment/piv05/clean-target`.
GUI betikleri bu dizinin `debug/pivra` ikilisini positional argümanla aldı.
Mevcut target üzerinde de aynı üç GUI regresyonu geçti. Weston prefix'i
`/workspace/pivra-environment/weston/usr`; aktivasyon betikleri değiştirilmedi.

### Glycin hatasının katmanları ve açık medya kapıları

`bwrap --unshare-all --die-with-parent --new-session --ro-bind /usr /usr
--symlink usr/lib /lib --symlink usr/lib64 /lib64 -- /usr/bin/true` exit 1:
`bwrap: setting up uid map: Read-only file system`.
Paketli bubblewrap 0.12.0'ın ldd bağımlılıkları çözülüyor; Codex'te `/proc` read-only,
NoNewPrivs=1, Seccomp=2. Bu ortamın UID map yazma kısıtı **genel Debian/Glycin
uyumsuzluğu kanıtı değildir**. Codex dışındaki native sandbox NOT RUN. Güvenlik
izolasyonu, mount veya yetki sınırları değiştirilmedi; bypass/fallback yapılmadı.

Ayrı risk: Debian aday loader 1.2.1 (1+) araştırılan Glycin 3.0.8 (2+) ile
protokol hattı bakımından eşleşmiyor; kurulu loader yok. Bu gerçek paketleme/ABI
riski sandbox kısıtından bağımsızdır. Glycin hâlâ koşullu ilk adaydır; decode PASS değil.
PIV-06'nın decoder gerektirmeyen dosya/URI sınırları ile gerçek fotoğraf açılışı
ayrılmalı; görüntü entegrasyonundan önce eşleşen loader/SDK, çalışan sandbox ve
JPEG/PNG/orientation deneyleri ön koşul (PIV-09 kapısı).

GStreamer video adayı olarak kalır. SDK, gtk4paintablesink ve Libav yok;
avdec_h264/avdec_h265 bulunmuyor. PIV-19 öncesi sink availability/aynı viewport'a
embedding ve iki backend'de gerçek pipeline testi gerekir. Libav kullanan codec
fixture'ları ayrıca paket+decode testi ister; diğer decoder factory'lerinin varlığı
oynatma kanıtı değildir. Ses/donanım/subtitle/seek/frame-step/export **NOT RUN**.
Fiziksel GNOME/Wayland, GPU, fractional scale ve Weston normal kapanış **NOT RUN**.

### Ayrı Faz 0 kararı

Görevler **5/5 DONE**. Buna rağmen literal faz kapısındaki **temiz native kurulumda**
derleme/test/pencere davranışı bağımsız yeniden üretilmedi. Boş Cargo target
sonuçları mevcut native SDK ortamını kullanır; temiz makine veya snapshot restore
kanıtı sayılmaz. Dolayısıyla **Faz 0 INCOMPLETE**, temiz kurulum kapısı **NOT RUN**.
Ortamı koruma talimatına uyuldu; kapıyı kapatmak için ortam sıfırlanmadı. Eksik
gelecek medya testleri belgelendi; bu araştırmanın DONE olması entegrasyona PASS vermez.
**PIV-06 başlatılmadı.**
