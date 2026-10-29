# PIV-16 — cancellable async image loading

24.10.2026 · `dev` · **DONE**, gerçek Arch native ve kullanıcı manuel kabulü tamamlandı.
Başlangıç: `d9a127f482041a7f0a6994d9427de45f94c4784b`, origin/dev fast-forward.
PIV-01–15 DONE ve kullanıcı kaynaklı native kanıtları korunur; yeni görev için
Codex'te tekrar üretilmiş veya yeni native PASS olarak kullanılmaz.

## Mimari incelemesi ve gerekli değişiklikler

Mevcut worker mimarisi korundu: gerçek Glycin yüklemesi `gio::spawn_blocking`
içinde worker'a ait GLib context'te çalışır; GTK güncellemeleri ana context'tedir.
Bwrap, loader protokolü, görüntü formatları ve kaynak limitleri değiştirilmedi.

Önceki LoadSlot zaten her yeni istekte yeni `gio::Cancellable` oluşturuyor,
öncekini iptal ediyor ve geç callback'i token kimliğiyle reddediyordu. İki eksik
bulundu: hızlı girişte worker sayısı sınırsız artabiliyordu; `on_result` observer'ı
GTK mutasyonlarından önce çalıştığından reentrant yeni yükleme/close eski
completion'ın kalan UI güncellemelerini tehlikeli yapabiliyordu.

Yeni küçük `image_requests.rs` admission/cancellation bileşeni:

- **Pencere/ImageView başına en fazla 3 admitted worker ve 1 pending istek.**
  Yuva doluysa yalnız son istek tutulur; eski pending closure ve timer bırakılır,
  henüz worker başlatılmadığı için decoder işi üretilmez.
- Pending için **5 saniyelik GLib admission deadline** vardır. Önceki işler
  kooperatif olarak bitmiyorsa yeni istek sonsuz spinner'da tutulmaz; güncel
  isteğe kontrollü “Image loading is busy. Please try again shortly.” hatası
  verilir. Otomatik retry yoktur. Bu **worker/decoder kill timeout'u değildir**;
  main-context dispatch gecikmesi ayrıca olabilir. İptali dikkate almayan üç iş
  bitene veya süreç kapanana kadar kaynak rezervasyonları korunur; zorla thread/
  sandbox öldürülmez. Kullanıcı daha sonra yeniden istekte bulunabilir.
- Yeni seçim/mesaj/close/destroy current ve tüm admitted cancellable'lara iptal
  gönderir; pending timer/closure bırakılır. Her admitted job'un join sonucu
  işlendiğinde rezervasyon kaldırılır ve yalnız hâlâ güncel pending başlatılır.
  Yinelenmiş/bilinmeyen finish token'ı yuvayı açamaz veya yeni işi başlatamaz.
- İstek kimliği dosya adı veya tamsayı değildir: canlı GObject token kimliği +
  cancelled durumu kullanılır. Eski callback kendi token'ını güçlü tuttuğundan
  canlı eski token ile yeni istek aynı kimlik olamaz. A→B→A ayrıdır; sayısal
  sayaçların saturating olması kimliği etkilemez/taşma yaratmaz.
- Ordinary `load` aynı path için **hâlâ aynı current iş çalışıyor/bekliyorsa**
  tekrar decode başlatmaz. Bu cache veya istek kimliği değildir. Explicit
  Yenile/`load_observed`/completion girişleri her zaman yeni token üretir;
  değişmiş aynı dosya tekrar yüklenebilir. Tamamlanmış fotoğraflar cache'lenmez.
- `on_completion` yalnız gerçekten admitted ve join olmuş işler için çağrılır;
  pending timeout/drop için worker tamamlandı diye işaretlenmez. `on_result`
  güncel UI güncellemelerinin **sonrasında**, son çağrı olarak çalışır. Böylece
  observer yeni yükleme başlatırsa eski completion sonraki spinner'ı silemez.
- Kapanışta cache'in texture referansı da bırakılır; task'lar widget'ları weak
  referanslarla tutar. Stale frame/error UI'ye ve texture dönüşümüne ulaşmadan
  düşer. Fit/Fill/Actual ve güncel Smooth/Nearest callback anında okunur;
  eski callback mode/filter/zoom/pan durumunu geri alamaz.

