# PIV-18 — Thumbnail ve fullscreen alt görev planı

25.10.2026 · dev · **TH-07 DONE — kullanıcı native kabulü. PIV-18 TESTING.**
TH aşamaları mevcut PIV-18 kapsamına bağlı alt görevlerdir; yeni PIV aşaması
oluşturulmaz, PIV numaraları ve diğer aşamaların sırası değiştirilmez.
PIV-18 TESTING; PIV-19 başlamadı.

## Sıra ve durum

| Aşama | Kapsam | Durum |
|---|---|---|
| TH-01 | GNOME thumbnail altyapısı | DONE — kullanıcı native kabulü başarılı |
| TH-02 | Thumbnail overlay | DONE — kullanıcı TH-03 talimatında native kabulü bildirdi |
| TH-03 | Otomatik gizlenme, `T`, kalıcı ayar | TESTING — kod/otomatik testler; native kabul bekleniyor |
| TH-04 | Thumbnail gezinme senkronizasyonu | DONE — kullanıcı Arch GNOME kabulü, 1b11074 |
| TH-05 | Sistem thumbnail üretimi | DONE — kullanıcı Arch GNOME kabulü, 1b11074 |
| TH-06 | Thumbnail genel kabul ve optimizasyon | DONE — kullanıcı native kabulü, 25.10.2026 |
| TH-07 | Tam ekran fotoğraf görüntüleme | DONE — kullanıcı Arch/GNOME fiziksel kabulü, 25.10.2026 |

TH-02 [kullanıcı native kabulüyle DONE](TH02_THUMBNAIL_OVERLAY_VALIDATION.md).
TH-03 [ayrı native kabul bekler](TH03_THUMBNAIL_VISIBILITY_VALIDATION.md).
TH-04/TH-05 [kullanıcı native kabulüyle DONE](TH05_THUMBNAIL_SYSTEM_VALIDATION.md).
TH-06 kullanıcı native kabulüyle DONE. TH-07 [kullanıcı native kabulüyle DONE](TH07_FULLSCREEN_VALIDATION.md). TH-03 bağımsız TESTING; PIV-18 kapsam kararı açık.

## TH-07 — Tam Ekran Fotoğraf Görüntüleme

TH-06 kabulünden sonra uygulandı; TH-03 görünürlük ve kısayol kurallarıyla bütünleşir.

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

## TH-07 kabul kapısı — otomatik kontroller / ayrı native kabul

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
İlk planlama kaydından sonra kullanıcı talimatıyla üretim fullscreen ve testler eklendi; native kabul henüz yapılmadı.


## Faz2 denetimi — 25.10.2026

TH-07 kullanıcı native/fiziksel kabulüyle DONE. TH-03 otomatik sinyal/timer ve
ayrı-süreç persistence testleri Codex'te PASS olsa da native yeniden başlatma/
Enabled kalıcılığına ayrı açık kabul verilmedi; TESTING korunur. TH-07'nin T ve
thumbnail gezinmesi kabulü o bağımsız maddeye otomatik taşınmaz.
PIV-18/Faz2 kapanışı için gerçek erken preview ve foreground durum göstergesi
kapsamları karara bağlanmalı. Full-only başarı PIV-18 DONE değildir.
[Faz2 ve0.1.0 raporu](PHASE2_RELEASE_READINESS.md).
