# PIV-18 — preview/full pipeline — TESTING

24.10.2026 · dev. Başlangıç `07f47b5fb21200d8337933893128b700e5b8329d`;
origin/dev fetch ve fast-forward kontrolü: zaten güncel, çalışma dizini temizdi.
PIV-01–17 DONE, son kullanıcının görev talimatındaki durum beyanına göre korunur.
İlk yerel tracking ref PIV-17 TESTING gösteriyordu. Push öncesi explicit dev
fetch ile `5046419` dahil PIV-17 kullanıcı native/manuel kabul kayıtları ve kenar
navigasyon tasarımı normal merge ile korundu. Bunlar Codex native sonucu değildir.

## Güncel kapanış denetimi — 25.10.2026

**PIV-18 TESTING; Faz 2 kapanışı henüz onaylanmadı.** TH-07 kullanıcı native
ve fiziksel kabulüyle DONE; TH-03 bağımsız native restart/kalıcılık kabulü açık.
Full-only yolunun native başarısı gerçek erken preview'yi tamamlamaz. Reddedilen
PIV-18C yöntemleri geri getirilmedi. Erken preview'nin sonraki sürüme ertelenmesi
hâlâ kullanıcı kapsam kararı gerektirir; bu rapor erteleme onayı değildir.

Madde10 için ayrıca somut güncel bulgu: load_request status metnini boşaltıp
hidden yapıyor; FullLoading yalnız model state'i, spinner'lar thumbnail içindir.
Başlık+retained eski fotoğraf ve dosya adlı hata mesajı var, aktif ana-fotoğraf
yükleme göstergesi yok. TH-06'da kaldırılan Loading yazısı geri getirilmedi.
Aşağıdaki24.10 tarihli “loading mesajı korunur” cümlesi tarihsel ilk uygulamadır;
güncel source davranışı bu denetim kaydıyla ayrılır. Gereksinimi tamamlanmış
saymak için gösterge tasarımı veya mevcut davranışın açık kabulü gerekir.

Kaynak snapshot `0d13d24449047e3d23a0e15133c47e2ca44ca86b`: Codex88 Rust,
62 Python, fixture bütünlüğü ve12 kontrollü GTK/compositor probe PASS. Tüm Rust
hedefleri de21 harness/274 test icrası ile PASS (ortak modül tekrarları dahil). Yeni
Codex secure decode testi NOT RUN; aynı kaynağın bilinen ortam engeli tekrar
çalıştırılmadı. Kullanıcı Arch kabulü ve Codex kontrollü sonuçları ayrı tutulur.
[Faz 2 / 0.1.0 ayrıntılı hazırlık ve açık kararlar](PHASE2_RELEASE_READINESS.md).

## API kanıtı ve uygulanabilir yol

İncelenen client: Cargo registry **glycin-3.0.8/src/api_loader.rs**:
`Image::specific_frame` dokümanı loader'ın istek alanlarını yok sayabileceğini
söyler; `FrameRequest::scale` yalnız protokol isteğini ayarlar. `next_frame`
sabit görüntü için yalnız bir kez çağrılabilir. Boyutlar `Image::details`,
orientation `transformation_orientation` ile erişilebilir. Mevcut loader
`apply_transformations(true)` kullanır; ikinci EXIF uygulaması eklenmedi.
Cancellable istek, frame lifetime ve global pool politikaları değiştirilmedi.

Yerel resmi GNOME Glycin 2.0.0 kaynak checkout'unda
`glycin-loaders/glycin-image-rs/src/main.rs` içindeki
`fn frame(&mut self, _frame_request: FrameRequest)` isteği kullanmaz ve decoder'ı
`take()` eder. Böyle bir yolda scale→full iki istek güvenilir erken-preview
sağlamaz; ikinci full decode üretmek de hedefe aykırıdır. Başka loader veya
Arch sürümünün davranışı bu kaynak incelemesinden doğrulanmış sayılmaz.
Client'ta embedded JPEG/RAW/TIFF preview seçen sertifikalı bir erişim yolu
bulunmadı. **Üretimde erken preview etkinleştirilmedi.** Full frame'den küçük
texture üretip erken preview diye raporlanmadı; decode gecikmesi iyileşmesi
iddiası yoktur.

