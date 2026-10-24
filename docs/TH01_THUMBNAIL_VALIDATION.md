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
child256MiB address space/1s CPU/sıfır regular-file output bütçesi vardır.
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
