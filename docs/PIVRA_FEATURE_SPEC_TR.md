# Pivra — Kararlaştırılan Özellikler ve Davranışlar

**Durum:** Mantıksal tasarım; **henüz kod yazılmadı**.  
**Hedef:** GNOME/Linux için Nautilus'tan açılan resimden başlayıp aynı klasördeki fotoğraf ve videolar arasında tek pencerede gezinilen hızlı görüntüleyici. Varsayılan video oynatıcısı olmayacak.  
**Güncelleme:** 20.10.2026

## 1. Temel kullanım ve arayüz

| No | Özellik | Kabul edilen davranış |
|---|---|---|
| 1 | Başlatma | Nautilus'ta fotoğrafa çift tık; klasördeki fotoğraf+videolara geçiş. Dahili dosya yöneticisi veya kütüphane yok. |
| 2 | Video ilişkisi | Video dosyasına doğrudan çift tıklamada GNOME'un varsayılan oynatıcısı kullanılır. |
| 3 | Sıralama | Karışık medyalar doğal dosya adı sırasıyla (1, 2, 10); Nautilus sıralamasından bağımsız. |
| 4 | Gezinti | Sol/sağ ok önceki/sonraki medya; normal gezinmede başa sarma ayarlardan açılabilir. |
| 5 | Başlangıç görüntüsü | Her yeni medya merkezlenir ve kullanılabilir alana sığdırılır; önceki zoom/konum aktarılmaz. |
| 6 | Görüntü ölçeği | Ekrana sığdır, en-boy oranını koruyarak doldur (gerekirse kırp), gerçek boyut (%100). Esnetme yok. |
| 7 | Yakınlaştırma | Fare tekerleği imleç merkezli zoom, %10–3200; interpolasyon veya nearest-neighbor seçimi. |
| 8 | Taşıma | Yakınlaşmış görsel fare sol tuşuyla sürüklenir. |
| 9 | Mini harita | Fotoğraf/video görüntüsü ekrandan taştığında sağ altta otomatik görünür; görsel alanı gösterir ve sürüklenebilir. Tam sığıyorsa gizlenir. |
| 10 | Küçük resim şeridi | Altta, fare yaklaşınca ana medyanın üzerine açılır; görüntüyü yeniden ölçeklemez; yatay kaydırılabilir, aktif medya ortalanır. |
| 11 | Şerit seçimi | Normalde tıklama medyayı açar; ayrı **Seç** modu, çoklu/aralık/Tümünü Seç/Seçimi Temizle ve adet gösterimi. Seçim modunda **Fotoğraf / Video / Tümü** filtresiyle seçilebilecek medya türü değiştirilebilir. Yalnız desteklenen dosyalar başlatılan toplu işleme alınır; uygun olmayan seçimler açıklanır. |
| 12 | Şerit önbelleği | Pivra küçük resimleri **yalnız RAM'de** tutar; kalıcı disk önbelleği yazmaz; bellek basıncında azaltır. |
| 13 | Ön yükleme | Yakın fotoğraflar akıllı hazırlanır; RAM/CPU/GPU baskısına uyum; videolar bütünüyle belleğe alınmaz. |
| 14 | Ağır RAW/TIFF | Varsa gömülü düşük çözünürlüklü önizleme hemen; tam çözümleme arka planda, hafif ilerleme göstergesi. |
| 15 | Canlı klasör | Dosya ekleme/silme/yeniden adlandırma listeye yansır. Açık dosya haricen değişirse otomatik yenilenmez, **Yenile** bildirimi. |
| 16 | Hatalı medya | Yalnız gerçekten okunabilen kısım gösterilir; bozuk dosya uyarısı; navigasyon kesilmez. |
| 17 | Bilgi paneli | İsteğe bağlı sağ panel EXIF/fotoğraf ve codec/FPS/süre/bitrate/video bilgileridir; açılınca viewport daralır, medya refit olur. |
| 18 | Arayüz | Minimal GNOME'a uygun; sabit alt toolbar yok; uygun zamanda beliren overlay kontroller. |
| 19 | Tam ekran | F11; slayt gösterisinden ayrı. |
| 20 | Renk/şeffaflık | Şeffaf alanlarda dama tahtası varsayılan; beyaz/koyu/sistem arka planı ayarlanabilir. |
| 21 | Renk yönetimi | Gömülü ICC/ekran profili otomatik, HDR destekli oturumda HDR, değilse SDR ton eşleme; gösterme dosyayı değiştirmez. |
| 22 | Yön | EXIF orientation ve video rotation metadata'sı uygulanır; kaydederken çift döndürme önlenir. |
| 23 | Pencereler | Nautilus'ta her ayrı dosya açılışı ayrı pencere; son pencere boyutu saklanır, konumu GNOME yönetir. |
| 24 | Tema | GNOME sistem teması varsayılan; Sistem/Açık/Koyu seçimi. |
| 25 | Geçmiş | Varsayılan kapalı; açılırsa son dosyalar/klasörler; temizlenebilir. Ayarlar geçmişten bağımsız kalıcıdır. |
| 26 | Kısayollar | Klavye ve desteklenen fare hareketleri yeniden atanabilir; çakışma denetimi ve sıfırlama. |