| Format | Embedded preview | Üretimde full decode | Yeni native sonuç |
|---|---|---|---|
| JPEG | Kullanılan yolda destek doğrulanmadı | Güvenli mevcut yol | NOT RUN |
| PNG | Kullanılan yolda destek doğrulanmadı | Güvenli mevcut yol | NOT RUN |
| TIFF | UNSUPPORTED / DEFERRED PIV-40/41 | UNSUPPORTED / DEFERRED PIV-40/41 | NOT RUN |
| RAW/DNG | UNSUPPORTED / DEFERRED PIV-40/41 | UNSUPPORTED / DEFERRED PIV-40/41 | NOT RUN |

## Uygulanan fallback ve sınırlar

`image_pipeline::Phase`: Requested, PreviewLoading, PreviewReady, FullLoading,
FullReady, Failed, Cancelled. Gerçek ImageView yolu doğrudan FullLoading →
FullReady/Failed, close/message iptalinde Cancelled kullanır. Eski callback
önce mevcut token/weak/visible kontrollerinden geçer, sonra phase günceller.
Preview geçiş fonksiyonları yalnız model testleriyle doğrulanır; bunlar gerçek
preview sunumu veya decode kanıtı değildir. Üretimde preview sağlayıcısı yoktur.

Tek decode, tek kalite (full), mevcut loading mesajı ve cache-hit texture reuse
korunur. Preview/full için çift texture sahipliği, ek worker veya ikinci decode
yoktur. Mevcut cache yalnız full kalite taşır; preview cache'e yazılmaz.
3 admitted + 1 pending, 5s admission, aktif seçim > tek background, cache
128 MiB/3 girdi/64 MiB girdi sınırları, monitor/metadata/Refresh/pressure,
Fit/Fill/Actual, zoom/pan/filter ve pencere izolasyonu değişmedi. Hesaplanan
cache byte'ları RSS/VRAM değildir. Kaynak medya, Cargo bağımlılıkları ve Bwrap
zorunluluğu korunur.

**Uygulanmayan/NOT RUN:** erken-preview piksel sunumu, preview→full sırasında
viewport/zoom/pan koordinat dönüşümü, preview üzerinde hafif full-loading
göstergesi, gerçek preview-success/full-failure UI yolu ve iki kalite arasında
cache promotion. Mevcut full-only fallback bu özellikleri yapmış sayılmaz.
Sertifikalı bir erken-preview sağlayıcısı olmadan bu işlemler etkin değildir.
PIV-18 TESTING kalır; native fallback başarısı tek başına erken-preview kabulü
anlamına gelmez.

## Test kanıtı

- PASS: 2 yeni Rust model testi; doğrudan full, preview/full başarı/hata
  kombinasyonları, late-preview suppression ve cancel terminal durumu.
- PASS: kontrollü gerçek GTK/GIO preload probe; FullLoading/FullReady ve close
  Cancelled assertion'ları eklendi. Synthetic pixel verisi gerçek decode değildir.
- PASS: `bash tests/check.sh` mevcut activation ile, stable/locked/offline/-j2:
  fmt/check/build/test/clippy all-targets -D warnings; **74 Rust** (50 unit +24
  integration), **30 Python observer**, fixture/hash bütünlüğü.
- PASS: 3 yeni Python rapor sürücüsü testi; full-only etiketleme, exit77'de
  tekrar etmeme, timeout'u FAIL kaydetme. Subprocess mock'ları decode değildir.
- BLOCKED/NOT RUN: strict secure preload probe Xvfb'de bir kez **exit77**;
  ilk görüntü oluşmadı, loader erken çıktı. Önceki Codex Bwrap/read-only proc
  engeli bağlamı korunur. Aynı başarısız native kapı tekrar çalıştırılmadı.

- PASS: 20 scoped grafik çağrısı (14 X11, 4 Sway, 2 Weston): navigation,
  watch/failures/race, async/timeout, mode/zoom/filter ve surface kontrolleri.
  Gerçek decode PASS değildir; Weston normal close NOT RUN. Tüm örneklerin
  locked/offline derlemesi, shell syntax ve whitespace PASS.

## Tek native komut

```bash
bash tests/run_preview_pipeline_probe.sh
```

