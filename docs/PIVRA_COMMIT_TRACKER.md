# Pivra — Planlı Commit ve İlerleme Takibi

**Kaynak:** [Özellik Şartnamesi](PIVRA_FEATURE_SPEC_TR.md) · [Yol Haritası](PIVRA_DEVELOPMENT_ROADMAP.md)  
**Branch:** `dev` · **Durum:** PIV-01 **DONE (21.10.2026)**; PIV-02 **DONE (21.10.2026, Xvfb GTK pencere doğrulaması)**; PIV-03 **DONE (21.10.2026)**; PIV-04 **DONE (21.10.2026; grafik test kategorileri ayrı)**; PIV-05 **DONE (21.10.2026; belgeleme/probe kapsamı)**; PIV-06 **DONE (21.10.2026; dosya/URI ve desktop giriş altyapısı; gerçek Nautilus NOT RUN)**; PIV-07 **DONE (21.10.2026; sırasız komşu medya keşfi)**; PIV-08 **DONE (21.10.2026; deterministik doğal sıra)**; PIV-09 **DONE (21.10.2026; Arch gerçek JPEG/PNG/EXIF, otomatik geometri ve güvenli aktif-decode kapanış PASS)**; PIV-10 **DONE (22.10.2026; native piksel/Fit/gezinti PASS ve kullanıcı fiziksel sol/sağ doğrulaması)**; PIV-11 **DONE (22.10.2026; Arch native probe exit0 ve gerçek GNOME Yenile PASS)**; PIV-12 **DONE (24.10.2026; Arch secure medya hata testi ve manuel iki pencere izolasyonu PASS)**; PIV-13 **DONE (24.10.2026; Arch gerçek secure Fit/Fill/%100 probe exit0 ve GNOME manuel kontrol PASS)**; PIV-14 **DONE (24.10.2026; Arch secure zoom/pan probe exit0 ve GNOME gerçek fare kabulü PASS)**; PIV-15 **DONE (24.10.2026; Arch secure resampling probe exit0 ve GNOME manuel Smooth/Nearest PASS)**; PIV-16 **DONE (24.10.2026; Arch secure async/aktif decoder kapanış exit0 ve GNOME manuel hızlı gezinme PASS)**; PIV-17 **DONE (kullanıcı native/manuel kabul kaydı)**; PIV-18 **TESTING**; PIV-19–60 TODO, başlatılmadı. Faz 0 **INCOMPLETE** (temiz native kurulum yeniden üretimi NOT RUN). Güncel kapanış kanıtı son bölümde; 20.10.2026 kayıtları tarihçedir. Burada listelenen `PIV-xx` kimlikleri gerçek Git commit SHA'sı **değildir**.

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
| PIV-06 | `feat: accept image path from Nautilus` | Nihai hedef: fotoğrafa çift tıklama. Dosya/URI + GIO/desktop Exec doğrulandı; gerçek Nautilus DESKTOP-06 NOT RUN; video varsayılanı değiştirilmedi | **DONE — 21.10.2026 (giriş altyapısı)** | `ad0bcde080ee36c88dc74bd58844d73e7bff8986`, `cac751927b6e3a967cf0ca31b04d06971a5f9fbc`, `69feda8d5cc7342d941b71d5e8878081f2792211` |
| PIV-07 | `feat: scan neighboring media in folder` | Tek dizinde sırasız fotoğraf/video adayları, açılan alias seçimi, arka plan keşfi ve açık hata durumları | **DONE — 21.10.2026** | `9255519e8c576f067184a2818e5917bfcb9ec724`, `91c81e895cd3859981805690af01a7cafddbf30a`, `9a998ba40c793ff49a6ee4fe7e617a6041f48cfe` |
| PIV-08 | `feat: apply natural filename ordering` | Doğal karma medya sırası, alias seçimi ve deterministik byte politikası doğrulandı | **DONE — 21.10.2026** | `5a109ac26bd98201e8de9f5a3971189e5969a8ba`, `5cf8469070b4b574c6d2292a92e0d8167763d8a1` |
| PIV-09 | `feat: render JPEG PNG with orientation` | Arch gerçek JPEG/PNG/EXIF, iki görüntü/otomatik geometri ve aktif secure-decode kapanış/temizlik PASS; iki decoder zaman çakışması NOT MEASURED | **DONE — 21.10.2026** | İlk uygulama: `1685139d47afdd55ac47699726f49a66aa270bfc`; yeni probe: `66af846ae4b153ad1415c7d8ff7f2612896817e7`; Arch kanıtı: `c42774da2b0448a4014500dc4545a4b399d58d28` |
| PIV-10 | `feat: navigate previous and next media` | 54 Rust + 30 Python, otomatik native pixel/Fit/rapid/error/independent window PASS (exit0); gerçek GNOME fiziksel sol/sağ PASS, Loading image securely mesajı yükleme sonrasında kayboluyor; Tab için odaklanacak düğme yok | **DONE — 22.10.2026** | `028b747`, `b69ba2c`, `8403d12`, `80e501d`; tam SHA ve kanıt son bölümde |
| PIV-11 | `feat: watch folder changes and external modifications` | Arch gerçek GIO ve Glycin Yenile probe exit0; GNOME Yenile yeni görüntü PASS | **DONE — 22.10.2026** | `078d028`, `5dd8f14`; tam SHA aşağıda |
| PIV-12 | `test: harden media failures and window isolation` | 57 Rust + 30 Python, 18 scoped GTK/GIO PASS; native decoder kabulü NOT RUN | **TESTING** | `7b48e94`, `2b76543`; tam SHA aşağıda |

## Faz 2 — Zoom, pan, ağır medya ve kaynak yönetimi

| ID | Planlı commit başlığı | Kabul / test kontrolü | Durum | SHA |
|---|---|---|---|---|
| PIV-13 | `feat: implement fit fill and actual size` | 59 Rust + 30 Python, 21 scoped GUI/helper ve 144 sentetik snapshot PASS; secure native mod kabulü NOT RUN | **TESTING** | `b576c35`, `5ad7b98`; tam SHA aşağıda |
| PIV-14 | `feat: implement cursor anchored zoom and pan` | 63 Rust + 30 Python, 24 scoped GTK/GIO ve 360 sentetik snapshot PASS; native zoom/pan kabulü NOT RUN | **TESTING** | `0bacddb`, `e1d9771`; tam SHA aşağıda |
| PIV-15 | `feat: add image resampling choices` | 64 Rust + 30 Python, 27 scoped GTK/GIO ve 462 sentetik snapshot PASS; native secure resampling NOT RUN | **TESTING** | `5fec698`, `fd36f56`; tam SHA aşağıda |
| PIV-16 | `feat: add cancellable async image loading` | 67 Rust + 30 Python, 33 scoped GTK/GIO PASS; 3+1 bound / stale success/error / reentrancy / rename; secure native NOT RUN | **TESTING** | `636498c`, `d9c93eb`; tam SHA aşağıda |
| PIV-17 | `perf: preload neighbors with bounded memory` | Öncelik mevcut medya; RAM baskısında önbellek azalır | DONE (kullanıcı native/manuel kabul kaydı) | `e2f7504`, `a4fd9e0`; 72 Rust/30 Python/20 GUI PASS, secure native NOT RUN; [kanıt](PIV17_VALIDATION.md) |
| PIV-18 | `perf: preview and full resolution image pipeline` | RAW/TIFF önizleme, yükleme göstergesi ve viewport sürekliliği | TESTING | `99d49dd`, `2905917`; full-only fallback, erken preview/upgrade NOT RUN; [kanıt](PIV18_VALIDATION.md) |

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
| 1 — Fotoğraf | 06–12 | 7 | 7 |
| 2 — Zoom/performans | 13–18 | 6 | 5 |
| 3 — Video temel | 19–24 | 6 | 0 |
| 4 — Video gelişmiş | 25–31 | 7 | 0 |
| 5 — Arayüz | 32–39 | 8 | 0 |
| 6 — Uyumluluk/paket | 40–47 | 8 | 0 |
| 7 — Edit (en son özellik geliştirmesi) | 48–57 | 10 | 0 |
| 8 — Son doğrulama | 58–60 | 3 | 0 |
| **TOPLAM** | **PIV-01–60** | **60** | **17** |

> PIV-01–17 tamamlandı (PIV-17 kullanıcı beyanı); PIV-18 TESTING, PIV-19–60 gelecekteki işlerdir. Görev sayacı 5/5 olsa da Faz 0 kapısı INCOMPLETE: temiz native kurulum yeniden üretimi NOT RUN. Görev kimlikleri commit SHA'sı değildir. İlk planlama ve önceki engel kayıtları aşağıda tarihçe olarak korunur.

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


## PIV-06 kapanış — 21.10.2026

**PIV-06: DONE (yetkilendirilen giriş altyapısı kapsamı).** PIV-05 sonundaki
"PIV-06 başlatılmadı" kaydı tarihçedir. Kullanıcının açık PIV-06 talimatıyla yerel
dosya/URI, GIO open ve decoder içermeyen pencere ilişkisi tamamlandı. Nihai Nautilus
çift tıklama kabulünün masaüstü kısmı **DESKTOP-06 NOT RUN** olarak açık bırakıldı.
**PIV-07 başlatılmadı. Faz 0 INCOMPLETE** kaydı korunur: bağımsız temiz native
kurulum doğrulaması bu görevde yapılmadı; PIV-06 talimatı kapının PASS olduğu anlamına gelmez.

### Git kanıtı ve değişiklikler

- Başlangıç: temiz `dev`; `origin/dev` uzak HEAD'i ile eşit `242ff09bcbd2d9cdaa87341b68a5b102bb41ee2b`.
- Dosya/URI, GIO open, pencere ilişkisi ve ilgili testler: `ad0bcde080ee36c88dc74bd58844d73e7bff8986`.
- Desktop kaydı ve GIO parse/launch testleri: `cac751927b6e3a967cf0ca31b04d06971a5f9fbc`.
- README sözleşmesi, yol haritası ve DESKTOP-06 kapısı: `69feda8d5cc7342d941b71d5e8878081f2792211`.
- Bu sonuç tablosu ayrı kapanış commit'inde; kendi SHA'sı kendi dosyasına yazılamaz.
  Kesin bitiş SHA'sı teslim raporu ve Git/uzak HEAD'den doğrulanır.
- `main`: `5d55ee599df4617bd51c09b1c7a008b8d0ece480`; değiştirilmedi.
- Değişenler: `src/{main,cli,app,window}.rs`, yeni `src/input.rs`; `tests/cli.rs`,
  `tests/gui_support.py`, üç GUI smoke; yeni `tests/desktop.rs`, `tests/desktop_smoke.py`,
  `data/io.github.onbeskarakterli.Pivra.desktop`; README, yol haritası, bu tablo.
- `Cargo.toml`, `Cargo.lock`, core modeller, fixture kaynakları ve PIV-05 teknik
  risk belgesi değişmedi. Sistem kurulumu/CI/lisans/decoder/player eklenmedi.

### PIV-06.a–g

| Alt görev | Durum | Gerçek kabul / sınır |
|---|---|---|
| a CLI | DONE | Tek yerel yol veya file URI, Unicode/boşluk/reserved byte/percent encoding; OS argümanları non-UTF-8 yerel isimleri korur. Help syntax genişledi; flag/alias/exit davranışları korundu; çoklu argüman reddedilir |
| b GIO | DONE | GFile -> NON_UNIQUE + HANDLES_OPEN -> activate/app.open -> connect_open yeniden kontrolü -> pencere. D-Bus single-instance activation ilan edilmedi; uzak URI/VFS uygulanmadı |
| c Pencere | DONE | Pencere ömrüne ait GFile/MediaItem, dosya adı başlığı ve açık decoder-yok placeholder. Global current-file yok; X11/Sway bağımsız kapanışları geçti |
| d Doğrulama | DONE | Regular-file/read-only open/descriptor kontrolü, içerik okunmaz; symlink alias korunur. Eksik/okunamayan/klasör/special/unsupported kontrollü hata. TOCTOU ve senkron yerel I/O sınırları README'de |
| e Desktop/MIME | DONE — teknik altyapı | desktop-file-validate, GIO parse ve kaynak entry ile gerçek gio launch geçti. Exec=pivra %u, image/jpeg + image/png, DBusActivatable=false. Kurulum/default değişikliği yok; **Nautilus DESKTOP-06 NOT RUN** |
| f Test | DONE | Aşağıda 26 Rust testi, zorunlu Cargo/fixture ve üç backend'in eski+yeni smoke sonuçları ayrı |
| g Belge/commit | DONE | README sözleşme/komutlar, yol haritası açık gate, bu gerçek SHA/sonuç tablosu; normal dev push ve uzak HEAD eşitliği teslim sırasında doğrulanır |

### Gerçek test sonuçları

Aktivasyon: `source /workspace/pivra-environment/gtk-activate.sh`.
`bash tests/check.sh` çalıştırıldı; bütün zorunlu Cargo kontrolleri tamamlandı:

| Kontrol | Sonuç |
|---|---|
| `cargo +stable fmt --check` | PASS |
| `cargo +stable check --locked --offline -j 2` | PASS |
| `cargo +stable build --locked --offline -j 2` | PASS |
| `cargo +stable test --locked --offline -j 2` | PASS: **26 = 16 unit + 10 integration**, 0 failed/ignored; 0 doctest |
| `cargo +stable clippy --locked --offline --all-targets -j 2 -- -D warnings` | PASS |
| `python3 tests/fixtures/generate.py --check` | PASS; kaynak fixture/manifest byte'ları değişmedi |
| Eski CLI | PASS: help/-h, version/-V, invalid flags/multiple args exit 2; no-display X11/Wayland exit 1. Help dosya sözleşmesiyle genişletildi |
| Yeni input/CLI | PASS: yol ve URI, localhost, Türkçe/boşluk/#/?/%/emoji, relative/non-UTF-8; malformed/remote/query/fragment URI, missing/directory/special/unsupported, symlink/dangling ve okunabilirlik. Geçerli CLI girdileri no-display durumda GTK aşamasına ulaşıyor |
| Boyut/decode sınırı | PASS: 8 GiB sparse .jpg admission, içerik yüklemeden; geçerli uzantı decode iddiası değil. Bu performans/codec testi değil |
| Desktop entry | PASS: desktop-file-validate; GIO KeyFile/DesktopAppInfo parse ve yalnız iki image MIME adayı. Kurulmamış uygulama için parse testi build executable'ını geçici KeyFile'a açıkça koyuyor |
| GIO gerçek desktop launch (Xvfb) | PASS: değiştirilmemiş kaynak entry, private PATH symlink/XDG dizinleri, gio launch percent-encoded Unicode URI; doğru dosya başlığı, normal WM close ve process termination. Detached uygulama exit kodu **OBSERVED değil**; doğrudan GUI testleri exit 0 kontrol ediyor |
| Python syntax / git diff --check | PASS |

Grafik komutlarının her biri hem eski boş pencere modunda hem de aşağıdaki dosya
modunda çalıştırıldı; dosya kopyalarının içerikleri değişmedi:

```sh
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- python3 tests/gui_smoke.py target/debug/pivra --files
dbus-run-session -- python3 tests/wayland_smoke.py target/debug/pivra --files
PIVRA_WESTON_PREFIX=/workspace/pivra-environment/weston/usr \
  dbus-run-session -- python3 tests/weston_smoke.py target/debug/pivra --files
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- python3 tests/desktop_smoke.py
```

| Backend | Boş pencere regresyonu | Yeni dosya açılışı |
|---|---|---|
| X11/Xvfb | PASS: iki mapped pencere, 960x640, normal close/exit 0, boş stdout/stderr | PASS: local path + encoded URI, dosya adları doğru; birinin kapanışı diğerini etkilemez; iki normal close/exit 0 |
| Sway 1.10.1 headless/Pixman | PASS: iki native xdg_shell, normal Wayland close/exit 0 | PASS: local path + encoded URI, app ID/PID/title/960x640; bağımsız iki normal close/exit 0; XWayland kapalı/DISPLAY yok |
| Weston 14.0.2 headless/Pixman | PASS: gerçek xdg_toplevel + buffer/protocol kanıtı | PASS: encoded URI, Unicode dosya başlığı, mapped SHM surface, ack_configure + buffer attach/commit; XWayland kapalı/DISPLAY yok |
| Weston normal kapanış | **NOT RUN** | **NOT RUN**; SIGTERM cleanup PASS sayılmadı |
| Fiziksel GNOME/Wayland | **NOT RUN** | **NOT RUN** |
| Gerçek Nautilus çift tıklama / default photo handler | **NOT RUN — DESKTOP-06** | gio launch/CLI başarısı bunun yerine geçmez |

### Açık risk ve sonraki kapılar

- **DESKTOP-06:** native kurulumdan sonra gerçek Nautilus dosya gönderimi, mevcut
  fotoğraf/video default'ları ve bağımsız pencere davranışı gerçek oturumda ayrıca
  doğrulanmalı. Bu geliştirme placeholder'ı sistem varsayılanı yapılmadı.
- Symlink/path kontrolleri noktasaldır; sonraki açılışa kadar kaynak değişebilir.
  Senkron metadata/open, yerel görünen network mount veya özel dosyayla kötü niyetli
  değiştirme karşısında gecikme yapabilir. Decoder entegrasyonu güvenli descriptor,
  iptal ve gerçek kaynağın yeniden doğrulanmasını ayrıca ele almalı; burada decoder yok.
- PIV-05 Glycin loader/ABI/SDK ve Codex bubblewrap `/proc` sandbox engeli, GStreamer
  SDK/gtk4paintablesink/Libav eksikleri aynen açık. Güvenlik izolasyonu kapatılmadı.
  PIV-09/PIV-19 gerçek görüntü/video kapıları PASS sayılmadı.
- PIV-07 için mevcut tek-file/URI hata ve pencere sözleşmesi korunmalı; klasör
  keşfi/sıralaması/gezinme ayrı, ayrıca yetkilendirilecek iştir. Uzantı/MIME adayları
  gerçek içerik veya decoder başarısı olarak kullanılmamalı.

**Kapanış: PIV-06 DONE; PIV-07 TODO, başlatılmadı.**


## PIV-07 kapanış — 21.10.2026

**PIV-07: DONE.** Kullanıcının yalnız PIV-07 talimatıyla aynı klasörde medya
adayları keşfi uygulandı. Önceki “PIV-07 başlatılmadı” kayıtları kendi görevlerinin
tarihçesidir. **PIV-08 başlatılmadı.** Doğal sıralama, decode/playback, gezinme,
watcher veya sonraki özellik eklenmedi. Faz 0 temiz native kurulum kapısı
INCOMPLETE, DESKTOP-06 gerçek Nautilus kapısı NOT RUN olarak korunur.

### Başlangıç / commit kanıtı

- Başlangıç: temiz `dev`, uzak `origin/dev` ile eşit `a6c28ad2dd34965dc1f936cbc53fc039706f0847`; PIV-06 DONE/push doğrulandı.
- Std-only keşif ve deterministik testler: `9255519e8c576f067184a2818e5917bfcb9ec724`.
- Pencere kataloğu/worker/UI ve gerçek GUI testleri: `91c81e895cd3859981805690af01a7cafddbf30a`.
- README/yol haritası politikası ve kanıt sınırları: `9a998ba40c793ff49a6ee4fe7e617a6041f48cfe`.
- Bu takip kaydı ayrı kapanış commit'idir. Kendi SHA'sı kendi içeriğine yazılamaz;
  kesin bitiş SHA'sı Git/uzak HEAD ve teslim raporunda doğrulanır.
- `main` referansı `5d55ee599df4617bd51c09b1c7a008b8d0ece480` değişmedi.

### PIV-07.a–h kabul durumu

| Alt görev | Durum | Kanıt / politika |
|---|---|---|
| a Mevcut yapı | DONE | PIV-06 CLI/URI/Unicode/GIO/bağımsız pencereler ve image/jpeg,image/png desktop kabulü korundu; input/app/CLI ve MIME entry değiştirilmedi |
| b Klasör | DONE | Açılan yolun lexical parent'ı; relative -> absolute; dosya veya parent symlink alias'ı canonical target'a çevrilmez. Tek dizin, recursion yok |
| c Adaylar | DONE | Image: jpg/jpeg/png/webp/gif/bmp/tif/tiff. Video: mp4/mkv/webm/mov/avi. ASCII extension case-insensitive; decode veya codec iddiası değil; doğrudan giriş jpg/jpeg/png olarak kaldı |
| d Katalog | DONE | std-only discovery -> mevcut MediaItem/MediaKind/MediaCollection; fotoğraf+video tek katalog. read_dir sırası korunur, hiçbir sıra garantisi/sort yok; GTK/GIO core'a eklenmedi |
| e Seçim | DONE | Açılan exact alias/yol katalogda aranır ve seçilir; bulunamazsa hata, rastgele komşu seçilmez. Pencere ömrüne ait katalog; title tamamlanan katalogdaki current item ve count'tan gelir |
| f Dayanıklılık | DONE | Fatal directory error ve per-entry warning ayrı; missing/dangling/permission/iteration hataları testli. Worker off-main-thread, close/destroy cooperative cancellation, late UI update koruması. Bloklu syscall iptali/gerçek network testi NOT RUN |
| g Test | DONE | Aşağıdaki 40 Rust testi, zorunlu kontroller, fixture/desktop ve X11/Sway/Weston regresyonları; gerçek katalog ve hata UI kanıtları |
| h Belge/Git | DONE | Küçük dev commit'leri ve gerçek SHA/test kayıtları; normal origin/dev push + uzak HEAD eşitliği teslim sırasında doğrulanır; main değişmedi |