## 2. Video ve animasyon

| No | Özellik | Kabul edilen davranış |
|---|---|---|
| 27 | Oynatma | Navigasyonla video açılınca otomatik oynar; başka medyaya geçince durur, ses kesilir. |
| 28 | Video sonu | Varsayılan son karede bekle; seçeneklerden farklı bitiş davranışı ayarlanabilir. |
| 29 | Video kontrolü | Oynarken 2 sn hareketsizlikte gizlenir, duraklatıldığında kalır, imleç kontrol üzerindeyken gizlenmez. |
| 30 | Kısayollar | Space oynat/duraklat; Ctrl+sol/sağ 5 sn sarma; virgül/nokta duraklatılmışken kare geri/ileri; yukarı/aşağı ses. |
| 31 | Kare adımı | Mümkün olan en doğru kare hassasiyetinde ileri/geri, değişken FPS zamanlaması gözetilir. |
| 32 | Zaman çizgisi | Hover ile önizleme karesi+zaman; tıklamadan oynatma pozisyonu değişmez. |
| 33 | Zoom ve pan | Aynı videoda oynatma, duraklama, seek ve kare adımında korunur; yeni medyada ekrana sığdırılır. |
| 34 | Pozisyon | Aynı pencere içinde videoya dönülünce kaldığı yerden otomatik oynar; pencere kapanınca unutulur. |
| 35 | Ses | Ses seviyesi ve mute durumu uygulamanın sonraki açılışlarında korunur. |
| 36 | Video kare kaydetme | Kaynak çözünürlüğünde, ekran/zoom'dan bağımsız; PNG varsayılan, JPEG/WebP, önerilen zaman damgalı ad. |
| 37 | Alt yazı/parçalar | Gömülü/harici SRT ve ASS; aynı adlı harici altyazıyı otomatik algılama; birden fazla ses parçası seçimi. |
| 38 | Alt yazı stili | ASS'nin özgün biçimi varsayılan korunur; yazı boyutu/renk/çerçeve/arka plan ayarı ve sıfırlama. |
| 39 | Formatlar | MP4/MKV/MOV/WebM/AVI/WMV/MPEG/3GP gibi konteynerler; H264/HEVC/VP8/VP9/AV1 vb.; donanım çözme, yazılıma geri düşme. Gerçek destek platforma bağlı. |
| 40 | Hareketli fotoğraf | GIF, animasyonlu WebP, APNG otomatik ve sonsuz döngü; Space ile duraklat; ayrılınca durur. |
| 41 | Güç yönetimi | Slayt gösterisi boyunca (duraklatılsa bile) ve normal modda video oynatılırken ekranın otomatik kapanması engellenir; normal video duraklayınca güç yönetimi normale döner. |

