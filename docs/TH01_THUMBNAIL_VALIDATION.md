# PIV-18 / TH-01 — GNOME thumbnail sağlayıcısı — TESTING

25.10.2026 · dev. Başlangıç `1624264aa72a9b27825853b812fb17f69e2e2451`,
çalışma ağacı temiz, origin/dev ile eşit. Yalnız TH-01 uygulandı.
PIV-18 TESTING; PIV-19 başlamadı. Kullanıcı native kabulü beklenir.

## Kapsam ve mimari

`src/thumbnail.rs`: GIO metadata/cache adaptörü ve kontrollü sonuç modeli.
Mevcut `FileKey` (path/device/inode/size/nanosecond mtime/ctime) yeniden kullanılır.
GIO blocking işi `query_on_worker` üzerinden mevcut blocking pool'a verilebilir;
`query` yalnız worker kullanımı içindir. Provider başına1 aktif sorgu, pending
kuyruk yok; eşzamanlı istek Busy döner. GTK mutation, timer polling veya katalog
scan yoktur. `Candidate::still_current` teslim öncesi source/cache kimliğini
ve cancellation'ı yeniden kontrol eder; bu point-in-time kanıttır.

`window::FolderWatch` olayları/revision, mevcut doğal katalog seçimi ve request
cancellable gelecekte tüketici tarafından kullanılabilir. Bu aşamada window'a
provider bağlantısı eklenmedi. `ImageView`, `preload`, `image_requests`,
`image_loader`, `settings`, Cargo.toml/lock değişmedi. PIV-17 full-resolution
texture LRU'su ile thumbnail cache birbirinden tamamen ayrıdır: bu provider
texture tutmaz, disk cache üretmez, yalnız sınırlı PNG baytlarını geri verir.
Normal uygulama henüz provider çağırmaz; performans kazanımı iddiası yoktur.

## Doğrulanan gerçek API'ler