### Gerçek test sonuçları

Mevcut ortam `source /workspace/pivra-environment/gtk-activate.sh` ile kullanıldı;
kurulum/bağımlılık değişikliği veya ortam sıfırlama yapılmadı.

| Komut / kontrol | Sonuç |
|---|---|
| `bash tests/check.sh` | PASS: fixture + tüm zorunlu Cargo adımları |
| `cargo +stable fmt --check` | PASS |
| `cargo +stable check --locked --offline -j 2` | PASS |
| `cargo +stable build --locked --offline -j 2` | PASS |
| `cargo +stable test --locked --offline -j 2` | PASS: **40 = 18 unit + 22 integration**, 0 failed/ignored; 0 doctest |
| `cargo +stable clippy --locked --offline --all-targets -j 2 -- -D warnings` | PASS; ilk test iterator'ındaki manual_inspect uyarısı inspect ile düzeltildi, son kontrol geçti |
| `python3 tests/fixtures/generate.py --check` | PASS; orijinal fixture/manifest değişmedi |
| Yeni discovery integration | **12 PASS**: tek/boş/no-candidate, karma 8 image + 5 video uzantısı/case, doğru seçim, unsupported/subdir/directory-link/special-file exclusion, missing, symlink alias/outside target, broken link, Unicode/non-UTF-8, relative/parent alias, mode-000 file ve mode-111 folder permission, 1,200 aday + 100 excluded, pre-cancellation |
| Yeni discovery unit | **2 PASS**: enumeration sonrası entry silinmesi + iteration permission error; iteration sırasında cancellation -> partial success dönmez. Bunlar 40 testin içinde |
| GUI bağımsız library | PASS: `rustc +stable --edition=2024 --crate-type lib src/lib.rs -o /tmp/pivra-piv07-core.rlib`; Cargo/GTK/GIO linkage olmadan |
| Desktop entry | PASS: desktop-file-validate + mevcut GIO parse testi; JPEG/PNG MIME kaydı değiştirilmedi |
| GIO desktop launch | PASS: Xvfb'de kaynak entry/private PATH/XDG ile URI -> doğru dosya + **1 candidate** title; normal WM close/process termination. Detached exit status NOT OBSERVED; doğrudan GUI testleri exit 0 kontrol ediyor |
| Python AST / git diff --check | PASS |

Her üç backend eski boş pencere ve dosya/URI modunda ayrı çalıştırıldı.
Mevcut `--files` komutları iki PNG, bir text-only video-model, unsupported dosya
ve nested PNG içeren private fixture klasörleri oluşturur. **3 candidates** title
beklentisi gerçek worker sonucunun UI'ye ulaştığını ve current selection'ın doğru
olduğunu doğrular; yalnız süreç/socket başlatılması PASS sayılmaz.

| Grafik kategori | Gerçek sonuç |
|---|---|
| X11/Xvfb eski pencere | PASS: iki mapped Pivra, 960x640, bağımsız normal WM close/exit 0 ve boş stdout/stderr |
| X11/Xvfb katalog | PASS: local path + encoded Unicode URI; iki seçili dosya adı + tam 3 aday; normal bağımsız close/exit 0; fixture kopyaları değişmedi |
| Sway 1.10.1 headless/Pixman eski + katalog | PASS: iki native xdg_shell/app ID/PID/geometri; dosya modunda iki doğru seçili ad + 3 aday; normal Wayland close/exit 0; XWayland kapalı, DISPLAY yok |
| Weston 14.0.2 headless/Pixman eski + katalog | PASS: mapped xdg_toplevel/SHM buffer, title, 960x640, ack_configure + buffer attach/commit; URI modunda doğru seçili ad + 3 aday; native Wayland, XWayland kapalı/DISPLAY yok |
| X11 discovery hata UI | PASS: broken candidate link -> **1 candidate, 1 warning**, controlled stderr; mode-111 klasör -> **folder scan failed**, controlled stderr. Normal kapatma exit 0 yalnız lifecycle başarısıdır, scan başarısı değildir |
| Weston normal kapanış | **NOT RUN**; SIGTERM cleanup normal kapanış/exit-0 başarısı sayılmadı |
| Fiziksel GNOME/Wayland / gerçek Nautilus | **NOT RUN**; DESKTOP-06 korunur |

Komutlar: README'deki mevcut Xvfb/Sway/Weston smoke'ları önce argsız, sonra
`target/debug/pivra --files` ile; ayrıca `tests/desktop_smoke.py` ve
`tests/discovery_smoke.py` Xvfb + D-Bus altında. Tüm GUI testlerinde mevcut GTK
aktivasyonu, Sway ve `/workspace/pivra-environment/weston/usr` kullanıldı.

### Açık riskler ve korunmuş sınırlar

- Sembolik file link'leri readable regular target için kabul edilir; alias yolları
  ayrı kalır, aynı inode/target'a göre dedup yapılmaz. Broken link uyarıdır;
  directory/special-file link adayı katalog dışıdır. Link target'ı dış klasörde
  olabilir fakat o klasör taranmaz. Dosyalar yalnız read-only açılır, byte okunmaz.
- Enumeration sırasında değişiklikler atomik snapshot değildir; metadata/open ve
  sonraki decode arasında TOCTOU sürer. Worker cancellation kooperatiftir; kernel
  veya yavaş network-mount syscall'ını kesmez. Gerçek network/mount, aşırı büyük
  dizin performansı, bloklu I/O sırasında erken close **NOT RUN**. Zaman O(entries),
  bellek O(candidates + issues); hard RAM/dizin limiti eklenmedi.
- Klasörün açılamaması veya seçili dosyanın kaybolması fatal scan durumudur;
  yalnız açılan item korunur ve UI/stderr başarısızlığı açık gösterir. Per-entry
  sorunlarında healthy items + warnings gelir; incomplete tarama tam başarı sayılmaz.
- Video-model fixture oynatılabilir video değildir; tüm uzantılar yalnız adaydır.
  PIV-05 Glycin loader/ABI/SDK/Codex sandbox ve GStreamer SDK/sink/Libav kapıları,
  DESKTOP-06 ve Faz 0 temiz native kurulum kanıtı açık kalır; bypass yapılmadı.
- Doğal sıralama, navigation, watcher, decoder/video, zoom/thumbnail/Edit yok.
  `Cargo.toml`, `Cargo.lock`, input/CLI/app, core media/catalog modelleri,
  desktop MIME kaydı ve orijinal fixture'lar değişmedi. Yeni CI/paket/lisans yok.

Değişen dosyalar: `src/lib.rs`, yeni `src/discovery.rs`, `src/window.rs`, yeni
`tests/discovery.rs`, yeni `tests/discovery_smoke.py`, `tests/gui_support.py`,
`tests/desktop_smoke.py`, README, yol haritası ve bu takip tablosu.

**Kapanış: PIV-07 DONE; PIV-08 TODO, başlatılmadı.**


## PIV-08 kapanış — 21.10.2026

**PIV-08: DONE; PIV-09 TODO, başlatılmadı.** Önceki görevlerin PIV-08
başlatılmadı/sıra garantisiz kayıtları tarihçedir. Kullanıcının PIV-08 talimatıyla
worker'ın tamamladığı aday listesine doğal sıra eklendi; decoder/player/gezinme yok.

### Git ve dosya kanıtı

- Başlangıç: temiz dev, origin/dev ile eşit `c0492a40c920fe2ca6cbd550a82243865c4c2bbd`; PIV-07 DONE/push doğrulandı.
- Comparator, worker entegrasyonu ve testler: `5a109ac26bd98201e8de9f5a3971189e5969a8ba`.
- Politika/README/yol haritası: `5cf8469070b4b574c6d2292a92e0d8167763d8a1`.
- Bu takip kaydı ayrı kapanış commit'idir; kendi SHA'sı kendi içeriğine yazılamaz.
  Kesin bitiş SHA'sı Git/uzak HEAD ve teslim raporunda doğrulanır.
- main referansı `5d55ee599df4617bd51c09b1c7a008b8d0ece480` değişmedi.
- Değişenler: yeni `src/natural.rs`, `src/lib.rs`, `src/discovery.rs`,
  `tests/discovery.rs`, README, yol haritası, bu tablo. Cargo.toml/lock, CLI/input,
  app/window, core media/catalog, desktop entry ve orijinal fixture'lar değişmedi.

### PIV-08.a–h

| Alt görev | Durum | Gerçek kanıt / sınır |
|---|---|---|
| a Denetim | DONE | PIV-07 read_dir sırası garantisizdi; selection opened exact path üzerinden. Belgeler/model/worker sınırları incelendi |
| b Comparator | DONE | Std-only natural::compare_paths; ASCII katlama, sayı significant-length/digit karşılaştırması; integer parse yok. Primary eşitse ilk farklı sayı bloğu genişliği, sonra raw filename/full-path byte tie-break |
| c Karma medya | DONE | Bütün adaylar toplandıktan sonra tek sort_unstable_by; tür gruplama yok; path/kind değişmez. Gerçek karma katalog sırası opposite creation order ile test edildi |
| d Seçim | DONE | Sıralama katalog kurulmadan önce; scan_neighbors opened exact alias/yolu sonra bulup seçiyor. foto10.jpg seçimi, alias/raw ad, boş/tek öğe testleri geçti |
| e Mimari | DONE | Worker'da in-place sort, comparator filesystem I/O/heap allocation yapmaz; crate eklenmedi. Cancel check sort öncesi/sonrası; sort içi preemption yok. Kapanış/weak UI guards değişmedi |
| f Test | DONE | 5 yeni comparator unit, 2 yeni real-filesystem integration; mevcut 1,200 aday testine tam sıra ve index 917 eklendi. 18-name corpus'ta antisimetri/transitivity/equality + 18 rotated/reversed permutation; finite test, formal proof değil |
| g Regresyon | DONE | Aşağıda 47 Rust ve tüm Cargo/fixture/desktop/X11/Sway/Weston kapsamları; normal Weston/physical GNOME NOT RUN |
| h Belge/Git | DONE | README açık locale/tie/cancellation politikası, gerçek SHA/test tablosu; normal origin/dev push ve uzak HEAD eşitliği teslim sırasında doğrulanır |

### Gerçek test sonuçları

Mevcut `source /workspace/pivra-environment/gtk-activate.sh` kullanıldı. Ortam veya
bağımlılık kurulumu/değişikliği yapılmadı. `bash tests/check.sh` nihai çalışması PASS:

| Kontrol | Sonuç |
|---|---|
| cargo +stable fmt --check | PASS |
| cargo +stable check --locked --offline -j 2 | PASS |
| cargo +stable build --locked --offline -j 2 | PASS |
| cargo +stable test --locked --offline -j 2 | **47 PASS: 23 unit + 24 integration**, 0 failed/ignored, 0 doctest |
| cargo +stable clippy --locked --offline --all-targets -j 2 -- -D warnings | PASS; ilk unnecessary_lazy_evaluations uyarıları unwrap_or ile düzeltildi; nihai tam check runner geçti |
| python3 tests/fixtures/generate.py --check | PASS |
| Comparator testleri | PASS: 1/2/10, karma uzantılar, aynı kök, padding/all-zero, çoklu sayı blokları, 10,000-digit değerler/padding, ASCII case, Türkçe/Unicode/raw byte, full-path fallback ve total-order corpus |
| Gerçek katalog testleri | PASS: iki ters dosya oluşturma sırasında aynı tam sıra; foto10.jpg index 6; alias/raw byte korunur. 1,200 aday tam 0..1199 doğal sıra + seçili 917.png index 917 |
| PIV-07 ve CLI/URI | PASS: empty/single, permissions, removed entry/iteration error, symlinks, cancellation; eski flag/exit/URI davranışları |
| Bağımsız core derleme | PASS: rustc +stable --edition=2024 --crate-type lib src/lib.rs -o /tmp/pivra-piv08-core.rlib; Cargo/GTK/GIO linkage olmadan |
| Desktop entry / GIO | PASS: desktop-file-validate, GIO parse ve kaynak entry/private PATH/XDG gerçek gio launch. Detached exit status NOT OBSERVED; normal WM close/process termination doğrulandı |
| git diff --check | PASS |

| Grafik kategori | Sonuç |
|---|---|
| X11/Xvfb, eski + --files | PASS: iki mapped pencere; dosya/URI selected name + 3 candidates; bağımsız normal WM close, exit 0, boş stdout/stderr ve değişmeyen fixture byte'ları |
| Sway headless, eski + --files | PASS: iki native xdg_shell, app ID/PID/geometri ve selection/count; normal Wayland close/exit 0; XWayland kapalı/DISPLAY yok |
| Weston headless/Pixman, eski + --files | PASS: gerçek mapped xdg_toplevel/SHM buffer; title/geometri, ack_configure + attach/commit; native Wayland, XWayland kapalı/DISPLAY yok |
| X11 discovery warning/fatal UI | PASS: partial warning ve permission-error başlıkları + controlled stderr; normal window close. Scan başarısızlığı lifecycle başarısıyla karıştırılmadı |
| Weston normal close | **NOT RUN**, cleanup SIGTERM başarı sayılmadı |
| Fiziksel GNOME/Wayland, gerçek Nautilus | **NOT RUN**, DESKTOP-06 açık |

Komutlar: README'deki Xvfb `tests/gui_smoke.py`, D-Bus Sway
`tests/wayland_smoke.py`, Weston prefix ile `tests/weston_smoke.py`; her biri
argsız ve `target/debug/pivra --files` modunda. Ek olarak Xvfb+D-Bus altında
`tests/desktop_smoke.py` ve `tests/discovery_smoke.py`. Gerçek sıra filesystem
catalog testlerinde doğrulandı; sıralı liste/gezinme UI'si eklenmedi.

### Politika ve açık riskler

ASCII 0–9 numeric token; non-ASCII byte'lar opak. Whole filename/extension primary
karşılaştırılır. Leading-zero width yalnız tüm primary eşitliğinde uygulanır:
foto2 < foto02 < foto002; a02b3 < a2b10. Sonra exact raw filename/full path;
aynı byte path eşit. İşaret/decimal/Unicode digit özel sayısal anlam taşımaz.
Türkçe locale collation, NFC veya Unicode case folding **uygulanmadı**; ASCII I/i
birincilde eşitken İ/ı ayrı kalır. Raw Linux filename byte'ları değişmez.

Sort O(N log N × L) byte işi, benchmark/latency garantisi değil. Std sort iptali
araya almaz; cancel sort öncesi/sonrasında ve enumeration sırasında kontrol edilir.
Network/blocked syscall, extreme-scale/RAM/TOCTOU riskleri PIV-07'deki gibi açık.
PIV-05 Glycin loader/ABI/SDK/Codex sandbox, Gst SDK/sink/Libav ve Faz 0 temiz native
kurulum kapıları açık; güvenlik bypass yok. DESKTOP-06 NOT RUN. Video default'ları,
MIME giriş kuralları, lisans ve CI değişmedi. **PIV-09 başlatılmadı.**


## PIV-09 doğrulama kaydı — 21.10.2026 — BLOCKED

**PIV-09 DONE değildir. PIV-10 başlatılmadı.** Kaynak kodu, fixture'lar, derleme ve
decoder öncesi kontroller hazır; gerçek JPEG/PNG/EXIF/Fit başarısı yok. Önceki
“PIV-09 başlatılmadı” kayıtları kendi görevlerinin tarihçesidir. DONE sayacı **8**
olarak kalır; Faz 0 INCOMPLETE ve DESKTOP-06 NOT RUN kapıları korunur.

### Başlangıç ve commit'ler

- Başlangıç: temiz dev, origin/dev ile eşit `310dc16e6b49a51586384200fcadead9c9f3cdfa`; PIV-08 DONE/push doğrulandı.
- Glycin/GTK source + lock/CLI/help: `1685139d47afdd55ac47699726f49a66aa270bfc`.
- 18 orijinal asset/probe/harness: `018d9d91648ff582b8f6b19981a4e7ed741334f0`.
- README/teknik belge/yol haritası: `bebf6f232625cfdf6a6d02f5cf71b38e441aed9e`.
- Bu takip kaydı ayrı commit'tir; kendi SHA'sı kendi dosyasına yazılamaz. Kesin
  bitiş SHA'sı Git/uzak HEAD ve teslim raporunda doğrulanır.
- main `5d55ee599df4617bd51c09b1c7a008b8d0ece480` değişmedi. Yalnız dev normal push.

### PIV-09.a–j

| Alt görev | Durum | Kanıt ve açık kapı |
|---|---|---|
| a Sürüm | DONE | Glycin =3.0.8, common 1.0.3/utils 4.0.4; gtk/gdk 0.10.3, GLib/GIO 0.21.5/adw 0.8.1; actual compile/link geçti. 3.x COMPAT_VERSION=2. Debian aday loader 1.2.1+ds-2 hâlâ 1+ |
| b Loader/sandbox | **BLOCKED** | Kurulu 2+ loader/config yok. Resmi tag 2.0.0 kaynak eşleşmesi incelendi, build/install edilmedi. Bwrap true probe exit 1, UID-map read-only; mevcut yetkiyle güvenli runtime sağlanmadı. Bypass yapılmadı |
| c Çözümleme | CODED / BLOCKED runtime | image_loader: GIO -> forced Bwrap -> MIME JPEG/PNG -> Glycin Frame; boyut/boyut bütçesi/pre-cancel testli. Başarılı gerçek decode NOT RUN |
| d GTK | CODED / BLOCKED pixels | Worker frame -> main-thread GdkTexture/Picture; loading/error label ve weak references. Hata/lifecycle gözlendi, actual image sunumu NOT RUN |
| e Fit | CODED / BLOCKED | Picture ScaleDown/can_shrink; amaç aspect/center/contain, intrinsic logical-size cap. Gerçek küçük/büyük fotoğraf ve resize/HiDPI sonucu NOT RUN |
| f EXIF | **BLOCKED** | apply_transformations(true), view'da ikinci dönüş yok. Fixture tag 1–8 yapısı doğrulandı; gerçek yön/piksel testleri NOT RUN. Double-rotation olmadığı runtime ile henüz sertifikalanmadı |
| g Async/ömür | TESTING | Worker GLib context, Cancellable + close/destroy, weak UI; iki bağımsız hata penceresi normal kapandı. Aktif gerçek decode sırasında close, başarılı paralel frame ve peak RSS NOT RUN |
| h Fixture | CODED / TESTING | 18 küçük deterministik asset, golden RGB/corner/hash/size manifest, tekrarlı encode aynı çıktı. Pixel probe derlendi, baseline kapısı exit 77; olumlu piksel/EXIF branch NOT RUN |
| i Regresyon | TESTING | 49 Rust + Cargo + eski boş pencere/core/desktop/keşif/sıra kontrolleri PASS. File-window scoped lifecycle/error PASS; görüntü başarı kapısı BLOCKED. Gerçek X11/native Wayland image piksel/Fit kapısı kapanmadı |
| j Git/belge | DONE | Gerçek SHA/test kaydı, README/teknik karar güncellemesi; dev normal push ve uzak HEAD eşitliği teslim sırasında doğrulanır |

### Ortam / iki bağımsız engel

Resmi imzalı Debian APT metaverisi üzerinden yalnız gerekli SDK'lar download-only
alınıp mevcut kullanıcı prefix'ine çıkarıldı: liblcms2-dev 2.16-2+deb13u2 ve
libseccomp-dev 2.6.0-2 (.pc 2.16/2.6.0). Rust/GTK activation ve sistem kurulumu
sıfırlanmadı/yükseltilmedi. Yardımcı prepare-image-sdk.sh mevcut prefix'i korur.

Glycin gdk4 özelliği nedeniyle etkili native GTK tabanı artık **4.16**; manifest
v4_16 kullanır. Mevcut **GTK 4.18.6/adw 1.7.6** uygundur. Rust/Cargo **1.99.1**;
deklarasyon 1.85, ayrı 1.85 derlemesi NOT RUN. Cargo.lock yeni Glycin istemci
bağımlılıklarını sabitler; GStreamer veya başka production codec eklenmedi.

Resmi GNOME glycin tag 2.0.0, tag objesi 902fc5525180cd18d281afcfc977cefa40a684c1,
commit **724f18c5f3519885995428a7e2e6e5df9ca2ab27** incelendi: client 3.0.0 /
utils 4.0.0 / loaders 2.0.0 eşleşmesi. Bu cache checkout'u derlenmiş/kurulmuş loader
sayılmaz; gereksiz geniş codec/editor bileşenleri kurulmadı. Yapılandırılmış
Debian 13 kaynaklarında uyumlu 2+ binary yok; loader hazırlığı açık kalır.

