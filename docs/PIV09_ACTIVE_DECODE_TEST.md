# PIV-09 — gerçek secure decode sırasında kapanış kapısı

Durum: **TESTING**. Yeni testin native pozitif yolu henüz kullanıcı tarafından
çalıştırılmadı. Codex'te baseline Bwrap başlangıcı BLOCKED/exit77; bu belge PASS
kaydı değildir. Mevcut `image_gui_probe` için kullanıcının `7d1acf4` üzerindeki
Arch/GNOME Wayland exit0 bildirimi takip tablosunda ayrı kaydedilmiştir.

## Arch Linux / GNOME Wayland komutu

Mevcut çalışan Rust/GTK/Glycin ortamını kullanın; sudo, yeni paket, sandbox
gevşetmesi, Codex aktivasyon betiği veya loader config değişikliği gerekmez.
Aşağıdaki komutlar deponun kökünde, gerçek GNOME Wayland oturumunda çalışır:

```sh
set -e
test "$(git branch --show-current)" = dev
git pull --ff-only origin dev
git rev-parse HEAD
cargo build --locked --example active_decode_close_probe --target-dir target
GDK_BACKEND=wayland python3 -B tests/active_decode_close.py \
  --binary target/debug/examples/active_decode_close_probe \
  --report /tmp/pivra-active-decode-close.json
```

Sonuç JSON'u: `/tmp/pivra-active-decode-close.json`. **Exit0 ve result=PASS birlikte**
başarılı kapıdır. Exit77 / NOT RUN gözlem veya decode kapısı sağlanamadı demektir;
exit1 FAIL'dir. Test güvenlik ayarı değiştirmeyi denemez. Python optimizasyonu
(-O / PYTHONOPTIMIZE) assertion'ları kaldırdığı için reddedilir.

## Aktif decode kanıtı

Sadece thread başlatılması, `load()` çağrılması veya loader PID'sinin görünmesi
aktif decode kabul edilmez. Test şu koşulların tamamını şart koşar:

1. Aynı süreçte ikinci bir gerçek üretim view'ı önce `wide.png` dosyasını güvenli
   Glycin üzerinden yükler; gerçek texture byte'ları saklanır. Bu baseline decode
   başarısızsa test exit77 ile biter; hiçbir aktif-decode PASS'i üretilmez.
2. Standart kütüphaneyle geçici, geçerli RGB8 PNG üretilir: 8000×4000 / 32 MP,
   tüm pikseller (17,83,191), tüm scanline filtreleri 0. Compressed dosya 64 MiB
   sınırının altındadır; 96 MB çıktı mevcut üretim limitlerine uygundur. Dosya
   test sonunda silinir; fixture manifesti değişmez ve başka decoder kullanılmaz.
3. Python yalnız kendi başlattığı probe'un yeni descendant süreçlerini izler.
   Aynı UID'li, gerçek executable adı `glycin-image-rs` olan loader aranır.
   PID starttime + pidfd ile kimlik sabitlenir; sinyal öncesi descendant üyeliği ve
   kimlik tekrar kontrol edilir. Başka oturum/process'lere sinyal gönderilmez.
4. Loader'ın `NoNewPrivs=1`, `Seccomp=2`, `CapEff=0` ve probe'dan ayrı user,
   mount, PID ve network namespace'leri doğrulanır. Üretim loader yolu yine
   `SandboxSelector::Bwrap` kullanır; hiçbir config/protokol/loader patch'i yoktur.
5. `/proc/<pid>/fd` üzerinden yalnız read-only açılan `memfd:glycin-frame` çıktısı
   tam 96 MB olmalı; ilk 96 byte beklenen RGB olmalı, son 96 byte hâlâ sıfır
   olmalıdır. Geçerli kaynakta sıfır kanal yoktur: ilk bölge yazılmış, son bölge
   henüz yazılmamış demektir. Yalnız boş allocate edilmiş veya bitmiş frame geçmez.
6. Tamamlanma yarışını kaldırmak için **yalnız bu loader** pidfd ile SIGSTOP alır.
   Tüm thread'lerin T durumuna girmesi beklenir; dosyanın loader tarafından hâlâ
   aynı inode ile sahiplenildiği, aynı boyutta writable/shared mapping'inin hâlâ
   loader'da bulunduğu ve çıktının hâlâ kısmi olduğu tekrar kontrol edilir.
   Böylece gözlemcinin açık tuttuğu ama decoder'ın çoktan bıraktığı bir buffer
   aktif decode sanılamaz. Gözlem izni yoksa veya yarış kaçırılırsa NOT RUN olur.
7. Rust tarafı kapanış checkpoint'inde gerçek secure `load()` işine girildiğini,
   henüz dönmediğini, paintable'ın boş ve loading durumunun sürdüğünü bağımsız
   doğrular. Sonra gerçek pencerenin `close()` metodu çalışır; üretim close-request
   handler'ı kendi GIO Cancellable'ını iptal eder.

