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
bash tests/run_active_decode_close.sh
```

Önce `dev` checkout'unu normal fast-forward pull ile güncelleyin. Bu tek betik dev
branch kontrolü yapar, gerçek HEAD'i yazar, `cargo build --locked` ile eşleşen
Rust example'ı `target` altında derler ve GDK_BACKEND=wayland ile Python sürücüsünü
başlatır. Böylece eski binary/yeni driver protokol karışıklığı önlenir.

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
   PID starttime + pidfd ile kimlik örnekleme başlamadan sabitlenir; descendant
   üyeliği ve kimlik bu pin aşamasında tekrar kontrol edilir. Sinyal pidfd üzerinden
   yalnız o süreç kimliğine gider; STOP sonrasında starttime yeniden doğrulanır. Başka oturum/process'lere sinyal gönderilmez.
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

`tests/test_active_decode_observer.py` 17 negatif/pozitif gözlemci guard testi
çalıştırır (gerçek decoder PASS'i değildir); core `tests/check.sh` içine eklenmiştir.
Üretim Cargo.toml/Cargo.lock ve mevcut fixture/pixel assertion'ları korunmuştur.
Önceki sürücü Codex JSON kaydı (tarihçe): [PIV09_ACTIVE_DECODE_CODEX.json](probes/PIV09_ACTIVE_DECODE_CODEX.json).
Codex'te 49 Rust + 17 Python guard PASS; fmt/check/build/Clippy PASS. Gerçek yeni
probe yalnız baseline decode kapısına kadar çalışabildi ve exit77 verdi.
**PIV-09 TESTING; aktif decode kapanışının native sonucu henüz yok. PIV-10 başlamadı.**


## 21.10.2026 — native NOT RUN teşhisi ve gözlemci v2

**Kullanıcı tarafından aktarılan Arch kanıtı; Codex tarafından bağımsız üretilmedi:**
Glycin 2.2.3 / Bwrap 0.13.1 ortamında gerçek image-rs loader 8.285 kez örneklendi,
memfd 30 kez görüldü; `/proc` veya Python erişim hatası olmadı ve pidfd mevcuttu.
96.000.000 byte buffer ve mapping görüldü. Tek ilk örneğin ilk 12 byte'ı
`1153bf1153bf1153bf1153bf`, son 12 byte'ı sıfırdı. Bu biçim beklentisini destekler;
tam 96-byte prefix/tail ve STOP sonrası sahiplik/mapping/worker koşulları aynı
checkpoint'te doğrulanmadığından yeni aktif-decode PASS'i değildir.

Önceki sürücüde kesin görülen maliyet/kör aralıklar:

- TARGET_STARTED, worker'ın başlatıldığı `view()` ve `present()` dönüşünden sonra
  yazılıyordu. Gözlemci bu bildirimi beklediğinden decode daha önce başlayabiliyordu.
- Her poll tam descendant/executable/fd keşfi yapıyor, sıra set sırasına bağlı
  kalıyordu. Tek kaybolan descriptor hatası kalan fd'lerin taranmasını atlatabiliyordu.
- Kısmi adaydan sonra pidfd açımı, yeniden tam ağaç taraması ve namespace/status
  okumaları yapılıyor, SIGSTOP daha sonra gönderiliyordu.
- Boolean retler hangi koşulun eksik kaldığını saklıyordu. İlk 12 byte ile sürücünün
  istediği tam 96 byte koşulları birbirinden ayrıştırılamıyordu.

Bunlar kod incelemesi bulgularıdır; kullanıcının exit77 sonucunun kesin tek nedeni
eldeki verilerle belirlenmedi. Sürüm/ABI uyumsuzluğu veya proc yasağı ilan edilmedi.

Yeni protokol: survivor gerçek baseline decode yaptıktan sonra START gönderilir;
Rust **TARGET_STARTING** yazar ve **ARMED** bekler. Gözlemci hazır olmadan target
view/loader oluşturulamaz. Bu yalnız test başlatma bariyeridir; üretim decoder'ı
yavaşlatılmaz veya değiştirilmez.

PID'ler deterministik sırayla keşfedilir, pidfd bir kez pin edilir. Geçici fd
kaybolmaları tek fd düzeyinde ele alınır. Yanlış boyutlu memfd açık tutulup doğru
frame'i gizlemez. Tam boyuttaki çıktı fd'si read-only cache edilir; ardından yalnız
bu fd yaklaşık 0,2 ms poll aralığıyla örneklenir, süreç ağacı tekrar taranmaz. Bu
sleep hedefidir; gerçek scheduler/okuma süreleri için JSON örnek zamanları ve
`max_sample_gap_seconds` incelenmelidir; sabit gerçek örnekleme frekansı iddiası yoktur.

Kısmi adayda SIGSTOP hemen gönderilir. Namespace/status/ownership/mapping kontrolleri
**durduktan sonra**, aynı sıkı koşullarla yapılır. Tanı stdout yazımları tüm canlı
örnekleme aralığı boyunca bufferlanır ve sonuçta basılır; aday→STOP yolunda stdout
yazımı yoktur. PID kimliği, tüm thread'lerin T durumu, tam 96-byte
prefix ve tail, fd/inode/writable-shared mapping, namespace/NoNewPrivs/seccomp/CapEff,
Rust load pending ve sonraki tüm kapanış/temizlik ölçütleri aynen zorunludur.

JSON ve ilk-geçiş DIAG satırları: baseline_decoded, target_starting, observer_armed,
loader_found, pidfd_pinned, memfd_found, size_correct, prefix_correct, tail_zero,
partial_candidate, stop_requested/stop_complete, fd_owned, mapping_verified,
sandbox_verified, checkpoint_certified ve kapanış/join/cleanup aşamalarını ayırır.
Sayaçlar her gözlemi sayar, stdout yalnız ilk geçişleri basar. İlk/son örnek ve son
kısmi aday 96-byte hex, fd/PID/inode ve örnek başlangıç/bitiş zamanlarını içerir.
`phase=candidate_before_stop` durdurulmuş atomik kanıt değildir; yalnız `phase=stopped`
ile yeniden kontrol ve tüm sonraki kapılar nihai PASS'e götürebilir. Ayrı örneklerde
prefix ve tail başarıları birleştirilemez. `first_uncompleted_stage`, `failed_stage`
(varsa) ve son örnek, geçişin nerede durduğunu gösterir. Ham bağımsız koşul sayaçları
tek başlarına başarı sertifikası değildir.

17 unit test içinde STOP sırası, tamamlanma yarışı, PID reuse, mapping/sandbox
reddi, izin reddi, stop timeout, 12/96-byte farkı, farklı örneklerin birleştirilmemesi,
kaybolan erken fd'nin sonraki frame'i gizlememesi ve cache poll sırasında ağaç
keşfinin tekrarlanmaması vardır. Bu unit senaryoları gerçek decode taklidiyle native
PASS üretmez; yalnız gözlemci kontrol akışını sınar.

Güncel Codex kanıtı: [observer v2 JSON](probes/PIV09_ACTIVE_DECODE_OBSERVER_V2_CODEX.json).
Baseline secure loader başlamadığı için `first_uncompleted_stage=baseline_decoded`,
exit77/NOT RUN. Native v2 checkpoint/close yolu henüz çalıştırılmadı. İki decoder'ın
zamansal çakışması NOT MEASURED kalır. **PIV-09 TESTING; PIV-10 başlamadı.**