Ayrı bwrap 0.12.0 probe:
`bwrap --unshare-all --die-with-parent --new-session --ro-bind /usr /usr
--symlink usr/lib /lib --symlink usr/lib64 /lib64 -- /usr/bin/true` -> **exit 1**,
`setting up uid map: Read-only file system`. Yeniden gözlem: /proc ro,nosuid,nodev,
noexec,relatime; NoNewPrivs=1, Seccomp=2/1 filter. Bu Codex UID-map kısıtı genel
native Linux/Glycin uyumsuzluğu değildir. Namespace/seccomp/mount/privilege sınırı
kurcalanmadı. Source **SandboxSelector::Bwrap** ister; Auto/NotSandboxed/fallback
kullanmaz; source directory expose kapalıdır. Loader eksikliği giderilse de bu
sandbox kapısı ayrıca geçmeden DONE olmaz.

### Gerçek komutlar ve sonuçlar

Mevcut gtk-activate.sh kullanıldı. Nihai `bash tests/check.sh` PASS:

| Kontrol | Gerçek sonuç |
|---|---|
| cargo +stable fmt --check | PASS |
| cargo +stable check --locked --offline -j 2 | PASS |
| cargo +stable build --locked --offline -j 2 | PASS |
| cargo +stable test --locked --offline -j 2 | **49 PASS = 25 unit + 24 integration**, 0 failed/ignored; 0 doctest |
| cargo +stable clippy --locked --offline --all-targets -j 2 -- -D warnings | PASS; probe'daki constant assertion const block'a alınarak ilk uyarı düzeltildi |
| Orijinal fixture --check | PASS; eski PNG/model bytes/manifest değişmedi |
| images/generate.py --check | PASS: 18 hash/size, decode iddiası yok |
| images/generate.py --write tekrar | PASS: aynı manifest hash'i; Pillow 12.3.0 yalnız trusted synthetic encoder. TIFF orientation tag 1–8 struct ile bağımsız kontrol edildi; yön sunumu değil |
| İki yeni image_loader unit | PASS: zero/extreme/pixel count; missing/sparse oversized/pre-cancel decoder başlamadan hata. Gerçek image decode testi sayılmaz |
| desktop-file-validate / GIO parse | PASS: aynı Exec=pivra %u ve JPEG/PNG MIME; yalnız açıklama source entegrasyona göre güncellendi |
| PNG real image_probe | **BLOCKED, exit 77:** No image loaders are configured. Config image_loader/image_editor boş |
| JPEG real image_probe --jpeg-first | **BLOCKED, exit 77:** aynı gerçek loader/config hatası |
| EXIF 1–8 / texture pixels / malformed metadata | **NOT RUN**, valid baseline başarısızken diğer error/metadata sonuçları PASS sayılmadı |
| Gerçek app Fit/resize/küçük/büyük/HiDPI/presented pixels | **NOT RUN/BLOCKED** |
| Python AST / git diff --check | PASS |

Probe komutu (Xvfb+D-Bus, GDK_BACKEND=x11/GSK_RENDERER=cairo/GTK_A11Y=none):
`cargo +stable run --locked --offline -j 2 --example image_probe`, ayrıca
`-- --jpeg-first`. Başarılı branch frame boyutu + gerçek texture.download quadrant
RGB/premultiplied-alpha kontrolü ve orientation 1–8 golden mapping içerir, fakat
bu makinede o branch çalışmadı. Bu application screenshot/Fit testi yerine geçmez.

### Grafik regresyonları ve anlamları

Boş pencere smoke'ları strict eski modda; dosya modları açık
`--files --expect-image-error` ile çalıştırıldı. Bu harness seçeneği app'e
aktarılmaz: gerçek image-error stderr'ini bekler ve **BLOCKED image decode/pixels**
yazar. Seçenek yoksa strict boş-stderr kontrolü korunur. Hata sessizce yutulmadı
ve hiçbir pozitif görüntü assertion'ı kapatılarak PASS üretilmedi.

| Backend / test | Sonuç ve sınır |
|---|---|
| X11/Xvfb boş pencere | PASS: iki mapped 960x640 Pivra; normal close/exit 0, boş stdout/stderr |
| X11/Xvfb file/URI | PASS yalnız katalog/lifecycle/error: doğru Unicode selected title + 3 candidates, iki kontrollü gerçek loader hatası ve normal bağımsız close/exit 0. **Image pixels BLOCKED** |
| Sway headless eski + file | PASS aynı scoped davranış: gerçek native xdg_shell, app ID/PID/geometri, normal Wayland close/exit 0; XWayland kapalı/DISPLAY yok. **Image pixels BLOCKED** |
| Weston headless eski + file | PASS yalnız mapped xdg_toplevel, SHM buffer/title/geometri/ack_configure/attach/commit ve kontrollü loader hatası; **fotoğraf pikselleri BLOCKED** |
| Weston normal close | **NOT RUN**; SIGTERM cleanup normal kapanış değildir |
| GIO desktop launch Xvfb | PASS source entry/private PATH/XDG -> associated file/catalog window + kontrollü loader hatası. Normal WM close/process disappearance; detached exit status NOT OBSERVED; fotoğraf başarısı değil |
| X11 discovery error UI | PASS partial warning/fatal scan durumları ve controlled loader error ayrı; fixture byte'ları değişmedi |
| Fiziksel GNOME/Wayland / Nautilus | **NOT RUN**, DESKTOP-06 açık |

Aynı README komutları kullanıldı; Sway/Weston private compositor ve mevcut native
prefix korundu. Fotoğraf piksellerinin X11/Wayland gerçek sunumu doğrulanamadı.

### Kaynak sınırları ve devam koşulu

64 MiB compressed, 32 MP/32768 edge ve 512 MiB returned-frame limitleri compile ve
preflight testlidir. Post-info/frame sınırı loader'ın metadata/transform/ara kopya
peak RSS'ini veya çoklu pencere toplamını hard-limit etmez. Tam decode latency,
iptal süresi, kaynak temizliği ve Fit resize gerçek fixture ile test bekler.
Glycin tek orientation uygular; view ikinci dönüş yapmaz. Picture ScaleDown/can_shrink
amaçlanan Fit/center/no-stretch ve logical intrinsic-size cap'tir; gerçek piksel
sunumu olmadığından bu davranış PASS değildir; fiziksel HiDPI %100 kanıtı yok.

2+ JPEG/PNG loader + namespace-capable güvenli native oturum sağlanınca aynı görevde
pozitif pixel/EXIF kapısı, gerçek app Fit/resize, bozuk image ve eşzamanlı/active
load close testleri tamamlanmalı. Bwrap bypass veya başka korumasız decoder
alternatifi bu kaydın çözümü değildir. PIV-05 GStreamer SDK/sink/Libav riskleri ve
Faz 0 temiz native kurulum kapısı açık. **PIV-09 BLOCKED; PIV-10 başlamadı.**

Son Git bütünlük kontrolünde NUL içermeyen truncated PNG metin olarak algılandı;
PNG imzasının zorunlu CR/LF byte'ları whitespace uyarısı üretti. Fixture içeriği
oynanmadı: images/.gitattributes JPEG/PNG'yi binary olarak tanımlar. Hash kontrolleri
tekrar geçti; final commit bu Git asset sınıflandırmasını da içerir.


## PIV-09 devam doğrulaması — 21.10.2026

Başlangıç `f5160711e93e88a388f0547aa679d9c7837c36cf`: temiz `dev`,
`origin/dev` ile eşitti. Önceki PIV-09 kodu korundu; üretim kaynakları,
Cargo.toml/Cargo.lock, fixture ve mevcut GUI assertion'ları değiştirilmedi.

| Gerçek commit | Değişiklik |
|---|---|
| `88021eef5890cf124eb5c5f52521e33c8aa9a11f` | İzole resmi loader kurulum yardımcısı, build manifesti ve gerçek istemci config discovery probe |
| `f08ff64c54a0d6ac4886b2f7f1d09c8ae7332dcf` | README, teknik yetenekler ve yol haritasında loader/sandbox ayrımı ve devam koşulları |

Resmi GNOME Glycin tag `2.0.0`, kaynak commit
`724f18c5f3519885995428a7e2e6e5df9ca2ab27` değiştirilmeden,
upstream Cargo.lock ile yalnız `glycin-image-rs` paketi derlendi.
Bu resmi bundle'ın zorunlu diğer Rust codec/editor yardımcı bağımlılıkları da
build'e dahildir; ayrı HEIF/JXL/RAW/SVG loader kurulmadı. Yalnız JPEG/PNG loader
kayıtları var, editor kaydı yok. Protokol ailesi **2+**, istemci **3.0.8**.
Build/hash kaydı: [PIV09_LOADER_BUILD.json](probes/PIV09_LOADER_BUILD.json).
`image-activate.sh` mevcut GTK aktivasyonuna yalnız özel GLYCIN_DATA_DIR ekler.
`ldd` çözümü PASS; sürüm/ABI hack'i veya sistem paketi değişikliği yok.

| Doğrulama | Gerçek sonuç |
|---|---|
| Loader helper tekrar çalıştırma / bash syntax | PASS; temiz kaynak ve aynı config korunarak cached build |
| `cargo run --locked --offline -j 2 --example loader_probe` | PASS: gerçek Glycin config API tam olarak image/jpeg ve image/png buldu; executable/config kontrol edildi. IPC handshake/decode değildir |
| Minimal Bwrap + gerçek JPEG strace | BLOCKED: namespace clone başarılı, ardından `openat(5,"uid_map",O_RDWR\|O_CLOEXEC) = -1 EROFS`; /proc ro |
| Süreç güvenliği | UID/GID 1000, CapEff 0, NoNewPrivs 1, Seccomp 2. Seccomp varlığı tek başına EROFS nedeni sayılmadı |
| İzinli alternatif yerel Docker ön kontrolü | BLOCKED: resmi digest-pinned Debian trixie, UID1000, cap-drop ALL/no-new-privileges/default security, read-only/network none ile Bwrap namespace oluşturma reddedildi. Alt seccomp/LSM/kernel politikası kesin ayrıştırılamadı; gevşetilmedi |
| Gerçek PNG `image_probe` / JPEG `--jpeg-first` | İkisi de **BLOCKED exit 77**: loader artık bulunuyor, fakat sandbox loader başlamadan UID-map kapısında hata veriyor |
| JPEG/PNG piksel / renk / EXIF 1–8 | **BLOCKED/NOT RUN**; baseline geçmedi, başarılı assertion yolları çalışmadı |
| Küçük/büyük gerçek görüntü, Fit/merkezleme/resize | **BLOCKED/NOT RUN** |
| Başarılı eşzamanlı decode / aktif yüklemede kapatma | **BLOCKED/NOT RUN**; hata-yolu pencere bağımsızlığı bunların yerine sayılmadı |
| `cargo fmt --check`, `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings` | **PASS**, mevcut `tests/check.sh`, stable/locked/offline/-j2; **49 = 25 unit + 24 integration**, 0 failed/ignored |
| Fixture bütünlüğü / CLI / katalog / doğal sıralama / desktop | PASS; eski fixture ve 18 yeni görüntü hash'i, desktop-file-validate/GIO parse korundu |
| X11/Xvfb | **4 scoped PASS**: boş pencere, file/URI, desktop launch, discovery error UI |
| Sway headless native Wayland | **2 scoped PASS**: boş ve file/URI pencere; gerçek xdg_shell, normal kapanış/exit0 |
| Weston headless native Wayland | **2 scoped PASS**: boş ve file/URI mapped xdg_toplevel/SHM/configure/attach/commit; normal kapanış **NOT RUN** |
| Fiziksel GNOME/Wayland ve Nautilus DESKTOP-06 | **NOT RUN** |

Sekiz grafik/helper çağrısı yalnız pencere/katalog/lifecycle veya kontrollü hata
kapsamında PASS'tir. Dosya modları açık `--expect-image-error` kullandı; bu app
seçeneği değildir ve gerçek loader hatasını şart koşar. Fotoğraf pikselleri
sunulduğu iddia edilmez. Weston SIGTERM temizliği normal kapanış değildir.

Devam için sahibi tarafından önceden izinli native/VM ortamında unprivileged
user namespace, kendi namespace'inde normal procfs üzerinden UID/GID map yazımı
ve Bwrap'ın mount/PID/network namespace işlemlerine mevcut güvenlik politikasınca
izin gereklidir. Önce minimal Bwrap true exit0, sonra aynı 2+ discovery ve gerçek
pixel/EXIF testleri, ardından uygulama Fit/resize/lifecycle testleri çalışmalıdır.
Bu oturumda mount/sysctl/seccomp/capability değişikliği, yetki yükseltme,
NotSandboxed veya korumasız decoder fallback yapılmadı. Codex proc kısıtı gerçek
native dağıtım uyumsuzluğu kanıtı değildir; IPC/pixel ABI çalışma zamanı hâlâ
onaylanmamıştır. PIV-05 GStreamer riskleri ve Faz 0 temiz native kurulum kapısı açık.
**PIV-09 BLOCKED; PIV-10 başlatılmadı.** Bu kayıt commit'i kendi SHA'sını içermez;
son SHA Git geçmişi/push raporundan doğrulanır.


## PIV-09 Arch Linux kullanıcı sonuçları ve eksik test değerlendirmesi — 21.10.2026

Başlangıç yerel HEAD: `459fbea40cc3048eabc14d8c996db1ab7023c726`, temiz `dev`.
Fetch sonrası `origin/dev` `242ff09bcbd2d9cdaa87341b68a5b102bb41ee2b` idi;
yerel HEAD'in atası olduğu doğrulandı. Fast-forward-only merge yerel değişiklik
yapmadı; reset/rebase/force-push kullanılmadı. `main` referansı
`5d55ee599df4617bd51c09b1c7a008b8d0ece480` olarak korundu.

### Kanıt A — kullanıcının aktardığı gerçek Arch/GNOME Wayland testleri

**Bu testleri Codex çalıştırmadı.** Kullanıcının belirttiği yerel test commit'i
`459fbea` (tam SHA yukarıda); ham log veya tam paket envanteri sunulmadı.

| Ortam / kontrol | Kullanıcının bildirdiği sonuç |
|---|---|
| Rust/Cargo | 1.99.1 |
| GTK4 / libadwaita | 4.24.1 / 1.10.0 |
| Glycin / Bwrap | Native Glycin 2.2.3, uyumlu 2+ loader'lar / Bwrap 0.13.1; Cargo istemcisi 3.0.8 olarak korunur |
| User namespace ve Bwrap izolasyon testi | PASS |
| PNG ve JPEG image_probe | PASS, exit 0 |
| EXIF orientation 1–8 piksel doğrulaması | PASS |
| Unicode dosya yolu / hatalı görüntülerde kontrollü hata | PASS |
| Gerçek GNOME/Wayland Pivra fotoğrafı | Görünür, merkezde, oran korunarak Fit: manuel PASS |
| Pencere resize / normal hatasız kapanış | Manuel PASS |
| cargo fmt --check | PASS |
| cargo check --locked / cargo build --locked | PASS |
| cargo test --locked | PASS: 49 test, 0 başarısız |
| cargo clippy --locked --all-targets -- -D warnings | PASS |
| Git çalışma dizini | Temiz |

Bu kanıt normal native ortamda güvenli decode ve manuel sunumun çalıştığını
bildirir; Codex sandbox'ının düzeldiği veya tüm dağıtımların doğrulandığı anlamına
gelmez. Arch native loader sürümü Cargo client sürümüyle karıştırılmadı.

### Kanıt B — bu devamda Codex'in bağımsız çalıştırdığı kontroller

Aktivasyon: `source /workspace/pivra-environment/piv09/image-activate.sh`.
Üretim src/Cargo bağımlılıkları, mevcut test assertion'ları ve fixture'lar korundu.

| Kontrol | Codex sonucu |
|---|---|
| `bash tests/check.sh` | PASS: iki fixture manifesti, fmt, check, build, test, all-targets Clippy -D warnings; stable/locked/offline/-j2 |
| Rust test toplamı | **49 PASS = 25 unit + 24 integration**, 0 failed/ignored; 0 doctest; CLI/katalog/doğal sıra/desktop dahil |
| Yeni example check / son fmt ve all-targets Clippy | PASS |
| X11/Xvfb boş ve file/URI, desktop launch, discovery hata UI | **4 scoped PASS**; file modlarında gerçek loader hatası açıkça beklendi |
| Sway headless boş ve file/URI | **2 scoped PASS**: gerçek native Wayland, normal close/exit0; file görüntü decode BLOCKED |
| Weston headless boş ve file/URI | **2 scoped PASS**: gerçek xdg_toplevel/configure/SHM/attach/commit; normal close NOT RUN |
| Yeni `image_gui_probe` Xvfb gerçek üretim-view yüklemesi | **BLOCKED exit77**; iki iş de secure loader başlangıcında hata verdi, pozitif görüntü/geometri assertion'ları çalışmadı |
| `git diff --check` | PASS |

GUI komutları önceki README kaydıyla aynıydı: X11 için GTK_A11Y=none,
GSK_RENDERER=cairo, GDK_BACKEND=x11 ve xvfb-run/dbus-run-session; Sway için
`dbus-run-session -- python3 tests/wayland_smoke.py`; Weston için özel prefix ve
`tests/weston_smoke.py`. File varyantlarında `target/debug/pivra --files
--expect-image-error`; desktop/discovery için binary ve expected-error seçeneği.
Bu sekiz çağrı decoded fotoğraf PASS'i değildir. Codex UID-map EROFS kısıtı aynı;
Bwrap zorunluluğu, host güvenliği ve loader izolasyonu değiştirilmedi.
Fiziksel GNOME sonucu yalnız Kanıt A'dır; gerçek Nautilus ve Weston normal close
hakkında yeni PASS iddiası yok. Faz 0 temiz native kurulum kapısı ayrı/açık kalır.

### Eklenen regresyon ve yalnız kalan PIV-09 kapıları

`examples/image_gui_probe.rs` doğrudan üretim image_view/image_loader'ını import
eder. Büyük PNG ve EXIF-6 JPEG için iki yükleme işini main loop öncesi başlatır;
iki gerçek texture'ı bekler. Gerçek Picture'ın WidgetPaintable/GSK snapshot'ında
alpha sınırlarıyla Fit boyutu/oranı/merkez ölçülür; render frame beklendikten sonra
resize'da tekrarlanır. İlk pencerenin normal kapanışından sonra ikincinin görünür
ve görüntüsünün doğru geometrili kalması gerekir. Bu offscreen GTK snapshot'ıdır,
compositor ekran fotoğrafı değildir; iki decoder'ın zaman aralıklarının çakıştığını
ölçmez. Yeni probe kullanıcı tarafından test edilen `459fbea` içinde yoktu.

1. **İki başarılı yükleme ve otomatik görüntü geometrisi/resize:** yeni probe
   çalışan güvenli native ortamda henüz çalıştırılmadı. Komut:
   `GDK_BACKEND=wayland cargo run --locked --example image_gui_probe`.
2. **Kanıtlı aktif decode sırasında kapanış:** bir secure decoder'ın gerçekten
   aktif olduğu gözlenmeli; pencere kapanınca stale callback/crash olmamalı,
   worker/loader temizliği sınırlı sürede tamamlanmalı, ikinci yüklü pencere
   etkilenmemeli. Önceden iptal edilmiş token unit testi, decoder başlamadan
   kapanış veya başarılı yüklemeden sonra normal kapanış bunu kanıtlamaz.

Bu iki eksik nedeniyle **PIV-09 TESTING**, DONE değil. Codex'e özel çalıştırma
engeli ayrı olarak BLOCKED kalır. Gerçek JPEG/PNG, EXIF ve manuel Fit kanıtını
tekrar eksik saymıyoruz; kaynak açıkça kullanıcı bildirimi. PIV-10 başlamadı.

| Gerçek yeni commit SHA | İçerik |
|---|---|
| `66af846ae4b153ad1415c7d8ff7f2612896817e7` | Gerçek üretim view'ı kullanan iki yükleme/snapshot geometri/resize gate |
| `c42774da2b0448a4014500dc4545a4b399d58d28` | README/teknik yetenek/yol haritasında kullanıcı Arch kanıtı ve TESTING durumu |

Bu takip kaydı ayrı commit'tir; kendi SHA'sı Git geçmişi/push raporundan doğrulanır.
Normal push sonrası origin/dev eşitliği ve main korunması ayrıca kontrol edilir.


## PIV-09 son test aşaması — aktif decode kapanışı, TESTING

Başlangıç: `7d1acf4932b6b7dfa21e42c00777b965cee54bcc`, temiz `dev`.
Üretim Cargo bağımlılıkları/lock ve fixture'lar korundu. `main` korunacak referans:
`5d55ee599df4617bd51c09b1c7a008b8d0ece480`. Normal push, geçmiş yazımı yok.

### Kullanıcı kanıtı — Arch Linux / gerçek GNOME Wayland

**Codex yürütmesi değildir.** Kullanıcı güncel `7d1acf4` commit'inde
`image_gui_probe` **exit0** bildirdi:

| Kullanıcının doğruladığı kapsam | Sonuç |
|---|---|
| Gerçek image_view snapshot Fit/center, 1600×1200 görüntü / 800×600 pencere | Kullanıcı PASS |
| Gerçek image_view snapshot Fit/center, 32×64 görüntü / 800×600 pencere | Kullanıcı PASS |
| İki bağımsız görüntü yüklemesi | Kullanıcı PASS |
| 480×320 yeniden boyutlandırma | Kullanıcı PASS |
| Başarılı decode sonrası bağımsız pencere kapatma | Kullanıcı PASS |