Bu yaklaşımın kaynak temeli: resmi GNOME Glycin 2.0.0 commit
`724f18c5f3519885995428a7e2e6e5df9ca2ab27` içindeki
`glycin-utils/src/image_rs.rs`: `SharedMemory::new` ardından
`decoder.read_image(&mut memory)`, sonra texture aktarımı. Ortak shared-memory
kaynağı `glycin-common/src/shared_memory.rs` buffer'ı `glycin-frame` adıyla oluşturur.
Test yalnız gözlenen uyumlu RGB image-rs düzenini kabul eder; başka loader veya
bellek formatında bu kanıt yöntemi desteklenmiyorsa PASS yerine NOT RUN üretir.
Native Arch loader kaynak/ABI'sinin tüm sürümleri için uyumluluk iddiası değildir.

## Kapanış, callback ve temizlik ölçümü

Kapanış bildirimi gelir gelmez gözlemcinin memfd referansı kapanır ve loader'a
SIGCONT gönderilir (iptal zaten süreci öldürmüşse ProcessLookupError normaldir).
Normal iptal mekanizmasının çalışması beklenir; başarı yolunda SIGTERM/SIGKILL
fallback'i kullanılmaz. Hata durumunda test prosesinin sonlandırılması PASS sayılmaz.

Üretim `image_view` için küçük dahili `build_with_completion` gözlem noktası
vardır: gerçek GIO worker JoinHandle await edildikten sonra, mevcut cancellation
ve weak-ref guard'larından önce GTK context'inde bildirim yapar. Normal `build`
no-op callback kullanır; yükleme/iptal/sandbox davranışı değişmez. Probe'un
loader wrapper'ı yalnız giriş/dönüş bilgisini tutar ve aynı üretim secure loader'ı
await eder; decoder veya sonuç taklidi yapmaz.

Başarı için kapanış isteğinden itibaren **10 saniye içinde**:

- secure load dönmeli ve gerçek GIO job'u GTK context'inde join edilmiş olmalı;
- kapalı window/picture/status weak ref'leri bırakılmış olmalı;
- kapanıştan sonraki paintable/label-text/label-visible mutation sayısı sıfır olmalı;
  join sonrasında 300 ms daha main context işlenir;
- diğer pencere görünür kalmalı; yüklenmiş texture byte'ları aynen korunmalı;
- sabitlenen loader ve gözlenen yeni helper PID/starttime kimlikleri `/proc`'tan
  kaybolmalı (zombie kabul edilmez); baseline dışı yeni descendant kalmamalı;
- probe normal exit0 vermeli ve stderr boş olmalıdır. Crash/panic/timeout FAIL'dir.

Kalıcı GIO thread-pool thread'lerinin OS seviyesinde yok olması beklenmez;
ölçülen worker işi join'idir. Gelecekteki tüm callback'lerin yokluğu matematiksel
olarak iddia edilmez: gerçek result completion, weak-ref bırakılması ve belirtilen
300 ms gözlem aralığı test edilir.

Bu, **gerçek, kısmen tamamlanmış bir decoder işleminin kontrollü pause altında
iptal testidir**. Doğal, hiç duraklatılmamış decode zamanlaması veya latency
benchmark'ı değildir. JSON stop/close/resume/settle zamanlarını ve cleanup süresini
kaydeder. Önce yüklenmiş ikinci pencere kullanıldığı için **iki decoder'ın çalışma
aralıklarının zamansal çakışması ölçülmez**; JSON'da NOT MEASURED kalır.

## Ortam sınırları ve regresyon

Linux `/proc` gözlemi ve Python pidfd API'leri gerekir. Kendi namespace'ine ait
izinli memfd/maps/namespace gözlemleri engelleniyorsa host güvenliği değiştirilmez,
root/ptrace yetkisi istenmez: test NOT RUN kalır. SIGSTOP/SIGCONT yalnız testin
kendi loader'ına zamanlama kontrolüdür; sandbox yetkisi veya erişimi genişletmez.

`tests/test_active_decode_observer.py` altı negatif/pozitif gözlemci guard testi
çalıştırır (gerçek decoder PASS'i değildir); core `tests/check.sh` içine eklenmiştir.
Üretim Cargo.toml/Cargo.lock ve mevcut fixture/pixel assertion'ları korunmuştur.
Codex JSON kaydı: [PIV09_ACTIVE_DECODE_CODEX.json](probes/PIV09_ACTIVE_DECODE_CODEX.json).
Codex'te 49 Rust + 6 Python guard PASS; fmt/check/build/Clippy PASS. Gerçek yeni
probe yalnız baseline decode kapısına kadar çalışabildi ve exit77 verdi.
**PIV-09 TESTING; aktif decode kapanışının native sonucu henüz yok. PIV-10 başlamadı.**