Mevcut Arch/GNOME Wayland terminalinde güncel dev checkout ile çalıştırılır.
Secure JPEG/PNG/EXIF baseline → phase/cache/preload → gerçek zoom/pan/filter →
async cancel/race/two-window → mevcut partial memfd/pidfd aktif-close checkpoint.
Kaynaklar testin geçici kopyalarıdır. Her alt komutun exit durumu ve full-only
etiketi `/tmp/pivra-preview-pipeline.json` içine yazılır; active-close detayları
`/tmp/pivra-preview-active-close.json`. İlk hata/77'de durur; tamamlanmayan aşamalar
PASS değildir. Hepsi exit0 olsa bile rapor yalnız **PASS full-only fallback**
der; early-preview ve preview/full continuity UNSUPPORTED/NOT RUN kalır.
Gerçek GNOME manuel kabulü ve desteklenebilir preview sağlayıcısı değerlendirmesi
açıktır. Faz 0 temiz kurulum, DESKTOP-06, PIV-39 ve kenar navigasyon tasarım kararı
korunur; PIV-19 başlamadı.

## Commit'ler

- `99d49ddbdfb73c88d1c638f5d1d0d07ae4c85c3d`: kalite yaşam döngüsü, full-only ImageView entegrasyonu.
- `29059174e800ed067414651c99b02c2e2a9c779e`: phase assertion'ları, tek native komut/JSON ve rapor testleri.

Uzak belge değişiklikleri kodu değiştirmedi; kod testleri tekrar edilmedi.

## Native render hatası — hedefli düzeltme — 24.10.2026

Kullanıcı güncel dev üzerinde `bash tests/run_preview_pipeline_probe.sh` için
**FAIL / exit101** bildirdi: `view_modes_probe.rs` render, `to_node()` None.
Önceki secure JPEG/PNG/EXIF, preload/cache/LRU/priority/cleanup aşamalarının
geçtiği kullanıcı tarafından bildirildi; bütün native komut PASS değildir.

Düzeltme: `bba76daa1879d8dfd310664bed47a9b83f239ce8`.
Üretim ImageView/decoder değişmedi. `loaded()` paintable varlığını bekliyor,
ancak bu GTK widget render cache'inin üretildiğini garanti etmiyor. `render()`
yalnız sabit 100ms settle kullanıyordu; anlık WidgetPaintable snapshot'ı yükleme,
layout veya compositor frame completion bariyeri değildir. İki pencere testinde
hedef pencere occluded da olabilir. Bu nedenle test senkronizasyon açığı düzeltildi;
Arch'taki tam tetikleyicinin bağımsız yeniden üretildiği iddia edilmez.

Yeni yardımcı hedef pencereyi present eder, WidgetPaintable'ı frame boyunca
canlı tutar, frame clock after-paint sinyalinden sonra gerçek snapshot alır.
Tick callback frame talebini sürdürür; nonempty node için en çok20s bekler.
Boş node sürerse mapped/visible/allocation/content/mode tanısıyla FAIL verir.
Tick ve sinyal bağlantısı bekleme sonunda kaldırılır. Sahte node, assertion
kaldırma veya skip yoktur; mevcut clipping/geometry/alpha/gradient piksel
assertion'ları aynı gerçek node üzerinde çalışmaya devam eder.

Önce hedefli Xvfb synthetic mode testi: **48 snapshot PASS**. Ardından bir kez
`bash tests/check.sh`: fmt, locked check/build/test, all-targets Clippy -D
warnings; **74 Rust +30 Python**, fixture/hash PASS. X11 synthetic zoom/pan ve
resampling PASS; headless Sway/Wayland synthetic mode **48 snapshot PASS**.
Whitespace PASS. Bunlar gerçek decoder/native Arch sonucu değildir.
Strict `view_modes_probe --zoom-pan` Codex'te yalnız bir kez **exit77 / NOT RUN**;
secure baseline oluşmadı, tekrar denenmedi. Güvenlik değiştirilmedi.

Arch kullanıcı yeniden aynı komutu çalıştırmalı:
`bash tests/run_preview_pipeline_probe.sh`.
Önceki native FAIL kaydı korunur; yeni native sonuç **bekleniyor**, PIV-18
**TESTING**. Early-preview/preview-full continuity sınırlamaları değişmedi.