## 3. Slayt gösterisi

| No | Özellik | Kabul edilen davranış |
|---|---|---|
| 42 | Başlatma | F5: geçerli medyadan tam ekran slayt; F11'den bağımsız. |
| 43 | Süre ve döngü | Fotoğraf süresi ayarlanabilir; video tamamen oynatılır; son dosyadan ilkine döner, normal gezinme döngüsü ayarından bağımsız. |
| 44 | Kumanda | Space duraklat/devam; oklarla elle geçiş; Esc çıkış. |
| 45 | Geçiş | Sadece slayt fotoğraflarında kısa, yaklaşık 300 ms crossfade; normal navigasyon kesintisiz/anlık. |

## 4. Düzenleme

| No | Özellik | Kabul edilen davranış |
|---|---|---|
| 46 | Mod | Düzenleme yalnız ayrı modda; normal görüntüleme kalabalıklaşmaz; kayda kadar tahribatsız. |
| 47 | Döndürme/çevirme | 90/180/270°, yatay/dikey çevirme, +/-180° serbest açı ve ufuk hizalama; boş köşelerin otomatik en az kayıplı kırpılması. |
| 48 | Kırpma | Serbest, özgün, 1:1, 4:3, 3:2, 16:9, 9:16, özel oran; en-boy oranı bozulmaz. |
| 49 | Boyut | Hedef piksel ölçüsü veya dosya boyutu; kalite/çözünürlük optimizasyonu ve tahmini sonuç. |
| 50 | Ton/renk | Parlaklık, kontrast, doygunluk ve renk düzeltmeleri. |
| 51 | Otomatik iyileştirme | Gerçek zamanlı önizleme; etki %0–100. |
| 52 | Kırmızı göz | Otomatik tespit/düzeltme, manuel seçim/düzeltme. |
| 52A | Gelişmiş ışık ve renk | Pozlama (Exposure), parlak bölgeler (Highlights), gölgeler (Shadows), renk sıcaklığı (Temperature), yeşil-magenta renk tonu (Tint) ve canlılık (Vibrance): altı bağımsız kontrol, anlık önizleme ve geri alma. |
| 53 | Keskinlik | Keskinlik (Sharpen), gürültü azaltma (Denoise), netlik (Clarity): üç ayrı kaydırıcı. |
| 54 | Öncesi/sonrası | Sürüklenebilir **dikey ayırıcı**, sol orijinal/sağ düzenlenmiş, ortak zoom/konum. |
| 55 | Geri alma | Ctrl+Z ve uygun işlemlerde yinele. |
| 56 | Kaydetme | Ctrl+S ve Ctrl+Shift+S Farklı Kaydet; mümkünse kayıpsız JPEG dönüşümü, ICC/EXIF koruma, güvenli atomik yazma. |
| 57 | Kaydedilmemiş değişiklik | Medya değişiminde, düzenleme modundan çıkışta, pencere kapatmada **Kaydet / Kaydetmeden Devam Et / İptal**. |
| 58 | Kapsam sınırı | Metin, ok, şekil, anotasyon yok; genel fotoğraf editörü değil. |

## 5. Dosya işlemleri, toplu dönüştürme, karşılaştırma