Önceki kayıttaki iki başarılı view/otomatik geometri kapısı artık kullanıcı kanıtıyla
kapanmıştır. İki decoder'ın zamansal çalışma aralıklarının çakışması **NOT MEASURED**;
bu yeni kapanış testi de ikinci pencereyi önceden yükler, eşzamanlı iki codec süresi
iddiası üretmez. Codex Bwrap UID-map EROFS engeli ayrı kalır.

### Kalan tek kapı ve eklenen test

[Ayrıntılı yöntem ve kesin Arch komutu](PIV09_ACTIVE_DECODE_TEST.md).
Yeni `tests/active_decode_close.py`, gerçek üretim view'ını kullanan
`examples/active_decode_close_probe.rs` çalıştırır. Gerçek secure image-rs
loader'ın 32 MP RGB çıktısında yazılmış prefix ve henüz yazılmamış tail aranır.
Kimliği pidfd/starttime/descendant üyeliğiyle sabitlenen kendi loader'ı kısa SIGSTOP
altında tekrar doğrulanır: aynı output fd/inode hâlâ loader'da ve writable shared
mapping aktif olmalıdır. Yalnız gözlemcinin açık tuttuğu eski buffer kabul edilmez.
NoNewPrivs/seccomp/etkin capability ve ayrı user/mount/PID/network namespace'leri
kontrol edilir. Rust tarafı gerçek load işine girildiğini ve henüz dönmediğini
şart koşar. Böylece queued, tamamlanmış veya yalnız PID'si bulunan iş PASS olmaz.

Pencere close-request cancellation'ı çalıştıktan sonra SIGCONT verilir; 10 saniye
sınırında secure load dönüşü ve gerçek GIO job join'i, kapalı widget weak-ref'lerinin
bırakılması, sıfır stale UI mutation, diğer pencerenin texture byte'larının aynı
kalması ve gözlenen loader/helper süreçlerinin zombiesiz kaybolması gerekir.
Join sonrası 300 ms gözlem vardır; sonsuza kadar callback yokluğu iddiası değildir.
GIO pool thread'lerinin kalması worker işi sızıntısı sayılmaz.

View'a küçük dahili job-completion observer eklendi; normal build no-op kullanır.
Üretim decode, sandbox ve cancellation/result guard'ları korunur. Bu kontrollü,
gerçek mid-frame işlemi pause ederek iptal testidir; kesintisiz doğal decode
zamanlaması veya latency benchmark'ı değildir. STOP/CONT güvenlik yetkisi genişletmez;
root/ptrace/sandbox değişikliği yok. İzinler veya loader bellek düzeni gözleme uygun
değilse ya da kısmi frame yarışı kaçırılırsa NOT RUN, asla sahte PASS yoktur.

### Bu aşamada Codex'in gerçekten yürüttüğü sonuçlar

| Kontrol | Sonuç |
|---|---|
| `bash tests/check.sh` | PASS: fmt/check/build/test/Clippy all-targets -D warnings; locked/offline stable -j2 |
| Rust | **49 PASS**, 25 unit + 24 integration, 0 failed/ignored; 0 doctest |
| Yeni Python observer guard testleri | **6 PASS**: boş/tamamlanmış output, yanlış piksel/boyut, PNG CRC/dimensions/scanlines, süreç kimliği ve bırakılmış buffer sahipliği |
| Fixture/CLI/katalog/doğal sıralama/desktop | PASS, mevcut kontroller ve fixture içerikleri korundu |
| Yeni Rust example check/build/Clippy | PASS |
| X11/Xvfb | 4 scoped PASS: boş, file/URI, desktop launch, discovery hata UI |
| Sway native headless Wayland | 2 scoped PASS: boş/file, normal close/exit0 |
| Weston native headless Wayland | 2 scoped PASS: mapped yüzey/configure/SHM/attach/commit; normal kapanış NOT RUN |
| Yeni gerçek active-close probe Xvfb | **BLOCKED / exit77**, baseline secure decode UID-map kapısında durdu; active checkpoint/close/cleanup pozitif yolları NOT RUN |
| Python -O assertion-disable guard | PASS: çalıştırmayı reddediyor, exit2; decoder testi değildir |
| git diff --check | PASS |

Grafik dosya modları açık expected-image-error kullandı; bu sekiz scoped sonuç
fotoğraf pikselleri PASS'i değildir. Yeni gerçek probe için böyle bir tolerans
yoktur: baseline gerçek fotoğraf yüklenmeden test başlayamaz.
Codex JSON'u: [PIV09_ACTIVE_DECODE_CODEX.json](probes/PIV09_ACTIVE_DECODE_CODEX.json),
gerçek driver/binary hash ve NOT RUN nedeni içerir. Native yeni active-close testi
henüz kullanıcı tarafından çalıştırılmadı. **PIV-09 TESTING**, DONE değildir.
Kalan yalnız native aktif-decode-close/cleanup sonucudur. PIV-10 başlatılmadı;
Faz 0 ve önceki bağımsız test kategorilerinin durumları kendiliğinden değiştirilmedi.

| Gerçek yeni commit SHA | İçerik |
|---|---|
| `9305bd26b1471cfc78e1039de69211f75a7b8c80` | Gerçek kısmi decode checkpoint'i, worker join observer, kapanış/temizlik probe'u ve guard testleri |
| `e54c07957eabf78eca1fd8499e1e027ab845f9c6` | STOP sonrası output fd/inode/mapping sahipliğini yeniden doğrulama; eski buffer yanlış pozitifini engelleme |
| `8359d7fa974e32d790a0e4a7110764711629a574` | Kullanıcı geometri PASS kaynağı, kesin native komut, yöntem/sınırlar ve Codex NOT RUN JSON'u |

Bu takip kaydı commit'inin kendi SHA'sı Git geçmişi ve son push raporundadır.


## PIV-09 aktif decode gözlemci teşhisi / v2 — 21.10.2026

Başlangıç temiz `dev`: `f363425732817ad562e7bda423f471f079f29e4a`.
PIV-09 **TESTING** kalır. `main` korunacak SHA:
`5d55ee599df4617bd51c09b1c7a008b8d0ece480`; PIV-10 başlamadı.

### Kullanıcı native teşhis kanıtı — bağımsız Codex çalıştırması değildir

Arch Glycin 2.2.3 / Bubblewrap 0.13.1; proc erişimi ve Python pidfd mevcut.
Kullanıcı gerçek glycin-image-rs sürecinin 8.285 kez örneklendiğini, beklenen
memfd'nin 30 kez görüldüğünü, mapping ve tam 96.000.000 byte boyut bulunduğunu,
Python/proc erişim hatası olmadığını aktardı. Tek ilk örnekten ilk 12 byte
`1153bf1153bf1153bf1153bf`, son 12 byte sıfır. Son native active-close sonucu
**NOT RUN/exit77**. Test edilen teşhis binary/driver tam SHA'sı bu yeni kullanıcı
bildiriminde ayrıca verilmedi; yeni v2 commit'lerine atfedilmedi.

Bu kanıt beklenen RGB/memfd biçimini destekler; 96-byte prefix/tail koşullarının
aynı STOP sonrası checkpoint'te geçmesi, loader sahipliği/mapping ve load pending
kanıtı değildir. Önceki kullanıcı JPEG/PNG/EXIF/geometri PASS'leri korunur; yalnız
aktif-decode kapanış/temizlik kapısı açık kalır. Decoder sürelerinin zamansal
çakışması NOT MEASURED kalır.

### İnceleme bulgusu ve değişiklik

Eski kod gözlemi target `view/present` döndükten sonra başlatıyordu. Her tur set
sıralı tam process/executable/fd keşfi yapıyordu; tek fd kaybolması kalan fd'leri
atlatabiliyordu. Kısmi adaydan sonra pidfd açımı, yeniden ağaç üyeliği ve
namespace/status okumaları SIGSTOP'u geciktiriyordu. Hangi boolean'ın başarısız
olduğu raporlanmıyordu. Bunlar kaynakta doğrulanan kör aralık/maliyetlerdir;
**native exit77'nin kesin tek nedeni eldeki kanıtla belirlenmedi.** Proc engeli
veya ABI uyumsuzluğu şeklinde yeni teşhis uydurulmadı.

V2, TARGET_STARTING/ARMED bariyeriyle gözlemci hazır olmadan target oluşturmaz.
PID'ler sıralı keşfedilir, kimlik/UID/executable/descendant ve starttime tekrar
kontrol edilerek pidfd örnekleme öncesi pin edilir. Doğru boyutlu çıktı fd'si
cache edilir; sonraki poll'lar ağaç taraması yapmaz, yaklaşık 0,2 ms hedef aralığı
kullanır. Bu gerçek frekans garantisi değildir: örnek başlangıç/bitişi ve en büyük
örnek aralığı JSON'a yazılır. Yanlış boyutlu memfd ve tekil kaybolan fd sonraki
adayları gizlemez. Kısmi adayda hemen pidfd SIGSTOP; pahalı namespace/status ve
aynı sıkı ownership/mapping/security koşulları STOP sonrasında yapılır.

Tüm canlı örnekleme aralığında stdout bufferlanır; sonuçta ilk aşama geçişleri ve
sayaçlar basılır. Baseline, loader, pin, memfd, boyut, tam 96-byte prefix, sıfır tail,
aynı örnekte partial, stop, sahiplik, mapping, sandbox, Rust pending checkpoint,
join ve cleanup ayrı raporlanır. İlk/son ve son kısmi örnekler faz/zaman/fd/inode
ile saklanır. Farklı örneklerdeki prefix ve tail başarıları birleştirilemez.
Tamamlanmış frame, kaybolmuş mapping, yanlış PID, izin reddi ve stop timeout hiçbir
zaman aktif checkpoint PASS'i olmaz. Üretim decoder/sandbox yavaşlatılmadı veya
gevşetilmedi; src/Cargo/data/fixture içerikleri bu devamda değişmedi.

### Bu devamda gerçekten çalıştırılan kısa kontroller

| Kontrol | Gerçek Codex sonucu |
|---|---|
| `bash tests/check.sh` | PASS: iki fixture manifesti; fmt, check, build, test, all-targets Clippy -D warnings; stable/locked/offline -j2 |
| Rust testleri | **49 PASS** (25 unit + 24 integration), 0 failed/ignored |
| Python gözlemci unit | **17 PASS**; önceki 6 guard ve 11 yeni tanı/race senaryosu |
| Yeni negatif senaryolar | 12/96-byte farkı, farklı örneklerin birleştirilmemesi, STOP öncesi tamamlanma, PID reuse, mapping/security reddi, izin reddi, tüm thread stop timeout; native decoder PASS'i değildir |
| Gözlem akışı unit | Arm-before-discovery, kaybolan erken fd sonrasındaki frame, dört cache okumasında yalnız pin/ilk-keşif ağaç kontrolleri; sıcak döngüde stdout flush yok |
| Rust example build/check ve shell wrapper `bash -n` | PASS |
| Python -O assertion guard | PASS: exit2 ile çalışmayı reddeder |
| Yeni gerçek probe Xvfb + zorunlu secure loader | **NOT RUN / exit77**: baseline secure decode başlamadı; first_uncompleted_stage=baseline_decoded |
| Yeni native v2 aktif checkpoint/kapanış/temizlik | **NOT RUN**, kullanıcı henüz çalıştırmadı |
| git diff --check; src/Cargo/data/fixture diff | PASS; production kaynakları/bağımlılıkları ve mevcut fixture/pixel assertion'ları korunur |

Yeni Codex kanıtı: [PIV09_ACTIVE_DECODE_OBSERVER_V2_CODEX.json](probes/PIV09_ACTIVE_DECODE_OBSERVER_V2_CODEX.json).
Önceki 8 scoped GUI regresyonu tarihçedir; bu yalnız gözlemci/test değişikliğinde
yeni görüntü/GUI PASS'i iddia edilmedi. Codex UID-map sandbox kısıtı aşılmadı.

**Yalnız native son test kullanıcıya kalır:** güncel dev checkout'unda
`bash tests/run_active_decode_close.sh`. Betik eşleşen Rust probe'u locked build
eder ve native Wayland sürücüsünü çalıştırır. Sonuç JSON'u
`/tmp/pivra-active-decode-close.json`; exit0 ve result=PASS birlikte gerekir.
[Ayrıntılı yöntem/yorumlama](PIV09_ACTIVE_DECODE_TEST.md).
PIV-09 TESTING, DONE değil; son native kanıttan sonra kapanış değerlendirilecek.

| Gerçek commit SHA | İçerik |
|---|---|
| `26c2846f940e1d867e8acf02090dc7ad35c8f48b` | Başlatma bariyeri, cached pidfd/frame gözlemi, ayrıntılı tanı, 17 guard ve tek native giriş betiği |
| `083bff60f3b4cd33a92b55e77a9dfadc6fb45239` | stdout backpressure'ını tüm canlı örnekleme döngüsünden çıkarma ve ilgili unit kontrolü |
| `cfe01b288b8e1daf9f7cce2dbd8f39eb4481a7b4` | Kullanıcı teşhis kanıtının sınırları, README/teknik/yol haritası/test komutu ve güncel Codex NOT RUN kaydı |

Bu takip kaydı ayrı commit'tir; kendi SHA'sı Git geçmişi ve push raporundadır.
Normal push sonrasında origin/dev eşitliği ve main korunması doğrulanır.


## PIV-09 V3 — baseline loader keşfi düzeltmesi, TESTING

Başlangıç: `69a98d3f7b93015cb3fc75c49c1356eeffa45436`, temiz `dev`.
`main` korunacak referans: `5d55ee599df4617bd51c09b1c7a008b8d0ece480`.
PIV-10 başlamadı. PIV-09 DONE değildir; son native V3 sonucu beklenir.

### Kullanıcı tarafından aktarılan native V2 kanıtı

**Codex'in bağımsız çalıştırması değildir.** Arch/GNOME Wayland V2 testi
NOT RUN/77, first_uncompleted_stage=loader_found ve 5.476 tarama bildirdi.
Ayrı 5.445 taramalık yerel teşhiste ilk taramada iki bwrap (142105, 142122) ile
`/usr/lib/glycin-loaders/2+/glycin-image-rs` (142123) zaten vardı ve loader tüm
taramalarda görünmeye devam etti. Başlangıçtaki bu loader'ın hedef görüntü için
gerçekte reuse edildiği bu kanıtla henüz gösterilmedi.

V2 `descendants(process.pid) - baseline` kullandığından bu başlangıç PID'lerini
keşiften tamamen çıkarıyordu. **Somut keşif hatası V3'te giderildi.**

### Yaşam döngüsü ve doğru hedef ilişkisi

Kilitli Rust Glycin **3.0.8** kaynakları incelendi: Loader::new varsayılan
Pool::global kullanır; pool.rs bağlı uygun config hash'li süreci tekrar verebilir.
Varsayılan retention **30 saniye**, paralel işlem sınırı varsayılanı sınırsızdır.
Image drop/cancellation request'e ait done(path) yapar; her yeni görüntü için
mutlaka yeni PID veya her cancellation'da anında process exit beklenemez.
Native Glycin 2.2.3 loader paketiyle Rust client sürümü karıştırılmadı. Kaynakta
reuse olanağı doğrulandı; Arch'ta gerçek reuse olmuş gibi PASS üretilmedi.

V3 ARMED öncesi tüm child PID/starttime kimliklerini ve probe+image-rs loader
frame dev/inode envanterini alır. Bwrap helper'ları kimlik olarak kaydedilir;
frame üreticisi olmayan helper fd'lerine gereksiz erişim istenmez. Gerekli frame
envanteri okunamazsa NOT RUN. Keşif baseline dahil tüm kendi loader'larını tarar;
origin PID **ve** starttime ile baseline/new olarak ayrılır. Eski inode'un boyutu
büyüse veya RGB koşulu aynı olsa da fresh kabul edilmez. Yalnız ARMED sonrasında
tek kontrollü 32 MP hedef işine uyan yeni inode'daki 96 MB frame değerlendirilir.

Tam 96-byte prefix/tail, pidfd, tüm thread STOP, yeniden kimlik, loader'ın aynı
inode/fd sahipliği ve writable-shared mapping, namespace/NoNewPrivs/seccomp/CapEff
ve Rust load-pending checkpoint'i korunur. Eski tamamlanmış veya eski kısmi
matching frame, yeni fakat tamamlanmış frame ve yalnız PID'nin bulunması PASS
olamaz. Üretim kaynakları, Cargo.lock/toml, fixture ve sandbox politikası değişmedi.

Test-only TARGET_LOAD_ENTERED/RETURNED ok/error telemetrisi yeni PID oluşmadığında
hedef işin başlamaması/erken hata/tamamlanmasını ayırır. Process inventory exe ve
origin kaydeder; target_frame_fresh ve eski frame reddi ayrı aşamalardır.
`candidate_loader_origin` yalnız adayı, `certified_loader_origin` ve
`reuse_observed` ancak fresh-frame + Rust pending checkpoint'i geçmiş ilişkiyi
anlatır. Codex baseline başarısızlığında not_observed/not_certified/null kalırlar.

Temizlik: önce gerçek job join/weak-ref/stale-callback/survivor byte'ları ve hedef
inode'un fd+mapping bırakılması gerekir. Bu aşamada pool'da canlı baseline süreç
sırf var diye leak sayılmaz; origin ile raporlanır. Sonra survivor/app normal
kapanmalı ve gözlenen baseline+new çocukların **tamamı** zombiesiz çıkmalıdır.
Toplam süre hedef close isteğinden itibaren hâlâ <10 s; dışarıdan kill ile PASS yok.
Son process-exit veya hedef kaynak temizliği koşulu kaldırılmadı.

### Bu devamda Codex'te gerçekten çalıştırılanlar

| Kontrol | Gerçek sonuç |
|---|---|
| `bash tests/check.sh` | PASS; iki fixture manifesti, fmt/check/build/test/all-targets Clippy -D warnings; stable/locked/offline -j2 |
| Rust regresyonu | **49 PASS** (25 unit+24 integration), 0 failed/ignored; 0 doctest |
| Python gözlemci regresyonu | **30 PASS**; eski 17 + 13 yeni baseline/telemetri/kaynak senaryosu |
| Yeni pozitif guard | Mevcut baseline loader taranır ve taze hedef frame'i örneklenir; native PASS değildir |
| Yeni negatif guard'lar | Eski matching partial/complete frame reddi, old inode büyümesi, unreadable inventory, PID reuse, tamamlanmış fresh frame, telemetri/protokol ayrımı, mapping-only referans, helper fd erişimi ve hedef/baseline kaynak ayrımı |
| Rust example locked/offline build/check/Clippy | PASS |
| Tek native wrapper `bash -n` | PASS; giriş komutu değiştirilmedi |
| Python -O assertion-disable guard | PASS: exit2 ile çalıştırmayı reddeder |
| Gerçek V3 probe / Xvfb | **NOT RUN/exit77**, baseline secure loader engeli; first_uncompleted_stage=baseline_decoded |
| Native V3 fresh-frame/reuse/checkpoint/kapanış/cleanup | **NOT RUN**, kullanıcı henüz çalıştırmadı |
| git diff --check ve üretim src/Cargo/data/fixture diff | PASS; üretim değişikliği yok |

Codex kayıt: [PIV09_ACTIVE_DECODE_OBSERVER_V3_CODEX.json](probes/PIV09_ACTIVE_DECODE_OBSERVER_V3_CODEX.json).
Unitlerdeki kontrol akışı simülasyonları gerçek decode PASS'i değildir. Önceki
kullanıcı decode/EXIF/Fit/iki görüntü/geometri PASS'leri korunur. İki decoder'ın
zamansal çakışması NOT MEASURED. Yeni GUI/piksel başarısı iddia edilmedi.

Güncel dev checkout'unda yalnız native son test için tek komut:
`bash tests/run_active_decode_close.sh`.
JSON `/tmp/pivra-active-decode-close.json`; result=PASS ve exit0 birlikte gerekir.
[Detaylı yöntem ve origin yorumlaması](PIV09_ACTIVE_DECODE_TEST.md).
**PIV-09 TESTING; native sonuç sonrası kapanış değerlendirilecek. PIV-10 başlamadı.**

| Gerçek commit SHA | İçerik |
|---|---|
| `775b5bce0f41a8a02ad8a9619e424013328a510f` | Baseline loader keşfi, taze hedef frame envanteri, target telemetri/origin ve pool-aware tam cleanup |
| `97db18ba790154aa7b1fd1d51a814ad2b43b6315` | Helper kimliği korunurken ilgisiz helper buffer erişimini önleme ve 30. regresyon |
| `7914d70b6d972830382a4418580b82ee52dba87a` | Pool yaşam döngüsü, kullanıcı V2 bulgusunun sınırları, güncel yöntem ve Codex V3 NOT RUN kanıtı |

Bu takip kaydı commit'inin kendi SHA'sı Git geçmişi/push raporundadır; normal push
sonrası origin/dev eşitliği ve main korunması ayrıca kontrol edilir.


## PIV-09 native V3 kapanışı — 21.10.2026 — DONE

