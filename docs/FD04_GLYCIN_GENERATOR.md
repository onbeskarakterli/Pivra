# FD-04 — Secure Glycin bellek içi thumbnail adaptörü

25.10.2026 — **TESTING: kod/kontrollü testler tamamlandı; native kabul NOT RUN.**
Başlangıç temiz/güncel dev: `df5b088c2be7dff70e215a08cd6803eebff946c0`.
FD-03 kod referansı `4f5c7d8af687d77ff15735f937340a97e4073309` korunur.

- Adaptör, minimum Error uzantısı ve testler:
  `f1dccab956e4699c18eae82fecf9a15a185ec946`.
- Native probe/script: `efdda35f8d8ebefdadc57f75efb0454fd07fc263`.

Referanslar: [FD-01](FD01_GNOME_BASELINE.md),
[FD-02](FD02_PROVIDER_INTERFACE.md), [FD-03](FD03_FREEDESKTOP_CACHE_READER.md).

## Mimari ve korunan sınır

`src/freedesktop_generator.rs::generate(&Request, SizeClass)` yalnız worker üzerinde
çağrılacak bağımsız, blocking API'dir. GTK penceresi, şerit veya provider seçimi
oluşturmaz. Kendi thread/pool/queue'su yoktur. Üretim akışına bağlanmadı.

Yeni modül **aynı `src/image_loader.rs` dosyasını path modülü olarak derler** ve
mevcut `preload` girişini kullanır. Boundary kopyalanmadı/değiştirilmedi:
Glycin 3.0.8, zorunlu Bwrap, apply_transformations(true), expose_base_dir(false),
JPEG/PNG MIME kısıtı ve mevcut dosya/frame sınırları aynıdır. Lib'deki private
`extern crate self as pivra` alias'ı, bu dosyanın mevcut crate referansını korur;
normal binary loader'ının çağrıları değişmez. Shared foreground fonksiyonu bu
modülde kullanılmadığından yeni path modülünde dead_code izni gerekçelidir.

Public API her zaman private SecureDecoder kullanır. Private Decoder seam ve
sentetik texture implementasyonları yalnız kontrollü testler içindir; kullanıcı
veya üretim için seçilebilir fallback/unsandboxed mode yoktur.

**Bu erken/reduced-resolution preview değildir.** Mevcut güvenli tam decode
beklenir, sonra host üzerinde yalnız çözülmüş pikseller resample edilir.
Glycin scale isteğinin küçük decode garanti ettiği varsayılmadı; bu API deneyi
tekrarlanmadı. Kaynak dosya için yeni JPEG/PNG parser/decoder eklenmedi.

## Bellek içi çıktı ve FD-05 bağlantısı

Candidate mevcut cache dosyasının kimliğini ve PNG verisini gerektirir. Henüz
cache dosyası olmadığı için sahte Candidate/metadata veya geçici thumbnail PNG
üretilmez. Minimum ayrı `GeneratedThumbnail` şunları taşır:

- mevcut kaynak FileKey, istenen FD-03 SizeClass;
- width/height, tight stride, immutable GLib Bytes içinde RGBA8 pikseller;
- gerçek decode boyutu/frame byte sayısı, download/output byte sayıları;
- monotonic decode ve resize süreleri.

FD-05 ileride FileKey'i yeniden doğrulayıp kaynak yolundan GIO URI ve güncel
mtime/size metadata'sını çıkarmalıdır. Dosya değişmişse bu eski pikselleri güncel
metadata ile kaydetmemelidir. Atomik PNG/cache yazımı ve failure marker üretimi
**uygulanmadı**. Native testin özel alpha PNG'si önceden kodlanmış TEST kaynak
fixture'ıdır; adaptör çıktısının cache'e kaydı değildir.

## Boyutlandırma, EXIF, renk/alfa