| No | Özellik | Kabul edilen davranış |
|---|---|---|
| 59 | Format dönüşümü | Tek dosyada Farklı Kaydet, çoklu şerit seçiminde toplu resim dönüştürme; hedef format, kalite, yeniden boyut, çıktı klasörü. |
| 60 | Arka plan işlemi | Dönüşüm sürerken fotoğraf/video gezinme açık; ilerleme ve per-file hata; iptal. |
| 61 | Çakışma | Mevcut dosyaların üzerine yazılmaz: foto.webp varsa foto (1).webp vb. ilk uygun yeni ad. |
| 62 | Duraklat/devam | Aktif dosya tamamlandıktan sonra kuyruk duraklar; devam kalanlardan; tamamlananlar tekrar işlenmez. |
| 63 | Bildirim | Pivra öndeyken uygulama içi, arka planda GNOME bildirimi; çift bildirim yok; sonuç sayıları ve çıktı klasörünü açma. |
| 64 | Kapanış | Aktif dönüştürmede **İşlemi İptal Et ve Çık / Pivra'ya Dön** sorulur; tamamlanan çıktılar kalır; yarım dosya bırakılmaz. |
| 65 | Dosya silme | Delete ile onaysız Çöp Kutusu; sonraki medyaya geç; Geri Al bildirimi; kalıcı silme yok. |
| 66 | Dosya adlandırma | F2 yalnız açık dosya; uzantı varsayılan korunur, isim çakışması önlenir; sıra yeniden hesaplanır. Toplu rename yok. |
| 67 | İki dosya karşılaştırma | Sağ tık Compare, aynı klasörden ikinci görsel; yan yana; ayrı gezinti; aktif paneli tıklayarak seç. |
| 68 | Eşzamanlı zoom | Karşılaştırmada açılabilir; her görsel kendi fit boyutuna göre aynı oranda büyür; başlangıç merkezi ortak. Transparan üst üste bindirme yok. |
| 69 | GNOME duvar kâğıdı | Açık resmi masaüstü duvar kâğıdı yap. |

## 6. Medya formatı ve ek kapsam

| No | Özellik | Kabul edilen davranış |
|---|---|---|
| 70 | Görsel formatları | JPEG, PNG, GIF, WebP, AVIF, HEIC, BMP, TIFF, APNG, RAW DNG/CR2/CR3/NEF/ARW, SVG, JPEG XL, ICO, PSD ve mümkün diğerleri. |
| 71 | Çoklu sayfa/katman | TIFF sayfaları, ICO çözünürlükleri ve decoder destekliyorsa PSD katmanları için yalnız gerektiğinde panel. |
| 72 | Desteklenmeyenler | Kendi dosya yöneticisi, medya kütüphanesi, histogram, kapsamlı panoya kopyalama, genel sürükle-bırak ve ek sıralama seçenekleri yok. |

## 6A. Çoklu monitör ve HiDPI — 69. madde (KARARLAŞTIRILDI)

| Alan | Kabul edilen davranış |
|---|---|
| Arayüz ölçeği | GTK arayüzü pencerenin bulunduğu monitörün HiDPI / kesirli ölçeklendirme ayarına otomatik uyarlanır. |
| Fit / Fill | Monitör ya da kullanılabilir viewport değiştiğinde hedef ekrana göre yeniden hesaplanır, en-boy oranı korunur. |
| Elle zoom ve pan | Kullanıcının elle yaptığı zoom ve kaydırma mümkün olduğunca korunur; yeni viewport'ta geçerli sınırlar içine alınabilir. |
| Monitör geçişi | Pencere farklı çözünürlük ve ölçekli monitöre taşınırken gereksiz sıfırlama ve görüntü bozulması önlenir. |
| Dosya | HiDPI davranışı kaynak medyayı veya kaydedilen çözünürlüğü değiştirmez. |

## 6B. Touchpad ve dokunmatik hareketleri — 70. madde (KARARLAŞTIRILDI)

| Hareket/bağlam | Kabul edilen davranış |
|---|---|
| Pinch (iki parmakla açma/kapatma) | İmleç/dokunma odağına uygun biçimde zoom in/out. |
| Büyütülmüş medya | Yatay/dikey kaydırma öncelikle fotoğraf veya videonun görünür alanını pan eder; yanlışlıkla dosya değişimini engeller. |
| Medya ekrana sığıyorsa | Yatay hareket önceki/sonraki medyaya geçiş yapabilir. |
| Dokunmatik ekran | Desteklenen donanımda uygun pan ve pinch hareketleri; fare/klavye davranışları daima kullanılabilir. |
| Uygulanabilirlik | Touchpad ve dokunmatik sürücü/Wayland gesture olayları gerçek donanımla test edilecek. |