## İkinci native snapshot hatası — resampling — 24.10.2026

Kullanıcı güncel dev için **FAIL / exit101** bildirdi:
`examples/support/resampling_cases.rs:20`, `resampling render node`.
Önceki mode düzeltmesiyle gerçek Fit/Fill/Actual ve iki pencere kontrollerinin
PASS olduğu kullanıcı tarafından bildirildi; tüm native komut PASS değildir.

Düzeltme SHA: `fcea3e8a4ac0f7cb87ca4eedf9eaec4a1140adb4`.
Resampling `pixels()` hâlâ settle + geçici WidgetPaintable kullanıyordu. Önceki
başarılı mode frame bariyeri `examples/support/render_frame.rs` ortak yardımcısına
taşındı. Mode, resampling ve aynı açığı taşıyan zoom/pan snapshot çağrısı bu
küçük ortak yardımcıyı kullanır. Widget referansları ve WidgetPaintable bütün
bekleme boyunca canlıdır. Gerçek after-paint, mapped/nonzero allocation ve
nonempty snapshot şartları korunur; 20s deadline başarısızsa tanılı FAIL verir.
Tick/sinyal bağlantısı temizlenir. Action bulunmayan synthetic pencerede hata
tanısı ayrıca panic üretmez. Sahte node, assertion kaldırma veya skip yoktur.
Smooth/Nearest, gerçek GSK piksel örnekleri, clipping ve gradient kontrolleri
aynen korunur. Üretim ImageView/decoder/cache/iş yaşam döngüsüne dokunulmadı.

Önce hedefli **X11 synthetic resampling PASS**: Nearest palette/konum, Smooth
bilinear, fractional/min/max/downscale, distinct output ve geometry. Ardından
bir kez zorunlu core helper: **74 Rust +30 Python**, fmt/check/build/test ve
all-targets Clippy -D warnings, fixture hash'leri PASS. Ek **3 Python rapor testi
PASS**. X11 **48 mode snapshot**, X11 zoom/pan/controller/gradient ve Sway
Wayland resampling PASS. Bunlar gerçek GTK/GSK üzerinde **synthetic texture**
testleridir; secure decoder ve fiziksel GNOME kabulü değildir. Whitespace PASS.

Bilinen Codex Bwrap engeli yeniden denenmedi. Yeni secure/native sonuç
**NOT RUN**; kullanıcının Arch native FAIL kaydı düzeltme sonrasında yeniden
çalıştırılmayı bekler. Aynı komut: `bash tests/run_preview_pipeline_probe.sh`.
PIV-18 **TESTING**; PIV-19 başlamadı. Önceki erken-preview sınırlamaları sürer.

## Normal uygulamada PNG MIME hatası — açık teşhis — 24.10.2026

Kullanıcı gerçek Arch/GNOME'da native paket **exit0** bildirdi; buna rağmen
normal `cargo run --locked -- <wide.png veya large.png>` geçerli PNG için
`Unknown image format: text/plain` verdi. **Gerçek kullanıcı senaryosu FAIL**;
paket başarısı bunu geçersiz kılmaz. Kök neden henüz doğrulanmadı ve bu değişiklik
PNG açılış hatasını çözmüş olarak sunulmaz. PIV-18 TESTING kalır.

Kod/test SHA `44560357e6a8061308c4666c269bc13461f86248`.
CLI → OpenedFile → app::launch → GIO open → window → ImageView → worker →
Glycin akışı incelendi. CLI ve URI yerel GFile'a dönüştürülüyor, katalog ilk
açılan dosyayı koruyor. MIME katalog uzantısından atanmaz. Glycin3.0.8
`src/api_common.rs::guess_mime_type`, `dbus.rs::GFileWorker` tarafından okunan
başlangıç baytlarını `gio::content_type_guess(None, bytes)` ile sınıflandırır.
Belirsiz/TIFF/XML/gzip durumunda filename de kullanabilir. `Unknown image format`
loader config seçimine aittir; burada decoder pikselleri henüz çözülmemiştir.
Kod incelemesiyle yanlış path/seçim kanıtı bulunmadı; Arch nedeninin farklı MIME
veritabanı, içerik değişimi veya başka bir ortam etkisi olduğu **kanıtlanmadı**.

