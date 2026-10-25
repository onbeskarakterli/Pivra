# PIV-18 / TH-03 — Otomatik görünürlük ve kalıcı tercih — TESTING

25.10.2026 · dev. Başlangıç `0090131f05cddf9b076dc6a61d1807d4ba833834`,
çalışma ağacı temiz, HEAD=origin/dev. Kullanıcı TH-03 talimatında TH-02 native
kabulünü bildirdi: TH-02 DONE. Ayrıntılı yeni native log/SHA verilmedi;
Codex bağımsız GNOME kabulü sayılmaz. TH-03 native kabul bekler.

Gerçek kod SHA: `f8bfd580708f103945aa6b310e9ae80166eeefbb`.
GTK gözlem/senkronizasyon testi SHA: `e5f6681bfcd3e63c857fbff60963815c2ec28a2f`.

## Uygulama

`src/thumbnail_overlay.rs` üç ayrı durumu yönetir:

| Available | Enabled | Fare/olay | Visible |
|---|---|---|---|
| false | herhangi | herhangi / T | Panel ve düğme gizli; kayıtlı tercih korunur |
| true | false | Alt kenar | Yalnız yukarı ok |
| true | true | Alt etkileşim bölgesi / panel | Panel ve aşağı ok |
| true | herhangi | Uzaklaşma | 500 ms sonra ikisi de gizli |
| true | false→true | T | Panel kısa süre gösterilir; fare uzaktaysa 1000 ms sonra gizli |

Enabled ve Available sürekli Visible anlamına gelmez. T ile kapatma paneli
hemen gizler; düğme normal fare kurallarına döner. Başlangıçta fare yaklaşmadıkça
kontroller gizlidir. Alt etkileşim bölgesi panel/düğme yüksekliğinden hesaplanır;
resize sonrası gerçek frame allocation değişiminde yeniden değerlendirilir.
GTK motion controller gözlemcidir; fotoğraf olaylarını tüketmez. Panel ile düğme
arasındaki boşluk da aynı bölgede olduğundan geçişte kaybolmazlar.

Tek pencereye özel one-shot GLib hide kaynağı vardır; dönüşte veya tercih
değişiminde kaldırılır. Kapanış/unmap sırasında kaynak ve frame-clock gözlemcisi
temizlenir; callback weak state kullanır. Frame-clock gözlemcisi frame istemez,
yalnız allocation yüksekliği değişince işlem yapar. Sürekli polling veya animasyon
yoktur; azaltılmış animasyon tercihine aykırı geçiş eklenmez.

T ana pencerenin capture controller'ındadır; Ctrl/Alt/Super/Meta birleşimleri ve
Editable/TextView odağı yakalanmaz. Thumbnail kontrolü odaklıyken de çalışır.
Yalnız Enabled değişir, ana fotoğraf seçimi/load/zoom/pan değişmez. Yeni fotoğrafın
ImageView yüklenmesi gizli kontrolleri açmaz. Thumbnail seçim senkronizasyonu yoktur.

## Kalıcılık ve pencere bağımsızlığı

`src/settings.rs`: `$XDG_CONFIG_HOME/pivra/thumbnails.conf` (varsayılan kullanıcı
config dizini) içinde yalnız `enabled=true/false` tutulur. İlk çalıştırma/missing/
bozuk dosyada false. Okuma32 baytla sınırlı; yazma GIO replace_contents ile atomik,
private dosya olarak yapılır. Yeni bağımlılık veya kurulması gereken GSettings
şeması yoktur; kaynak koddan native Cargo çalıştırmasıyla uyumludur.

Yazma hatası stderr'de raporlanır; bellek içi tercih ve fotoğraf görüntüleme devam
eder. UI thread'indeki ortak tercih açık pencerelere weak abonelerle bildirilir;
Visible, fare konumu ve timer pencereye özeldir. Ayrı uygulama süreçleri açılışta
aynı dosyayı okur; süreçler arasında canlı ayar değişikliği yayını eklenmedi.
Unavailable durumunda tercih silinmez. Görünürlük/fare konumu diske yazılmaz.

TH-02 bağımsız yüzen panel/düğme, %25 CSS arka planı, opak hücreler, çerçevesiz
stil ve1px seçili çizgisi korunur. Fotoğraf viewport'u daralmaz; ortak dock yoktur.
GNOME provider/factory, decoder, cache, preload kaynakları değişmedi. Şu anda ne
Enabled ne de Visible thumbnail üretim/sorgu kuyruğu başlatır; TH-05 geliştirilmedi.

## Test kanıtları

`examples/thumbnail_overlay_probe.rs` gerçek production ImageView/overlay, GTK
controller sinyalleri, GLib timer'ları ve gerçek GSK snapshot kullanır; ana görsel
ve hücreler **sentetik TEST** verisidir. Üretim loader'ı değiştirilmez. Otomatik
motion sinyalleri kontrollü verilir: gerçek production handler'ları taşıyan
motion controller yalnız otomatik modda widget'tan ayrılıp testte tutulur.
Böylece compositor fiziksel pointer crossing/reset olayları sentetik girişleri
karıştırmaz; enter/motion/leave davranışı handler'larda değişmez. Manuel modda
controller widget'a bağlıdır ve gerçek olay akışını kullanır. Bu test gerçek Glycin/Bwrap veya fiziksel mouse başarısı değildir.

PASS:

- Available/Enabled/Visible kombinasyonları, gerçek button activation ve T;
  Ctrl/Alt/Super ve gerçek Entry odağında t harfinin yakalanmaması.
- 500 ms gecikme, geri girişte iptal, hızlı giriş/çıkış ve panelde kalma;
  kısa T gösteriminin bitmesi, resize ile bölgenin motion olmadan güncellenmesi.