Eşlenmiş GIO rename sırasında hâlâ pending/running olan **JPEG/PNG** isteği yeni
supported ada retarget edilir, önceki token iptal edilir; mode ve filter korunur.
Tamamlanmış görüntü sırf adı değişti diye otomatik decode edilmez. Atomic save
(temp→seçili path) otomatik yeniden decode başlatmaz, mevcut Yenile akışı korunur.
Diğer katalog/unsupported politikaları ve `X candidates` arayüzü değiştirilmedi.

`image_loader.rs` yalnız private `LoadedFrame = glycin::Frame` type alias'ı ve
ona eşdeğer dönüş imzasıyla değişti. Decoder/sandbox uygulaması aynı kaldı.
Alias kontrollü probe'nin UI success dalını sentetik frame ile sınamasına izin
verir; **üretim binary'sinde test enum'u, fake decoder veya fallback yoktur**.

## Ölçülebilir sınırlar ve kanıt ayrımı

`ImageView::load_stats()` probe diagnostics: running/pending, started/completed,
max_running, dropped_pending, suppressed, timed_out. Bunlar **admitted GIO job**
sayılarıdır; gerçek decoder PID sayısı, CPU süresi veya pool stop kanıtı değildir.
Ürün arayüzüne diagnostics/status toolbar eklenmedi.

Glycin 3.0.8 kaynak incelemesi: cancellable isteğe bağlanır; cancel handler ve
Image Drop request-path'e `process.use_().done(path)` gönderir. UsageTracker
pool kullanımını bırakır; global pool varsayılan idle retention **30 saniyedir**.
Bu nedenle worker join, Frame drop ve token cancel tek başına pooled loader
PID'sinin hemen çıkması demek değildir. Aynı pool başka pencerenin işini de
sunabilir; uygulama process-global pool'u veya diğer pencerenin token'ını iptal
etmez. Güvenli aktif decoder/process cleanup kapısı PIV-09'un mevcut gözlemcisidir.

## Gerçek test sonuçları

