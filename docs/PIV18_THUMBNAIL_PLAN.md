# PIV-18 — Thumbnail ve fullscreen alt görev planı

25.10.2026 · dev · **Yalnız planlama; geliştirme yetkisi değildir.**
TH aşamaları mevcut PIV-18 kapsamına bağlı alt görevlerdir; yeni PIV aşaması
oluşturulmaz, PIV numaraları ve diğer aşamaların sırası değiştirilmez.
PIV-18 TESTING; PIV-19 başlamadı.

## Sıra ve durum

| Aşama | Kapsam | Durum |
|---|---|---|
| TH-01 | GNOME thumbnail altyapısı | DONE — kullanıcı native kabulü başarılı |
| TH-02 | Thumbnail overlay | DONE — kullanıcı TH-03 talimatında native kabulü bildirdi |
| TH-03 | Otomatik gizlenme, `T`, kalıcı ayar | TESTING — kod/otomatik testler; native kabul bekleniyor |
| TH-04 | Thumbnail gezinme senkronizasyonu | TODO |
| TH-05 | Sistem thumbnail üretimi | TODO |
| TH-06 | Thumbnail genel kabul ve optimizasyon | TODO |
| TH-07 | Tam ekran fotoğraf görüntüleme | TODO — yalnız planlandı |

TH-02 [kullanıcı native kabulüyle DONE](TH02_THUMBNAIL_OVERLAY_VALIDATION.md).
TH-03 [ayrı native kabul bekler](TH03_THUMBNAIL_VISIBILITY_VALIDATION.md).
TH-04–TH-07 yalnız plandır; bu görevde kodlanmadı.

## TH-07 — Tam Ekran Fotoğraf Görüntüleme

TH-06 sonrasında uygulanacak; TH-03 görünürlük ve kısayol kurallarıyla bütünleşir.

1. Ana fotoğraf alanında çift tıklama fullscreen açar/kapatır.
2. `F` fullscreen açar/kapatır.
3. `Esc` fullscreen modundan çıkar.
4. Fullscreen sırasında başlık çubuğu, menüler ve pencere kontrol düğmeleri görünmez.
5. Fotoğraf tüm ekran alanını kullanır; görüntü oranı korunur.
6. Thumbnail şeridi ve alt orta düğmesi mevcut bağımsız, yüzen overlay tasarımını korur;
   fotoğraf viewport'undan yer ayırmaz.
7. Thumbnail şeridinin görünürlüğü TH-03 kurallarına uyar.
8. Sol/Sağ fotoğraf gezinmesi korunur; gizli şerit gezinme nedeniyle kendiliğinden açılmaz.
9. `T` kısayolu fullscreen sırasında da çalışır.
10. Çift tıklama yalnız ana fotoğraf alanında etkindir; thumbnail veya düğme etkileşimi
    fullscreen tetiklemez.
11. Fullscreen aç/kapa mevcut fotoğrafı yeniden decode etmez.
12. Normal pencereye dönüşte pencere boyutu ve mevcut görüntüleme tercihleri korunur
    (Fit/Fill/Actual, zoom/pan ve resampling dahil).
13. Birden fazla Pivra penceresinin fullscreen durumu birbirinden bağımsızdır.
14. GTK/libadwaita'nın yerleşik fullscreen mekanizması kullanılır; yeni pencere
    oluşturulmaz, görüntüleme motoru yeniden başlatılmaz.
15. Native Arch Linux GNOME Wayland testi ile klavye/fare regresyonları kabul koşuludur.

## TH-07 kabul kapısı — henüz çalıştırılmadı

- Gerçek GNOME'da çift tıklama, `F`, `Esc`, `T` ve Sol/Sağ; focus ve overlay
  etkileşimleriyle birlikte doğrulanır. Thumbnail/düğmeye çift tıklamanın fullscreen
  başlatmadığı ve gizli şeridin gezinmede açılmadığı sınanır.
- Fullscreen'de başlık/menü/kontrol yokluğu, tüm ekranın kullanılması, oran ve overlay
  yerleşimi; normal pencereye dönüşte boyut ve tercihlerin korunması doğrulanır.
- Aynı fotoğraf/texture korunur ve ek decode başlamadığı ölçülür; iki gerçek pencerenin
  fullscreen durumlarının bağımsızlığı ve mevcut görüntüleme regresyonları sınanır.
- Uygun GTK/X11/Wayland otomatik kontrolleri ile native Arch GNOME görsel/klavye/fare
  kabulü ayrı raporlanır. Çalıştırılmayan test PASS sayılmaz; native kabul olmadan DONE yoktur.

Mevcut secure Glycin, asenkron yükleme, cancellation, cache ve preload korunur.
Bu belgeyle üretim kodu veya test uygulaması eklenmemiştir.