**Kanıt kaynağı:** Kullanıcının kendi Arch Linux / GNOME Wayland bilgisayarından paylaştığı terminal çıktısı ve `/tmp/pivra-active-decode-close.json`; Codex tarafından çalıştırılan test **değildir**. Test edilen `dev` commit'i: `c3e0aaf9e1d19ee73f35d1167b7363ce48c70d86`. Komut: `bash tests/run_active_decode_close.sh`. **Çıkış 0**, JSON `result=PASS`, `observer_version=3`.

- `READY survivor_loaded` ve `TARGET_LOAD_ENTERED` doğrulandı. Önceden mevcut Glycin image-rs loader'ının hedef için gerçekten tekrar kullanıldığı **`reuse_observed=true`**, `certified_loader_origin=baseline` ile kanıtlandı; V2'nin baseline PID'lerini dışlaması bu nedenle geçersizdi.
- Yeni hedef frame'i: 8000×4000 RGB, **96.000.000 byte**, doğru 96-byte RGB prefix ve sıfır tail; yeni inode; pidfd/SIGSTOP ile tüm thread stop; loader'ın aynı fd/inode ve writable-shared mapping sahipliği kontrol edildi. `checkpoint_certified`, `mapping_verified`, `sandbox_verified` PASS. NoNewPrivs=1, Seccomp=2, CapEff=0; user/mount/PID/network namespace ayrılığı gözlendi.
- Gerçek yükleme sürerken pencere kapatıldı; hedef decoder `Operation was explicitly canceled` döndürdü. `job_joined=true`, `widgets_released=true`, `stale_mutations=0`, `survivor_pixels_unchanged=true`; `target_resources_released` ve `children_cleaned` PASS.
- `close_to_cleanup_seconds=0.31790098699275404` (10 s kabul sınırının altında), toplam çalışma `elapsed_seconds=0.7865941640047822`. Son temizlikte `Cleanup SIGCONT unavailable: [Errno 3] No such process` mesajı, ilgili loader'ın önceden çıkmış olmasından kaynaklandı; testin çıkışı 0 ve kaynak temizliği başarılı.
- Önceki kullanıcı Arch testleri: gerçek JPEG/PNG piksel, EXIF orientation 1–8, Unicode/bozuk dosya kontrolleri, manuel GNOME Fit/resize/close ve `image_gui_probe` iki görüntü/snapshot/geometri/normal kapanış **PASS**. Codex'in son belgelenmiş kısa regresyonları: **49 Rust + 30 Python gözlemci** PASS, format/check/build/Clippy PASS (V3 aşaması). Bunlar kullanıcıdaki native testle karıştırılmamalıdır.

**Karar: PIV-09 DONE.** İki decoder'ın eşzamanlı gerçek çalışma aralığı `NOT MEASURED`; test bunun yerine iki bağımsız yükleme ve aktif yüklemede iptali kanıtlar. Faz 0 temiz native kurulumun bağımsız tekrarı, DESKTOP-06 gerçek Nautilus testi ve diğer bağımsız kapılar **açık** kalır. PIV-10 **TODO / başlamadı**. Üretim/sandbox politikası değiştirilmedi, yalnız sonuçlar belgelendi. Bu kayıt commit'inin SHA'sı Git geçmişinden doğrulanır; burada önceden tahmin edilmez.


## PIV-10 — önceki/sonraki medya — 22.10.2026 — TESTING

Başlangıçta yerel `c3e0aaf` temiz dev idi. Origin fetch refspec'i yalnız main'i
kapsadığından dev açık ref ile fetch edildi; `origin/dev`'e fast-forward yapıldı:
**başlangıç SHA `c11c7b79bf0dce97516c3a1f8505c5f020e9193e`**. Kullanıcının
PIV-09 DONE kayıtları `8991b78`/`c11c7b7` kaynakları ve yukarıdaki native kanıt
aynen korundu. `main` korunacak SHA `5d55ee599df4617bd51c09b1c7a008b8d0ece480`.
Yeni kurulum/reset yapılmadı; mevcut Rust/GTK/Glycin SDK/Xvfb/Sway/Weston kullanıldı.

### Uygulanan davranış

- MediaCollection::select_previous/select_next sınırlıdır; ilk/sonda false ve
  seçim aynı kalır. Boş/tek öğe güvenli no-op; doğal sıralama ve başlangıçta açılan
  alias seçimi korunur. Karışık video/fotoğraf listesi filtrelenmez/yeniden sıralanmaz.
- Her pencerenin ayrı context/kataloğu, previous-media/next-media action'ları ve
  capture-phase Left/Right handler'ı vardır. Ctrl/Alt/Super okları tüketilmez.
  Başlık seçilen adla ve mevcut aday/uyarı suffix'iyle güncellenir. Yeni toolbar yok.
- Tek ImageView yeniden kullanılır. Her yeni seçim önceki GIO Cancellable'ını iptal
  eder, paintable'ı temizler ve ScaleDown Fit'i yeniler. Completion GTK context'inde
  yalnız güncel ve iptal edilmemiş token'la kabul edilir; daha önce bitmiş ama GTK
  kuyruğundaki eski sonuç da yeni seçim/video/hata durumunu ezemez. Close/destroy
  handler'ları her tuşta yeniden bağlanmaz. PIV-09 probe build/observer girişleri korunur.
- JPEG/PNG aynı zorunlu Bwrap image_loader yolundadır. Video atlanmaz; açık
  playback-unavailable durumu gösterilir, sonraki fotoğrafa geçilir. Diğer format
  adaylarında JPEG/PNG kapsamı açık mesajdır. Video motoru veya video default'u yok.
- Silinmiş/bozuk görüntü hatası gezinmeyi kilitlemez. Katalog snapshot kalır;
  silinmiş öğe kontrollü hata verir, file watcher/refresh eklenmez. Kaynak dosyalar
  değişmez; testteki silme yalnız private temp kopyadadır. Sarma/zoom/Edit/CI yok.

### Gerçek Codex kontrolleri

| Kontrol | Gerçek sonuç ve sınır |
|---|---|
| cargo fmt --check | PASS |
| cargo check --locked / cargo build --locked | PASS; stable/offline/-j2 helper |
| cargo test --locked | **54 PASS = 30 unit + 24 integration**, 0 failed/ignored; 0 doctest |
| cargo clippy --locked --all-targets -- -D warnings | PASS |
| Eski Python observer regresyonları | **30 PASS**; native decode sayılmaz |
| Yeni katalog unit (3) | Boş/tek, sınırlar, opened seçim, doğal 1/2/10 fotoğraf-video, 1000 hızlı alternatif, iki katalog bağımsızlığı PASS |
| Yeni LoadSlot unit (2) | Pending veya bitmiş eski callback kimliği, replacement/video/close iptali, 1000 replacement ve iki pencere token bağımsızlığı PASS |
| Fixture bütünlüğü / CLI / desktop / katalog / doğal sıra | PASS; core testler ve iki fixture manifesti |
| Yeni gerçek X11 tuş smoke'u | PASS: Tab sonrası Left/Right, doğal karma sıra, sınırda durma, Ctrl filtreleme, hızlı alternatifler, iki launch bağımsızlığı ve normal WM close |
| Yeni üretim-window navigation_probe Xvfb | Scoped PASS, gerçek mapped GDK yüzey ve GTK controller/key sinyalleri; explicit --expect-image-error, fotoğraf pikselleri BLOCKED |
| Yeni navigation_probe Sway | Scoped PASS: native Wayland/DISPLAY yok/XWayland kapalı; controller sinyali, compositor fiziksel key enjeksiyonu değildir |
| Yeni navigation_race_probe Xvfb | PASS yalnız kontrollü Err-worker GTK/GIO yarışları: ikinci sonuç önce, eski sonuç sonra; video geçişi, pending close/join/weak-ref ve diğer pencere durumu korunur |
| Eski X11 boş/file/desktop/discovery | 4 scoped PASS; dosyalı modlarda gerçek loader hatası beklendi |
| Eski Sway boş/file | 2 scoped PASS; mapped native surface ve normal close/exit0 |
| Eski Weston boş/file | 2 scoped PASS; mapped xdg_toplevel/SHM/configure/attach/commit. Normal close NOT RUN |
| Yeni navigation_probe strict, Xvfb | **BLOCKED exit77**: ilk gerçek JPEG loader başlamadı. Yeni başarılı piksel/Fit/late-success-frame yolu NOT RUN |
| Rust examples build / Python AST / shell syntax / git diff --check | PASS |
| Cargo/lock, üretim decoder, CLI/input, discovery/natural, data ve fixture diff | Başlangıçla aynı; kaynak medya ve desktop varsayılanları değiştirilmedi |

**12 başarılı scoped GUI/helper çağrısı** = 8 eski + 4 yeni. Piksel PASS toplamı
değildir; strict native gate bu sayıya dahil edilmez. Mock/hata worker yalnız
ayrı test example'ındadır; üretimde decoder/fallback değildir, hiç frame/piksel
üretmez. Gerçek Bwrap izolasyonu kapatılmadı. Bozuk fixture'da Codex'in gözlediği
hata secure-loader başlangıcıdır; gerçek corrupt decode recovery PASS sayılmaz.
Silinmiş private dosyanın gerçek metadata NotFound hatası ise ayrıca doğrulandı.
İlk gerçek native baseline geçtikten sonraki geçerli resim hatası FAIL'dir;
yeni probe bunu ortam skip'i/exit77 diye gizlemez.

PIV-09'un **kullanıcı Arch** pixel/EXIF/Fit/active-close DONE kanıtı tarihiyle
korunur; bu devamda Codex'te gerçek görüntü işlendiği anlamına gelmez. PIV-10
native render ve fiziksel GNOME/focus sonuçları henüz yok.

### Yalnız kalan PIV-10 kabul kanıtları

Güncel dev checkout'unda tek otomatik native komut:
`bash tests/run_navigation_probe.sh`.
HEAD'i yazar, locked matching example'ı derler ve gerçek GDK_BACKEND=wayland
çalıştırır. Native doğrulamada --expect-image-error kullanılmaz. Private iki
pencerede gerçek JPEG/PNG boyut/RGB/alpha, actual-view snapshot Fit/center,
rapid sonrası son frame, video/hata geçişleri ve bağımsız close doğrulanmalıdır.

Ardından normal Pivra'da fiziksel Left/Right'ı fotoğraf alanı ve başlık düğmeleri
arasında mouse/Tab ile odak değiştirerek doğrulamak gerekir; video mesajından
fotoğrafa geçiş ve ikinci pencerenin bağımsızlığı görülmelidir. Bu fiziksel
GNOME/Wayland kontrolü Codex'te yapılmış değildir. [Tam komut/matris](PIV10_VALIDATION.md).

**PIV-10 TESTING**, DONE değil. PIV-11 ve sonrası başlamadı. Faz 0 temiz native
kurulum INCOMPLETE, DESKTOP-06 gerçek Nautilus NOT RUN ve PIV-05 video bağımlılık
riskleri kendi kapsamlarında açık kalır. Toplam DONE görev sayacı PIV-09 dahil 9'dur;
PIV-10 bu sayaca eklenmedi.

| Gerçek commit SHA | İçerik |
|---|---|
| `028b74720e5690d5f2509044895f18a8b6639b73` | GUI bağımsız bounded previous/next model ve 3 katalog regresyonu |
| `b69ba2c0ff13da35d6f464460669073b4a600130` | Window-local actions/keys/başlık, reusable cancellable view ve 2 slot regresyonu |
| `8403d125db65f1997c7f9872b6e68bb11fe3954c` | Gerçek X11 key, Sway/GTK gezinme, deterministik callback yarışları ve native pixel probe |
| `80e501dbf2500f342863f309492770c814dcb986` | Gerçek baseline sonrası valid-image hatasını FAIL sınıflandıran strict native gate |
| `9fd4b3d59df858a57a7fbbcd8df6c87f2fc0e965` | README/teknik/yol haritası ve ayrıntılı test matrisi; PIV-09 DONE/PIV-10 TESTING ayrımı |

Bu takip commit'inin SHA'sı Git geçmişi ve push raporundadır. Normal origin/dev
push sonrası remote eşitliği ve main korunması doğrulanır; force/rebase yoktur.


## PIV-10 Arch native kabul ve kapanış — 22.10.2026 — DONE

**Kaynak:** Kullanıcının Arch Linux / GNOME Wayland terminal çıktısı ve manuel gözlemi; Codex testi değildir. **Test commit'i**: `c53d4071368b2b45ceb8ecba06f949f4c5188b52`. Komut: `bash tests/run_navigation_probe.sh`; **exit0** ve gerçek secure-navigation pixel/Fit/center PASS. Otomatik native probe: natural mixed order, sınırlar, hızlı ardışık oklar, hata sonrası devam, bağımsız pencereler, normal close ve kaynak dosya değişmezliği PASS. Bozuk PNG ve silinmiş dosyanın konsoldaki beklenen hataları kontrollü kurtarma kapsamındaydı.

Kullanıcı gerçek fotoğraflarla normal Pivra'da **fiziksel sol/sağ oklarla fotoğraflar arasında geçişin çalıştığını** doğruladı. Her medya değişiminde görünen `Loading image securely` yükleme tamamlandığında kayboluyor; bu beklenen geçici durum. Arayüzde gezinme düğmeleri bulunmuyor; PIV-10 bunları kapsamamıştı. Odaklanabilir başlık düğmesi olmadığından Tab ile odak aktarımı fiilen uygulanamadı; **Tab sonrası focus PASS iddiası yoktur**. Video üzerinden fiziksel gezinme ayrıca kullanıcı tarafından raporlanmadı; otomatik native probe video mesajı/karışık sıralamayı doğruladı. Bu ayrımlar saklanır.

Codex kaynaklı önceki kısa testler **54 Rust + 30 Python observer PASS**, 12 scoped GUI/helper PASS. **PIV-10 DONE**: mevcut kabul kapsamındaki klavye gezinmesi, gerçek secure render ve kontrollü durum akışı doğrulandı. Faz 0 bağımsız temiz kurulum **INCOMPLETE**, DESKTOP-06 Nautilus **NOT RUN**, PIV-11 **TODO**. Yeni production değişikliği yapılmadı; belge güncellemesinin commit SHA'sı Git geçmişindedir.


## PIV-11 — 22.10.2026 — TESTING

Başlangıç/origin dev: `b83aa93e7b6d754c4cc2f55e7f6ff584046e49b6`.
PIV-01–10 DONE korunur; PIV-10 native kabul kullanıcı tarafından bildirilmiştir.
Mevcut Rust/GTK/loader/Xvfb/Sway/Weston ortamı korundu, kurulum veya üretim
bağımlılığı değişmedi. Yalnız `dev` kullanıldı.

| Gerçek commit SHA | İçerik |
|---|---|
| `078d0283a0e87af7e59489c51a44a9ec2c121429` | GIO klasör/symlink hedef izleme, 120 ms coalescing, tek worker, seçim/rename/boş durum, açık Yenile ve güncel callback kontrolü; 3 katalog regresyon testi |
| `5dd8f14d806dbabdbe046ed586eea94167b9a1a1` | Gerçek üretim pencereli folder watch probe ve tek native giriş; PIV-10 probunun canlı silinme davranışına uyarlanması |

**Codex'te çalıştırıldı:** `tests/check.sh` tamamen PASS: fmt, locked/offline check,
build, test (**57 Rust**, 33 unit + 24 integration, 0 failed/ignored), clippy
`--all-targets -- -D warnings`; **30 Python observer**, model fixture manifestleri
ve 18 görüntü hash'i PASS. Tüm örnekler locked/offline derlendi. Cargo.toml ve
Cargo.lock değişmedi. CLI/katalog/doğal sıra/desktop regresyonları korunur.

**14 scoped GUI/helper çağrısı PASS:** X11 blank/files/desktop/discovery,
gerçek xdotool gezinmesi, controlled race, navigation probe ve folder watch
probe (8); headless Sway blank/files/navigation/folder watch (4); Weston
blank/files (2). Yeni folder watch X11/Sway çağrıları gerçek GIO olaylarıyla
çalıştı; `--expect-image-error` yalnız kontrollü decoder hatası kapsamıdır.
Doğrudan yazma/atomic save bildirimi, seçili olmayan değişiklik, doğal mixed
sıra, 50 olaylık burst + hızlı gezinme, rename/silinme/boş katalogdan toparlanma,
başka klasördeki symlink hedefi değişikliği/kaybı, klasör kaybı, bağımsız pencere
ve kapanış sonrası weak pencere serbest bırakılması doğrulandı.

**NOT RUN:** PIV-11 gerçek secure pixel/refresh gate ve PIV-09 image_gui_probe
Codex'te exit77. Read-only `/proc` UID-map/Bwrap engeli sürer; güvenlik gevşetilmedi.
Eski kullanıcı PIV-09/10 piksel kanıtları yeni PIV-11 için PASS sayılmadı. Sway
probe GTK action/sinyalidir, fiziksel GNOME girişi değildir. Weston normal
kapanış NOT RUN kalır. Controlled race mock'u gerçek decoder testi değildir.

**Kalan kabul:** Arch/GNOME Wayland'de
`bash tests/run_folder_watch_probe.sh` ve yalnız ekle/sil/rename gezinmesi,
bildirim, Yenile sonrası yeni görüntü, kullanıcı eyleminden önce görüntünün
korunması için görsel kontrol. Varsayılan native probda ilk secure baseline
engeli exit77; sonrasındaki hatalar FAIL. [Ayrıntılı test matrisi ve sınırlar](PIV11_VALIDATION.md).

**PIV-11 TESTING; DONE değil. PIV-12 başlamadı.** Faz 0 temiz native kurulum
INCOMPLETE ve DESKTOP-06 gerçek Nautilus NOT RUN bağımsız kalır. Sayaçta PIV-10'un
önceden DONE olmasına rağmen güncellenmemiş faz/toplam sayısı 5/10 olarak düzeltildi;
PIV-11 sayaca eklenmedi. Main beklenen SHA:
`5d55ee599df4617bd51c09b1c7a008b8d0ece480`. Normal push ve uzak SHA kontrolü teslim
sonunda ayrıca doğrulanır; bu belge kendi commit SHA'sını önceden tahmin etmez.

### PIV-11 belge ve uzak dal doğrulaması

Belge/test matrisi commit'i:
`4340c448bf1e4d5583248c3a476dc285930d0675`.
Normal push başarılı: `b83aa93..4340c44 dev -> dev`. Açık refspec ile fetch sonrası
`HEAD = origin/dev = 4340c448bf1e4d5583248c3a476dc285930d0675` ve çalışma dizini
temiz doğrulandı. `ls-remote` main'in
`5d55ee599df4617bd51c09b1c7a008b8d0ece480` kaldığını doğruladı. Bu son kanıt kaydı
ayrı belge commit'idir; kendi SHA'sı Git geçmişinde bulunur. PIV-11 TESTING kalır.


## PIV-11 final native acceptance — DONE (2026-10-22)

At `f96a0db029fa0e5157318d4bdfd4ef494d3baf33`, user ran `bash tests/run_folder_watch_probe.sh`: exit0/PASS, real filesystem events and secure Glycin refresh, previous frame retained until explicit user refresh action. In normal GNOME/Wayland application, user observed the refresh banner and confirmed clicking Yenile displays the modified image after a loading animation. Previous new-window impression was not confirmed by terminal logs (two separate cargo run commands); actual GtkWindow object identity not manually instrumented. Native acceptance met; PIV-11 DONE, PIV-12 TODO. The mixed Turkish/English UI strings are scheduled for PIV-39 i18n; Phase 0 clean install INCOMPLETE and DESKTOP-06 Nautilus NOT RUN remain independent.


## PIV-12 — 22.10.2026 — TESTING

Başlangıç: `1ceaa04e202e94a320d2dd3f632f48aa1554a7bd`, güncel origin/dev'e
fast-forward. PIV-01–11 DONE ve kullanıcı kaynaklı native kanıtlar korunur.

| Gerçek SHA | Değişiklik |
|---|---|
| `7b48e9492c9ad063b831836fb1c7c6afcfa9215a` | Unsupported/video Yenile decoder çağrısını engelleyen minimal düzeltme; mevcut üretim probuna medya hataları, sağlam resme dönüş, iki pencere/bildirim ve gerçek GTK button clicked nesne-kimliği kontrolleri; native giriş |
| `2b76543c969cb3b96565a958c9ee5e6220416e65` | ImageView A/B/C, bağımsız bekleyen worker ve stale result notification testleri; üretim pencere + gerçek monitor ile kontrollü worker yarış probu; Sway senaryo argümanı |

**PASS — Codex:** fmt/check/build/test/clippy (`--locked`, helper stable/offline/-j2,
clippy `--all-targets -- -D warnings`); **57 Rust** (33 unit + 24 integration,
0 failed/ignored), **30 Python observer**, fixture/18 image hash'i ve tüm
örneklerin locked/offline derlenmesi. CLI/doğal sıra/katalog/desktop regresyonları
korundu. Yeni Rust unit sayısı eklenmedi; eksikler GTK/GIO probe düzeyindedir.

**PASS — 18 scoped GUI/helper çağrısı:** X11 10, Sway 6, Weston 2. Eski PIV-09–11
pencere/gezinti/folder/erişim testleri, genişletilmiş kontrollü ImageView yarışı,
yeni medya hatası (X11/Sway) ve yeni window/monitor yarışı (X11/Sway). Gerçek
GIO olayları ve GTK nesneleri kullanıldı. Yeni native senaryo bozuk JPEG/PNG,
yarım/boş/yanlış içerik, unsupported/video Yenile, hatadan gezinme, hatalı seçimde
katalog değişimi, tekrarlanan Yenile, silinme, bağımsız frame/notice ve close
kaynak temizliğini kapsar. Kontrollü worker'da dış değişiklik otomatik load
başlatmadı; tam 7 başlangıç gözlemi ve gecikmiş callback guard'ları doğrulandı.

