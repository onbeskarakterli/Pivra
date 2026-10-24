# Pivra — Geliştirme Yol Haritası

**Durum:** PIV-01–08 DONE (giriş, keşif ve doğal sıra); PIV-09 DONE (Arch gerçek JPEG/PNG/EXIF, geometri ve aktif-decode kapanış/temizlik PASS); PIV-10 DONE (native gerçek secure piksel/Fit ve fiziksel sol/sağ gezinme PASS); PIV-11 DONE (Arch native ve GNOME Yenile PASS); PIV-12 DONE (24.10.2026 native + manuel kabul); PIV-13 TESTING; PIV-14–60 TODO. Faz 0 INCOMPLETE: temiz native kurulumun bağımsız yeniden üretimi NOT RUN; temiz Cargo target derlemesi ve headless regresyonlar geçti. [PIV-05 kanıtı ve ayrı faz kapısı](PIVRA_TECHNICAL_CAPABILITIES.md#102-piv-05-ve-faz-0-kapanışını-ayrı-değerlendirme).
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

## PIV-09 güvenli görüntü yükleme — 21.10.2026

Glycin 3.0.8/Bwrap zorunlu istemci, async worker, GTK Picture ScaleDown ve
EXIF-tek-dönüşüm kaynakları derlendi. Native GTK minimumu 4.16 oldu. Gerçek valid
JPEG/PNG probe'ları loader/config eksikliğiyle exit 77 verdi; ayrıca Codex UID-map
sandbox probe read-only /proc nedeniyle başarısız. Bypass yok. PIV-09 **BLOCKED**;
gerçek JPEG/PNG/EXIF/Fit/resize/lifecycle kapıları kapanmadı. Olumlu source branch
compile olması PASS değildir. 49 Rust ve scoped window/catalog regresyon kanıtı
README/teknik belge/takip tablosundadır. PIV-10 başlamadı; Faz 0 INCOMPLETE sürer.

### PIV-09 devam güncellemesi — 21.10.2026

Resmi2.0.0 image-rs loader kullanıcı prefix'inde hazır ve client3.0.8 ile yalnız
JPEG/PNG discovery PASS. Native Codex'te clone başarılı ama uid_map open EROFS;
standart kısıtlı Docker yolu da namespace oluşturmaya izin vermedi. Güvenlik
profili gevşetilmedi. PIV-09 BLOCKED kalır: gerçek piksel/EXIF/Fit/resize/lifecycle
kapıları için önceden izinli namespace destekli ayrı native oturum gerekir.
49 Rust + 8 scoped GUI/helper regresyonu PASS, image success değil. PIV-10 başlamadı.


### PIV-09 Arch Linux kullanıcı kanıtı — TESTING

`459fbea` üzerinde kullanıcı gerçek Arch/GNOME Wayland oturumunda güvenli Bwrap,
PNG/JPEG image_probe exit0, EXIF 1–8 pikselleri ve manuel Fit/resize/normal kapanış
PASS bildirdi. Bu Codex tarafından bağımsız yürütülmedi. Codex UID-map EROFS
kısıtı ayrı kalır; önceki BLOCKED genel durumunu bu yeni kanıtla TESTING izler.
Kalan kapılar: yeni gerçek-view iki görüntü/otomatik snapshot geometri probe'unun
native ortamda çalışması ve kanıtlı aktif decode sırasında kapanış/temizlik.
49 core ve 8 scoped GUI regresyonu Codex'te yeniden geçti; yeni pozitif görüntü
probe'u Codex'te exit77/BLOCKED. Ayrıntılar README ve takip tablosundadır.
PIV-09 DONE değildir; PIV-10 başlatılmadı. Faz 0 kapısı ayrı değerlendirilir.


### PIV-09 son test aşaması

Kullanıcı `7d1acf4` üzerinde yeni image_gui_probe exit0 (iki görüntü, gerçek-view
snapshot Fit/center, resize ve decode sonrası bağımsız kapanış) bildirdi; bu
Codex yürütmesi değildir. Yalnız aktif decode sırasında kapanış/temizlik sonucu
beklenir. [Yeni kontrollü test ve native komut](PIV09_ACTIVE_DECODE_TEST.md) gerçek
secure loader'ın kısmi çıktı yazımını kanıtlamadan pencere kapatma PASS'i vermez.
Codex baseline sandbox kapısında exit77; güvenlik gevşetilmedi. 49 Rust + 6 guard
regresyonu PASS. PIV-09 TESTING, DONE değil; PIV-10 başlamadı.


### PIV-09 gözlemci v2 — teşhis ve son native kapı

Kullanıcı beklenen memfd boyutu/ilk-son 12 byte'ı ve erişim hatası olmadığını
bildirdi; bu aktif checkpoint PASS'i değildir. Gözlem test başlatılmadan önce
ARMED bariyeriyle hazırlanır, doğru boyutlu frame fd'si cache edilir; tam 96-byte
ve STOP/kimlik/mapping/sandbox kriterleri korunarak aşama tanıları eklenir.
Native ortamda tek komut: `bash tests/run_active_decode_close.sh`.
Codex: 49 Rust + 17 Python unit PASS; gerçek secure baseline exit77/NOT RUN.
Yeni native sonuç bekleniyor; PIV-09 TESTING, PIV-10 başlamadı.


### PIV-09 V3 — mevcut loader keşfi

Kullanıcının V2 native loader_found engeli baseline PID çıkarımının somut kör
noktasını gösterdi. V3 tüm kendi loader'larını tarar; eski frame inode'larını
reddederek yalnız taze hedef 96 MB çıktısını önceki sıkı checkpoint'le kabul eder.
Glycin 3.0.8 istemci havuzunun 30 s retention davranışı kaynakta doğrulandı;
Arch'ta gerçek reuse native sonuç olmadan iddia edilmez. Origin ve target yaşam
döngüsü raporlanır; cleanup hedef kaynak bırakma + normal app/çocuk exit olarak
aynı 10 s bütçede korunur. 49 Rust + 30 Python PASS; native test Codex'te NOT RUN.
Komut `bash tests/run_active_decode_close.sh`; PIV-09 TESTING, PIV-10 başlamadı.


### PIV-09 V3 native kapanış — DONE (21.10.2026)

Kullanıcı gerçek Arch Linux / GNOME Wayland üzerinde `c3e0aaf9e1d19ee73f35d1167b7363ce48c70d86` commit'inde `bash tests/run_active_decode_close.sh` çalıştırdı: **exit 0**, JSON `result=PASS`. Mevcut Glycin image-rs pool loader'ının tekrar kullanımı (`reuse_observed=true`), hedefe ait taze 96 MB kısmi RGB frame, zorunlu Bwrap izolasyonu, SIGSTOP sonrası fd/inode/mapping ve worker pending doğrulandı. Aktif decode sırasında gerçek pencere kapatıldı; iptal, job join, widget bırakma, sıfır stale mutation, diğer pencere piksel bütünlüğü ve tüm çocuk süreç temizliği **PASS**; kapanıştan temizliğe **0,318 sn**. Önceki gerçek JPEG/PNG, EXIF 1–8 ve otomatik Fit/resize kanıtları da PASS. Bunlar **kullanıcı kaynaklı native sonuçlar**, Codex çalıştırması değil.

**PIV-09 DONE**; decoder'ların zamansal çakışması `NOT MEASURED`. Faz 0 temiz native kurulum ve DESKTOP-06 Nautilus kontrolleri kendi bağımsız kapılarıyla açık kalır. PIV-10 **TODO**. Ayrıntılar [commit tracker](PIVRA_COMMIT_TRACKER.md#piv-09-native-v3-kapanışı--21102026--done) içindedir.


## PIV-10 — 22.10.2026, TESTING

Pencereye ait doğal sıralı karışık MediaCollection üzerinde sınırlı previous/next
seçim, capture-phase sol/sağ controller ve pencere başlığı entegrasyonu eklendi.
Tek ImageView yeniden kullanılır; önceki token iptal edilir ve geç callback yalnız
güncel kimlikle kabul edilir. Her yeni JPEG/PNG Fit/center başlar; videolar atlanmaz,
oynatma yerine açık unavailable durumuna geçilir. Bozuk/silinmiş öğeden devam mümkündür.
Sınır/tek/boş ve pending callback guard'ları otomatik kontrollüdür.

54 Rust + 30 Python observer PASS; 12 scoped GUI/helper PASS. Strict yeni piksel
testi Codex sandbox kapısında exit77/BLOCKED. [Detay ve native tek komut](PIV10_VALIDATION.md).
Yeni native render/rapid sonuç ve fiziksel GNOME tuş/focus kanıtı olmadan DONE yok.
PIV-09 kullanıcı native kanıtıyla DONE kalır. PIV-11, Faz 0 temiz native kurulum
ve DESKTOP-06 kapsamında yeni başarı iddiası yok; bu görevler ilerletilmedi.


### PIV-10 native Arch kapanış — DONE (22.10.2026)

Kullanıcı `c53d4071368b2b45ceb8ecba06f949f4c5188b52` üzerinde `bash tests/run_navigation_probe.sh` çalıştırdı: **exit0/PASS**. Gerçek secure JPEG/PNG pikselleri, Fit/center, doğal karışık katalog, rapid navigation, bozuk/silinmiş dosyadan devam ve bağımsız pencereler PASS. Normal Pivra'da fiziksel sol/sağ tuşlarıyla gerçek fotoğraf geçişi kullanıcı tarafından ayrıca doğrulandı; `Loading image securely` mesajı yükleme bitince kayboluyor. PIV-10 arayüzünde odaklanabilir gezinme/başlık düğmesi olmadığından Tab sonrası focus kontrolü **NOT APPLICABLE**; bu ayrıca test edilmiş gibi gösterilmez. Video mesajının native otomatik doğrulaması PASS; fiziksel video geçişi kullanıcıca ayrıca raporlanmadı.

**PIV-10 DONE**, PIV-11 TODO. Faz 0 bağımsız temiz native kurulum **INCOMPLETE** ve DESKTOP-06 gerçek Nautilus **NOT RUN** kalır. Ayrıntılar [PIV-10 kapanış kanıtı](PIVRA_COMMIT_TRACKER.md#piv-10-arch-native-kabul-ve-kapanış--22102026--done).


## PIV-11 — 22.10.2026, TESTING

Canlı GIO katalog izleme, seçim/rename koruması, kayıpta kontrollü durum ve
kullanıcı eylemiyle güvenli Yenile uygulandı. 57 Rust + 30 Python, fixture
bütünlüğü ve 14 scoped GUI çağrısı PASS. Gerçek native pixel/refresh gate Codex'te
exit77/NOT RUN; PIV-11 DONE değil. Tek native komut:
`bash tests/run_folder_watch_probe.sh`. [Ayrıntılı kanıt](PIV11_VALIDATION.md).
PIV-01–10 DONE korunur; PIV-12 başlamadı. Faz 0 ve DESKTOP-06 açık kalır.


### Planlanan dil standardizasyonu — PIV-39 GNOME entegrasyonları

Kullanıcının 22.10.2026 gözlemi: GNOME arayüzü İngilizce, bölge Türkiye iken PIV-11'in `Dosya değiştirildi — Yenile` bileşeni Türkçe; `Loading image securely` ve video uyarıları İngilizce. Metinler kodda sabit; bölge ayarı dil seçimi olarak yorumlanmamalı. **PIV-39 kapsamında** uygulamaya gettext/GNOME uyumlu merkezi i18n altyapısı, en az `tr` ve `en` katalogları, tüm kullanıcıya görünen mesajların çeviriye taşınması ve `LANGUAGE/LC_MESSAGES/LANG` dil seçimi testleri eklenmesi planlandı. İngilizce GNOME + Türkiye bölgesi → İngilizce Pivra; Türkçe arayüz dili → Türkçe Pivra. İleride kapsamlı localization işi PIV-39'da açılacak; **henüz kodlanmadı**, PIV-11'in kaynak kapsamı genişletilmeyecek.

### PIV-11 kullanıcı native bulgusu — açık pencere yenileme regresyonu

Kullanıcı Arch/GNOME'da `f96a0db029fa0e5157318d4bdfd4ef494d3baf33` commit'inde `bash tests/run_folder_watch_probe.sh` **exit0/PASS** bildirdi: gerçek GIO olayları ve secure Glycin reload; beklenen silinme/klasör kaybı hata logları test kapsamı. Normal uygulamada seçili fotoğrafın haricen değiştirilmesi üzerine altta **Yenile** seçeneği görüntülendi. Ancak kullanıcı **Yenile'ye basınca mevcut pencerenin kapandığını ve yeni pencerenin açıldığını** bildirdi. Bu beklenen in-place refresh değildir; window identity ve signal/action yaşam döngüsü incelenmeli. Yeni pencere nedeninin kodda doğrulandığı iddia edilmez.

**Karar:** PIV-11 `TESTING` kalır. Başarılı otomatik probe, fiziksel buton tıklamasındaki regresyonu kapsamıyor olabilir. Düzeltme kabulü: gerçek GTK button click ile **aynı GtkWindow nesnesinin korunması**, mevcut ImageView'da taze frame, popup/ek pencere oluşmaması, eski image'ın kullanıcı onayına kadar korunması, banner'ın doğru temizlenmesi, worker guard/monitor güvenliği ve bağımsız pencere testleri; native GNOME manuel tekrar kontrolü. Faz 0 / DESKTOP-06 ayrı açık; PIV-12 başlatılmaz.


## PIV-11 final native acceptance — DONE (2026-10-22)

At `f96a0db029fa0e5157318d4bdfd4ef494d3baf33`, user ran `bash tests/run_folder_watch_probe.sh`: exit0/PASS, real filesystem events and secure Glycin refresh, previous frame retained until explicit user refresh action. In normal GNOME/Wayland application, user observed the refresh banner and confirmed clicking Yenile displays the modified image after a loading animation. Previous new-window impression was not confirmed by terminal logs (two separate cargo run commands); actual GtkWindow object identity not manually instrumented. Native acceptance met; PIV-11 DONE, PIV-12 TODO. The mixed Turkish/English UI strings are scheduled for PIV-39 i18n; Phase 0 clean install INCOMPLETE and DESKTOP-06 Nautilus NOT RUN remain independent.


## PIV-12 — 22.10.2026 — TESTING

Medya hataları ve gerçek GTK/GIO pencere/monitor izolasyonu güçlendirildi.
Yenile'nin video/unsupported öğeyi decoder'a göndermesi küçük bir düzeltmeyle
engellendi. Mevcut 57 Rust + 30 Python PASS; 18 scoped GUI/helper çağrısı PASS.
A/B/C, tekrarlanan Yenile, seçili silinme ve diğer pencerede bekleyen yükleme
sırasında kapanış deterministik kontrollü hata worker'larıyla doğrulandı;
gerçek Glycin başarısı değildir. 7 native decoder kapısı Codex'te exit77/NOT RUN.
Tek yeni native komut: `bash tests/run_media_failure_probe.sh`.
[Ayrıntılı kanıt/kalan kabul](PIV12_VALIDATION.md). PIV-01–11 DONE korunur,
PIV-12 DONE değil; PIV-13 başlamadı. PIV-39 tr/en i18n kararı, Faz 0 temiz
kurulum ve DESKTOP-06 ayrı açık işler olarak kalır.


## PIV-12 native ve manuel kabul kapanışı — 24.10.2026 — DONE

Kullanıcı `b12970f6a3a7ea2dddc890a747977d5d68437a63` üzerinde `bash tests/run_media_failure_probe.sh` çalıştırdı: **exit0**, gerçek secure Glycin, bozuk/yarım/boş/yanlış format JPEG/PNG hataları, sağlam kareye geri dönüş, katalog/seçim/monitor/bildirim izolasyonu, Yenile ve close cleanup PASS. Bozuk dosya hata logları beklenen test senaryolarıydı. Gerçek GNOME/Wayland manuel testinde kullanıcı iki Pivra penceresi açtı; birincide kırmızı→yeşil→bozuk→mavi gezinmesini, diğer pencerenin etkilenmemesini ve birinci kapatılınca ikincinin çalışmaya devam etmesini **PASS** bildirdi. Aktif gerçek secure decode sırasında silme/kapatma ve decoder süreçlerinin zamansal çakışması **NOT MEASURED**; bu ölçümler PASS sayılmaz. PIV-12 DONE; PIV-13 TODO. Faz 0 bağımsız temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN, PIV-39 tr/en i18n planı açık.


## PIV-13 — 24.10.2026 — TESTING

Fit/Fill/%100, küçük stateful header menüsü ve F6/F7/F8 eklendi. Modlar mevcut
texture üzerinden çalışır; yeni yükleme Fit'e döner, pencere modları bağımsızdır.
%100 = bir kaynak piksel / GTK mantıksal birimi; fiziksel HiDPI 1:1 iddiası yok.
59 Rust + 30 Python ve 21 scoped GUI/helper çağrısı PASS; üç sentetik render
çağrısında toplam 144 snapshot PASS, gerçek decoder kanıtı değildir. Sekiz
secure kapı Codex'te exit77/NOT RUN; native pozitif yollar PASS sayılmadı.
Tek native komut: `bash tests/run_view_modes_probe.sh`.
[Ayrıntılı matris ve native kabul](PIV13_VALIDATION.md). PIV-01–12 DONE korunur,
PIV-13 DONE değildir. PIV-14, serbest zoom/pan ve video modu başlatılmadı.
Faz 0 temiz kurulum, DESKTOP-06 ve PIV-39 tr/en i18n kendi kapsamlarında açık.
