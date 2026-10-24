# PIV-18 / TH-02 — Thumbnail overlay — TESTING

25.10.2026 · dev. Başlangıç `e503e8fec645ddf79b2b73b4b34b9c03573f8eec`,
temiz çalışma ağacı, origin/dev ile eşit. Kullanıcı TH-01 native kabulünün
başarılı olduğunu bildirdi; bu yeni Codex GNOME sonucu değildir.
Yalnız TH-02 uygulanmıştır. TH-03–TH-06 ve PIV-19 başlamadı.

## Değişiklik

`src/thumbnail_overlay.rs` mevcut ImageView GtkOverlay'ine ölçüme katılmayan,
viewport içinde clip edilen bağımsız alt panel ve düğme ekler; ortak dock yoktur.
Panel ScrolledWindow ile yatay
kaydırılır. Hücre dış sınırı96×80 GTK mantıksal piksel; içerik ayrı non-measuring
Overlay içinde clip edilir. Görünür hücre çerçevesi/arka planı yoktur; selected class yalnız altta1px
accent vurgu çizgisi çizer (inset shadow, layout alanı ayırmaz). CSS arka planında
alpha(window_bg_color,0.25) kullanılır; panel veya hücre opacity azaltılmaz.
Tema renkleri ve GTK logical ölçüleri kullanılır. Stylesheet GTK thread'inde
bir kez kaydedilir; pencere başına provider biriktirilmez.

Alt orta ToggleButton kapalıyken pan-up, açıkken pan-down gösterir; yerel transient
state taşır. Kapalıyken panel görünmez; düğme her zaman yalnız kendi dar
allocation alanını kaplar. Açıkken de düğme yanında geniş input katmanı yoktur.
Şerit kenarlardan8px içeride kalır; scrollbar gerekirse görünür. Panelin alt
boşluğu düğmenin GTK doğal yükseklik isteğinden hesaplanır. Button callback
yalnız panel görünürlüğü, icon ve tooltip değiştirir. Fotoğraf transform/texture/load çağrısı yoktur.
Callback weak widget referansları kullanır; sahiplik Overlay'dedir.

`src/image_view.rs` yalnız component kurulumu ekler. TH-01 discovery capability
Unavailable ise kontrol gizlidir; GNOME'da initial cache-only desteğiyle düğme
sunulur. Yeni source/cache sorgusu veya factory çağrısı yoktur.
Normal uygulamada **No thumbnails available** boş durumu vardır. Sahte hücreler
normal kullanıcı thumbnail'i gibi sunulmaz. Henüz gerçek thumbnail yükleme,
seçim senkronizasyonu, üretim veya placeholder spinner yoktur.

TH-03 öncesindeki geçici davranış: düğme desteklenen fotoğraf penceresinde
görünür kalır; yalnız tıklama ile açılır/kapanır. Otomatik fare görünürlüğü,
animasyon, T ve kalıcı Enabled tercihi kodlanmadı. Mode/zoom/pan/fit/texture,
katalog veya resize davranışı değiştirilmedi. Üretim loader/preload/request/
settings/thumbnail-provider kaynakları ve Cargo.toml/lock değişmedi.

## Doğrulama

`examples/thumbnail_overlay_probe.rs` gerçek production ImageView/component ve
GTK olaylarını kullanır; **ana görüntü de thumbnail hücreleri de sentetiktir**.
Pencere başlığında TEST MAIN IMAGE / TEST PLACEHOLDERS yazılıdır; hücreler
TEST1–32 etiketlidir. Örnekteki loader stub hiçbir secure decode başarısı sayılmaz.
Üretim uygulamasında stub veya test içeriği yoktur.

PASS — TH-02 X11 synthetic GTK/GSK testi:

- Açık/kapalı overlay sırasında Picture ve Overlay allocation'ları eşit.
  800×600 ve320×300 taleplerinde Fit/Fill/Actual Picture snapshot byte'ları
  değişmedi; aynı paintable korundu.
- Gerçek scroll/drag controller sinyalleriyle zoom/pan sonrası aç/kapa pikselleri
  aynı kaldı. İki ilk synthetic decode dışında ek load/admitted job başlamadı.
- Panel bounds viewport içinde, düğme alt-orta, ikonlar iki yönde doğru.
  Outer cell allocation96×80; yatay adjustment hareket edebiliyor.
- Kapalı dock genişliği80px altında, gizli strip unmapped; fotoğraf alt kenarında
  düğme dışındaki pick sonucu Picture. Görünmez geniş input katmanı yok.