Dört sınıfın maksimum kenarları 128/256/512/1024'tür. Küçük görüntü büyütülmez;
büyük görüntünün bütün alanı oran korunarak küçültülür, crop/stretch yoktur.
Integer boyutlarda floor/max(1) nedeniyle oran yuvarlaması en çok bir pikseldir.
Sıfır, taşma, aşırı edge/pixel ve budget ihlalleri reddedilir.

EXIF dönüşümünü yalnız mevcut Glycin loader uygular; ikinci rotation/flip yoktur.
GDK TextureDownloader güvenli decoded texture'ı **unpremultiplied RGBA8/sRGB**'ye
çevirir. GdkPixbuf bilinear yalnız bu piksel buffer'ını ölçekler; dosya decode etmez.
Sonuç tight RGBA satırlarına sınırlı kopyalanır. Alfa korunur; tamamen saydam pikselin
görünmeyen RGB değeri korunacağı iddia edilmez. HDR/geniş gamut/yüksek bit derinliği
çıktıda 8-bit sRGB'ye dönüşür; kayıpsız yüksek hassasiyet iddiası yoktur. Ana fotoğrafın
renk/viewport/render tercihleri ve pipeline'ı değiştirilmedi.

## Kaynak, bellek ve iş sınırları

- Mevcut kaynak sınırı 64 MiB; ortak loader ayrıca 32 MP / 32768 edge / 512 MiB
  frame üst sınırını korur.
- Adaptör **256 MiB conservative decode accounting (32 byte/pixel)** ile mevcut
  preload girişini kullanır: yaklaşık **8.39 MP** üzeri kaynaklar frame decode
  edilmeden LimitExceeded alır. Bu bilinçli, ana loader'dan daha sıkı bağımsız
  thumbnail sınırıdır; örneğin 24 MP fotoğraf bu prototipte kabul edilmez.
- Beklenmeyen gerçek frame buffer >256 MiB de reddedilir. Loader'ın döndürdüğü
  frame bu kontrolden önce zaten oluşturulmuştur; decoder içi geçici allocation
  veya toplam süreç RSS'i için 256 MiB hard cap iddia edilmez.
- RGBA download en fazla64 MiB; kabul edilmiş boyutlar normalde <=32 MiB tight
  RGBA gerektirir. Kaynak büyüklüğünde ayrıca gereksiz Vec kopyası yapılmaz:
  mevcut frame/texture referansı → gerekli format download'u → Pixbuf byte reference.
- Nihai çıktı en fazla1024*1024*4 =4 MiB; output boyutlandırma sırasında row/control
  ve stride/buffer/overflow kontrolleri yapılır. Future strip integration mevcut
  64 texture/16 MiB bütçesine ayrıca admission/LRU uygulamalıdır.
- Standalone admission fail-fast en çok2 aktif iş ve dev/inode bazlı tekilleştirme
  uygular; hardlink ve symlink alias'ları aynı gerçek dosyaya çakışamaz. Busy için
  kuyruk/retry oluşturulmaz. RAII tüm error/cancel yollarında kaydı bırakır.

Bu admission **mevcut strip worker permit'inin yerine geçmez**. Henüz bağlı
olmadığından uygulama genelinde birleşik2-worker garantisi sertifikalandırılmadı.
FD-06 aynı mevcut global permit'i paylaşmalı; iki bağımsız limitin toplamını
uygulamada artırmamalıdır. Normal fotoğraf açılışı bu adaptörü beklemez/çağırmaz.

## Kimlik, iptal/deadline ve hatalar

Request/FileKey/Cancellable yeniden kullanılır. Dosya kimliği decode öncesi,
preflight sonrası, decode sonrası, resampling/download sonrası ve yayın öncesi
karşılaştırılır. Kaynak silinmesi SourceUnavailable, değişim SourceChanged olur.
Kaynak symlink'i desteklenir; farklı hedefe retarget/replacement FileKey
karşılaştırmasıyla reddedilir. Admission gerçek dev/inode'u kullanır.
Kaynaklarda write/chmod/delete yoktur; fixture manipülasyonları özel test dizinindedir.