Üretim değişikliği yalnız hata teşhisidir: secure loader load hatasında aynı
GFile backend ile en çok512 bayt salt okunur, gerçek path/URI, content-only MIME,
uncertain, PNG/JPEG/other-truncated imzası ve örnek uzunluğu hataya eklenir.
Bu ikinci okuma hata anındaki ayrı point-in-time kanıttır; Glycin'in önceki
okumasıyla aynı içerik görüldüğünü garanti etmez. Ham dosya metni dökülmez.
Uzantıdan MIME zorlama, signature'ı decode sayma, loader/sandbox bypass veya
üretim decoder yolunu değiştirme yoktur. Bozuk içerik secure loader'da hata kalır.

Yeni `application_open_probe` doğrudan üretim `app::launch` ve GIO-open sinyalini
çalıştırır. Original fixture klasöründeki wide.png, large.png, no-exif.jpg;
path ve file URI için dosya ilişkisinin korunmasını, gerçek texture boyutlarını,
indirilen texture'da opak piksel bulunmasını, normal close ve kaynak byte
bütünlüğünü kontrol eder. Pencere/render kodunu taklit etmez, CLI alt süreç
olmadığı açıkça belirtilir; aynı uygulama başlatma fonksiyonu kullanılır.
Native JSON paketine image_probe sonrasında bu kapı eklendi. Bir hata sonrası
diğer aşamalar PASS olmaz. İlk loader early exit için77, MIME hatası içinFAIL
verir. Mevcut paket testi gerçek uygulama girişini atlayamaz.

**PASS — Codex:** hedefli yanlış uzantı/PNG/JPEG/text/empty header teşhis birim
testi; sonra bir tam zorunlu core: fmt/check/build/test/all-targets Clippy
-D warnings, **75 Rust +30 Python**, fixture/hash. Native rapor sürücüsü **3
Python** PASS. Normal binary X11 GUI path/URI error-mode smoke PASS (görüntü
decode başarısı değildir). Whitespace PASS.

**NOT RUN:** application_open_probe bir kez Codex Xvfb'de exit77. Gerçek
production app girişinde wide.png için path/URI doğru, `image/png`,
uncertain=false, PNG signature, sampled=124; ardından secure loader early exit.
Arch text/plain sorunu yeniden üretilemedi; bilinen Bwrap engeli tekrar denenmedi.
PNG/JPEG gerçek normal açılış regresyonu için native sonuç bekleniyor.

Kullanıcı aynı normal komutu yeniden çalıştırmalı; hata sürerse ek teşhis
satırıyla birlikte bildirmeli:

```bash
GDK_BACKEND=wayland cargo run --locked -- "$PWD/tests/fixtures/images/wide.png"
```

Paket komutu değişmedi: `bash tests/run_preview_pipeline_probe.sh`.
Geçerli PNG/JPEG kullanıcı senaryosu doğrulanmadan bu açık hata kapanmaz.

## Arch native ve manuel full-only kabul — 24.10.2026

Kullanıcı güncel `dev` üzerinde `bash tests/run_preview_pipeline_probe.sh` çalıştırdı; **exit 0**. Normal uygulama girişinde wide.png, large.png, no-exif.jpg için path ve URI, gerçek secure texture/pixel sonuçları PASS. Preload/cache, Fit/Fill/Actual, Smooth/Nearest, zoom/pan, async A-B-A, iki pencere izolasyonu ve PIV-09 active secure decoder close/cleanup PASS. Hesaplanan cache bütçesi gerçek RSS/VRAM ölçümü değildir.

İlk manuel denemelerde fixture klasörü gezinmesindeki bilerek geçersiz `text-content.jpg`, `broken.jpg` ve `broken.png` hata kayıtları görüldü; yeni hata teşhisi `text-content.jpg` için content-only MIME `text/plain` ve geçersiz imzayı gösterdi. Bu, ilk geçerli PNG'nin decoder tarafından hatalı çözüldüğü anlamına gelmez. Kullanıcı temiz klasörde geçerli fotoğrafların açıldığını ve Sol/Sağ gezinme, zoom, Fit/Fill kontrollerinin çalıştığını doğruladı. GDK frame-timing warning gözlendi; bu tek başına decoder hatası veya crash kanıtı değildir.