| Kontrol | Sonuç |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo check --locked`, `cargo build --locked` | PASS, mevcut stable/offline/-j2 helper |
| `cargo test --locked` | **67 PASS** = 43 unit + 24 integration, 0 failed/ignored |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| Tüm örneklerin locked/offline derlenmesi | PASS |
| Python aktif decoder gözlemci unit testleri | **30 PASS**, gerçek decoder başarısı değildir |
| Eski fixture/manifest bütünlüğü | PASS, byte'lar değiştirilmedi |
| GTK/GIO/helper çağrıları | **33 scoped PASS**: X11 19, Sway 12, Weston 2 |
| Gerçek secure decoder kapıları | **11 NOT RUN / exit77** |
| Shell/Python syntax, diff whitespace | PASS |

### 1. GIO model/unit

Üç yeni admission unit testi: farklı token/aynı-path ABA kavramı ve pencere
bağımsızlığı; 3+1 kapasite ve yalnız son pending başlatılması; duplicate finish
reddi; timeout'un completion sayılmaması, close/pending iptali. Test timer'ı
kontrollü sıfır deadline ile GLib context'te dispatch edilir; ordering yalnız
sleep gecikmesine bırakılmaz. İstek kimliği için 10.000 token replacement kontrolü
vardır; önceki LoadSlot testleri korunur.

### 2. Gerçek GTK/GIO, kontrollü success/error ve sentetik texture

`async_cancel_probe --controlled`, gerçek üretim ImageView/admission/callback
kodunu gerçek GTK pencereleri/GIO worker'larıyla kullanır. **Test backend'i**
iptali bilerek bekletir; bırakma barrier'ları işlerin dönüş sırasını belirler.
Success payload'u düz renkli sentetik frame'dir; eski success/error'un üzerine
gelememesi gerçek GSK snapshot merkez pikselinden de kontrol edilir.

- A/B/C worker'ları başlar; C blue success önce, eski A success/B error sonra:
  yalnız blue piksel, eski hata/spinner değişmez. Stale success için texture
  üretilmediği constructor sayısıyla ayrıca doğrulanır.
- A/B/A aynı path, farklı token ve farklı sentetik payload: son A green kalır;
  geç ilk A/B mode/manual zoom ve filtreyi değiştiremez.
- Üç noncooperative worker tutulurken **1000** farklı istek: started artmaz,
  pending yalnız 1; kapasite açıldığında yalnız `queued-999` başlar. Ölçülen
  max_running=3. GLib idle callback işlerin bitmesini beklemeden çalışır.
- Result observer yeni yükleme başlatır: yeni Picture boş/loading visible kalır,
  eski completion yeni spinner'ı silemez. Ardından yeni success görüntülenir.
- Aynı dosya iki ayrı pencerede yüklenir; birinin close/cancel'ı diğerini iptal
  etmez. Normal aynı-active-path `load` tekrarları worker başlatmaz.
- **14 admitted iş join edilir**, yalnız **6** güncel success texture'a çevrilir;
  iptal sinyalleri, 0 running/pending, weak widget/window bırakma doğrulanır.

`--controlled --queue-timeout`: gerçek beş-saniyelik admission timer'ı ile
kontrollü busy UI hatası; running=3/pending=0/timed_out=1, worker completion=0,
retry yok; close sonraki pending'i başlatmadan bırakır, eski üç job bırakılıp
join edildiğinde 0/0 olur ve texture üretilmez. Deadline başarısı **decoder stop
PASS'ı değildir**. Bu ikisi X11 ve Sway'de toplam dört çağrıdır.

`window_failure_race_probe --rename-pending` X11/Sway'de gerçek özel dosya
rename'i ve GIO event'iyle pending JPEG/PNG'nin yeni path'e yönelmesini,
mode/filter korunmasını, eski path hatasının geç gelince bastırılmasını,
tamamlanmış dosyanın sonraki rename'inde decode olmamasını ve bağımsız pencereyi
sınar. Worker sonucu burada da kontrollü hatadır, gerçek decoder değildir.

Eski 27 GUI/helper çağrısı korunur; bu altı yeni çağrıyla **33** olur. Eski
window/monitor yarış testi artık cancelled iki barrier'ı admission yuvası açmak
için explicit bırakır; eski hataların ters tamamlanması/silinme/Refresh/close
assertion'ları korunur. Video/unsupported geçişi, hızlı gerçek X11 tuşları,
Yenile/monitor/bozuk medya, mode/zoom/pan/resampling ve eski **462 sentetik
snapshot** regresyonu PASS. Yeni controlled success center snapshot'ları da
sentetiktir. Bunlar Glycin JPEG/PNG/EXIF PASS'ı olarak raporlanmaz.

Sway gerçek headless Wayland'dir (DISPLAY/XWayland yok); sinyaller fiziksel GNOME
kullanıcı kabulü değildir. Weston mapped surface PASS; normal close **NOT RUN**.
Idle dispatch canlılığı, job cap ve GObject lifetime ölçüldü; hardware GPU,
FPS/latency/process RSS benchmark'ı **NOT RUN / NOT MEASURED**.

### 3. Gerçek secure decode / kaynak temizliği

Ayrı girişlerle image_probe PNG/JPEG-first, image_gui_probe, strict navigation,
strict folder watch, strict medya hatası, active_decode_close, strict view_modes,
strict zoom/pan, strict resampling ve yeni strict async_cancel_probe denendi:
**11 exit77 / NOT RUN**. İlk secure baseline oluşmadı; bilinen Codex Bwrap /
read-only proc UID-map engeli sürer. Native pozitif yollar yalnız derlendi.

Şu ayrım korunur: GIO token cancel gözlendi; stale UI suppression ve kontrollü
worker join/weak cleanup PASS; **aktif gerçek decoder stop ve sandbox/child
process temizliği bu ortamda NOT RUN**. Eski kullanıcı PIV-09 memfd checkpoint
kabulü korunur; yeni kaynakta tekrar çalıştırılmış sayılmaz. PIV-09 Python
protokolü/gözlemci başarı koşulları değiştirilmedi; Rust wrapper yalnız aynı
LoadedFrame alias'ını sağlar. Sandbox gevşetilmedi, NotSandboxed/bypass yoktur.

## Native Arch/GNOME kabulü — tek komut

Güncel `dev` üzerinde gerçek GNOME Wayland oturumunda:

```bash
bash tests/run_async_cancel_probe.sh
```

HEAD yazdırılır, locked build yapılır, GDK_BACKEND=wayland kullanılır. Sırasıyla:

1. Mevcut image_probe gerçek JPEG/PNG/EXIF 1–8 piksel kapısı.
2. Yeni async_cancel_probe **varsayılan secure modu**: aynı dosyada iki pencere,
   101 istekli A/B burst ardından son C'nin boyut ve gerçek quadrant pikselleri,
   max3/0 pending drain, A/B/A, pending mode/filter seçimi, bozuk→sağlam,
   bir pencere kapanırken diğerinin yüklemeye devamı ve kaynak byte bütünlüğü.
3. Değiştirilmemiş PIV-09 `active_decode_close.py` sürücüsü + güncel Rust probe:
   gerçek partial memfd/pidfd/SIGSTOP sonrası mapping/identity checkpoint,
   aktif secure close, join/weak cleanup ve çocuk process kontrolü.

Son JSON: `/tmp/pivra-async-active-close.json`. Tüm aşamalar exit0 ve JSON
`result=PASS` olmadan bütün native kapı PASS değildir; exit77 bir engel/NOT RUN,
diğer sıfır dışı sonuç FAIL'dir. Betik controlled/synthetic mod çalıştırmaz ve
kullanıcı fotoğraflarına yazmaz. İlk başarısız aşamada durur; sonraki aşamalar
çalışmış sayılmaz. Native async burst decoder'ın tam aktif zaman aralığını veya
iki decoder overlap'ini sertifikalandırmaz; üçüncü aşamadaki **mevcut PIV-09
checkpoint** aktif decoder kanıtıdır. Yeni wrapper'da yalnız program başladı diye
PASS yapılmaz.

Kısa manuel kabul: iki pencereyle hızlı Sol/Sağ ve A→B→A, değişen fotoğrafta
Yenile→gezinme→Yenile, silinmiş/bozuk/video öğesinden devam; yükleme sırasında
F6/F7/F8 ve Smooth/Nearest, ardından zoom/pan; bir pencereyi yükleme sürerken
kapatıp diğerinin aynı dosyayı açmaya devam etmesini gözleyin. `X candidates`
arayüz düzenlemesi bu görevde yapılmadı.

**PIV-16 TESTING**, native SHA/sonuç ve kullanıcı kabulü olmadan DONE değil.
PIV-17 ön yükleme/cache, PIV-18 preview/full-res, yeni format/filtre/video/edit/
i18n/paketleme/CI başlamadı. PIV-01–15 DONE; Faz 0 temiz kurulum INCOMPLETE,
DESKTOP-06 Nautilus NOT RUN ve PIV-39 tr/en i18n planı ayrı açık işlerdir.

## Gerçek kaynak commit'leri

| SHA | İçerik |
|---|---|
| `636498cb123cf92cc723f432251704c3b7566793` | 3+1 admission/deadline, unique cancellable identity/stats, duplicate-load coalescing, close cache release, observer reentrancy ve pending rename düzeltmesi; eski probe compatibility/barrier uyarlaması |
| `d9c93ebc104bdda9ea6b7ee3171ef7707e3b0062` | Controlled success/error/pixel ve timeout probe'ları, secure async native yolu, PIV-09 checkpoint'i içeren tek native giriş, Sway argümanları |


## PIV-16 native ve manuel kapanış — 24.10.2026 — DONE

Kullanıcı gerçek Arch/GNOME ortamında `0822e4b2028d414304ef3b2078bc5bc6d6da8852` üzerinde `bash tests/run_async_cancel_probe.sh` çalıştırdı: **exit 0**. Gerçek JPEG/PNG/EXIF baseline PASS; secure async latest-pixel, A→B→A, hata toparlanma, iki pencere ve kaynak bütünlüğü PASS. `LoadStats`: running=0, pending=0, started=11, completed=11, max_running=3, dropped_pending=97, suppressed=6, timed_out=0. Ek PIV-09 aktif secure decode close checkpoint'inde partial memfd/frame, pidfd/mapping/sandbox, worker join, stale mutation=0, survivor pixels ve child cleanup PASS. `Cleanup SIGCONT unavailable: No such process` süreç zaten çıktığı için fatal değildi; toplam test exit0. Burst içinde decoder active interval/overlap **NOT MEASURED**. Kullanıcı gerçek uygulamada hızlı Sol/Sağ, A→B→A ve yükleme sırasında görüntüleme tercihlerini denedikten sonra olumlu manuel yanıt verdi; ayrıntılı ayrı ölçüm iddiası yok. **PIV-16 DONE; PIV-17 TODO.** Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 i18n planı açık.
