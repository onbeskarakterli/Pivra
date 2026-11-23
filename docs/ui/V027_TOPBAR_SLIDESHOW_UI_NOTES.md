# Pivra — Üst bar ve slayt gösterisi arayüzü değişiklik notları

Tarih: 2026-11-23
Durum: UYGULANDI / TESTING — `feature/v0.2.7-toolbar-ux`; fiziksel GNOME kabulü bekleniyor.
Kapsam: Aşağıdaki kayıtlı isteklerin uygulanması; GTK/EGL hata araştırması bu çalışmanın dışındadır.

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

İlk plan kaydı korunmuştur. Uygulama yalnız yetkilendirilen toolbar feature dalındadır; dev/main, paketler, GPU motoru ve GTK yamaları değiştirilmemiştir. Geçici XWayland varsayılanı korunur (`main.rs` farkı yalnız rustfmt satır düzenidir). Mevcut native kabul ve GTK kaynaklı açık sorunlar çözülmüş sayılmaz.

## Uygulama ve odaklı doğrulama

- Ayrı görüntüleme (Fit/Fill/100% ve filtre), magnifier ve birleşik Play/süre/geçiş grupları. Yeni kırpma/düzenleme motoru yoktur.
- Manuel süre 1–3600 tam saniyedir; geçersiz giriş mevcut süreyi değiştirmez. Hazır 3/5/10/15/30 seçenekleri korunur. Fade yaklaşık 300 ms; No transition eski snapshot/tick kaynağını da temizler.
- Süre, geçiş, görüntüleme ön ayarı, Smooth/Nearest ve magnifier tekerlek-zoom tercihi mevcut atomik/private XDG ayar yoluyla saklanır. Fotoğrafa özgü geçici zoom/pan konumu başka medyaya taşınmaz.
- Toast'lar mevcut alanda tek güncel slotu paylaşır; eskisi dismiss edilir, timeout sonrası slot temizlenir. Tekrarlanan F5 çoğaltmaz; resume eski pause mesajını kaldırır, yeni mesaj uydurmaz.
- `cargo fmt --check`, `cargo check --locked`: PASS. Yalnız ilgili 14 slideshow model testi ve 1 interval ayar testi: PASS. Tam regresyon/Clippy/release paketi bu görevde çalıştırılmadı.
- Gerçek GTK kontrolleri, doğrulama, fade/none, F5/F11/Space/oklar/Esc, toast değişimi ve iki pencere: X11/Xvfb ve headless Sway PASS; ikinci temiz süreçte beş tercihin geri yüklenmesi her iki ortamda PASS. Fotoğraf pikselleri açıkça kontrollüdür; native Glycin/AMD kabulü değildir.
- 8 saniyelik gerçek stock video / 3 saniyelik fotoğraf süresiyle mevcut odaklı EOS, pause/seek ve hata-koruma kapısı X11'de PASS. İlk Sway fade testi kısa görünürlüğü sonradan örneklerken timeout verdi; aynı koşullar gerçek after-paint callback'inde kaydedilerek test bariyeri düzeltildi ve geçti. Üretim süresi uzatılmadı, assertion gevşetilmedi.
- Tekrar: `python3 -B tests/toolbar_ux_smoke.py --session x11` (veya `sway`), etkin mevcut SDK ile. Native görsel kabul: `cargo run --locked -- /path/to/photo`; header gruplarını ve restart sonrası tercihleri kontrol edin. Fiziksel GNOME testi henüz yapılmadı.