**BLOCKED/NOT RUN — 7 exit77:** PNG/JPEG-first image_probe, image_gui_probe,
strict navigation, strict folder watch, strict yeni medya hatası probu ve
active_decode_close. İlk secure baseline yüklenemedi; bilinen Codex Bwrap /
read-only proc kısıtı sürer. Aktif test raporu baseline_decoded aşamasında kaldı,
loader not_certified ve decoder_time_overlap NOT MEASURED. Gerçek decoder
hatası sınıflandırması/sağlam piksele dönüş/aktif secure iptal PASS sayılmadı.
Mock/kontrollü hata sonuçları gerçek decoder kanıtı değildir. Sway probe
sinyalleri fiziksel GNOME giriş testi değildir; Weston normal close NOT RUN.

Kalan native komut: `bash tests/run_media_failure_probe.sh`. Sonrasında gerçek
GNOME'da bozuk medyadan sağlam görüntüye gezinme ve bir penceredeki
hata/Yenile/kapanışın diğerine etkisizliği kullanıcı tarafından doğrulanmalı.
Aktif secure decode sırasında silme/kapanış zamanlaması yeni medya probunda
ölçülmez; deterministik worker kanıtı ayrı tutulur. [Tam matris](PIV12_VALIDATION.md).

**PIV-12 TESTING; native kabul olmadan DONE değil. PIV-13 başlamadı.** PIV-39
tr/en i18n kararı, Faz 0 temiz native kurulum INCOMPLETE ve DESKTOP-06 Nautilus
NOT RUN korunur. Sayaç PIV-11'in mevcut DONE kaydına göre 6/11'e düzeltildi;
PIV-12 eklenmedi. Cargo.toml/lock, kaynak fixture'lar, decoder güvenliği,
kurulum, CI ve lisans değişmedi. Main beklenen SHA
`5d55ee599df4617bd51c09b1c7a008b8d0ece480`; normal push sonunda ayrıca doğrulanır.

### PIV-12 belge ve push kanıtı

Belge/matris commit'i `2b7e5751a041ea3f3a628aed1e8defcba524ff53`.
Normal push `1ceaa04..2b7e575 dev -> dev` başarılı; açık dev refspec'iyle fetch
sonrasında HEAD/origin/dev aynı SHA ve çalışma dizini temiz doğrulandı.
`ls-remote` main'i değişmemiş
`5d55ee599df4617bd51c09b1c7a008b8d0ece480` olarak gösterdi. Başlangıca karşı
Cargo.toml, Cargo.lock, image_loader.rs, image_view.rs ve fixture farkı yok.
Bu son kayıt kendi SHA'sını içermez; ayrı belge commit'i Git geçmişindedir.
PIV-12 TESTING, native kabul bekleniyor.


## PIV-12 native ve manuel kabul kapanışı — 24.10.2026 — DONE

Kullanıcı `b12970f6a3a7ea2dddc890a747977d5d68437a63` üzerinde `bash tests/run_media_failure_probe.sh` çalıştırdı: **exit0**, gerçek secure Glycin, bozuk/yarım/boş/yanlış format JPEG/PNG hataları, sağlam kareye geri dönüş, katalog/seçim/monitor/bildirim izolasyonu, Yenile ve close cleanup PASS. Bozuk dosya hata logları beklenen test senaryolarıydı. Gerçek GNOME/Wayland manuel testinde kullanıcı iki Pivra penceresi açtı; birincide kırmızı→yeşil→bozuk→mavi gezinmesini, diğer pencerenin etkilenmemesini ve birinci kapatılınca ikincinin çalışmaya devam etmesini **PASS** bildirdi. Aktif gerçek secure decode sırasında silme/kapatma ve decoder süreçlerinin zamansal çakışması **NOT MEASURED**; bu ölçümler PASS sayılmaz. PIV-12 DONE; PIV-13 TODO. Faz 0 bağımsız temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN, PIV-39 tr/en i18n planı açık.


## PIV-13 — 24.10.2026 — TESTING

Başlangıç: `f9d7157cb4da22bc763b7fc2bf4f0607923f4df8`, güncel origin/dev'e
fast-forward. PIV-01–12 DONE ve kullanıcı kaynaklı Arch/GNOME kanıtları korunur.

| Gerçek SHA | Değişiklik |
|---|---|
| `b576c35f30e0fb42b1e971c8162afe9fa7f37873` | Test edilebilir Fit/Fill/%100 geometri modeli; kaynak mantıksal boyutunda clip paintable; ImageView texture/mod state'i; pencere menüsü ve F6/F7/F8 |
| `5ad7b98bf0f020c8fa7bb21c54165a2148b662f7` | Render/native test probu, tek native giriş; mevcut worker/window yarışlarında mod reset/izolasyon/menü etiketi/tuş testleri; Sway sentetik argümanı |

**PASS — Codex:** fmt, locked check/build/test ve clippy all-targets -D warnings
(mevcut stable/offline/-j2 helper); **59 Rust** = 35 unit + 24 integration,
0 failed/ignored; **30 Python observer**, eski fixture ve 18 image hash'i; tüm
örneklerin locked/offline derlemesi, shell/Python syntax, diff whitespace.

**PASS — 21 scoped GUI/helper:** X11 12, Sway 7, Weston 2. PIV-09–12'nin 18
mevcut pencere/gezinti/folder/Yenile/hata/controlled race çağrısı korunur.
Yeni üç sentetik mode render çağrısı Xvfb scale1, Xvfb GDK_SCALE=2 ve headless
Sway: her birinde 48, toplam **144 snapshot**. Gerçek üretim sunumu, gerçek
atanmış widget boyutu, alpha sınırı, kaynak degrade renk örnekleri, merkez,
en-boy oranı ve clip doğrulandı. **Sentetik texture decoder PASS'ı değildir.**

Mod değişiminde worker sayısının artmaması, F6/F7/F8 capture sinyalleri, menü
etiketi, iki pencerenin farklı modu, yeni yüklemede Fit, gecikmiş callback ve
mode/close kontrollü GTK/GIO worker testlerinde doğrulandı. Gerçek decoder
etkinliği/overlap bu testlerde ölçülmez. Sol/Sağ gerçek X11 tuş regresyonu PASS.
Sway sinyalleri fiziksel GNOME girişi değildir; Weston normal close NOT RUN.

**BLOCKED/NOT RUN — 8 exit77:** image_probe PNG/JPEG-first, image_gui_probe,
strict navigation, strict folder watch, strict PIV-12 medya hatası,
active_decode_close ve yeni strict view_modes_probe. İlk secure baseline
oluşmadı; bilinen Codex Bwrap/read-only proc kısıtı sürer. Yeni native
EXIF/mod/render/two-texture-window pozitif yolu yalnız derlendi, PASS değil.
Bwrap zorunluluğu korunur; loader/sandbox bypass veya decoder fallback yoktur.

**Kalan native kabul:** `bash tests/run_view_modes_probe.sh` önce mevcut gerçek
JPEG/PNG/EXIF piksel probunu, sonra yeni mod/resize/EXIF geometri ve bağımsız
iki gerçek frame penceresi probunu çalıştırır. Fiziksel GNOME menü/F6–F8,
resize, yeni medyada Fit ve pencere bağımsızlığı ayrıca kullanıcıca doğrulanmalı.
%100 tanımı **1 kaynak piksel = 1 GTK mantıksal birimi**, HiDPI fiziksel 1:1
değildir; gerçek HiDPI monitör testi NOT RUN. [Tam matris](PIV13_VALIDATION.md).

**PIV-13 TESTING, DONE değil. PIV-14 başlamadı.** Önceki PIV-12 DONE kaydıyla
uyumsuz kalan faz/toplam sayaçları 7/12 olarak düzeltildi; PIV-13 sayaca eklenmedi.
Faz 0 bağımsız temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve
PIV-39 tr/en i18n kararı korunur. Kurulum, Cargo.toml/lock, image_loader,
kaynak fixture'lar, CI ve lisans değişmedi. Main beklenen SHA
`5d55ee599df4617bd51c09b1c7a008b8d0ece480`; normal push sonunda kontrol edilir.

### PIV-13 belge ve uzak dal kanıtı

Belge/matris commit'i `2644a433d0b0471d4778fb3509706342e2f69d68`.
Normal push `f9d7157..2644a43 dev -> dev` başarılı; açık dev refspec'iyle fetch
sonrasında HEAD/origin/dev aynı SHA olarak doğrulandı. `ls-remote` main'in
`5d55ee599df4617bd51c09b1c7a008b8d0ece480` kaldığını gösterdi. Başlangıca karşı
Cargo.toml, Cargo.lock, image_loader.rs ve fixture farkı yok. Bu son kanıt
kaydı ayrı belge commit'idir; kendi SHA'sı Git geçmişindedir. PIV-13 TESTING.


## PIV-13 native kabul ve kapanış — 24.10.2026 — DONE

Kullanıcı Arch Linux / GNOME Wayland üzerinde `d60c33078c08720524413d204f7131b34df2ff5b` ile `bash tests/run_view_modes_probe.sh` çalıştırdı: **exit0/PASS**; gerçek Glycin JPEG/PNG/EXIF 1–8 decode ve Fit/Fill/Actual render geometrisi, kırpma, merkez, resize, iki bağımsız pencere, yeni medyada Fit sıfırlama, bozuk medyadan toparlanma ve kaynak bütünlüğü doğrulandı. Kullanıcı normal GNOME uygulamasında modları manuel kontrol ettiğini bildirdi. %100 modunda büyük fotoğrafın sadece kesitinin görünmesini ve fareyle sürükleyerek gezinemediğini gözledi; ilki Actual davranışı, ikincisi henüz uygulanmamış **PIV-14 pan** kapsamıdır, PIV-13 hatası değildir. Gerçek HiDPI fiziksel 1:1 ölçek testi NOT RUN; Actual = bir kaynak piksel / GTK mantıksal birimi. Önceki Codex ortamı exit77 sonuçları geriye dönük PASS değildir. **PIV-13 DONE; PIV-14 TODO**. Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN, PIV-39 i18n planı açık.


## PIV-14 — 24.10.2026 — TESTING

Başlangıç `8b31944dacef310ef3df3b617a241c6f27d005ba`, güncel origin/dev'e
fast-forward. PIV-01–13 DONE ve kullanıcı kaynaklı native kabul korunur.

| Gerçek SHA | Değişiklik |
|---|---|
| `0bacddbd8cd4fd2e266ed63c1886e33f35468408` | İmleç ankrajlı 0.1–32 manuel ölçek, bounded pan/resize; ayrı preset/manual geometri, reset nesli, Picture-only controller'lar, cached paintable ve Zoom göstergesi |
| `e1d977124e939897b129e759293df866682a0de3` | GTK scroll/drag/cancel ve degrade snapshot, iki pencere/resize/aktif drag-close, pending-input regresyonu; tek native giriş ve Sway argümanı |

**PASS — Codex:** fmt, locked check/build/test ve all-targets clippy -D warnings
(mevcut stable/offline/-j2 helper); **63 Rust** = 39 unit + 24 integration,
0 failed/ignored; **30 Python observer**, eski fixture/18 image hash'i; tüm
örneklerin locked/offline derlenmesi, shell/Python syntax ve whitespace.

**PASS — 24 scoped GUI/helper:** X11 14, Sway 8, Weston 2. Önceki PIV-09–13'ün
21 çağrısı korunur. Yeni zoom/pan sentetik probu Xvfb scale1/scale2 ve Sway'de
72'şer snapshot (**216**); eski mode probunda toplam144: birlikte **360**.
Gerçek üretim controller/paintable/GSK ile kaynak renkleri, clip, ankraj,
yakınlaştırma/uzaklaştırma, dört köşe, %10/%3200 limitleri, her preset başına
1000 drag update'i, iki yönde 100 scroll, resize, stale gesture/reset/cancel,
aktif drag-close ve bağımsız ikinci pencere doğrulandı. Texture ve paintable
kimliği input boyunca korunur. **Sentetik texture decoder PASS'ı değildir.**
Modelde dört yeni unit, pending worker'da girişlerin texture/ek worker üretmemesi,
F6/F7/F8/Sol/Sağ/Yenile/monitor/hata/weak cleanup regresyonları PASS. Kontrollü
workerlar gerçek Glycin değildir. Sway controller sinyalleri fiziksel GNOME
mouse girişi değildir; Weston normal close NOT RUN kalır.

**BLOCKED/NOT RUN — 9 exit77:** PNG/JPEG-first image_probe, image_gui_probe,
strict navigation, strict folder watch, strict medya hatası, active_decode_close,
strict view_modes ve yeni strict view_modes --zoom-pan. İlk secure baseline
oluşmadı; bilinen Codex Bwrap/read-only proc kısıtı sürer. Native pozitif
zoom/pan/EXIF yolu yalnız derlendi, PASS değildir; decoder overlap ölçülmedi.
Sandbox gevşetilmedi, NotSandboxed/bypass/fallback eklenmedi.

Kalan tek native komut **`bash tests/run_zoom_pan_probe.sh`**. Önce gerçek
JPEG/PNG/EXIF piksel probu, ardından üretim zoom/pan sinyalleri ve önceki gerçek
mode/render/resize/two-window kaynak bütünlüğü kapısı çalışır. Fiziksel mouse
ankrajı, dört köşe, sınırlar, resize, preset reset, menü/Yenile ve pencere
izolasyonu kullanıcıca test edilen SHA ile bildirilmeli.
[Tam matris ve preset sınır davranışı](PIV14_VALIDATION.md).

**PIV-14 TESTING; native kabul olmadan DONE değil. PIV-15 başlamadı.** PIV-13'ün
mevcut DONE kaydına göre faz2/toplam sayacı 1/13'e düzeltildi; PIV-14 eklenmedi.
%100 kaynak piksel / GTK mantıksal birimi anlamında kalır, fiziksel HiDPI 1:1
iddiası yoktur. Fiziksel HiDPI ve input FPS/latency ölçümü NOT RUN. Faz 0 temiz
native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 tr/en planı
korunur. Kurulum/Cargo.toml/lock/loader/kaynak fixture/CI/lisans değişmedi.
Main beklenen SHA `5d55ee599df4617bd51c09b1c7a008b8d0ece480`; push sonunda kontrol edilir.

### PIV-14 belge ve uzak dal kanıtı

Belge/matris commit'i `e739a09418fd9e6aa30fc5de0dd3f5e174d4e33f`.
Normal push `8b31944..e739a09 dev -> dev` başarılı; açık dev refspec'iyle fetch
sonrasında HEAD/origin/dev aynı SHA doğrulandı. `ls-remote` main'i değişmemiş
`5d55ee599df4617bd51c09b1c7a008b8d0ece480` gösterdi. Başlangıca karşı
Cargo.toml, Cargo.lock, image_loader.rs ve fixture farkı yok. Bu son kayıt
ayrı belge commit'idir; kendi SHA'sı Git geçmişindedir. PIV-14 TESTING.


## PIV-14 Arch native ve gerçek fare kabulü — 24.10.2026 — DONE

Kullanıcı `6c55eab27d4c5bcf980246c2c69fb08671d2d2be` üzerinde `bash tests/run_zoom_pan_probe.sh` çalıştırdı: **exit 0**. Gerçek Glycin JPEG/PNG/EXIF baseline ve üretim render/mod/geometri, iki pencere izolasyonu testleri PASS; betiğin `view_modes_probe --zoom-pan` yolunu çağırdığı doğrulandı. Bozuk PNG testindeki decoder hata satırı beklenendi. Kullanıcı ayrıca fiziksel GNOME/Wayland fare testi için **olumlu sonuç** bildirdi: %100 görüntü üzerinde sürükleme, imleç ankrajlı yakınlaştırma, Fit sıfırlama ve resize akışı için bir problem bildirilmedi. Ayrı ölçülmeyen HiDPI fiziksel piksel eşlemesi, gerçek decoder süreç zaman çakışması ve FPS/gecikme ölçümleri PASS sayılmaz. **PIV-14 DONE; PIV-15 TODO.** Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 i18n kararı açık kalır.


## PIV-15 — 24.10.2026 — TESTING

Başlangıç `28505e4d377759f1550b20d8b199841ce6aa80dd`, origin/dev fast-forward.
PIV-01–14 DONE ve kullanıcı kaynaklı native kabul korunur.

| Gerçek SHA | Değişiklik |
|---|---|
| `5fec698673ea331eb6bf43922381349e26b2cd09` | Smooth/Nearest pencere state/action/modeli, aynı texture ve geometry üzerinde gerçek GSK filtresi, mevcut menü bölümü, zoom helper imza uyarlaması |
| `fd36f560203f7de6830ef3c3a256c33df53a91f9` | CPU referans + gerçek GSK piksel/geometry/lifetime testleri; pending tercih/late callback, exact-color PNG/manifest/üretici, native giriş |

**PASS — Codex:** fmt, locked check/build/test ve clippy all-targets -D warnings
(mevcut stable/offline/-j2 helper). **64 Rust** = 40 unit + 24 integration,
0 failed/ignored; **30 Python observer**; önceki fixture hash'leri ve yeni
resampling-checker deterministik byte/manifest kontrolü; tüm örneklerin
locked/offline derlemesi, shell/Python syntax ve diff whitespace.

**PASS — 27 scoped GUI/helper:** X11 16, Sway 9, Weston 2. Eski PIV-09–14'ün
24 çağrısı korunur. Yeni üç resampling çağrısında 34'er (**102**) snapshot;
eski360 ile toplam **462 sentetik snapshot**. Kesin 2×2 kaynağın 2/4/8×,
kesirli, %10/%3200, Fit/Fill/Actual çizimi; Nearest opaque palette/spatial
örnekler ve 16×16 kaynakta deterministik0.5× küçültme; Smooth encoded-channel
Cairo bilinear referansına 5/255 tolerans ve gerçek ara değerler doğrulandı.
Bunlar sentetik source ile **gerçek üretim GSK rendering** testidir; gerçek
Glycin başarısı veya hardware GPU/physical mouse kabulü değildir.

Gradient photo örneğinde filtre sırasında rendered source koordinatları,
zoom/pan/clip ve nearest roundtrip byte eşitliği korundu. 1000 hızlı filtre
geçişinde aynı manuel paintable ve ardından weak adapter release PASS.
Pending worker'da Nearest seçimi, yeni istekte ve late callback sonrası
korunması, diğer pencerenin Smooth kalması ve 7 worker sayısı/close regresyonu
PASS; controlled worker gerçek decoder değildir. Sway action/sinyalleri fiziksel
GNOME girişi değildir. Weston normal close NOT RUN kalır.

**BLOCKED/NOT RUN — 10 exit77:** PNG/JPEG-first image_probe, image_gui_probe,
strict navigation/folder watch/media failure, active_decode_close, strict
view_modes, strict zoom-pan ve yeni strict resampling. İlk secure baseline
oluşmadı; bilinen Codex Bwrap/read-only proc kısıtı sürer. Yeni native pozitif
PNG checker/palette, EXIF filtre, iki gerçek texture penceresi ve aktif drag
filtre roundtrip'i yalnız derlendi, PASS değil. Güvenlik bypass'ı eklenmedi.

GTK4.10+ append_scaled_texture / GSK Linear/Nearest mevcut GTK min4.16 içinde
kullanılır; bağımlılık/ABI hack'i yok. GSK uygulaması backend'e aittir; Cairo
renk/precision referansı GPU'ya genellenmez. %100 GTK mantıksal piksel tanımı
korunur, fiziksel HiDPI/FPS/RSS ölçümü NOT RUN/NOT MEASURED.

Kalan tek native komut **`bash tests/run_resampling_probe.sh`**. Secure JPEG/PNG/
EXIF baseline sonrası gerçek üretim render/filter ve exact-color PNG kapısı;
aktif drag'da filtre değişiminde aynı frame/konum, boş paintable olmaması,
yeni medyada tercih, bozuk medyadan dönüş, iki pencere ve kaynak bütünlüğü
kontrolleri hazırlanmıştır. Kullanıcı gerçek mouse/görsel kabulünü test SHA'sıyla
bildirmeli. [Tam matris ve backend sınırları](PIV15_VALIDATION.md).

**PIV-15 TESTING; native kabul olmadan DONE değil. PIV-16 başlamadı.** Mevcut
PIV-14 DONE kaydına göre faz2/toplam sayaç2/14'e düzeltildi; PIV-15 eklenmedi.
Tercih pencere içinde korunur, restart için persistence eklenmedi. Faz 0 temiz
native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 tr/en planı
korunur. Cargo.toml/lock, decoder/sandbox, eski medya fixture'ları, kurulum,
CI ve lisans değişmedi; manifest yalnız yeni özgün checker satırıyla genişledi.
Main beklenen SHA `5d55ee599df4617bd51c09b1c7a008b8d0ece480`; push sonunda kontrol edilir.

### PIV-15 belge ve uzak dal kanıtı

