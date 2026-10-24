# PIV-18 / TH-02 — Thumbnail overlay — TESTING

25.10.2026 · dev. Başlangıç `e503e8fec645ddf79b2b73b4b34b9c03573f8eec`,
temiz çalışma ağacı, origin/dev ile eşit. Kullanıcı TH-01 native kabulünün
başarılı olduğunu bildirdi; bu yeni Codex GNOME sonucu değildir.
Yalnız TH-02 uygulanmıştır. TH-03–TH-06 ve PIV-19 başlamadı.

## Değişiklik

`src/thumbnail_overlay.rs` mevcut ImageView GtkOverlay'ine ölçüme katılmayan,
viewport içinde clip edilen alt dock ekler. Panel ScrolledWindow ile yatay
kaydırılır. Hücre dış sınırı96×80 GTK mantıksal piksel; içerik ayrı non-measuring
Overlay içinde clip edilir. Border seçili/seçili değilken aynı2px alanı ayırır,
selected class accent çerçevesini etkinleştirir. CSS arka planında
alpha(window_bg_color,0.84) kullanılır; panel veya hücre opacity azaltılmaz.
Tema renkleri ve GTK logical ölçüleri kullanılır. Stylesheet GTK thread'inde
bir kez kaydedilir; pencere başına provider biriktirilmez.

Alt orta ToggleButton kapalıyken pan-up, açıkken pan-down gösterir; yerel transient
state taşır. Kapalıyken panel görünmez, dock yalnız küçük düğme genişliğindedir;
yatay görünmez bir input engeli kalmaz. Açıkken dock fill olur, şerit kenarlardan
8px içeride kalır; scrollbar gerekirse görünür. Button callback yalnız görünürlük,
align, icon ve tooltip değiştirir. Fotoğraf transform/texture/load çağrısı yoktur.
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
görsellerin opaklığı/çerçevesi ve fiziksel mouse/trackpad davranışı kullanıcı
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
seçili çerçeve/thumbnail opaklığı, yatay kaydırma, dar pencere/HiDPI ve iki pencere
bağımsızlığını kontrol et. Normal Pivra'da bu aşamada boş şerit beklenir.
Seçili ana fotoğraf ile placeholder'ın eşleştirilmesi beklenmez (TH-04 işi).
Otomatik gizleme/T/kalıcı tercih beklenmez (TH-03 işi).

**TH-02 TESTING**; kullanıcı kabulü olmadan TH-03'e geçilmez.
PIV-18 TESTING; PIV-19 başlamadı.

## Gerçek commit'ler

- `604b5761904c195a2f0bdf55f08f4a5d377ed59c`: non-measuring overlay component + ImageView kurulumu.
- `b1bb6f41573fd41148a066d7de878e867b8b3b52`: synthetic UI/GSK/counter/isolation probe ve native betik.