- İki pencerenin Enabled eşitliği/Visible bağımsızlığı; pending hide sırasında
  kapanış ve diğer pencerenin piksellerinin korunması. Yeni medya yüklemesi
  gizli kontrolleri açmıyor. Ek toggle kaynaklı decode işi yok.
- Gerçek GSK karşılaştırması: Fit/Fill/Actual ve zoom/pan sürekliliği; panel
  altında düğme dışındaki pikseller fotoğrafla aynı; açık/kapalı pick-through,
  alpha/hücre/underline, yatay scroll ve dar pencere kontrolleri.
- Settings birim testleri: missing/bozuk dosya, restart reload, değişimin abonelere
  yayılması, ölü abonelerin temizliği ve yazılamayan depolamanın nonfatal olması.
  Ayrı süreçlerde false→true→false yazma/yeniden açma testi de PASS.
- Rust fmt/check/build/test ve all-target Clippy -D warnings;82 Rust testi
  (58 unit+24 integration),30 Python gözlemci testi, fixture bütünlüğü.
  Locked/offline örnek build'leri, shell syntax ve whitespace kontrolü PASS.

İlk Clippy koşusunda listener field type-complexity FAIL; type alias ile
düzeltildi, son Clippy PASS. Resize testinde ilk X11/Wayland denemeleri deadline
FAIL verdi: GTK default-size ipucu, gecikmiş configure yanıtıyla değişebiliyordu.
Test gerçek frame/roundtrip ve gerçek allocation+Visible koşulunu birlikte bekler;
assertion kaldırılmaz, sahte boyut/frame üretilmez. Headless Sway için denenen
ek floating kuralı gereksizdi ve kaldırıldı. Bu tarihsel başarısız denemeler PASS
olarak sunulmaz. X11 küçülmesinde fiziksel imlecin pencere dışında kalıp
leave/reset oluşturduğu ayrıca gözlendi; otomatik controller izolasyonu bu
sentetik/güncel fiziksel giriş karışmasını giderdi. Üretimin gerçek leave'de
gizlenme davranışı değiştirilmedi.

PASS: X11 açık tema; X11 koyu tema+GDK_SCALE=2; headless Sway/Wayland.
Son X11 ve Sway koşuları nihai kontrollü sinyal/gerçek resize helper'ıyla geçti.
PASS: mevcut PIV-16 async_cancel_probe --controlled ve PIV-17 preload_probe
--controlled regresyonları (X11). Bu ikisi de sentetik frame/denetimli worker
kanıtıdır; gerçek decoder iptali, decoder overlap veya RSS ölçümü değildir.

NOT RUN: gerçek Arch GNOME/Wayland fiziksel fare/klavye ve görsel kabulü; yeni
secure Glycin decode/cancellation native paketi. Bilinen Codex Bwrap uid-map
engeli tekrar denenmedi; hiçbir sandbox koruması gevşetilmedi.

## Tek Arch GNOME komutu

```bash
bash tests/run_thumbnail_visibility_probe.sh
```

Önce özel geçici config ile otomatik UI ve ayrı-süreç persistence testleri koşar;
ardından iki **TEST görüntülü** pencereyi manuel inceleme için açar. Manuel oturum
normal Pivra thumbnail tercihini okur/yazar; kaydedilmiş Enabled başlangıçta
terminalde gösterilir. İki pencereyi kapatınca biter. Aynı komutu yeniden açarak
tercihin korunmasını kontrol et; otomatik testler gerçek tercihi sıfırlamaz.

Manuel: fareyi alta getir/uzaklaştır; düğme ve T ile aç/kapat; panelde kaydır,
düğmeden panele geç; hızlı giriş/çıkış ve resize dene. İki pencere Enabled'ı
paylaşmalı, geçici görünürlükleri ayrı olmalı. Fotoğrafta bant/daralma olmamalı;
Fit/Fill/Actual/zoom/pan korunmalı. Normal Pivra'da gerçek fotoğrafla Sol/Sağ
gezinirken etkin fakat gizli şeridin açılmadığını ayrıca kontrol et. Bu TEST
pencerelerinde gerçek katalog/thumbnail seçim bağlantısı veya üretim yoktur.

TH-03 TESTING. TH-04/TH-05/TH-07 kodlanmadı; PIV-18 TESTING, PIV-19 başlamadı.


## Faz2 kabul kanıtı değerlendirmesi — 25.10.2026

TH-06 ve TH-07 kullanıcı kabulü thumbnail gezinmesi/T/fullscreen bütünleşmesini
ve manuel kullanımın sorunsuzluğunu destekler. Ancak bu son beyan native uygulamayı
yeniden başlattıktan sonra Enabled tercihini doğruladığını ayrıca söylemez.
Codex bu denetimde gerçek ayrı süreçlerde false→true→false reload testini ve
GTK görünürlük/timer/resize kontrollerini tekrar PASS doğruladı; bu fiziksel
Arch kabulü değildir. **TH-03 TESTING** korunur, kalan dar kabul native restart/
kalıcılık ve belirtilen fare görünürlük listesine açık kullanıcı onayıdır.

Mevcut `bash tests/run_thumbnail_visibility_probe.sh` komutunda otomatik prefix
özel config kullanır; son manuel iki pencere normal kullanıcının thumbnail
tercihini okuyup yazabilir. Manuel tercih değişiminden sonra pencereleri kapatıp
aynı komutu yeniden açarak başlangıç değerini doğrula. Önceki native NOT RUN
kayıtları silinmez. [Ayrı kabul ve Faz2 raporu](PHASE2_RELEASE_READINESS.md).