- Selected CSS class ve panel/cell opacity1; bir pencerenin toggle'ı diğerini
  açmadı. Bir pencere kapanırken diğerinin Picture pikselleri değişmedi.

PASS — aynı TH-02 testi X11 **GDK_SCALE=2** ve headless **Wayland/Sway**.
Mantıksal layout/identity sınandı; fiziksel GNOME/HiDPI ekran görsel kabulü değildir.
Ek3 controlled GUI regresyonu PASS: PIV-16 async cancel, PIV-17 preload/cache ve
window-failure-race. Toplam6 scoped grafik çağrısı (3 TH-02 +3 eski controlled).

PASS — bir zorunlu core çalıştırmasıyla fmt/check/build/test,80 Rust(56 unit+24
integration),30 Python observer ve fixture/hash doğrulaması. İlk all-targets
Clippy probe'daki ok().expect() için FAIL verdi; doğrudan expect() düzeltmesinden
sonra yalnız Clippy ve fmt tekrar koştu, PASS. Tam paket gereksiz yere tekrarlanmadı.
Tüm örneklerin locked/offline derlemesi, shell syntax ve whitespace PASS.

NOT RUN — yeni kodda gerçek secure decode/native fotoğraf paketi; bilinen Bwrap
engeli tekrar denenmedi. Native GNOME CSS görünümü, arka planın yarı saydamlığı,
görsellerin opaklığı/alt vurgu çizgisi ve fiziksel mouse/trackpad davranışı kullanıcı
kabulü bekler. CSS alpha/class/opacity testi bu görsel kabulün yerine geçmez.
Ölçülmüş hız/RSS/VRAM kazanımı iddiası yoktur.

## Tek Arch/GNOME manuel komutu

```bash
bash tests/run_thumbnail_overlay_probe.sh
```

GNOME Wayland terminalinde locked build yapar, aynı otomatik UI kontrollerini
koşar, ardından **iki TEST penceresini** manuel inceleme için açık bırakır.
İkisini kapatınca biter. Bu komut gerçek thumbnail üretmez, source dosyası almaz,
GNOME factory veya Glycin bypass çağırmaz. Test ana görüntüsü gradient, thumbnail
hücreleri renkli TEST placeholder'lardır; gerçek fotoğraf gösterimi değildir.

Manuel kabulde düğmeye tıklama/ikon, alt-orta konum, arka plan saydamlığı,
seçili ince alt çizgi/thumbnail opaklığı, yatay kaydırma, dar pencere/HiDPI ve iki pencere
bağımsızlığını kontrol et. Normal Pivra'da bu aşamada boş şerit beklenir.
Seçili ana fotoğraf ile placeholder'ın eşleştirilmesi beklenmez (TH-04 işi).
Otomatik gizleme/T/kalıcı tercih beklenmez (TH-03 işi).

**TH-02 TESTING**; kullanıcı kabulü olmadan TH-03'e geçilmez.
PIV-18 TESTING; PIV-19 başlamadı.

## Gerçek commit'ler

- `604b5761904c195a2f0bdf55f08f4a5d377ed59c`: non-measuring overlay component + ImageView kurulumu.
- `b1bb6f41573fd41148a066d7de878e867b8b3b52`: synthetic UI/GSK/counter/isolation probe ve native betik.


## Görsel geri bildirim düzeltmesi — 25.10.2026 — TESTING

Kullanıcının Arch GNOME geri bildirimi: test penceresinde standart başlık/close
kontrolleri yoktu, panel fazla opaktı ve hücre çerçeveleri istenmiyordu.
Düzeltme SHA `bd5d6b77f801015c26c3dac11e6218307a88ae79`.

- Panel ve düğme arka planı %84→**%25** opacity. Widget opacity1 kalır;
  thumbnail/test görselleri soluklaştırılmaz.
- Hücre background/border kaldırıldı. Selected durumda yalnız alt1px tema
  accent çizgisi (inset shadow) var; normal/selected allocation aynı96×80.
  Mevcut8px hücre aralığı ve yatay scroller korunur.
- Probe, normal `window.rs` kalıbıyla dikey content + standart **adw::HeaderBar**
  ve **adw::WindowTitle** kullanır. Start/end title buttons etkin, header mapped
  ve title widget mevcut olması sınanır. Başlık Pivra — TEST, subtitle açıkça
  synthetic/test; iki manuel pencere ve close/isolation testi korunur.