Belge/matris commit'i `470efbf4eb3e1ee3b29f45dade3cd4008aa161d5`.
Normal push `28505e4..470efbf dev -> dev` başarılı; açık dev refspec'iyle fetch
sonrasında HEAD/origin/dev aynı SHA doğrulandı. `ls-remote` main'i değişmemiş
`5d55ee599df4617bd51c09b1c7a008b8d0ece480` gösterdi. Başlangıca karşı Cargo.toml,
Cargo.lock, image_loader.rs, eski pixels-2x2/model-video ve images fixture'larında
fark yok; manifest yeni checker girdisiyle genişledi. Bu son kayıt ayrı belge
commit'idir; kendi SHA'sı Git geçmişindedir. PIV-15 TESTING.


## PIV-15 Arch native ve GNOME manuel kabul kapanışı — 24.10.2026 — DONE

Kullanıcı `3313113246baca8b2e19073b0ed20c64f569a50c` üzerinde `bash tests/run_resampling_probe.sh` çalıştırdı: **exit 0**, gerçek Glycin JPEG/PNG/EXIF 1–8, secure PNG checker Nearest palette ve Smooth interpolasyon, aynı decoded texture/paintable, yeni fotoğrafta tercih, iki pencere izolasyonu, kaynak bütünlüğü PASS. Beklenen bozuk PNG hata logları test verisindendi. Kullanıcı normal Pivra'da küçük checker görselinde Smooth/Nearest arasında görünür piksel farkını ve değiştirirken zoom/konumun korunduğunu **manuel PASS** olarak onayladı. Fill görüntüyü alanı dolduracak şekilde ölçeklediğinden bu davranış doğrudur; %100 hatası doğrulanmadı. Çok küçük test görsellerinde %3200 manuel zoom sınırı yetersiz hissedilebilir; mevcut PIV-14 sınırı korunur, artırma önerisi ayrı değerlendirmedir, PIV-15 kabulünü engellemez. Gerçek GPU/HiDPI eşlemesi, FPS/RSS ölçümü ve decoder overlap **NOT MEASURED**; Codex exit77 kayıtları geriye dönük PASS sayılmaz. **PIV-15 DONE, PIV-16 TODO.** Faz 0 bağımsız temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 i18n planı açık kalır.


## PIV-16 — 24.10.2026 — TESTING

Başlangıç `d9a127f482041a7f0a6994d9427de45f94c4784b`, origin/dev fast-forward.
PIV-01–15 DONE ve kullanıcı kaynaklı native kanıtlar korunur.

| Gerçek SHA | Değişiklik |
|---|---|
| `636498cb123cf92cc723f432251704c3b7566793` | Mevcut worker hattına 3+1 admission/5s deadline, token identity/stats, active-load dedup, close/cache cleanup, result observer reentrancy ve pending JPEG/PNG retarget; eski probelara alias/barrier uyarlaması |
| `d9c93ebc104bdda9ea6b7ee3171ef7707e3b0062` | Gerçek GTK/GIO controlled success/error, latest pixel/A-B-A/1000 pending/timeout/same-file isolation testleri; secure async yolu, PIV-09 aktif checkpoint dahil tek native giriş |

**PASS — Codex:** fmt, locked check/build/test, all-targets clippy -D warnings
(stable/offline/-j2 helper); **67 Rust** = 43 unit + 24 integration,
0 failed/ignored; **30 Python observer**, bütün fixture/manifest hash'leri,
tüm örneklerin locked/offline derlemesi, shell/Python syntax ve whitespace.

**PASS — 33 scoped GUI/helper:** X11 19, Sway 12, Weston 2. Eski 27 çağrı korunur.
Yeni controlled success/error ve 5s admission timeout X11/Sway; gerçek GIO
pending rename X11/Sway: altı çağrı. C önce/A-B sonra pikselleri, aynı-path
A/B/A farklı token, eski success için texture üretilmemesi, mode/filter/spinner
koruması, 1000 istekte running3/pending1 ve yalnız queued-999 admission, main
idle dispatch, reentrant yeni yükleme göstergesi, same-file iki pencere ve weak
cleanup doğrulandı. 14 admitted iş join, yalnız 6 current success texture;
ordinary same-active-path tekrarı worker başlatmadı. Timeout worker completion
sayılmadı; close pending'i başlatmadan bıraktı. **Synthetic success ve controlled
worker verisi gerçek decoder sonucu değildir.** Eski 462 sentetik snapshot ve
gezinti/Refresh/silinme/video/unsupported/monitor/zoom/pan/resampling korundu.

**BLOCKED/NOT RUN — 11 exit77:** PNG/JPEG-first image_probe, image_gui_probe,
strict navigation/folder watch/media failure, active_decode_close, strict
view_modes/zoom-pan/resampling ve yeni async_cancel_probe. İlk secure baseline
oluşmadı; bilinen Codex Bwrap/read-only proc kısıtı sürer. Gerçek decoder stop ve
child/sandbox cleanup bu ortamda PASS değildir. GIO token cancel gözlemi,
stale UI suppression, kontrollü job join ve gerçek process cleanup ayrı tutulur.
Glycin SDK cancel/drop request-path done gönderir; pool idle retention30s,
running0 PID exited demek değildir. PIV-09 gözlemci başarı kriteri/protokolü
korunur. PIV-09–15 eski kullanıcı native kanıtı yeni kod için PASS yapılmadı.

Kalan tek native komut **`bash tests/run_async_cancel_probe.sh`**: mevcut gerçek
JPEG/PNG/EXIF baseline, secure 101-istek async/son C pikselleri/A-B-A/iki pencere/
kaynak bütünlüğü ve sonra mevcut PIV-09 partial memfd/pidfd/SIGSTOP aktif kapanış
checkpoint'i/process temizliği. JSON `/tmp/pivra-async-active-close.json`;
tüm exit0 + result PASS olmadan birleşik native kapı PASS değildir. Async burst
içindeki tam decoder aktif aralığı/overlap ayrıca NOT MEASURED kalır.
[Tam yöntem/kalan manuel kabul](PIV16_VALIDATION.md).

**PIV-16 TESTING, native kabul olmadan DONE değil; PIV-17 başlamadı.** PIV-15'in
mevcut DONE kaydına göre faz2/toplam sayaç3/15'e düzeltildi; PIV-16 eklenmedi.
Main beklenen SHA `5d55ee599df4617bd51c09b1c7a008b8d0ece480` push sonunda
kontrol edilir. Rust/GTK kurulumu, Cargo.toml/lock, loader/sandbox uygulaması,
fixture byte'ları, CI/lisans değişmedi; loader'a yalnız eşdeğer LoadedFrame
alias'ı eklendi. X candidates arayüzü değiştirilmedi. Faz 0 temiz native kurulum
INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 tr/en planı korunur. Hardware
GPU/FPS/latency/RSS benchmark'ı NOT RUN/NOT MEASURED; Weston normal close NOT RUN.

### PIV-16 belge ve uzak dal kanıtı

Belge/matris commit'i `0d35712b7e5ed08f772e0a27ba5c4d3e225f86ac`.
Normal push `d9a127f..0d35712 dev -> dev` başarılı; açık dev refspec'iyle fetch
sonrasında HEAD/origin/dev aynı SHA doğrulandı. `ls-remote` main'i değişmemiş
`5d55ee599df4617bd51c09b1c7a008b8d0ece480` gösterdi. Başlangıca karşı Cargo.toml,
Cargo.lock, viewing.rs, fixture'lar ve PIV-09 Python aktif gözlemcisinde fark yok.
Loader diff'i yalnız eşdeğer LoadedFrame alias'ıdır, sandbox uygulaması aynı.
Bu son kayıt ayrı belge commit'idir; kendi SHA'sı Git geçmişindedir. PIV-16 TESTING.


## PIV-16 native ve manuel kapanış — 24.10.2026 — DONE

Kullanıcı gerçek Arch/GNOME ortamında `0822e4b2028d414304ef3b2078bc5bc6d6da8852` üzerinde `bash tests/run_async_cancel_probe.sh` çalıştırdı: **exit 0**. Gerçek JPEG/PNG/EXIF baseline PASS; secure async latest-pixel, A→B→A, hata toparlanma, iki pencere ve kaynak bütünlüğü PASS. `LoadStats`: running=0, pending=0, started=11, completed=11, max_running=3, dropped_pending=97, suppressed=6, timed_out=0. Ek PIV-09 aktif secure decode close checkpoint'inde partial memfd/frame, pidfd/mapping/sandbox, worker join, stale mutation=0, survivor pixels ve child cleanup PASS. `Cleanup SIGCONT unavailable: No such process` süreç zaten çıktığı için fatal değildi; toplam test exit0. Burst içinde decoder active interval/overlap **NOT MEASURED**. Kullanıcı gerçek uygulamada hızlı Sol/Sağ, A→B→A ve yükleme sırasında görüntüleme tercihlerini denedikten sonra olumlu manuel yanıt verdi; ayrıntılı ayrı ölçüm iddiası yok. **PIV-16 DONE; PIV-17 TODO.** Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 i18n planı açık.


## PIV-17 — 24.10.2026 — TESTING

