# PIV-18 / TH-04 regresyonları ve TH-05 — TESTING

25.10.2026 · yalnız dev. Başlangıç `fe7b6f7b56671999aef2604f96333f4202767cfd`.
Temiz çalışma ağacı, origin/dev'den 24 commit fast-forward ile güncellendi.
Geri alınmış deneme yamaları yeniden uygulanmadı; main değiştirilmedi.

## Kök neden — kısa teknik rapor

Eski akış: thumbnail click → `window.rs` select → `display_selected()` →
`update_thumbnail_catalog()` → `set_catalog()` → bütün GTK düğmelerini kaldır/
yeniden oluştur → eski işi iptal et/yedi dosyayı tekrar sorgula → yeniden hizala.

1. **Scroll/odak:** Tıklanan düğme varsayılan olarak odak alıyordu. Seçim callback'i
   aynı düğmeyi kaldırıp yeni bir widget ağacı kuruyordu. GTK odak kaydırması da
   bu ağaç değişimine katılıyordu. Başlangıç koduyla ayrı bir geçici GTK kontrolünde
   odaklanmış103. hücreye tıklamada adjustment **10400 → yaklaşık385** oldu
   (150 ms gözlem, kayma sürerken); normal viewport758 ve content upper12472 idi.
   Yeni regresyon ayrıca eski kodda `selection rebuilt stable catalog cells`
   assertion'ıyla FAIL verdi. Kontrol kaynakları çalışma ağacına yama olarak
   uygulanmadı; bu ölçüm gerçek decoder/native GNOME testi değildir.
2. **İlk hizalama:** Üretim penceresi önce yalnız açılan dosyayı, sonra asenkron
   klasör taramasını verir. Ayrıca gizli/henüz ölçülmemiş scroller adjustment'ı
   geçici değerlere sahiptir. Eski pending hizalama bu geçici aralıkta tüketilebilir;
   nominal104px adım da ölçülmüş hücre sınırının yerine kullanılıyordu. Yeni kod
   ilk tamamlanan taramayı tek seferlik bildirir ve yalnız mapped scroller'ın gerçek
   page/son hücre bounds sertifikasından sonra seçili hücreyi hizalar. Başlangıç
   anchor'ı ile scroll-restoration birbirini dışlar; sonraki seçim bunu tekrarlamaz.
3. **Eksik yükleme:** Eski `center-3..center+4` aralığı yalnız yedi kaydı inceliyordu.
   Adjustment değişimi yeni iş istemiyordu; her seçim bütün işi iptal ediyordu ve
   sonuçlar topluca bekletiliyordu. Bu yüzden yatay kaydırma diğer öğeleri yüklemiyordu.

## Uygulanan çözüm

- `thumbnail_strip.rs`: katalog yolu aynıysa hücreler ve yatay adjustment korunur;
  yalnız seçili stil değişir. Gerçek üyelik değişiminde yollarla reconcile edilir,
  güncel global katalog indeksi hücre callback'ine aktarılır. Mouse click odak
  taşımaz; klavye odaklanabilirliği kaldırılmaz. İçerik non-measuring Overlay'de
  tutulur: gerçek thumbnail geldiğinde96×80 hücre ölçüsü değişmez.
- Hücre oluşturma ilk görünürlüğe kadar ertelenir; GTK idle başına32 hücre yapılır.
  Gizlenince yarım inşa ve işler durur. Tüm uygun fotoğraflar katalogda kalır.
- Öncelik: görünür hücreler → iki komşu → diğer pending kayıtlar. Pencere başına
  bir iş, süreç genelinde en fazla iki thumbnail işi; aynı dosya aynı anda iki kez
  işlenmez. Yeni görünür öğeye öncelik gerektiğinde eski uzak iş iptal edilir;
  eski işin sonucu dönmeden pencereye ikinci iş açılmaz. Epoch/token/path ve weak
  state kontrolleri eski sonuçların yeni kataloğa veya kapalı pencereye yazmasını önler.
- GIO/freedesktop cache ilk tercihtir. URI/MTime/Size, PNG container/CRC,
  FileKey ve source/cache kimliği kontrolleri korunur. Kaynak rescan'ında stat işleri
  worker'dadır; yalnız kimliği değişen thumbnail temizlenir. Önce katalog reconcile,
  sonra revalidation yapılır; eşzamanlı silme/değiştirme eski epoch'u tüketmez.
- Her sonuç kendi hücresine uygulanır. Görünür pending/running için spinner,
  hata için placeholder/tooltip vardır. Hata paint döngüsünde tekrar denenmez;
  yalnız hover ile gizle/göster de başarısız kaynağı yeniden admit etmez. Kaynak/
  katalog invalidation'ı tekrar değerlendirebilir. Busy retry yalnız bekleyen iş
  için kısa, iptal edilebilir one-shot'tır; boş/gizli durumda sürekli polling yoktur.