- Yeni auto-hide, fareye göre görünürlük, T veya kalıcı ayar yok. TH-03 için
  mevcut dock/strip/toggle widget kimlikleri ve yerleşim korunur.

PASS: gerçek GSK snapshot'ta panel padding bölgesinin alfa değeri60–68/255
(yaklaşık %25), thumbnail iç örneği255/255, seçili hücrenin üst/yan örnekleri0
(çerçeve/background yok), altta çizgi var. Normal durumda alt örnek0; iki stil
aynı texture/render allocation'a sahip. Bunlar synthetic texture testleridir.
Header title/button ayarları, önceki Fit/Fill/Actual/zoom/pan byte sürekliliği,
scroll, gizli panel pick-through, iki pencere ve ek load olmaması korunur.

PASS: X11 açık tema; X11 **scale2+koyu tema**; Wayland/Sway; eski controlled
async/preload/window-failure-race — toplam6 scoped grafik çağrısı. Tek zorunlu
core80 Rust/30 Python observer, fmt/check/build/test/all-target Clippy -D warnings
ve fixture hash; tüm örneklerin locked/offline build'i, whitespace PASS.
Native GNOME görsel/physical input kabulü **NOT RUN**, kullanıcı yeniden test
edecek. Glycin/factory/preload/cache/ImageView production koduna dokunulmadı;
Bwrap/decode denemesi tekrarlanmadı. TH-02 TESTING; TH-03'e geçilmedi.

Tek native komut aynı: `bash tests/run_thumbnail_overlay_probe.sh`.
İki TEST penceresinde standart başlık/close kontrollerini, daha saydam paneli,
çerçevesiz opak test görsellerini ve yalnız alttaki seçili vurgu çizgisini incele.
Fareyle otomatik göster/gizle bu kabulde beklenmez; TH-03 işidir.


## Alt bant düzeltmesi — 25.10.2026 — TESTING

Kod/test SHA: `eba1a07fefc925c44527b0a9f3b775e19e65869a`.
Kullanıcı gerçek GNOME Wayland'da şerit altında istenmeyen koyu bant bildirdi.
İncelemede şerit ile düğmenin ortak `thumbnail-controls` dikey kutusunda olduğu,
açılınca bu kutunun Fill ile tam genişliğe yayıldığı görüldü. Kutuda uygulamaya
ait açık bir background CSS kuralı yoktu; native temanın tam çizim katkısı Codex'te
bağımsız yeniden üretilmedi. Sorunlu ortak geniş allocation kaldırıldı:

- Şerit ve düğme artık GtkOverlay'in iki bağımsız, non-measuring/clip edilen
  çocuğudur. Ayrı dock/status-bar widget'ı yoktur. Düğme her zaman alt ortada dar
  kalır, panel düğmenin üzerinde yüzer. Fotoğraf allocation'ı değiştirilmez.
- %25 panel/düğme arka planı, opak test görselleri, çerçevesiz hücreler ve1px
  seçili çizgisi aynen korunur. Provider/Glycin/preload/cache kaynakları değişmedi.
- Yeni gerçek GSK bileşik snapshot regresyonu: açık panelin altındaki tüm
  düğme-dışı pikseller Picture snapshot'ıyla birebir aynı olmalı. Ortak dock'un
  yokluğu ve doğrudan overlay ebeveynliği de sınanır. Açık ve kapalı durumda
  düğmenin sağ/sol alt kenarında gerçek GTK pick sonucu Picture olmalıdır.

PASS: hedefli X11; X11 koyu tema+GDK_SCALE=2; headless Wayland/Sway —3 GTK
çağrısı. Mevcut Fit/Fill/Actual/zoom/pan byte sürekliliği, resize, scroll,
alpha/hücre/underline, iki pencere izolasyonu ve ek load olmaması kontrolleri
korunup geçti. PASS: tek `tests/check.sh` çalıştırmasıyla fmt/check/build/test/
all-target Clippy -D warnings,80 Rust testi,30 Python gözlemci testi ve fixture
bütünlüğü. Whitespace kontrolü PASS.

NOT RUN: fiziksel Arch GNOME görsel kabulü; kullanıcı aynı
`bash tests/run_thumbnail_overlay_probe.sh` komutuyla şeridin altında yatay koyu
bant olmadığını, düğmenin bağımsız yüzdüğünü ve iki pencereyi kontrol edecek.
Synthetic UI testi gerçek secure decoder testi değildir. Bilinen Bwrap engeli
yeniden denenmedi. TH-02 TESTING; TH-03 başlamadı.