- GIO `GFile.query_info`: `thumbnail::path`, `thumbnail::is-valid`,
  `thumbnail::failed`, `standard::content-type`, `time::modified`.
  Yerel GLib header/GIR ve [GLib2.84.4 glocalfileinfo.c](https://github.com/GNOME/glib/blob/2.84.4/gio/glocalfileinfo.c)
  ile doğrulandı. Path byte-string olarak alınır; filename UTF8 olma şartı yoktur.
- [GLib thumbnail-verify.c](https://github.com/GNOME/glib/blob/2.84.4/gio/thumbnail-verify.c)
  standardın Thumb::URI ve Thumb::MTime alanlarını zorunlu, Thumb::Size'ı varsa
  eşleşmesi gereken alan olarak doğrular. URI/MTime parser'ı yeniden yazılmadı.
  MD5 filename eşleşmesi tek başına kabul ölçütü değildir.
- GNOME library capability API:
  `gnome_desktop_thumbnail_factory_new(GNOME_DESKTOP_THUMBNAIL_SIZE_NORMAL)` ve
  `gnome_desktop_thumbnail_factory_can_thumbnail(factory, uri, mime, mtime)`.
  [Gerçek GNOME header](https://github.com/GNOME/gnome-desktop/blob/master/libgnome-desktop/gnome-desktop-thumbnail.h)
  ve [uygulaması](https://github.com/GNOME/gnome-desktop/blob/master/libgnome-desktop/gnome-desktop-thumbnail.c)
  incelendi. can_thumbnail kayıtlı/etkin thumbnailer ve failed marker durumunu
  kontrol eder, yalnız **üretim denenebilir** anlamına gelir; gerçek üretim PASS
  değildir. TH-01 helper generate/save/queue çağırmaz.

GNOME factory için yeni Rust FFI/ABI tahmini yapılmadı: küçük C helper gerçek
kurulu development header ve pkg-config modülüyle derlenir; yalnız0/1 üretir.
Native betik kurulu `gnome-desktop*` modülünü listeden bulur, paket kurmaz.
Codex'te bu library/typelib bulunmadı. Gerçek D-Bus thumbnailer servisi bu ortamda
kanıtlanmadı; servis adı/metot imzası uydurulmadı. Bu sürüm D-Bus adapter eklemez.
GNOME factory mevcutsa onun standart API'si kullanılır; değilse GIO cache-only.
C helper bu aşamanın native doğrulama adaptörüdür; paketlenmiş/kurulmuş ürün
helper'ı iddiası yoktur. Gelecekte UI tüketicisine helper dağıtımı ayrı karardır.

## Yetenek ve sonuç modeli

| Durum | Koşul | Tüketici davranışı |
|---|---|---|
| Unavailable | GNOME desktop token'ı yok | Thumbnail kontrolünü sunma |
| CacheOnly | GNOME GIO cache backend mevcut, doğrulanmış generation yeteneği yok | Var olan geçerli kayıt kullanılabilir; generation talebi yok |
| CacheAndGenerate | Gerçek factory helper en az bir uygun inputta can_thumbnail=true bildirdi | Kaynağa özel generation_eligible de true ise daha sonraki TH-05 isteğine uygun |
| TemporarilyFailed | Daha önce başarılı factory probe sonradan hata/timeout verdi | Cache sorgusu devam eder, generation_eligible=false; mevcut fotoğraf etkilenmez |

`generation_eligible` son sorgulanan dosyaya aittir; global capability ile aynı
şey değildir. Başka MIME/failed marker için false olması, daha önce bilinen
factory yeteneğini yok etmez. İlk sorgudan önce CacheOnly ihtiyatlı varsayımdır.
Factory bulunmaması ile boş cache aynı olay değildir: boş cache Missing döner,
GNOME'da CacheOnly kalır. GNOME dışı masaüstleri desteklenmiyor. Bu aşamada
kontrol/şerit henüz yoktur; yukarıdaki görünürlük politikası TH-02 tüketicisi
sözleşmesidir, uygulanmış UI davranışı değildir. Enabled tercih kodlanmadı.

Outcome: Hit(Candidate), Missing, FailedCached, Stale, Invalid(reason),
SourceUnavailable(reason), Cancelled, Busy, UnsupportedDesktop. FailedCached
standart geçerli failed marker'dır; sonsuz yeniden deneme başlatılmaz.

## Cache kabul koşulları ve güvenlik

1. Yerel source regular file olmalı, FileKey alınır; doğru file URI GIO'dan gelir.
2. GIO thumbnail metadata parser'ı path-only sorguda bile dosyayı mmap eder.
   Bu yüzden önce standart cache konumları (normal/large/x-large/xx-large ve
   fail/gnome-thumbnail-factory) MD5 URI adıyla **yalnız resource preflight** için
   kontrol edilir. Her mevcut aday regular, symlink olmayan ve≤2MiB olmalıdır.
   Bu işlem standart validity API'sinin yerine geçmez.
3. Seçilen cache≤2MiB read bound ile okunur. PNG signature, IHDR boyutu,
   edge1–1024, chunk bounds/count≤2048, CRC, IDAT varlığı ve terminal IEND aranır.
   Bu bir PNG decoder değildir; IDAT inflate/piksel çözümleme yapılmaz.
4. Gerçek GIO is-valid=true ve aynı cache path zorunludur. URI, MTime, varsa Size
   GIO tarafından karşılaştırılır. Sonunda source/cache FileKey tekrar eşleşir.
5. Source ctime(ns), cache mtime(ns)'dan yeni ise Stale: aynı path/inode değişimi,
   korunmuş saniye mtime/size nedeniyle eski thumbnail kabul edilmez. Bu
   muhafazakâr ek kontrol bazı cache'leri reddedebilir; wall-clock sapması veya
   timestamp'ı korunarak taşınmış cache için false negative mümkündür.

Hit **yalnız metadata + container doğrulanmış Candidate** demektir; decode veya
görsel güvenilirliği PASS değildir. CRC'si yeniden hesaplanmış bozuk IDAT gibi
piksel hataları metadata katmanında kanıtlanamaz. Sonraki aşama piksel tüketmeden
önce güvenli decoder ve güncel identity/token kontrollerini uygulamak zorundadır.
Bu görevde hiçbir thumbnail GTK'de gösterilmez. Source dosyası/cache silinmez,
yazılmaz veya thumbnail üretimi başlatılmaz.

Source/cache yarışlarında stale sonuç teslim edilmez; daha sonra tüketici
still_current'ı yeniden çağırmalıdır. GIO'nun native metadata parser'ının kendi
süresi/allocation davranışı provider tarafından bütünüyle kontrol edilmez;
stat→GIO arasındaki aynı-kullanıcı dosya değişimi mutlak resource izolasyonu
sağlamaz. Preflight ve tekrar kimlik kontrolü bu riski azaltır; GIO standardına
ve yerel cache backend'ine güvenilir. Source metadata I/O hard realtime değildir.

Factory helper güvenilir, açıkça verilmiş executable'dır; dosya yolu/URI shell'e
interpole edilmez, ayrı argv kullanılır.≤16KiB URI/256 byte MIME,1s parent timeout,
child256MiB address space/1s CPU/dosya başına1MiB regular-file büyüme bütçesi vardır.
Helper stdout yalnız32 byte okunur; cancellation/timeout metadata child'ını
kill/wait eder, Pivra/Glycin decoder süreçlerine dokunmaz. GNOME factory kendi
failed cache doğrulamasında native GdkPixbuf kullanabilir; bu **Glycin Bwrap
izolasyonuyla eşdeğer değildir**. Factory probe kaynak fotoğrafı decode etmez
ve generation çağırmaz; gerçek generation güvenlik/öncelik değerlendirmesi TH-05
ön koşuludur. Sistem thumbnail servisinin güvenliği otomatik onaylanmadı.

## Gerçek test kanıtları

- PASS: 5 yeni Rust unit: dört capability, PNG corruption/bounds/CRC/cancel,
  candidate replace/rename/cache delete, Busy/no queue, başarılı factory-mock
  sonrası kayıpta TemporarilyFailed ve cache sorgusunun devam etmesi.
- PASS: **11 Python/GIO entegrasyon testi**. Özel temp dizin/XDG cache ve GNOME
  desktop token fixture'ı; gerçek GIO URI/MTime/Size validation. Hit, boş cache,
  stale/wrong URI/Size, CRC corruption, source delete, rename, aynı path/same
  mtime atomic replacement, unsupported desktop, failed marker, symlink ve
  oversized cache. MIME helper mock/native GNOME session değildir.
- PASS: `bash tests/check.sh` mevcut activation/stable/locked/offline/-j2 ile
  bir kez: fmt/check/build/test, all-targets Clippy -D warnings; **80 Rust =56
  unit+24 integration**,30 Python observer, fixture/manifest bütünlüğü.
  Mevcut native rapor sürücüsünün3 Python testi de PASS: toplam **44 Python**.
- PASS: probe build, shell syntax, whitespace. Normal app üretim dosyalarında
  fark yoktur. Thumbnail işi hiçbir normal window'a bağlanmadığından GUI/native
  decoder paketleri ve bilinen Bwrap engeli tekrar çalıştırılmadı.
- Ortam probe'u gerçek Codex desktop token'ında Unavailable/UnsupportedDesktop
  verdi; GNOME factory pkg-config modülü yok. Factory helper'ın gerçek header
  ile derlemesi/çalışması ve gerçek CacheAndGenerate **NOT RUN**. Bu sonuç
  CacheOnly/GNOME test fixture'larıyla karıştırılmadı. Native pixel decode yok.

## Arch GNOME doğrulaması

GNOME terminalinde mevcut fotoğraf için:

```bash
bash tests/run_thumbnail_provider_probe.sh /tam/yol/fotograf.jpg
```

Kaynak cache'ini önceden Nautilus gibi mevcut sistem uygulaması üretmişse bu
komut var olan cache'i sorgular. Source/temp test cache oluşturmaz. Factory
module mevcutsa C helper kurulu header/library ile Werror derlenir; yoksa cache
adaptörü çalışır. Derleme hatası PASS veya servis yokmuş gibi gizlenmez. Hiçbir
paket yüklenmez. GNOME token yoksa Unavailable doğrudur; cache boşken Missing
başarısız decode anlamına gelmez. Çıktı gerçek capability, eligibility, outcome,
URI/path ve Hit durumunda byte count/current bilgisini verir. Exit0 gözlemin
bittiğini belirtir; thumbnail generation/decode/native kabul PASS değildir.

Beklenen native kabul: gerçek GNOME factory derlemesi ve capability; güncel
cache Hit (metadata), boş cache Missing, kaynak değişimiyle eski cache Stale,
absence durumunda kontrollü CacheOnly. Gerçek generation henüz çalıştırılmaz.
**TH-01 TESTING**, kullanıcı kabulü beklenir. TH-02 overlay, TH-03 T/kalıcı
Enabled, TH-04 seçim senkronizasyonu, TH-05 üretim/öncelik ve TH-06 UI/native
matrisi bu görevde kodlanmadı. Viewport, zoom/pan/filter değişmedi.

## Commit'ler

- `c8eb5bdf011cae2e86e3c4448df62ed173590038`: provider/cache/capability modeli.
- `88f3e94f484ced10df9fb32d9ec36425571ce722`: gerçek GIO testleri ve native factory probe.

## Native modül seçimi/SIGPIPE düzeltmesi — 25.10.2026 — TESTING

Kullanıcı Arch GNOME'da eski listing→awk erken exit pipeline'ında141 bildirdi.
Exit kaldırılınca iki modül aynı değişkene geçti: gnome-desktop-3.0 ve
 gnome-desktop-4. Bu çıktı tek bir seçilmiş/uyumlu module kanıtı değildi.
Düzeltme SHA `1443a4762d5d8db852b4d0d027bfb1872ddf5c69`.

Betik `set -euo pipefail` korur. pkg-config listing'i önce tamamen alınır;
sonraki awk tüm veriyi tüketir. Tekilleştirilmiş adaylar version-descending
sıralanır (4, sonra3.0), her adayın flags ve helper compile/link sonucu ayrı
kontrol edilir. İlk Werror derlemesi başarılı olan **tek** module seçilir.
Başarısız aday tanısıyla sonraki adaya geçilir. Listing hatası/modül yokluğu/
hiçbir adayın derlenememesi açık cache-only fallback mesajı verir; paket kurulmaz.
Derleme hataları sessiz PASS veya gerçek factory kabulü sayılmaz.

### CacheOnly/Missing teşhisi

- **initial CacheOnly normal varsayılandır:** Provider kurulurken henüz kaynak
  MIME/factory yeteneği sınanmamıştır. Bu sistem generation yapamaz iddiası değildir.
- Sorguda source identity/metadata başarılıysa ve helper configured ise,
  **cache bulunmasa da** helper URI/MIME/mtime ile çağrılır. Missing daha sonra
  cache path yokluğunu bildirir. Dolayısıyla boş cache tek başına CacheOnly
  nedeni değildir; can_thumbnail=true + Missing mümkündür.
- **CacheAndGenerate** yalnız gerçek helper exit0 ve tam `1\n` cevabı
  (`can_thumbnail=true`) gözlendiğinde verilir. `generation_eligible` dosyaya
  özel sonucu bildirir. Sadece library bulunması/derlenmesi yeterli değildir.
- `can_thumbnail=false`: factory o URI/MIME/mtime için üretim denenebilirliği
  bildirmedi. Etkin thumbnailer kaydı olmaması, ayarlarla devre dışı olması,
  geçerli failed marker veya cache içinde bir input gibi kaynaklara bağlıdır.
  API false dönüşü tek başına bunlardan hangisini kanıtlamaz.
- İlk helper hatası/timeout da henüz doğrulanmış generator olmadığından CacheOnly
  bırakır. Önceden başarılı factory sonradan kaybolursa TemporarilyFailed olur.
  Yeni `FactoryObservation` bu durumları **NotConfigured / NotAttempted /
  MimeUnavailable / Eligible{mime} / Ineligible{mime} / Failed{mime,error}** olarak
  ayrı gösterir. Child exit/stdout hatası artık teşhis bilgisinde bulunur.
- Kullanıcının eski CacheOnly/Missing/exit0 çıktısı helper'ın çağrıldığını veya
  true/false/başlatma hatasını ayırt ettirmiyordu. Native neden henüz belirlenmedi;
  yetenek zorla yükseltilmedi, yeni çıktı ile yeniden doğrulanacak. Exit0 yine
  yalnız metadata gözleminin tamamlanmasıdır, generation PASS değildir.

PASS:6 yeni driver regresyonu; listing pipe buffer'dan büyük, iki modülde yalnız4
seçimi;4 compile fail→3.0 fallback; hiç modül yok; tüm compile'lar fail; listing
hatası; helper false→CacheOnly/Ineligible + Missing. Fake pkg-config/compiler ve
factory stub kullanılır; gerçek Rust provider çalışır. GNOME native kabulü değildir.
Rust mevcut factory testine Eligible→Ineligible→Failed ayrımı ve geçici hata
assertion'ları eklendi. 11 gerçek GIO cache entegrasyon testi PASS.
Bir tam core80 Rust/30 Python,3 rapor testi,6 yeni driver+11 GIO testi:
**80 Rust +50 Python PASS**; fmt/check/build/test/all-target Clippy -D warnings,
fixture/hash, shell syntax ve whitespace PASS.

Codex ortamında factory module yok ve GNOME desktop token yok: gerçek ortam
probe'u Unavailable / NotConfigured / UnsupportedDesktop verdi. Gerçek GNOME
module compile/helper can_thumbnail sonucu **NOT RUN**; Bwrap/decode denenmedi.
Üretim Glycin/ImageView/cache/preload/viewport davranışı değişmedi.

Arch GNOME'da tek yeniden doğrulama komutu:

```bash
bash tests/run_thumbnail_provider_probe.sh "$PWD/tests/fixtures/images/large.png"
```

Beklenen çıktı tek `GNOME factory module:` satırında yalnız bir module adı;
sonrasında capability, generation_eligible, factory observation ve outcome.
Gerçek4 derlenemezse tanıdan sonra3.0 seçilebilir; ikisi de yoksa kontrollü
fallback. Factory=Failed ise hata/exit, Ineligible ise MIME ile birlikte raporlanır.
**TH-01 TESTING**, native kullanıcı kabulü beklenir; TH-02 başlamadı.


## GNOME native link ve SIGXFSZ düzeltmesi — 25.10.2026 — TESTING

Kullanıcı Arch'ta gnome-desktop-4 ile doğrudan `g_object_unref` link hatası,
helper'da SIGXFSZ25 bildirdi. Düzeltme:
`ccced8089f425a07d6580c70570e725ac2296e14`.

### Bağımlılık/API uyumluluğu

Güncel gerçek GNOME header'daki factory_new/can_thumbnail imzaları ve
[GNOME meson pkg-config tanımı](https://github.com/GNOME/gnome-desktop/blob/master/libgnome-desktop/meson.build)
incelendi: gnome-desktop-4 public requires listesi GSettings schemas içerir;
helper'ın doğrudan kullandığı GObject sembolünün link satırında bulunacağı
varsayılamaz. Doğrudan bağımlılık **gobject-2.0** aynı kurulu pkg-config sistemi
üzerinden seçilen tek GNOME module ile birlikte sorgulanır. Compiler komutunda
source/object önce, seçilen GNOME library ve GObject/GLib library bayrakları
sonradır; transitive DSO bağımlılığına güvenilmez. GNOME3 ve4 flag'leri birleştirilmez.
Gerçek kurulu header ile gerçek helper derlenir; diğer ABI'den header/library
ödünç alınmaz. İlk derlenebilir tek aday politikası ve pipefail korunur.

### Dosya sınırı ve kanıtın sınırı

Eski RLIMIT_FSIZE=0 herhangi bir regular-file/memfd büyümesini engelliyordu.
GNOME factory constructor GSettings başlatır ve thumbnailer yapılandırmasını
okur; runtime/IPC küçük dosya veya shared-memory fd gerektirebilir. Bağımsız
stub factory'de gerçek memfd+ftruncate(4096) ile eski0 ceiling altında kernel
SIGXFSZ25 tekrar üretildi. **Bu, kullanıcının GNOME sürecinde hangi dosyanın
sinyali tetiklediğinin syscall trace'i değildir**; o ayrıntı NOT RUN kalır.

Current/hard FSIZE **1MiB per-file**: küçük metadata/IPC için pay, sınırsız
write değil. Bu toplam filesystem kotası değildir. SIGXFSZ ignore edilmedi;
limiti aşan gerçek işlem yine Failed olabilir. Generation/save çağrısı yoktur.
AS256MiB, CPU1s, parent timeout1s, cancellable ve child kill/wait korumaları
aynı. Factory helper'ın güvenliği hâlâ Glycin Bwrap ile eşdeğer sayılmaz.
Gerçek runtime1MiB'tan fazlasını gerektirirse failure gizlenmez; ek native
kanıt olmadan limit tekrar artırılmaz.

### Test sonuçları

- **PASS:**5 gerçek C/helper testi. Sistem GObject ile Werror compile/link;
  yalnız GNOME API fonksiyonları stub. Sıfır limit→SIGXFSZ yeniden üretimi;
  mevcut limitler altında4KiB memfd başarılı ve kaynak limitleri korunmuş;
  stdout1/0 korunur; factory_new NULL→exit2; gerçek provider helper exit2'yi
  FactoryObservation::Failed olarak tutar, Missing ve CacheOnly ile dönebilir.
- **PASS:**6 module-driver testi; tek4 seçimi, compile fail→3.0, link satırında
  source sonrası GObject/GLib sırası, controlled fallback ve false sonucunun
  korunması. pkg-config/compiler mock'ları gerçek GNOME ABI testi değildir.
- **PASS:**11 GIO cache entegrasyon,5 Rust provider unit, fmt --check, shell
  syntax ve git diff --check. Toplam **5 Rust +22 Python** hedefli regresyon.
  Üretim Rust/viewer değişmediğinden büyük core/GUI paketleri tekrar koşulmadı.
- **NOT RUN:** gerçek Arch GNOME factory library compile/run, signal-free
  initialization ve gerçek can_thumbnail sonucu. Codex'te library yoktur;
  buradaki stub başarıları native kabul yerine geçmez. Bwrap denenmedi.

Tek Arch komutu değişmedi:
`bash tests/run_thumbnail_provider_probe.sh "$PWD/tests/fixtures/images/large.png"`.
Beklenen: tek uyumlu module ve gerçek factory=Eligible veya Ineligible.
Gerçek hatada Failed korunur; Missing cache boşluğudur, tek başına hata değildir.
**TH-01 TESTING; kullanıcı native kabulü beklenir. TH-02 başlamadı.**