## 6C. Yerel Linux paketleri — 71. madde (KARARLAŞTIRILDI)

| Konu | Karar |
|---|---|
| Genel ilke | Pivra Linux dağıtımlarında **yerel (native) paket** olarak sunulacak; Flatpak/AppImage öncelikli veya zorunlu dağıtım kanalı olmayacak. |
| Arch tabanlı | PKGBUILD ile paketleme; uygun olduğunda AUR üzerinden erişim. |
| Debian/Ubuntu tabanlı | DEB paketleme. |
| Fedora/RHEL/openSUSE ailesi | Hedef dağıtıma uygun RPM paketleme. |
| Diğer dağıtımlar | Uygun yerel paket biçimi veya dağıtımın doğal build/kurulum yönergeleri; bütün dağıtımlara tek bir evrensel paket vaat edilmez. |
| Kütüphaneler | Dağıtımların güncel ve desteklenen GTK4, libadwaita, GStreamer vb. yerel sistem paketlerinden yararlanılır. Asgari sürümler ve codec/eklenti bulunabilirliği doğrulanır. |
| Dosya izinleri | Normal Linux kullanıcı ve dosya sistemi izinleri geçerlidir; Flatpak sandbox/portal sınırlamaları hedeflenmez. |
| Doğrulama | Desteklenen dağıtım/sürüm matrisi, GTK/GStreamer sürüm farkları ve paket kurulumu gerçek sistemlerde test edilecek. |

## 7. Henüz karara bağlanmayanlar / teknik doğrulama

| Konu | Durum |
|---|---|
| Codec/RAW/HDR/PSD/JPEG XL desteğinin tam sınırları | **UYGULAMA TESTLERİYLE DOĞRULANACAK.** |

> Bu doküman özellik ve davranış kararlarının kaydıdır; geliştirme veya commit edilmiş kaynak kod değildir. Push sırasında otomatik CI istenmiyor; yeni workflow eklenirse yalnızca manuel çalıştırma düşünülmeli.


## 8. Teknik mimari kararı — Rust (20.10.2026)

**Karar:** Pivra'nın ana uygulama dili **Rust** olacaktır. GNOME'a doğal uyumlu bir arayüz ve medya işlevleri birbirinden ayrılmış modüler bir yapı hedeflenir. Bu bölüm teknoloji tercihini kaydeder; henüz kodlama başlatma yetkisi değildir.

| Katman / araç | Tercih | Durum ve not |
|---|---|---|
| Ana dil | **Rust** | **KARARLAŞTIRILDI** |
| Kullanıcı arayüzü | **GTK4 + libadwaita**, Rust için gtk4-rs / libadwaita bindings | **HEDEF MİMARİ**; GNOME arayüzü |
| Video motoru | **GStreamer** / gstreamer-rs | **İLK TERCİH**; GTK4 içinde oynatma, ses, altyazı, seek, frame-step ve zoom prototipte doğrulanacak |
| Görsel çözümleme | **Glycin** ve formatlara özgü uygun çözücüler | **İLK ADAY**; RAW, HDR, HEIC, JXL, PSD vb. gerçek destek tek tek test edilecek |
| Görüntü işleme | **libvips** ve gerektiğinde özel görüntü işlemleri | **İLK ADAY**; Rust entegrasyonu, kalite ve işlem kapsamı test edilecek |
| Dosya ve klasör olayları | **GIO** | **HEDEF ALTYAPI** |
| Kalıcı uygulama ayarları | **GSettings** | **HEDEF ALTYAPI**; görüntüleme geçmişi varsayılan kapalı |
| Derleme/bağımlılıklar | **Cargo** | **KARARLAŞTIRILDI** |
| Dağıtım | **Dağıtıma özgü yerel paketler** (Arch PKGBUILD; Debian/Ubuntu DEB; Fedora/RHEL RPM; diğerleri için uygun yerel biçim) | **KARARLAŞTIRILDI**; Flatpak/AppImage hedef değil; her dağıtımın güncel sistem kütüphaneleri kullanılır, sürüm/bağımlılık uyumluluğu ayrıca test edilir |