**Kabul:** Native full-only fallback ve belirtilen manuel görüntüleme davranışları PASS. **Kapsam açığı:** gerçek erken preview sağlayıcısı, preview→full viewport sürekliliği ve preview sırasında full-loading UX henüz uygulanmadı; PIV-18 **TESTING**, DONE değil. PIV-19 başlatılmaz.


## PIV-18B — ayrı erken-preview araştırması

[API/loader kanıtı, standalone probe ve sınırlamalar](PIV18_PREVIEW_FEASIBILITY.md).
Mevcut full-only açılış değişmedi. Erken preview kanıtlanmadı; PIV-18 TESTING.


## PIV-18C — tarihsel deney; reddedildi ve kaldırıldı (25.10.2026)

Embedded JPEG thumbnail ve reduced-resolution libjpeg prototipleri
`f69b0edef8f1e5f33f404e234c20d49a08bae170` ile denendi; üretime bağlanmadı.
O zamanki hedefli metadata/sürücü testleri geçti, tek Codex Bwrap denemesi77
verdi; gerçek erken frame NOT PROVEN kaldı. Bunlar tarihsel sonuçlardır.

Kullanıcı iki yöntemi de reddetti. Deneysel decoder, Python sürücüsü/testi,
native betik ve prototip belgesi depodan kaldırıldı. Decoder silinmesi uzak
`87aa792a8f03a8979bd181733bfa9196d4ee313b` commit'inde zaten yapılmıştı;
bu temizlik kalan dosyaları ve aktif belge referanslarını kaldırır.
Silinen prototip için native test, geliştirme veya üretim entegrasyonu beklenmez.
PIV-18B Glycin araştırması ve mevcut secure full-only üretim davranışı korunur.
**PIV-18 TESTING; PIV-19 başlamadı.**


## Native view mode gezinme regresyonu — 25.10.2026 — TESTING

Kullanıcı `bash tests/run_preview_pipeline_probe.sh` için yeni native **FAIL**
bildirdi: `examples/view_modes_probe.rs:268`, left=`actual`, right=`fit`.
Secure JPEG/PNG/EXIF, normal uygulama path/URI, preload/cache ve çoğu render
kontrolünün PASS olduğu bildirildi; bütün native paket PASS değildir. Önceki
kabul ve FAIL kayıtları korunur. Başlangıç dev `99de95124fe80c655e703883d29b3a4e2aaa9128`.

### Kanıtlanan neden

Bu yalnızca “biraz daha bekle” hatası değildir. Üretimdeki
`4abc1668a2cc9b69b9742e6212677efb62f1972c` kararı Fit/Fill/Actual preset'ini
fotoğraf gezinmesinde korur; TH-05/06 kayıtları ve preload_probe aynı davranışı
zaten doğrular. `ImageView::load_request` yeni istekte mode/transform/texture'ı
korur; güncel callback başarılı yeni texture'ı kabul ettiğinde transform reset
eder. `ViewMode::from_id("actual")` Actual döndürür; yalnız `manual` gibi preset
olmayan state `unwrap_or_default()` ile Fit'e döner. Bu yüzden yeni frame hazır
olduktan sonra da Actual kalması doğrudur; üretim hatası bulunmadı/değiştirilmedi.

İkinci eski varsayım `loaded()` içinde yalnız paintable.is_some() beklenmesiydi.
TH-06 retention sonrasında bu önceki fotoğraf olabilir. Bozuk dosyada paintable
None beklemek de kabul edilmiş “önceki fotoğraf + dosya adlı hata” davranışına
aykırıydı. Sorunlar ortak test yardımcılarında giderildi, ürün Fit'e zorlanmadı.

### Düzeltme ve sıkı bariyer

Kod/test commit'i **`ffe9ab5cb2a11af7e2825214d7a3a07993b735f9`**.

- `loaded` seçili dosyayı gerçek pencere başlığından, hedef frame boyutunu ve
  önceki **strong-referenced paintable'dan farklı** yeni sunumu birlikte bekler.
  Eski obje hayatta tutulduğu için pointer yeniden kullanımı kabulü yanıltmaz.
  Aynı boyutlu resampling2→4 geçişinde de boyut tek başına yeterli değildir.
  Geçerli hedefte gerçek hata görünürse hemen FAIL;20s deadline tanı verir.
  Sabit settle/sleep eklenmedi, state sahte atanmadı veya assertion atlanmadı.
