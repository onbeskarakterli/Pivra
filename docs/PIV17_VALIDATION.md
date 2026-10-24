# PIV-17 — ön yükleme ve sınırlı RAM önbelleği

24.10.2026 · dev · **TESTING**. PIV-01–16 DONE korunur; PIV-18 başlamadı.
Başlangıç: `e67bebf5e5aef7ffa64b12d997596d489e6ad241`.
Uygulama: `e2f7504c19f5b49d405e9f45c36d954e84e301a3`.
Probe/test: `a4fd9e083176ce885b9557db10860db5f513608e`.

## Politika ve kaynak sahipliği

Her pencerenin ayrı RAM cache'i vardır: en çok **3 girdi, 128 MiB hesaplanan
bütçe, girdi başına 64 MiB**. Aktif görüntü pinlenir; diğer girdiler LRU ile
çıkarılır. Sonraki komşu, önceki komşudan korunur. Maliyet checked
`width × height × 32` hesabıdır: iki RGBA32F temsilini muhafazakâr biçimde
hesaba katar. Aynı GDK texture referansları yeniden kullanılır. Bu sayı **RSS,
VRAM veya toplam uygulama belleği değildir**; decoder/pool, renderer, zoom
snapshot'ları ve uçuş halindeki frame'ler bütçeye dahil değildir. Çoklu pencere
maliyeti pencere sayısıyla artar. Bütçe üstü aktif fotoğraf gösterilebilir,
cache'e alınmaz ve bu seçimde ön yükleme yapılmaz. Cache varsayılanının amacı
aktif + iki küçük/orta komşuyu tutarken ek sahipliği sınırlamaktır; çok büyük
fotoğraflarda hız kazanımı garantisi yoktur.

Doğal katalogdan seçili dosya hariç en yakın sonraki JPEG/PNG, ardından en yakın
önceki JPEG/PNG seçilir. Video ve desteklenmeyen adaylar atlanır, sarma yoktur.
Aktif iş bitmeden background admission yoktur; aynı anda yalnız **bir**
background worker vardır. PIV-16'nın 3 admitted + 1 latest pending ve 5 saniye
admission süresi korunur. Kullanıcı isteği kalan kapasiteyi hemen kullanabilir;
eski background iptal edilir. Ön yüklenen dosya seçilirse mevcut iş yeni UI
token'ına devredilir, ikinci decoder başlatılmaz. Hata veren komşu geçici olarak
karantinaya alınır (en çok dört path); diğer komşu devam eder. Monitor olayı veya
Yenile bu karantinayı temizler. Background callback, seçime devredilmediği sürece
picture/spinner/status/mode üzerinde değişiklik yapmaz.

Cache anahtarı path + device/inode + uzunluk + nanosaniyeli mtime/ctime'dır.
Worker cache lookup öncesi/sonrası metadata'yı, decode öncesi/sonrası kimliği
kontrol eder; I/O sırasında cache mutex'i tutulmaz. Değişen decode cache'e
alınmaz. GIO dosya/rename/alias olayları cache'i muhafazakâr olarak tamamen
boşaltır, eski background işini iptal eder. Aktif path olayı epoch'u da değiştirir;
eski cache/promoted sonuç kabul edilmez. Mevcut ekrandaki görüntü ve Yenile
bildirimi PIV-11 politikasını korur. Explicit Yenile cache'i atlar ve taze decode
ister. Cache hit yeni fotoğrafı Fit/merkezde açar, zoom/pan sıfırlanır,
Smooth/Nearest tercihi korunur. EXIF yine yalnız güvenli Glycin yolunda uygulanır.

Güvenlik/üretim bağımlılıkları değişmedi. Header doğrulamasından sonra, next_frame
çağrısından önce, background bütçesine sığmayan boyut reddedilir; header/loader
başlatmanın da kaynak maliyeti vardır. Yeni format, disk cache, CI veya UI debug
alanı yoktur; `X candidates` metni korunur.

## Bellek baskısı ve ölçümler

Gerçek GIO **GMemoryMonitor::dup_default / low-memory-warning** sinyaline zayıf
referansla bağlanılır, kapanışta bağlantı kaldırılır. Bu ortamda seçilen tür
`GMemoryMonitorDBus`; system bus bağlantısı yoktur (`Could not connect: No such
file or directory`). Gerçek OS basınç bildirimi **NOT RUN**. Kontrollü enjeksiyon
PASS: ön yükleme pencerenin ömrü boyunca durdurulur, komşular çıkarılır, bütçe
her uyarıda yarılanır. Aktif texture UI'da korunur; daralan cache'e sığmıyorsa
cache sahipliği bırakılır. Aktif seçime devredilmiş iş basınçla iptal edilmez.