- RAM: pencere başına en fazla64 texture ve16 MiB konservatif pixel sahipliği
  (`texture_cost`) tutulur; LRU eviction widget paintable'ını da bırakır. Bütçeye
  sığmayan görünür kümeler her paint'te decode/evict döngüsüne girmez; evicted
  öğeler viewport/katalog değiştiğinde tekrar değerlendirilir. Bu RSS/VRAM ölçümü
  değildir; katalog/hücre metadata'sı O(n)'dir.120'den büyük kataloglarda gerçek
  bellek/gecikme benchmark'ı yapılmadı, sanallaştırılmış liste iddiası yoktur.
- %25 panel, opak görseller, normal hücrede çerçevesiz stil ve mevcut TH-04
  tema rengindeki2px seçili outline korunur. Bağımsız yüzen düğme/panel vardır;
  alt dock veya viewport'tan ayrılan alan eklenmez. T/auto-hide/persistent preset,
  main Glycin, PIV-16 admission ve PIV-17 prefetch/cache kodları korunur.

## GNOME üretim sınırı

`build.rs`, tek modül seçer: önce gnome-desktop-4, sonra gnome-desktop-3.0.
`native/thumbnail_factory.c` gerçek public generate/save/can_thumbnail API'siyle
-Werror compile/link edilmeden kullanılmaz; GIO/GObject/GdkPixbuf doğrudan ve doğru
link sırasıyla eklenir. Public API referansı:
[GNOME43.2 header](https://github.com/GNOME/gnome-desktop/blob/43.2/libgnome-desktop/gnome-desktop-thumbnail.h)
(generate/save cancellable/error imzaları42+). Modüller karıştırılmaz; uyumsuz/eski
API veya eksik geliştirme paketinde cache-only fallback ve açık build uyarısı vardır.

Eksik/stale cache için helper GNOME sistem factory'sinden gerçek üretim ve cache
save ister. Geçerli mevcut cache yeniden üretilmez; fail marker/invalid güvenlik
sonuçları gizlenmez. Same-second stale cache için lookup tekrar kullanımı önlenir.
Kaynak kimliği generate öncesi/sonrası ve cache save sonrası kontrol edilir;
save sırasında değişirse yalnız yeni üretilen aynı cache inode'u discard edilir.
Cache mtime generation başlangıcına damgalanır; ctime kontrolüyle birlikte kullanılır.
Bu metadata doğrulaması kriptografik içerik kimliği veya ortak cache üzerinde
atomik dosya sistemi transaction'ı değildir.

Helper limitleri:1 GiB AS,15s CPU,4 MiB dosya; parent wall timeout20s.
Özel0700 TMPDIR, process-group cancel/kill/reap ve scratch temizliği vardır.
**Rlimit sandbox değildir:** source üretimi GNOME'un kurulu thumbnail backend ve
izolasyon politikasını kullanır. Yeni bağımsız JPEG/PNG decoder eklenmedi.
Cache pikselleri ve ana fotoğraflar mevcut **zorunlu Glycin Bwrap** yolundan geçer;
NotSandboxed/fallback eklenmedi. Native paketleme helper'ı binary yanında
`pivra-thumbnail-factory` olarak sağlamalıdır; Cargo kaynak derlemesi OUT_DIR'deki
API-checked helper'ı kullanır. Paketleme/CI bu görevde uygulanmadı.

## Doğrulama ve sınırlar

| Kontrol | Sonuç ve kanıt sınıfı |
|---|---|
| fmt/check/build/test, locked/offline; all-target Clippy -D warnings | PASS —86 Rust (62 unit+24 integration); gerçek decode sonucu değildir |
|30 Python active-decode observer regresyonu ve fixture bütünlüğü | PASS — gözlemci/asset kontrolleri |
|11 cache +6 modül seçimi +5 metadata-helper +7 generation-helper Python testi | PASS —29 thumbnail testi; son7 gerçek C + GLib/GIO/GdkPixbuf, **GNOME API stub** |
| Helper kaynak değişimi generate/save sırasında, limitler, cached/stale/unsupported/error | PASS — C API stub; gerçek GNOME üretimi değildir |
| Rust helper cancel/process-group/scratch cleanup ve global iki iş/dedup limiti | PASS — kontrollü kendi child süreçleri; native GNOME child cleanup kanıtı değildir |
|120 JPEG/PNG kaydı; ilk/79./119.; singleton→full scan; dar pencere→büyütme | PASS — gerçek GTK/GIO, kontrollü thumbnail frame'leri |
| İlk120 cache işinde120 decode sınırı; görünür kayıt önce; tüm katalog progresif | PASS — gerçek GIO cache/identity + **sentetik pixel boundary** |
| Scroll→click, stable GTK objects/selection, hızlı seçim, source/cache replace/delete/corrupt, decode sırasında cache kimliği değişimi, birlikte membership/content değişimi | PASS — GTK/GIO ve kontrollü hata enjeksiyonu |
| Sabit hata hover'da admit edilmez; hidden UI inşa/decode yok; kapanış/iki pencere | PASS — kontrollü GTK olayları; fiziksel mouse değildir |
| X11 açık; X11 koyu+scale2; headless Sway/Wayland | PASS — mapped GTK/frame ve üretim window.rs seçim/başlık/Left-Right akışı |
| Eski TH-02/03 geometry/pixel/alpha/T/hide/bant ve PIV-16 async/PIV-17 preload | PASS — kontrollü frame/worker regresyonları |
| Gerçek GNOME helper derleme/link/runtime ve gerçek Glycin thumbnail pikselleri | NOT RUN — Codex'te GNOME development modülü yok; helper preflight exit77 |
| Arch GNOME fiziksel fare/klavye/görsel kabulü; gerçek RSS/VRAM/latency | NOT RUN / NOT MEASURED |

Başarısız ilk testler PASS sayılmadı: eski hücre-rebuild assertion'ı negatif
kanıttır; yeni implementation'da synchronous adjustment callback borrow guard'ı
düzeltildi. Test display/DPI kapasitesi ve resize sonrasında sentetik pointer
konumu düzeltildi. `async_cancel_probe` ve `preload_probe` önce uzaktaki mevcut
c9e16cc/4abc166/edf5e22 kararlarıyla çelişen loading-text/Fit-reset beklentilerinde
FAIL verdi: bunlar hidden loading + FullLoading ve korunmuş Actual preset olarak
assert edilir. Texture identity hâlâ ayrıca doğrulanır; hata/iptal assertion'ları
kaldırılmadı. Native renderer yerine stub kullanımı üretim uygulamasına taşınmadı.
Bilinen Codex Bwrap uid-map kısıtı tekrar denenmedi, güvenlik gevşetilmedi.

## Tek Arch GNOME komutu

```bash
bash tests/run_thumbnail_system_probe.sh
```

Komut helper API build'ini kontrol eder; yoksa açıkça77/NOT RUN ile durur. Özel
geçici config/cache ve120 test JPEG/PNG kopyasında gerçek secure decode ön koşulunu
tek kez dener. Ardından gerçek cache pikselleri, eksik JPEG/bozuk PNG cache'inin
GNOME üretimi, source replacement sonrası gerçek portrait frame/pikselleri ve GTK
senaryolarını doğrular. Thumbnailer'ın ölçek davranışı varsayılmaz; gerçek boyutlar
basılır. Sonunda iki üretim Pivra penceresini **özel TEST fixture'larıyla** açık
bırakır. --native-thumbnails modunda bu pencerelerin main ve cache decoder'ı gerçektir.
Normal kullanıcının fotoğrafları veya thumbnail/config dosyaları değiştirilmez.
İki pencere kapanınca özel kaynak/config/cache dizinleri temizlenir.

Manuel:80. öğeyle açılan şeridin başlangıç konumunu; scroll→thumbnail click'te
main fotoğraf/başlık/outline ve scroll sürekliliğini; yatay tekerlek, hızlı Sol/Sağ,
T/hover, daralt/büyüt, iki pencere ve kapanışı kontrol et. Gizli şerit Sol/Sağ ile
kendiliğinden açılmamalı; panel viewport'u daraltmamalı veya alt bant üretmemeli.
Gerçek büyük klasör görsel/performans kabulü ayrıca gereklidir.

TH-04 regresyonları ve TH-05 **TESTING**. Native sonuç olmadan DONE değildir.
PIV-18 TESTING; TH-06/TH-07 ve PIV-19 başlamadı. Faz0 temiz kurulum ve DESKTOP-06
Nautilus kapıları bu görevle kapanmaz.


## Gerçek commit'ler

- `0f9db6232afccb3a7274805e1a71834db93ad053`: opsiyonel API-checked native GNOME
  generation helper, cache-first adapter, kaynak/save guard'ları, kaynak sınırları,
  process-group/scratch cleanup ve C API negatif testleri.
- `f9283d59f0917cacf58f09cf3929748d125d10ff`: stable GTK projection/initial scan
  anchor, progresif queue/LRU, hidden-work ve retry/epoch politikası;120 kayıtlı
  controlled/native probe, üretim pencere testleri ve tek Arch komutu.

Yeni bağımsız decoder/Rust bağımlılığı eklenmedi; Cargo.lock değişmedi.
Source metadata/GTK bookkeeping maliyeti ve gerçek GNOME üretim güvenliği/
performansı için native kabul açıktır. Süreç RAM'i16 MiB ile sınırlı iddiası yoktur;
16 MiB yalnız thumbnail texture sahipliği bütçesidir.