### Modüler sorumluluklar

1. **Medya/katalog yöneticisi:** Nautilus'tan açılan dosya ve klasör, doğal sıralama, dosya olayları, gezinme konumu.
2. **Görüntüleme motoru:** Decode, fotoğraf/video viewport'u, zoom/pan, renk profilleri, kaynak kontrollü hazırlama.
3. **Video motoru:** GStreamer oynatma yaşam döngüsü, ses/altyazı, kare adımı, seek ve video önizlemesi.
4. **Düzenleme motoru:** Geri alınabilir, kayda kadar tahribatsız işlemler; format dönüştürme ve atomik yazma.
5. **Arka plan iş yöneticisi:** Kaynak önceliği, iptal, duraklat/devam, durum/ilerleme ve bildirimler.
6. **GTK4/libadwaita sunum katmanı:** Pencere, şerit, paneller, kısayollar ve etkileşimler; medya işlemleriyle gevşek bağlılık.

### Kodlama öncesi doğrulama ölçütleri

- Büyük fotoğraf/RAW açılışı, zoom ve RAM tüketimi; önizlemeden tam çözüme geçiş.
- GTK4 içinde 4K video, donanım hızlandırma ve yazılımsal fallback; fotoğraf ↔ video geçişleri.
- Frame-step, harici/gömülü altyazı ve ASS biçimlendirme; video kaynağından orijinal çözünürlükte kare kaydı.
- ICC/HDR ve desteklenen gerçek formatların her birinin platforma göre doğrulanması.
- Her aşamada birim/entegrasyon testleri ve gerçek GNOME kullanım testi; otomatik push tetiklemeli CI yok (eklenirse yalnız manuel workflow_dispatch).

**Sınır:** Bu karar, daha önce kabul edilmiş 72 özellik ve 7. bölümdeki açık maddeleri değiştirmez. Teknik doğrulamada bir aday uygun çıkmazsa kullanıcıyla alternatif belirlenir. **Henüz kaynak kodu değiştirilmeyecek; kalan tasarım kararları sonraki görüşmeye bırakıldı.**

### Karar günlüğü — 20.10.2026

- **67. madde — KARARLAŞTIRILDI:** Gelişmiş ışık ve renk kontrollerinin altısı da eklenecek (Exposure, Highlights, Shadows, Temperature, Tint, Vibrance). Normal görüntüleme ekranında değil, düzenleme modunda bağımsız ayarlar olarak bulunacak; anlık önizleme ve geri alma davranışı diğer düzenlemelerle aynı olacak.

- **68. madde — KARARLAŞTIRILDI (20.10.2026):** Küçük resim şeridindeki ayrı Seç modunda Fotoğraf / Video / Tümü filtresi bulunur. Bu filtre çoklu seçim kapsamını belirler. İşlemler yalnız destekledikleri medya türlerine uygulanır.

- **69. madde — KARARLAŞTIRILDI (20.10.2026):** Çoklu monitörde akıllı HiDPI uyarlaması; Fit/Fill yeniden hesaplanır, kullanıcı zoom/pan tercihi mümkün olduğunca korunur.

- **70. madde — KARARLAŞTIRILDI (20.10.2026):** Akıllı touchpad/dokunmatik hareketler; pinch zoom, büyütülmüş görüntüde öncelikle pan, ekrana sığdığında yatay hareketle medya geçişi. Donanım desteği gerçek cihazlarda doğrulanacak.

- **71. madde — KARARLAŞTIRILDI (20.10.2026):** Tüm hedef Linux dağıtımları için dağıtıma özgü yerel paketleme; Flatpak tercih edilmez. Güncel sistem kütüphaneleri ve medya motorları kullanılır; fiili paket kapsamı desteklenen dağıtım/sürüm matrisi ile doğrulanır.
