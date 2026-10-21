# Pivra — Geliştirme Yol Haritası

**Durum:** PIV-01–08 DONE (giriş, keşif ve doğal sıra); PIV-09–60 TODO. Faz 0 INCOMPLETE: temiz native kurulumun bağımsız yeniden üretimi NOT RUN; temiz Cargo target derlemesi ve headless regresyonlar geçti. [PIV-05 kanıtı ve ayrı faz kapısı](PIVRA_TECHNICAL_CAPABILITIES.md#102-piv-05-ve-faz-0-kapanışını-ayrı-değerlendirme).
**Başlangıç branch:** `dev` · **Kararlı sürümler:** `main`  
**Kaynak şartname:** [PIVRA_FEATURE_SPEC_TR.md](PIVRA_FEATURE_SPEC_TR.md)  
**Görev ve commit takibi:** [PIVRA_COMMIT_TRACKER.md](PIVRA_COMMIT_TRACKER.md)

## Temel kurallar

1. Ana dil Rust; GTK4 + libadwaita. Görüntüde Glycin, videoda GStreamer/gstreamer-rs, düzenlemede libvips **aday**. Gerçek uygunluk prototiplerle doğrulanır.
2. **Önce görüntüleme, sonra video. Edit modu en sona bırakılır** ve yalnız `Edit` eyleminden açılır. Normal görüntüleyici düzenleme araçlarıyla kalabalıklaşmaz.
3. Her aşamada küçük, geri alınabilir, tek amaçlı commit. Bir commit bir iş/konu; ilgili testler ve belge güncellemesi aynı committe olabilir.
4. `dev` aktif geliştirme, `main` yalnız doğrulanmış sürüm. Mevcut kaynak kodu ve tasarım kararları korunur.
5. Otomatik push/PR CI **yok**. GitHub Actions kurulursa yalnız `workflow_dispatch`, kullanıcı tetiklemesiyle.
6. Her iş için uygulama + birim/entegrasyon testleri + ilgili manuel GNOME testi; başarısız testle iş tamamlandı sayılmaz.
7. Harici medya, disk ve metadata işlemleri güvenli; görüntüleme dosyaları değiştirmez. Hatalı dosyada gezinme bozulmaz.
8. Dağıtım yalnız **native Linux paketleri**: Arch PKGBUILD/AUR, DEB, RPM, diğer sistemlere uygun yerel seçenekler. Flatpak/AppImage kapsam dışı.
9. Belirli codec, HDR, RAW ve donanım özellikleri önceden garanti edilmez; yetenek algılama, fallback ve doğrulama yapılır.
10. Pivra sistemin varsayılan video oynatıcısı yapılmaz. Fotoğrafla açılan pencerede aynı klasördeki videolar da gezilir.
11. Aşamalar bağımlılık sırasıyla yürür; faz kapısı geçilmeden sonraki fazın özellikleri tamamlandı işaretlenmez.
12. Bu doküman kod yazma yetkisi vermez; geliştirmeye kullanıcı açıkça başladığında geçilir.

## Modüler hedef mimari

- **app/ui:** GTK4/libadwaita pencereleri, overlay kontroller, input ve paneller.
- **catalog:** klasör tarama, doğal ad sırası, dosya sistemi olayları, gezinme ve pencere bağımsızlığı.
- **image:** fotoğraf çözümleme adaptörleri, kaynak çözünürlüğü, fit/fill/100%, zoom/pan, renk.
- **video:** GStreamer oynatma yaşam döngüsü, seek, frame-step, ses/altyazı, render.
- **jobs/cache:** iptal edilebilir ön yükleme, RAM içi thumbnail, kaynak önceliği, toplu kuyruk.
- **settings/platform:** GSettings, GNOME/GIO, HiDPI, güç yönetimi, tema, dosya ilişkileri.
- **edit:** kullanıcı `Edit` dediğinde aktive edilen ayrı tahribatsız düzenleme hattı; yalnız en son faz.
- **packaging/tests:** birim, entegrasyon, gerçek GNOME smoke/performance, yerel Linux paketleri.

Bu isimler mantıksal modüllerdir; gerçek Rust crate/dizin yapısı PIV-01/02 sırasında belirlenir.

## Fazlar ve çıkış kapıları

| Faz | Commit hedefleri | Ürün çıktısı | Tamamlanma kapısı |
|---|---|---|---|
| 0 — Mimari ve temel doğrulama | PIV-01–05 | Derlenen Rust GNOME iskeleti, ortam/bağımlılık kontrolleri, test temeli | Temiz kurulumda `cargo build`, `cargo test`, uygulama penceresi ve açık test raporu |
| 1 — Fotoğraf çekirdeği | PIV-06–12 | Nautilus'tan açılan görüntü, klasörde doğal sırada gezinme, fotoğraf render, fit/fill/100% | JPEG/PNG açılışı, 1/2/10 sıralama, kararlı gezinme, doğru yön, bozuk dosyada çökme yok |
| 2 — Zoom ve kaynak yönetimi | PIV-13–18 | İmleç odaklı zoom/pan, RAM cache/ön yükleme, büyük görsel tepkiselliği, mini harita temeli | Büyük fotoğrafta UI donmaz; zoom sınırları ve iptal edilen decode doğrulanır |
| 3 — Video çekirdeği | PIV-19–24 | Aynı dosya sırası içinde video, otomatik oynatma/temiz kapatma, ses ve zaman çubuğu | Fotoğraf → video → fotoğraf akışı, kaynakların serbest kalması, sessiz geçiş |
| 4 — Gelişmiş video | PIV-25–31 | Frame-step, hover scrub önizleme, altyazı/ses izleri, video zoom, kare kaydetme | Video test matrisi ve mümkün/uygulanamayan özellikler belgeli |
| 5 — Tam görüntüleyici arayüzü | PIV-32–39 | Şerit ve seçim modu, mini harita, bilgi paneli, slayt, kısayollar, touchpad/HiDPI, karşılaştırma | Tüm temel kullanım senaryoları GNOME'da manuel doğrulanmış |
| 6 — Uyumluluk ve yayın hazırlığı | PIV-40–47 | Format genişletme, ICC/HDR, hata dayanıklılığı, performans regresyonu, native paketler | Destek matrisi, paket kurulum testi, tekrarlanabilir testler, görüntüleme/video test sürümü |
| 7 — **Edit ve toplu işleme (son faz)** | PIV-48–57 | Ayrı Edit modu, dönüş/kırp/boyut/renk/iyileştirme/karşılaştırma, güvenli kayıt ve toplu dönüştürme | Orijinal koruması, undo, güvenli kayıt, job iptali/çakışma testleri; normal viewer sade kalır |
| 8 — Son bütünleştirme | PIV-58–60 | Genel test, paket sonlandırma, belgeli release adayı | Açık kritik hata yok, tamamlanma kanıtları var, `main` geçişi ayrıca onaylı |

**Önemli:** Faz 6'daki sürüm *görüntüleyici + video test sürümü* olarak çıkabilir. Edit modunun geliştirilmesi ilk görüntüleme/video testini bloklamaz. Resmî sürüm adı/sürüm numarası ayrıca belirlenir.

## Kabul ve test standardı

Her takip girdisi için:

- **Başlamadan:** teknik hedef, bağımlılık ve şartname maddesi net.
- **Kodlandı:** sınırlı değişiklik, hata yolları ve iptal/kaynak temizleme ele alınmış.
- **Otomatik test:** uygun unit/integration testleri başarıyla çalışmış; test komutu ve özet kaydedilmiş.
- **Manuel test:** GUI/gerçek sistem davranışı gerekiyorsa kullanıcıdan doğrulama alınmış; **doğrulanmadıysa PASS yazılmaz**.
- **İnceleme:** formatlar, dosya güvenliği, bellek kaynakları, regresyon değerlendirilmiş.
- **Commit:** SHA ve tarih takip tablosuna eklenmiş.
- **Kapanış:** gerekli tüm kanıtlar mevcutsa DONE.

Takip durumları: `TODO`, `IN_PROGRESS`, `BLOCKED`, `CODED`, `TESTING`, `DONE`. Belge planlandığında tüm satırlar `TODO` durumundadır.

## PIV-04 test stratejisi

GNOME/Wayland birincil hedef, X11 ikincil uyumluluk hedefidir. Rust/CLI, X11/Xvfb, headless Wayland compositor ve fiziksel GNOME/Wayland sonuçları ayrı raporlanır; biri diğerinin yerine geçmez. Headless Wayland için gerçek mapped yüzey kanıtı gerekir; yalnız PID/socket yeterli değildir. Normal kapatma kontrolü yapılamayan bir compositor için bu alan ayrıca NOT RUN kalır.

Deterministik küçük fixture'lar, kaynak/lisans durumu ve SHA-256 manifesti `tests/fixtures/README.md` içinde tanımlıdır. Model testleri gerçek decoder/tür algılama iddiası taşımaz; yazma/temizlik yalnız testin özel geçici dizininde yapılır. Fiziksel GNOME/Wayland, HiDPI, touchpad, çoklu monitör, erişilebilirlik ve donanım render ayrı gelecek doğrulamalarıdır. Yerel komutlar ve kategori bazlı güncel kanıtlar README ve commit takip tablosundadır.

## Ölçüm başlıkları (eşik uydurulmaz)

- Soğuk/sıcak ilk görüntü açılışı, 1/10/1000+ dosyalı klasör açılışı.
- Fotoğraf ve karma fotoğraf/video gezinme gecikmesi; video start/stop/seek.
- RAM/VRAM/CPU, ön yükleme iptali ve uzun süreli kullanımda kaynak birikimi.
- Büyük RAW/TIFF önizlemeden tam render'a geçiş, zoom/pan kare akıcılığı.
- 1080p/4K ve codec/container matrisi, donanım decode/fallback, frame-step doğruluğu.
- Wayland/HiDPI, dokunmatik/touchpad, çok pencere, ağ/çıkarılabilir disk ve klasör değişimi.
- Korunan orijinal dosya, EXIF/ICC, atomik kayıt, yarıda kesilen dönüştürme.

Ölçümler kullanılan cihaz, dağıtım, GNOME/GTK/GStreamer sürümüyle birlikte kaydedilir. Hedef eşikler gerçek prototip sonuçlarına göre belirlenir.

## Açık teknik riskler ve durdurma noktaları

| Risk | Karşılık / karar kapısı |
|---|---|
| Glycin format/RAW/HDR yetenekleri | Hedef örnek dosya seti; eksik özellik için alternatif adapter; kullanıcıya kapsam bildirimi |
| GStreamer frame-step, ASS, timeline önizlemesi | Küçük doğrulama prototipleri; kısıt kanıtlanırsa çözüm/alternatif görüşülür |
| GTK4'te yüksek çözünürlüklü zoom/texture akışı | Benchmark, tiled/preview stratejisi, kontrollü bellek ve iptal |
| Rust ↔ libvips bağlayıcısı / sistem sürümleri | Son Edit fazı öncesi bağımlılık ve bakım değerlendirmesi |
| Native paket sürüm ayrışması | Arch/DEB/RPM için minimum sürüm ve smoke test matrisi, hedeflenen dağıtımlar için koşullu destek |
| Açık ürün kararları | Ana özellikler kararlaştırılmış; alt düzey belirsizlikler test aşamasında kayıt altına alınır |

## Repo belgeleri

- `docs/PIVRA_FEATURE_SPEC_TR.md` — kullanıcı onaylı ürün şartnamesi, ana referans.
- `docs/PIVRA_DEVELOPMENT_ROADMAP.md` — bu fazlar, kabul kapıları ve geliştirme yöntemi.
- `docs/PIVRA_COMMIT_TRACKER.md` — 60 planlı commit/görev, ilerleme ve SHA alanları.

Belgedeki planlı commitler **henüz Git commit'i değildir**. Her biri iş bittiğinde gerçekten oluşturulur; commit SHA'sı sonradan işlenir.

## PIV-06 kapsam ve açık desktop kapısı — 21.10.2026

Kullanıcının PIV-06 talimatıyla yerel dosya/URI girişi, GIO open yaşam döngüsü,
pencereye ait dosya modeli ve decoder içermeyen placeholder tamamlandı. Yol
haritasındaki nihai “fotoğrafa çift tıklama” kabulünün CLI/GIO/desktop Exec kısmı
teknik olarak doğrulandı; **DESKTOP-06: gerçek Nautilus çift tıklama ve varsayılan
fotoğraf uygulaması davranışı NOT RUN**. Varsayılan uygulama değiştirilmedi.
Kaynak desktop kaydı yalnız image/jpeg ve image/png giriş adayları sunar;
bu MIME listesi decode desteği anlamına gelmez. Gerçek render PIV-09 kapsamıdır.

PIV-05 riskleri ve ayrı Faz 0 temiz native kurulum kapısı açık kalır. Kullanıcının
sınırlı PIV-06 yetkilendirmesi bu kapının PASS olduğu anlamına gelmez. PIV-07
başlatılmadı; öncesinde bu giriş sözleşmesi korunmalı, tarama yeni görev olarak
ayrıca yetkilendirilmeli. DESKTOP-06 ve decoder/player ortam kapıları uygun gerçek
native oturum/sonraki entegrasyon doğrulamasında izlenmeli; X11/Sway headless testi
fiziksel GNOME doğrulamasının yerine geçmez.

## PIV-07 keşif sınırı — 21.10.2026

Tek üst dizinde, std-only keşif modülü fotoğraf/video uzantı adaylarını aynı
MediaCollection'a toplar; açılan alias/yol seçili kalır. UI taramayı GIO worker'ına
verir; pencere başına model ve kooperatif iptal vardır. Sıra read_dir sırasıdır ve
**garantili değildir**. PIV-08 doğal sıralama henüz uygulanmadı/yetkilendirilmedi.
Kısmi tarama uyarıları ile fatal erişim hataları açık UI/stderr durumlarıdır;
uzantı adaylığı decode/playback başarısı sayılmaz. Doğrudan giriş/desktop MIME
sözleşmesi genişletilmedi. Ayrıntılı politika/testler README ve takip tablosunda.

PIV-05 medya/sandbox riskleri, DESKTOP-06 gerçek Nautilus ve Faz 0 temiz native
kurulum kapıları açık kalır. Fiziksel GNOME ve Weston normal kapanış NOT RUN.

## PIV-08 doğal sıra — 21.10.2026

PIV-07 read_dir sırası artık katalog kurulmadan önce worker'da ortak doğal sıraya
çevrilir; açılan exact alias/yol sıralamadan sonra seçilir. ASCII katlama,
overflow'suz sayı blokları, primary eşitliğinde kısa sayı bloğu ve ham filename/
full-path byte tie-break kullanılır. Türkçe locale collation, Unicode normalization
ve non-ASCII case-fold yok. Raw Linux adları korunur. Ayrıntılı politika ve 47 test
kanıtı README/takip tablosundadır. PIV-08 DONE; PIV-09 başlamadı. Medya/sandbox,
DESKTOP-06 ve Faz 0 temiz native kurulum kapıları açık kalır; fiziksel GNOME ve
Weston normal kapanış NOT RUN. Decoder/player/gezinme eklenmedi.
