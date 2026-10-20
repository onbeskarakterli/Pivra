# Pivra — Planlı Commit ve İlerleme Takibi

**Kaynak:** [Özellik Şartnamesi](PIVRA_FEATURE_SPEC_TR.md) · [Yol Haritası](PIVRA_DEVELOPMENT_ROADMAP.md)  
**Branch:** `dev` · **Durum:** yalnız plan. Burada listelenen `PIV-xx` kimlikleri gerçek Git commit SHA'sı **değildir**.

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
| PIV-01 | `chore: initialize Rust Cargo app` | Cargo derlenir; temel dizin ve lisans/README korunur | TODO | — |
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
| 0 — Temel | 01–05 | 5 | 0 |
| 1 — Fotoğraf | 06–12 | 7 | 0 |
| 2 — Zoom/performans | 13–18 | 6 | 0 |
| 3 — Video temel | 19–24 | 6 | 0 |
| 4 — Video gelişmiş | 25–31 | 7 | 0 |
| 5 — Arayüz | 32–39 | 8 | 0 |
| 6 — Uyumluluk/paket | 40–47 | 8 | 0 |
| 7 — Edit (en son özellik geliştirmesi) | 48–57 | 10 | 0 |
| 8 — Son doğrulama | 58–60 | 3 | 0 |
| **TOPLAM** | **PIV-01–60** | **60** | **0** |

> Plan ve takip dokümanlarının oluşturulması gerçek Git commit'i üretir; ancak tabloda yer alan **60 gelecekteki uygulama commit'i** henüz gerçekleştirilmemiştir.