İlk8 byte GIO ile sadece signature kabulü için okunur; uzantıya güvenilmez.
JPEG/PNG decode kararı ve doğrulaması mevcut secure loader'dadır. Bu ön kontrol
bozuk görüntünün decoder'da başarılı olduğunu kanıtlamaz.

Her iş için child Cancellable ve worker-owned GLib MainContext oluşturulur. Parent
iptali child'a iletilir. GLib deadline source'u decode sürerken child'ı iptal eder;
millisecond süre **yukarı** yuvarlanır ve callback gerçek Instant deadline'ını
tekrar kontrol eder. User cancel timeout'a önceliklidir. Timer/cancel handler RAII
ile kaldırılır; parent token başarı/timeout nedeniyle iptal edilmez. Sonuç için
süre/kimlik yeniden kontrol edilir. Decode future'ı iptali cevaplayıp cleanup
olana kadar await edilir; işi arkada bırakan thread timeout/fallback yoktur.

Senkron filesystem/GDK resampling çağrıları preempt edilmez; dönüşte kontrol
edilerek geç sonuç yayımlanmaz. Mutlak wall-clock termination veya gerçek loader
cleanup garantisi bu ortamda ölçülmedi; native aktif cancellation hâlâ açıktır.

FD-02 Error'e yalnız SourceChanged, LimitExceeded(String), Busy eklendi. Bunlar
önceki modelde ayrı ifade edilemiyordu; geniş trait/model yeniden tasarımı yoktur.
Cancelled/TimedOut/SourceUnavailable/InvalidSource/GenerationFailed korunur.
ProviderUnavailable/ProviderTemporarilyUnavailable bir dosyanın decode hatasından
üretilmez. Unsupported/truncated signature ve kanıtlanmış yanlış MIME InvalidSource;
bilinen boundary budget mesajları exact match ile LimitExceeded olur.

**Sınıflandırma sınırı:** korunan image_loader ayrıntılı Glycin hatasını String
olarak verir. Tanınan bir PNG/JPEG header'ı sonrası bozuk payload ile decoder
altyapısı hatasını her durumda güvenilir ayırmak mümkün değildir. Bunlar özgün
hata ayrıntısı korunarak GenerationFailed olur; uydurma provider arızası/uygunluk
kararı verilmez. Güvenilir daha ayrıntılı sınıflandırma ileride boundary'den typed
hata gerektirir; bu görevde korunan üretim boundary'si değiştirilmedi.

## Test matrisi — gerçek sonuçlar

Mevcut SDK image-activate.sh, Rust1.99.1/GTK4.18.6/adw1.7.6 kullanıldı.

| Kontrol | Sonuç |
|---|---|
| `cargo test --locked --offline --lib freedesktop_generator::tests -j2` | **17 PASS**, kontrollü model/GDK/GIO; gerçek decoder başarısı değil |
| `cargo check --locked --offline -j2` | PASS |
| `cargo test --locked --offline -j2` | **145 PASS**, 0 fail/ignored:125 önceki +17 yeni + aynı paylaşılan loader'ın3 tekrar testi |
| `cargo clippy --locked --offline --all-targets -j2 -- -D warnings` | PASS |
| Native probe ve mevcut thumbnail overlay probe build | PASS |
| Thumbnail Python suite | **37 PASS**, kaynakları değiştirilmedi |
| Xvfb/X11 katalog | PASS:120 girişim/120 tekil başarı/0 iptal; kontrollü pikseller, GTK/GIO click/scroll/iki pencere |
| Sway/Wayland katalog + cancel race | PASS:122 girişim=120 başarı+2 iptal; kontrollü pikseller, gerçek GTK/Wayland; fiziksel GNOME testi değil |
| `bash tests/run_freedesktop_generator_probe.sh` | **NOT RUN / exit77**, tek giriş denemesi; Bwrap loader process exit1, geçerli PNG content-only MIME image/png |
| Yeni adaptörle gerçek JPEG/PNG/EXIF1–8/alpha/renk, native frame/time/RSS ölçümü | **NOT RUN**, ön koşulda duruldu; sonraki native dosyalar denenmedi |
| Yeni adaptörün gerçek aktif decoder cancellation/child cleanup/window-close entegrasyonu | **NOT RUN / doğrulanmadı**; fake worker temizliği native cleanup değildir; window wiring FD-06 kapsamı |
| `cargo fmt --check` | **FAIL, önceden mevcut**: image_view.rs:447, active_decode_close_probe.rs:211, application_open_probe.rs:65/83/97/135 |
| Yeni/değişen Rust dosyalarının hedefli rustfmt kontrolü, shell syntax, git diff --check | PASS |