Probe cache hit/miss/rejected/evictions, entries/bytes/budget, preload
started/completed/cancelled/decoded/promoted, foreground decode ve maksimum
admission gecikmesini raporlar. Admission gecikmesi istekten UI tarafındaki
worker admission'a kadardır; OS thread başlangıcı veya piksel sunum süresi
ölçümü değildir. Metadata erişim hatası cache lookup sayılmaz. Controlled
X11/Sway örnekleri: cache hits=3, misses=6, evictions=5; ilk ortadaki fotoğraf
foreground decode=1, ardından background decode=2. Ön yükleme kapalı karşılıkta
iki fotoğraf foreground decode=2, background=0. Bu iş sayısı karşılaştırmasıdır;
**daha hızlı/daha az RAM** sonucu, gerçek RSS/VRAM/FPS ve decoder aktif zaman
çakışması **NOT MEASURED**. Glycin global pool'un 30 saniyelik idle retention'ı
korunur; uygulama running=0 sayacı loader PID'sinin çıktığı anlamına gelmez.

## Codex sonuçları

- **PASS:** `bash tests/check.sh` (mevcut activation betiğiyle), fmt/check/build/
  test/clippy all-targets -D warnings; locked/offline/stable/-j2. **72 Rust**
  (48 unit + 24 integration), 0 failed/ignored; **30 Python observer**;
  fixture/manifest bütünlüğü. Tam paket kod tamamlandıktan sonra bir kez koştu.
- **PASS:** tüm örneklerin locked/offline derlemesi, yeni shell syntax,
  `git diff --check`.
- **PASS — model:** next/previous doğal sınırlar, boş/tek öğe, karışık
  video/unsupported, metadata/atomic rename/delete, overflow, büyük girdi reddi,
  LRU/pin/öncelik ve pressure; background kapasitesi/promotion.
- **PASS — controlled GTK:** yeni probe X11 ve Sway. Texture nesne kimliği,
  Fit/Nearest/Smooth, cache hit'te decode olmaması, eviction/re-decode,
  atomic-save/delete, sabit bütçe/enjekte pressure, off karşılaştırması;
  deterministik barrier ile promotion, foreground admission, Yenile yarışı,
  sessiz bozuk komşu, 300 seçim, iki pencere ve close/join/cache clear.
  Bunlar **sentetik pikseller ve kontrollü worker** verileridir.
- **PASS — diğer 18 GUI çağrısı:** X11 navigation/watch/media-failures/
  window-race/navigation-race/async/timeout/modes/zoom/filter/basic/files/desktop
  (13); Sway async/watch/basic (3); Weston basic/files (2). Yeni Sway probe dahil
  runner 19, ayrıca yeni X11 controlled probe: **20 scoped grafik çağrısı**.
  Error-mode decoder hataları kontrollü UI sonucu olarak beklenir, decode PASS
  değildir. Sway gerçek Wayland, XWayland kapalı; Weston yüzey kontrolü normal
  kapanış kontrolü değildir. Weston normal close **NOT RUN**.
- **BLOCKED/NOT RUN:** yeni strict `preload_probe` Xvfb altında bir kez denendi,
  **exit 77**, secure loader/bwrap erken exit1; ilk görüntü oluşmadı. Bu deneme
  UID-map stderr'ini ayrıca vermedi; önceki read-only `/proc`/Bwrap engeli
  kaydı korunur, yeni kesin kernel teşhisi iddiası yoktur. Güvenlik gevşetilmedi.
  Yeni gerçek JPEG/PNG/EXIF/cache/promotion/aktif decoder close sonuçları PASS
  değildir; aynı native kapılar tekrar çalıştırılmadı.

## Tek native komut ve kalan kabul

Güncel dev checkout'unda, mevcut Arch/GNOME Wayland terminalinde:

```bash
bash tests/run_preload_probe.sh
```

Bu komut locked build → gerçek JPEG/PNG/EXIF 1–8 `image_probe` → secure
`preload_probe` → PIV-09 gerçek partial memfd/pidfd/STOP aktif decode kapanış
checkpoint'i sırasını kullanır; ilk başarısızlıkta durur. Son JSON:
`/tmp/pivra-preload-active-close.json`. Özel geçici kopyalar kullanılır; kullanıcı
fotoğraflarına yazılmaz. Native preload probe cache isabeti/tahliye/kimlik,
texture reuse ve kaynak bütünlüğünü sınar. Ek native öncelik/promotion/kapanış
senaryosundaki barrier yalnız **decoder öncesi worker girişindedir**: başarılı
çıktılar gerçek secure decoder'dan gelir, fakat bu barrier aktif decoder anını
kanıtlamaz. running/pending/background sıfır ve close sonrası boş cache ile
worker drain doğrulanır; gerçek process cleanup ayrıca son PIV-09 checkpoint'i
ile doğrulanır. Gerçek **background decoder** aktif aralığı/process kapanışı ve
iki decoder'ın zamansal çakışması bu sayaçlardan çıkarılamaz.

Tüm komut exit0 ve son JSON PASS sonucu kullanıcıdan alınacak; gerçek GNOME'da
hızlı Sol/Sağ, sıcak komşu, dosya değiştir/Yenile ve iki pencere sırasında görünür
hata/spinner olmaması ayrıca manuel kabul bekler. Native sonuç gelmeden DONE
verilmez. Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN,
PIV-39 tr/en planı açık kalır.