- Actual→2.png geçişinde pending eski frame ve Actual korunması, **yeni frame
  sonrasında da Actual** olması assert edilir. Fit render'ı ayrıca gerçek mode
  action'ıyla sınanır; bu otomatik Fit reset diye sunulmaz.
- Bozuk3.png'de aynı eski paintable, `File: 3.png`, görünür hata/previous-photo
  açıklaması ve Fill korunması gerekir. Loader process exit'i invalid-media
  başarı diye kabul edilmez.
- Küçük4.png'de gerçek scroll controller ile `manual` oluşturulur. Hızlı
  4→broken3→cached2 sırasında önce manual kalmalı; **yeni2 frame'i gelince Fit**
  olmalıdır. Eski hata yeni seçime taşınmamalı. Cached tall GDK texture kimliği
  ve resampling, diğer pencerenin Fill/texture durumu ayrıca korunur.
- Close sonrası sabit settle yerine gerçek weak-window release beklenir.
- Ortak loaded yardımcısını kullanan resampling native checker'ın dört çağrısı
  aynı bariyere geçirildi. Nearest palette/Smooth/clipping assertion'ları aynen
  kalır; broken-media None yerine retained-photo koşulları güçlendirilir.
  Checker'ın8x scroll başlangıcı gerçek Actual action'ıyla belirlenir; tiny-image
  Fit zaten manual MAX_SCALE üstündeyse zoom-up'ın doğru biçimde no-op olması
  testte yanlış “manual olmadı” hatasına dönüştürülmez. Bu fake state değildir.

### Kontrollü yeniden üretim ve test sonuçları

`view_modes_probe --controlled-navigation` yalnız örnek programda açıkça seçilen
sentetik loader sınırıdır. Aynı üretim window/actions/ImageView ve aynı isolation
fonksiyonu çalışır; gerçek dosyayı host'ta decode etmez. Varsayılan/native yol
her zaman mevcut secure_image_loader'a gider; uygulama loader'ı değişmedi ve
hata halinde sentetik fallback yoktur. Native pipeline betiği bu flag'i kullanmaz.

Eski assertion kontrollü X11 koşusunda aynı leftActual/rightFit ile **exit101
FAIL** verdi. Düzeltme sonrası X11 ve izole Sway/Wayland aynı navigation/publish,
preset/manual reset/cache/error/iki pencere testleri **PASS** verdi. Başarılı
çıktı açıkça `CONTROLLED pixels, not Glycin` der; native başarı olarak sunulmaz.

PASS:fmt/check/test/all-target Clippy -D warnings ve examples build (locked,
offline, mevcut stable SDK, -j2); **88 ana Rust testi**, ayrıca view_modes_probe
harness'inde20 unit testi (ortak kaynak tekrarları dahil). X11 controlled-nav,
48 synthetic mode snapshot, synthetic zoom-pan ve resampling; controlled async
A/B/A/reverse late callback, cache/preload/priority, window-failure/navigation
race regresyonları PASS. Sway gerçek Wayland yüzeyinde controlled-nav PASS.
Python Wayland sürücüsü syntax kontrolü ve git diff --check PASS.

**Native secure yeniden test BLOCKED/NOT RUN:** aynı üretim loader'ının bilinen
Codex Bwrap/proc ortam engeli tekrar denenmedi, güvenlik gevşetilmedi. Native
resampling checker'ın değiştirilmiş default yolu derlendi; gerçek secure checker
pikselleri bu oturumda çalıştırılmış gibi PASS sayılmaz. Önceki kullanıcı native
FAIL kaydı, gerçek native sonuç gelene kadar kapanmış değildir.

Arch/GNOME'da aynı komut:

```bash
bash tests/run_preview_pipeline_probe.sh
```

Çalıştırılan SHA ve JSON/exit sonucu kaydedilmeli. Başarı bile yalnız full-only
paketi doğrular; erken-preview/foreground gösterge kapsam kararları ayrı kalır.
**PIV-18 TESTING**; main ve sonraki görevler değiştirilmedi.