17 kontrollü test dört sınıf/yatay/dikey/kare/küçük/büyük; rounding/overflow/budget;
RGBA/alfa quadrant gerçek postdecode resampling; source delete/replace/symlink;
iptal önceliği; çalışan fake future'ın deadline ile iptali/temizliği; admission
hardlink/dedup/count/drop; iki gerçekten aktif kontrollü worker'da tek isteğin
iptali ve diğerinin başarıyla devam etmesini kapsar. Hiçbir fake EXIF decode
başarısı iddia edilmez; gerçek EXIF native probe'da ayrıca sınanacaktır.

Native denemesi logunda secure loader Bwrap komutu exit1 ile sonlandı; ilk decode
başarılamadı. Bilinen Codex namespace/proc engelini aşacak değişiklik yapılmadı,
Bwrap tekrar döngüsü veya bypass yoktur. Bu log tek başına yeni bir uid-map teşhisi
iddiası değildir. Önceki Arch test sonuçları yeni adaptörün native kabulü sayılmaz.

## Arch için tek native komut

```bash
bash tests/run_freedesktop_generator_probe.sh
```

Probe kendi test worker'ında public secure generate girişini kullanır. İlk geçerli
PNG prerequisite sonrası JPEG/PNG, dört sınıf/large.png, portrait/tiny, EXIF1–8 gerçek
renk köşeleri, özel alpha PNG, recognizable corrupt JPEG/PNG hataları, bağımsız
sonuç ve kaynak kimliği kontrollerini çalıştırır. Sonuç/decoded boyutu, buffer
byte'ları, monotonic süreler ve Linux parent VmRSS/VmHWM basılır. Parent RSS,
loader child RSS'i veya tüm işin peak RAM'i değildir; tek ölçüm genel hızlandırma
iddiası değildir. Temp alpha/corrupt kaynakları test sonunda temizlenir;
kalıcı/shared thumbnail veya failure marker yazılmaz. Bu headless worker probe'u
fiziksel GNOME pencere/fare kabulü değildir. Aktif native cancellation/cleanup
sertifikalandırmadığını çıktısında da açıkça belirtir.

## Kapsam sonu

src/thumbnail.rs, thumbnail_strip.rs, image_loader.rs, image_view.rs, window.rs,
GNOME sidecar, build.rs, FD-03 okuyucusu ve Cargo.toml/lock başlangıçla aynı
(git diff --exit-code doğrulandı). GNOME üretim akışı ve mevcut source/cache
algoritmaları değişmedi. Yeni bağımlılık/framework/decoder veya üçüncü taraf
uygulama kodu kopyası yok; proje lisansı/paketleme kararları değişmedi.

FD-04 **TESTING** kalır: native renk/EXIF/güvenli decode ve gerçek cleanup kanıtı
olmadan DONE denmez. FD-05–FD-09, aktif Provider Manager, XDG cache writer,
paketleme/yayın/AUR ve main değişikliği yapılmadı.
