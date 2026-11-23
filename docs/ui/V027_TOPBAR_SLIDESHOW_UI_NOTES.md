# Pivra — Üst bar ve slayt gösterisi arayüzü değişiklik notları

Tarih: 2026-11-23
Durum: İSTEK / PLANLAMA — henüz uygulanmadı.
Kapsam: Gelecek Pivra arayüz çalışması. Mevcut slideshow hata ayıklama ve GTK çalışmalarından bağımsızdır.

## Üst bar düzeni

Tek bir seçenek/menü altında toplanmış kontroller ayrıştırılacak. Üst barda **her özellik ayrı bir kutu/grup** olarak gösterilecek:

1. **Resim kırpma biçimi** seçimi: üst barda kendi kutusu.
2. **Zoom ikonu/kontrolü**: üst barda kendi kutusu.
3. **Slayt gösterisi grubu**: aynı kutunun içinde
   - slayt gösterisini başlatan **Play** düğmesi,
   - **süre seçimi**,
   - **geçiş şekli seçimi**.

Kontroller birbirinden ayırt edilebilir olmalı; slideshow Play/süre/geçiş türü aynı kutuya ait olmalı.

## Slayt gösterisi ayarları

- Süre, hazır seçeneklerin yanında **manuel girilebilmeli**.
- Son seçilen **özellikler, seçenekler, geçiş şekli ve süre** uygulama tekrar açıldığında korunacak biçimde **kalıcı kaydedilmeli**.
- Mevcut F5/Space/oklar/Esc davranışları korunmalı; video, fotoğraf intervali dolduğunda değil gerçek bitişinde ilerlemeli.
- Geçiş biçimlerinin sunumu ve kalıcı tercihi üst bardaki slideshow grubunda olmalı (hangi geçiş biçimlerinin bulunacağı uygulama aşamasında netleştirilecek).

## Baloncuk / toast bildirimleri

Slayt gösterisi durum değişimlerinde uygulama içi baloncuk gösterilecek:

- Başlangıç: `Slideshow started`
- Space ile duraklatma: `Slideshow paused`
- Slayt gösterisi sona erdiğinde / kapatıldığında: `Slideshow finished`

Duraklatma ve devam ettirme davranışı korunacak. Devam ettirme için mesaj metni ayrıca kararlaştırılmadı; uydurulmuş bir metin zorunlu kabul edilmesin.

## Sınırlar

Bu belge kullanıcı talebinin kaydıdır; kod, paket, GTK veya dev/main dalı değiştirilmemiştir. Mevcut özelliğin native kabulü ve GTK kaynaklı açık sorunlar çözülmüş sayılmaz.