Başlangıç `e67bebf5e5aef7ffa64b12d997596d489e6ad241` (güncel origin/dev'den FF).

| Gerçek SHA | Değişiklik |
|---|---|
| `e2f7504c19f5b49d405e9f45c36d954e84e301a3` | Per-window RAM cache, next/previous politika, metadata/monitor invalidation, tek background admission/promotion, GMemoryMonitor, bounded header kontrolü; model testleri |
| `a4fd9e083176ce885b9557db10860db5f513608e` | Controlled/native preload probe, tek native giriş; eski mock/probe sınırlarının uyarlanması |

**PASS:** fmt, locked check/build/test, all-targets Clippy -D warnings;
stable/offline/-j2 mevcut helper ile **72 Rust = 48 unit + 24 integration**, 0
failed/ignored; **30 Python observer**, fixture hash'leri, tüm örnekler, shell
syntax ve whitespace. **20 scoped grafik çağrısı PASS**: 14 X11 (yeni controlled
probe dahil), 4 Sway, 2 Weston. Kontrollü cache/texture reuse/Fit/filter,
LRU/re-decode, metadata değişimi/silinme, pressure enjeksiyonu, promotion,
foreground priority, Yenile yarışı, 300 seçim ve iki pencere close/drain geçti.
Bu sonuçlar gerçek decoder piksel/aktif process kanıtı değildir.

**BLOCKED/NOT RUN:** strict preload probe bir kez secure baseline denedi,
exit77 / bwrap loader early exit1. Önceki Codex read-only proc engeli sürer;
yeni denemede UID-map stderr'i ayrıca alınmadı. JPEG/PNG/EXIF/cache gerçek native
kabulü ve yeni kodda aktif secure process cleanup PASS sayılmadı; aynı engel
tekrarlanmadı. GMemoryMonitorDBus seçildi, system bus yok; gerçek OS basınç
sinyali NOT RUN. RSS/VRAM/FPS/hız kazanımı ve decoder overlap NOT MEASURED.

[Tam politika, komutlar ve test ayrımları](PIV17_VALIDATION.md).
Native tek komut: **`bash tests/run_preload_probe.sh`**; image_probe → secure
preload_probe → mevcut aktif-decode observer, JSON
`/tmp/pivra-preload-active-close.json`. Native worker-entry barrier gerçek aktif
decoder checkpoint'i sayılmaz; process cleanup son bağımsız checkpoint'le
sınanır. Yeni native/manual GNOME kabulü beklenir, **PIV-17 TESTING**.
PIV-01–16 DONE, Faz 0/DESKTOP-06/PIV-39 açık kayıtları korunur; PIV-18 başlamadı.
Cargo.toml/lock, fixture içerikleri, UI candidates metni, Bwrap zorunluluğu ve
Rust/GTK ortamı değiştirilmedi. Belgeler bu iki gerçek SHA'yı kaydeder; belge
commit'inin SHA'sı Git geçmişi ve teslim raporundadır. Normal push sonrası dev
HEAD/origin eşitliği doğrulanacak; ek push-kanıt commit'i oluşturulmayacak.


## PIV-18 — TESTING — 24.10.2026

Başlangıç `07f47b5fb21200d8337933893128b700e5b8329d`, güncel origin/dev.
PIV-01–17 DONE kullanıcı talimatına göre korunur. PIV-17 için yeni native
ölçüm/log verilmedi; yeni Codex native PASS üretilmedi.

- `99d49ddbdfb73c88d1c638f5d1d0d07ae4c85c3d`: full-only kalite yaşam döngüsü.
- `29059174e800ed067414651c99b02c2e2a9c779e`: GTK phase assertion, native JSON kapısı, rapor testleri.

PASS: 74 Rust, 30 eski +3 yeni Python, fmt/check/build/test/all-target Clippy,
fixture bütünlüğü; controlled phase/cache/cancel testi. Strict secure test bir
kez exit77: NOT RUN. Gerçek early-preview mevcut loader yolunda doğrulanmadı;
ikinci full decode eklenmedi. Preview/full viewport sürekliliği ve native kabul
NOT RUN; model geçiş testleri bunların yerine geçmez.
[Tüm yöntem/sınırlar](PIV18_VALIDATION.md). Native: `bash tests/run_preview_pipeline_probe.sh`.
PIV-18 TESTING; PIV-19 başlamadı. Faz 0/DESKTOP-06/i18n açık.

Grafik regresyon: 20 çağrı PASS (14 X11, 4 Sway, 2 Weston); full-only/kontrollü
veri ayrımı korunur. Weston normal close NOT RUN. Tüm örneklerin locked/offline
derlemesi, shell syntax ve whitespace PASS.


## PIV-17 Arch native ve manuel kabul kapanışı — 24.10.2026 — DONE

Kullanıcı `07f47b5fb21200d8337933893128b700e5b8329d` üzerinde `bash tests/run_preload_probe.sh` çalıştırdı, **exit 0**. Gerçek secure JPEG/PNG/EXIF baseline, cache hit/texture reuse/Fit/filter, next-before-previous preload, LRU eviction, atomic replace/delete identity, hesaplanan bellek bütçesi, injected memory pressure, source integrity, worker-entry promotion ve foreground priority, window close/drain **PASS**. Örnek: hits=3, misses=6, evictions=5; enabled başlangıcı foreground=1/background=2; disabled foreground=2. Native secure secondary probe promoted=1, cancelled=2, started=completed=4, cache entries/bytes=0 after drain. PIV-09 gerçek active decoder memfd/pidfd/mapping/sandbox close checkpoint, job join, survivor pixels, children cleanup PASS. `Cleanup SIGCONT unavailable` hedef PID zaten çıkmışken fatal değildi. Kullanıcı gerçek GNOME arayüzünde hızlı Sol/Sağ gezinmenin sorunsuz olduğunu bildirdi; ayrı gerçek RSS/VRAM/FPS/hız kazanımı, OS gerçek memory-pressure sinyali ve decoder overlap **NOT MEASURED/NOT RUN**. **PIV-17 DONE; PIV-18 TODO.** Faz 0 temiz kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN, PIV-39 i18n açık.

### PIV-18 native mode snapshot düzeltmesi — TESTING

Kod/test SHA `bba76daa1879d8dfd310664bed47a9b83f239ce8`.
Kullanıcı Arch native komutu exit101 / `rendered image` (WidgetPaintable boş
node) bildirdi. Testte sabit100ms bekleme yerine GTK after-paint/frame bariyeri,
hedef present ve bounded gerçek snapshot kontrolü eklendi. Ürün kodu değişmedi;
node/piksel assertion'ları korundu. Arch kök tetikleyicisi bağımsız üretilmedi.
PASS: önce hedefli X11 48 snapshot; sonra bir tam core (74 Rust/30 Python,
fmt/check/build/test/Clippy/fixtures), X11 zoom/pan/resampling ve Sway 48 mode
snapshot. Strict secure gate bir kez exit77: NOT RUN. Arch native **FAIL kaydı
korunur / düzeltme sonrası yeniden test bekleniyor**. Komut değişmedi:
`bash tests/run_preview_pipeline_probe.sh`. [Ayrıntı](PIV18_VALIDATION.md).
PIV-18 TESTING; PIV-19 başlamadı. Main/üretim bağımlılıkları/sandbox değişmedi.

### PIV-18 ikinci native snapshot düzeltmesi — TESTING

`fcea3e8a4ac0f7cb87ca4eedf9eaec4a1140adb4`: doğrulanmış after-paint yardımcı
ortak test modülüne taşındı; resampling ve zoom/pan geçici snapshot'ları buna
bağlandı. Smooth/Nearest piksel/clipping assertion'ları aynı; ürün kodu değişmedi.
Kullanıcı önceki gerçek mode/iki pencere testlerini PASS, yeni resampling aşamasını
FAIL/exit101 bildirdi. Yeni Arch sonucu bekleniyor; native PASS ilan edilmedi.
PASS: hedefli X11 resampling, sonra tek core (74 Rust/30 Python, zorunlu Rust
kontrolleri/fixture), 3 Python rapor testi; X11 48 mode snapshot ve zoom/pan,
Sway resampling. Synthetic veri ile gerçek decoder ayrımı korunur. Secure kapı
NOT RUN; bilinen Bwrap engeli tekrar denenmedi. [Ayrıntı](PIV18_VALIDATION.md).
Native komut: `bash tests/run_preview_pipeline_probe.sh`. PIV-18 TESTING.

### PIV-18 normal uygulama PNG/MIME hatası — TESTING, açık

Kullanıcı native paketi exit0 bildirdi; normal GNOME uygulamasında iki geçerli
PNG `Unknown image format: text/plain` ile **FAIL**. Kök neden henüz bulunmadı;
ürün açılış sorunu giderildi/PASS iddiası yoktur.
`44560357e6a8061308c4666c269bc13461f86248`: aynı production app::launch/GIO-open
akışını kullanan PNG/JPEG path/URI probe, paket kapısı ve hataya sınırlı GFile
MIME/signature/path teşhisi. Uzantıya göre MIME zorlama/sandbox değişikliği yok.
PASS: hedefli içerik teşhis testi, tek core75 Rust/30 Python, 3 Python rapor testi,
normal binary X11 error-mode input smoke. NOT RUN: gerçek app giriş probe'u bir
kez exit77, Codex wide.png içeriğini image/png olarak algıladı; Arch text/plain
tekrar üretilemedi. [Kanıt ve manuel komut](PIV18_VALIDATION.md). PIV-18 TESTING;
kullanıcının normal GNOME yeniden testi gereklidir. PIV-19 başlamadı.


### PIV-18B — erken önizleme araştırması — TESTING

Başlangıç `b8924b85778e0386f81714dea6b99487ccbaa44a`.
`b0caa4f99e30031271b870d381b7fafee1d53fd0`: bağımsız secure Glycin3.0.8
specific_frame(scale) → same Image next_frame → fresh full kıyas probe'u.
Üretim açılışı/decoder/cancel/cache değişmedi. Kaynakta image-rs scale isteğini
yok sayıyor, full buffer okuyor ve still frame'i tüketiyor. Bu loader yolunda
gerçek erken preview önerilmez; native farklı loader davranışı ayrıca ölçülmeli.
PASS: probe build/1 unit/fmt/hedefli Clippy/shell/whitespace. Tek Codex secure
deneme77: NOT RUN, gerçek frame/zaman ölçümü yok; tekrar edilmedi.
[Format/method matrisi ve alternatif planı](PIV18_PREVIEW_FEASIBILITY.md).
Native: `bash tests/run_preview_feasibility_probe.sh`; exit0 preview PASS değildir.
PIV-18 TESTING, PIV-19 başlamadı; mevcut full-only native kabul korunur.


### PIV-18C — JPEG embedded/reduced prototipleri — tarihsel, reddedildi/kaldırıldı

`f69b0edef8f1e5f33f404e234c20d49a08bae170`: bounded EXIF IFD1 çıkarımı,
ayrı Bwrap libjpeg-turbo helper baseline 1/8 IDCT; üretim modülleri değişmedi.
PASS: C Werror compile, 6 metadata/driver test, shell/whitespace. Tek Bwrap77
(uid map read-only proc): frame/zaman/RSS/gerçek iptal NOT RUN. Glycin scale
deneyi tekrar edilmedi. Gerçek erken preview NOT PROVEN; no-thumbnail ve
progressive/arithmetic JPEG sınırlamaları açık.
Deney belgesi ve çalıştırma betiği 25.10.2026 kullanıcı kararıyla kaldırıldı;
yukarıdaki test sonuçları yalnız tarihsel kayıttır.
PIV-18 TESTING, PIV-19 başlamadı; yeni decoder üretime bağlanmadı.


### PIV-18C temizlik kararı — 25.10.2026

Kullanıcı embedded-thumbnail ve libjpeg reduced-resolution prototiplerini
reddetti; üretime entegre edilmeyecek. Decoder silinmesi uzak
`87aa792a8f03a8979bd181733bfa9196d4ee313b` ile alındı; kalan deney sürücüsü,
metadata testi, native betik ve prototip belgesi bu temizlikte kaldırıldı.
Üç mevcut belgede aktif prototip linkleri/test/entegrasyon beklentileri kaldırıldı.
Silinen prototip için yeniden native test veya devam geliştirmesi beklenmez.
PIV-18B Glycin araştırması ve bütün üretim kodu korunur.
**PIV-18 TESTING; PIV-19 başlamadı.**

Temizlik doğrulaması PASS: beş hedef path yok; değişen yollar yalnız izin verilen
silmeler ve üç belge; kalan prototip link/komut referansı yok; üretim src/Cargo/
examples ve PIV-18B betiği değişmedi. Mevcut 30 Python observer +3 native rapor
sürücüsü testi PASS; git diff --check PASS. Üretim kodu değişmediğinden Rust/GUI
paketleri tekrar çalıştırılmadı; bu bir native decode PASS iddiası değildir.

## PIV-18 / TH-01 — GNOME thumbnail sağlayıcısı — 25.10.2026 — TESTING

Yalnız TH-01; TH-02–TH-06 kullanıcı kabulü bekler, kodlanmadı.
Başlangıç `1624264aa72a9b27825853b812fb17f69e2e2451`, temiz/güncel dev.

| Gerçek SHA | Değişiklik |
|---|---|
| `c8eb5bdf011cae2e86e3c4448df62ed173590038` | İzole GIO thumbnail cache provider, FileKey, metadata/container validation, bounded worker, dört capability/controlled outcome |
| `88f3e94f484ced10df9fb32d9ec36425571ce722` | Gerçek GIO cache testleri, GNOME factory can_thumbnail metadata helper ve tek native giriş |

PASS:5 yeni Rust unit;11 GIO entegrasyon testi. Tek core80 Rust(56 unit+24
integration),30 Python observer;3 mevcut native rapor testi: toplam44 Python.
Fmt/locked check/build/test/all-targets Clippy -D warnings, fixture hash'leri,
probe build, shell/whitespace PASS. İzole GNOME token/cache fixture'ı gerçek
GNOME session değildir; factory mock sonucu gerçek factory PASS sayılmaz.
Codex gerçek ortamda GNOME library yok, capability Unavailable; gerçek factory
C derleme/çalışması ve CacheAndGenerate native kabulü NOT RUN. Thumbnail piksel
decode/generation yok; Bwrap engeli tekrar denenmedi.

GIO gerçek Thumb::URI/MTime/Size doğrulamasına ek PNG bound/CRC ve source/cache
FileKey kontrolleri var; Hit bir **untrusted-pixel Candidate**, decode PASS değil.
Source/cache yazılmaz/silinmez. GNOME factory capability güvenliği Glycin Bwrap
ile eşdeğer sayılmaz; TH-05 generation güvenlik/öncelik işi bu görevde yok.
[TH-01 API/kanıt/sınırlar/native komut](TH01_THUMBNAIL_VALIDATION.md).
Native: `bash tests/run_thumbnail_provider_probe.sh /tam/yol/fotograf.jpg`.
Normal app'e bağlantı/UI/şerit/T/kalıcı ayar eklenmedi; üretim ImageView/loader/
cache/preload/cancellation/settings ve Cargo bağımlılıkları korundu.
**TH-01 TESTING; PIV-18 TESTING; PIV-19 başlamadı.**

### TH-01 native SIGPIPE/modül seçimi ve CacheOnly teşhisi — TESTING

`1443a4762d5d8db852b4d0d027bfb1872ddf5c69`: listing tamamen okunur, adaylar
tekilleştirilip sürüm sırasıyla tek tek derlenir; yalnız bir başarılı module
seçilir. Pipefail korunur,141 nedeni olan erken pipe consumer kaldırıldı.
Helper configured/attempted/eligible/ineligible/failed ve MIME artık probe
çıktısında ayrılır. CacheOnly başlangıç varsayılanıdır; Missing yalnız cache
path yokluğudur. CacheAndGenerate sadece helper can_thumbnail=true ile mümkün;
eski native CacheOnly çıktısının gerçek sebebi yeni Arch çıktısı bekler.
PASS:80 Rust+50 Python (6 yeni module-driver,11 GIO cache,30 observer,3 rapor),
zorunlu Rust kontrolleri/fixture/shell/whitespace. Modül seçim testlerinde4
seçildi,4 derlenmezse3.0 seçildi; compiler/factory stub gerçek GNOME değildir.
Codex'te gerçek GNOME factory bulunmadı: native module compile/çalışma NOT RUN.
[Teşhis/tek Arch komutu](TH01_THUMBNAIL_VALIDATION.md). TH-01 TESTING;
TH-02/PIV-19 başlamadı. Glycin/preload/viewport ve görüntüleme kodu korunur.


### TH-01 GNOME native link/SIGXFSZ düzeltmesi — TESTING

`ccced8089f425a07d6580c70570e725ac2296e14`: seçilen tek GNOME pkg-config
module'ünün yanında doğrudan GObject bağımlılığı, source sonrası library link
sırası; GNOME ABI flag'leri karıştırılmaz. FSIZE0→dosya başına1MiB; bellek/CPU/
parent timeout/cancel/child cleanup aynı. Eski0 limit gerçek memfd ftruncate
stub testinde SIGXFSZ25 verdi; native GNOME'da tetikleyen dosya trace edilmedi.
PASS:5 Rust provider +22 Python hedefli regresyon (5 C/helper+6 module driver+
11 GIO cache), fmt/shell/whitespace. C testlerinde gerçek GObject, stub GNOME API;
bu native GNOME başarı iddiası değildir. Gerçek GNOME compile/init/can_thumbnail
NOT RUN; kullanıcı Arch testi beklenir. Gerçek true/false/Failed korunur,
Missing tek başına hata değildir. [Kanıt/komut](TH01_THUMBNAIL_VALIDATION.md).
Üretim görüntüleyici/Glycin/preload/UI değişmedi. TH-01 TESTING; TH-02 yok.

## PIV-18 thumbnail aşamaları — 25.10.2026

| Aşama | Güncel durum | Kanıt |
|---|---|---|
| TH-01 provider | DONE — NATIVE ACCEPTED | Kullanıcı TH-02 talimatında native kabul başarılı bildirdi; yeni ayrıntılı log/SHA verilmedi, Codex testi sayılmaz |
| TH-02 overlay | DONE | Kullanıcı TH-03 talimatında native kabulü bildirdi; yeni ayrıntılı log/SHA verilmedi |
| TH-03 | TESTING | Otomatik gizlenme, `T`, kalıcı ayar; ayrı native kabul bekleniyor |
| TH-04 | DONE | DONE — kullanıcı Arch GNOME kabulü, 1b11074 |
| TH-05 | DONE | DONE — kullanıcı Arch GNOME kabulü, 1b11074 |
| TH-06 | TESTING — TH-06.1/06.2, native kabul bekleniyor | Thumbnail genel kabul ve optimizasyon |
| TH-07 | TODO | Tam ekran fotoğraf görüntüleme; PIV-18 alt görevi, yalnız planlandı |

TH-02 başlangıç `e503e8fec645ddf79b2b73b4b34b9c03573f8eec`, temiz/güncel dev.

| Gerçek SHA | Değişiklik |
|---|---|
| `604b5761904c195a2f0bdf55f08f4a5d377ed59c` | Yeni bağımsız thumbnail overlay component; mevcut ImageView'e non-measuring kurulum, pencereye özel toggle |
| `b1bb6f41573fd41148a066d7de878e867b8b3b52` | Açık TEST placeholder UI probe ve native manuel giriş; GSK/geometry/counter/isolation testleri |

PASS:80 Rust/30 Python observer, fmt/check/build/test/fixture, örneklerin locked
build'i. Clippy ilk probe ok-expect uyarısı sonrası düzeltildi; all-targets
Clippy -D warnings ve fmt tekrar PASS.6 scoped grafik çağrısı PASS: TH-02 X11,
X11 scale2, Sway; eski async/preload/window-failure controlled probeları.
Bunlar synthetic frame/GTK olay testleridir, secure/native pixel PASS değildir.
Şerit açık/kapalı allocation ve Fit/Fill/Actual/zoom/pan pikselleri, aynı texture,
sabit96×80 cell, scroll, kapalı input pick-through, ikon, pencere izolasyonu
ve ek decode-job olmaması doğrulandı. Native GNOME alpha/tema/çerçeve/fiziksel
input kabulü NOT RUN. Bilinen Bwrap engeli tekrarlanmadı.
[TH-02 ayrıntılı kanıt ve manuel komut](TH02_THUMBNAIL_OVERLAY_VALIDATION.md):
`bash tests/run_thumbnail_overlay_probe.sh` — açıkça TEST iki pencere, gerçek
thumbnail değil. Normal app boş strip gösterir. Factory/generation/selection/T/
kalıcı ayar/auto-hide yok. Üretim Glycin/preload/request/cache/settings ve
Cargo bağımlılıkları korundu. **TH-02 TESTING; PIV-18 TESTING; PIV-19 yok.**


### TH-02 görsel düzeltme — 25.10.2026 — TESTING

`bd5d6b77f801015c26c3dac11e6218307a88ae79`: panel/düğme arka planı%25,
çerçevesiz hücre ve yalnız1px selected underline; probe'a normal Pivra kalıbında
adw HeaderBar/WindowTitle ve standart close kontrolleri. TEST etiketleri ve iki
pencere korunur. GTK/GSK alfa/opaque/çerçeve-yok/alt-çizgi ve sabit allocation
assertion'ları eklendi. Üretim Glycin/factory/preload/cache değişmedi.
PASS:80 Rust/30 Python, zorunlu Rust/fixture kontrolleri, örnek build'leri;
X11 light, X11 dark+scale2, Sway ve3 eski controlled regresyon (toplam6 GUI çağrısı).
Gerçek GNOME görsel kabulü NOT RUN. [Ayrıntı/aynı native komut](TH02_THUMBNAIL_OVERLAY_VALIDATION.md).
TH-02 TESTING; auto-hide/T/kalıcı ayarlar yok, TH-03'e geçilmedi.


### TH-02 alt bant düzeltmesi — 25.10.2026 — TESTING

`eba1a07fefc925c44527b0a9f3b775e19e65869a`: açılınca tam genişliğe yayılan ortak
thumbnail-controls kutusu kaldırıldı; strip ve dar alt orta düğme bağımsız
non-measuring GtkOverlay çocukları oldu. %25 CSS/çerçevesiz hücre/ince çizgi
korundu. GSK bileşik snapshot'ta panel altındaki düğme-dışı piksellerin fotoğrafla
aynılığı, açık/kapalı GTK pick-through ve ortak dock bulunmaması regresyonu eklendi.
PASS: X11, X11 dark+scale2, Sway Wayland;80 Rust/30 Python; fmt/check/build/test/
all-target Clippy -D warnings ve fixture/whitespace. Gerçek Arch GNOME görsel
kabulü NOT RUN; native komut `bash tests/run_thumbnail_overlay_probe.sh`.
[Ayrıntı](TH02_THUMBNAIL_OVERLAY_VALIDATION.md). Secure decoder tekrar denenmedi;
Glycin/provider/preload/cache değişmedi. TH-02 TESTING; TH-03 başlamadı.


### TH-07 fullscreen planlama kararı — 25.10.2026

Kullanıcı talimatıyla TH-06 sonrasına **TH-07 — Tam Ekran Fotoğraf Görüntüleme**
eklendi. Yeni PIV numarası yok; mevcut PIV-18 kapsamına bağlıdır.
[Tam plan ve15 gereksinim](PIV18_THUMBNAIL_PLAN.md),
[yol haritası](PIVRA_DEVELOPMENT_ROADMAP.md).
Çift tıklama/`F`/`Esc`, gizli chrome, TH-03 görünürlüğü/`T`, Sol/Sağ ve overlay
etkileşim ayrımı, decode yapmadan aynı pencerede fullscreen, normal boyut/tercih
koruması ve pencere bağımsızlığı planlandı. Native Arch GNOME Wayland ve klavye/fare
regresyonları zorunlu kabul kapısıdır; bu testler henüz çalıştırılmadı.
Yalnız belgeler değişti; TH-03 ve TH-07 kodlaması başlamadı. TH-01 DONE;
TH-02 TESTING ve alt bant native kabulü ayrı devam eder. PIV-18 TESTING;
PIV-19 başlamadı. Mevcut tarihsel test/commit kayıtları korunur.


## PIV-18 / TH-03 — 25.10.2026 — TESTING

Başlangıç `0090131f05cddf9b076dc6a61d1807d4ba833834`, temiz ve origin/dev ile eşit.
Kullanıcı bu görevin talimatında TH-02 native kabulünü bildirdi: **TH-02 DONE**.
Ayrı yeni native log/test SHA verilmedi; Codex GNOME kabulü olarak sunulmaz.

| Gerçek SHA | Değişiklik |
|---|---|
| `f8bfd580708f103945aa6b310e9ae80166eeefbb` | Pencereye özel motion/hide/T; ortak kalıcı Enabled, atomik küçük config dosyası; GTK/persistence testleri ve tek native giriş |
| `e5f6681bfcd3e63c857fbff60963815c2ec28a2f` | Otomatik motion sinyallerini fiziksel pointer reset olaylarından ayırma, gerçek frame/roundtrip/allocation ile resize doğrulaması |

PASS:82 Rust (58 unit+24 integration),30 Python observer, fixture bütünlüğü;
fmt/check/build/test, son all-target Clippy -D warnings ve örnek build'leri.
Settings missing/bozuk/yazılamayan config, abonelik ve ayrı süreç persistence
kontrolleri PASS. GTK: X11 light, X11 dark+scale2, Sway Wayland; eski controlled
PIV-16 cancellation ve PIV-17 preload/cache PASS. İlk type-complexity lint ve
resize test sürücüsü deadline FAIL'leri düzeltildi; ayrıntılı geçmiş
[TH-03 doğrulama kaydında](TH03_THUMBNAIL_VISIBILITY_VALIDATION.md).
Test ana görselleri/hücreleri sentetik; fiziksel GNOME klavye/fare/görsel kabulü
ve yeni gerçek secure decode paketi NOT RUN. Bwrap engeli tekrar denenmedi;
Glycin/provider/cache/preload kodu değişmedi.

Native tek komut: `bash tests/run_thumbnail_visibility_probe.sh`.
İki TEST penceresinde hover/T/resize/Enabled paylaşımı ve Visible izolasyonu;
yeniden çalıştırarak persistence, normal uygulamada gizli şeridi açmayan Sol/Sağ
ayrıca kullanıcı tarafından kontrol edilecek. **TH-03 TESTING; TH-04/TH-05/TH-07
kodlanmadı; PIV-18 TESTING, PIV-19 başlamadı.**


## TH-04 kök neden / TH-05 tam thumbnail sistemi — 25.10.2026 — TESTING

Başlangıç `fe7b6f7b56671999aef2604f96333f4202767cfd`; temiz dev,24 remote commit
ff-only getirildi. Başarısız geçmiş yamalar yeniden uygulanmadı.

| Gerçek SHA | Kapsam |
|---|---|
| `0f9db6232afccb3a7274805e1a71834db93ad053` | API-checked optional GNOME factory sidecar; cache-first generation, SourceFileKey/stat/save guard'ları, limit/kill/reap/scratch ve7 C API test senaryosu |
| `f9283d59f0917cacf58f09cf3929748d125d10ff` | Stable hücreler/scroll, ilk tamamlanan scan anchor'ı, tüm katalog için viewport-priority queue, LRU/hidden-work/retry/epoch;120 JPEG/PNG ve gerçek window.rs callback testleri |

Eski focused-cell rebuild kontrolünde scroll10400→~385 (150ms) gözlendi;
eski regression hücre identity assertion'ı FAIL. Yeni kod seçimde hücreleri
kaldırmaz, odak taşımaz; singleton→asenkron katalog ve gerçek allocation tek
initial hizalama kapısıdır. Eski7-öğelik pencere yerine tüm pending katalog
progresif işlenir. Son controlled testte120/120, failed0,64 texture/4MiB
konservatif sahiplik; aynı stabil seçimde tekrar decode0.

PASS:86 Rust (62 unit+24 integration), fmt/check/build/test/all-target Clippy
-D warnings;30 Python observer +29 thumbnail Python testi (11 cache,6 modül,
5 metadata helper,7 generation helper), fixture/hash ve shell/whitespace.
C helper testleri gerçek C/GLib/GIO/GdkPixbuf + GNOME API stub'dır; gerçek
GNOME generation diye sayılmaz. Current GUI: X11 light, X11 dark+scale2 ve
Sway Wayland; legacy TH-02/03 UI, controlled PIV-16/PIV-17 PASS.
Testte mevcut loading-text/preset kararlarıyla çelişen eski assertion'lar
FullLoading/hidden text ve Actual korunması/aynı texture ile güncellendi;
ürünün kabul edilmiş davranışı tersine çevrilmedi.

NOT RUN: gerçek GNOME helper link/runtime, secure Glycin thumbnail pikselleri
ve fiziksel Arch GNOME kabulü. Codex'te GNOME development modülü yok;
`--require-generation-helper` exit77 verdi. Bilinen Bwrap uid-map engeli yeniden
denenmedi. Rlimit sandbox olarak sunulmaz; source generation GNOME sistem
politikasını, cache/main pixel decode zorunlu Glycin Bwrap'ı kullanır.

Tek native komut: `bash tests/run_thumbnail_system_probe.sh` — private fixture/
config/cache, gerçek cache ve generation gate, ardından iki üretim TEST penceresi.
[Kök neden, sınırlar ve kabul adımları](TH05_THUMBNAIL_SYSTEM_VALIDATION.md).
TH-04/TH-05 TESTING; TH-06/TH-07 ve PIV-19 başlamadı. PIV-18 TESTING, Faz0 temiz
native kurulum ve DESKTOP-06 açık. Main ve üretim Glycin/preload/cache kaynakları
korundu; yeni Rust decoder/dependency/CI eklenmedi.


## TH-04 / TH-05 native kapanış — 25.10.2026 — DONE

Kullanıcı `1b11074e6ae6b00cdd79c0c6dae985cbb6ea6dce` (malformed GNOME PNG cache
regeneration düzeltmesi) sonrası `bash tests/run_thumbnail_system_probe.sh`
çalıştırdı: gnome-desktop-4,120/120, failed0, gerçek secure pikseller;0/79/119
initial alignment, scroll/click/hızlı seçim/stable cells/no redundant decode,
missing/cache/source replace-delete, cancellation/close ve iki pencere PASS.
Kullanıcı iki pencereyi manuel inceleyip sorun bildirmedi, ikisini kapattığında
komut terminale hatasız döndü. Kullanıcı raporudur; Codex native çalıştırması
değildir. Ayrı yeni detaylı log verilmedi. TH-05 DONE; bu kanıt TH-04 seçim/
hizalama/scroll/pencere bağımsızlığı koşullarını da karşıladığından TH-04 DONE.
[Native kabul kaydı](TH05_THUMBNAIL_SYSTEM_VALIDATION.md). PIV-18 TESTING;
TH-06 yetkili yeni kapsam, native kabul gelmeden DONE değil; TH-07 TODO.
Faz0/DESKTOP-06 ve diğer PIV kapıları bu kapanışla değişmedi.


### TH-06.1 — 25.10.2026 — TESTING

Şerit alt/start/end margin0 ile viewport alt kenarında; görünür panelde düğme
natural panel yüksekliği+4 ile üst ortada, kapalı panelde bottom4px. Bağımsız
non-measuring overlay ve%25/opaque/selected-frame tasarımı korunur.
PASS:86 Rust, fmt/check/all-target Clippy; X11/Sway geometri ve gerçek GSK
Photo snapshot sürekliliği; koyu tema+scale2 altında120 kayıt TH-05 regresyonu.
Native GNOME fiziksel kabul NOT RUN. [Kayıt/komut](TH06_VALIDATION.md).
TH-06 TESTING, TH-07 TODO; PIV-18 genel kapanışı yapılmadı.


### TH-06.2 — 25.10.2026 — TESTING

Kök neden yeni dosya istenirken prepare() ile mevcut texture'ın temizlenmesiydi;
negatif GTK testi bunu gösterdi. Hazır eski görüntü/zoom/pan/cache pin korunur,
yeni texture callback'te tek sunumla değişir. ActualPaintable texture kimliği
kontrolü eklendi. Hatalı hedef dosya adı + önceki fotoğraf açıklamasıyla Failed
kalır; Loading yazısı/animasyon/bypass yok. Mevcut token/worker/cache/preload
implementasyonları korunur.

PASS:86 Rust, fmt/check/all-target Clippy, examples build; X11 kontrollü tam-frame
geçiş/pan/hata-recovery, cache/priority/promotion/LRU ve iki pencere/GIO yarışları;
Sway/Wayland async ve120/120 kontrollü thumbnail katalog regresyonu. Fiziksel
GNOME ve gerçek Glycin TH-06 kabulü NOT RUN, bilinen Bwrap engeli tekrar denenmedi.
[Detay/tekil native test girişleri](TH06_VALIDATION.md).

Gerçek commit'ler: `0b50b7a8a0e514b2ea7743b7ff81c983b7a12acc` TH-04/05 kapanışı;
`8c077954bd14a0d41b716ad71d8095cb44c5247d` TH-06.1 geometri. TH-06.2 kod/test/kayıt
bu bölümün eklendiği commit'tedir (`git log --oneline -- docs/TH06_VALIDATION.md`);
kendi SHA'sı teslim raporunda belirtilir. TH-06 TESTING, TH-07 TODO, PIV-18 TESTING.
