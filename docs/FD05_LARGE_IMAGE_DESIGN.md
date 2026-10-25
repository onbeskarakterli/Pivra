# FD-05 — Büyük fotoğraf desteği teknik kararı

25.10.2026 — **Analiz tamam; limit değişikliği uygulanmadı.**
FD-04 baseline `f1dccab956e4699c18eae82fecf9a15a185ec946`,
FD-05 writer `d6cc7d07845841060787113bc4ec522596016ba7`.
Bu belge ölçümle doğrulanmış destek iddiası değil, ayrı uygulama kararı önerisidir.

## Mevcut engeller

1. FD-04 DECODE_BUDGET=256 MiB, mevcut texture_cost=32 byte/pixel:
   268435456/32=8388608 pixel ≈8.39 MP. Bu gerçek RGBA8 buffer boyutu değildir;
   ana cache'in iki worst-case16-byte temsil muhasebesi thumbnail'a da uygulanır.
2. Ortak image_loader::validate_dimensions ayrıca32,000,000 pixel ve32768 edge
   sınırı uygular; image.load/details sonrası ve frame sonrası doğrular.
3. MAX_FRAME_BYTES=512 MiB gerçek dönen frame buffer'ına frame oluşturulduktan
   sonra uygulanır. FD-04 ayrıca gerçek frame256 MiB ve RGBA download64 MiB sınırına
   sahiptir. Kaynak dosya64 MiB ile sınırlıdır; compressed boyut pixel sayısı değildir.

Dolayısıyla yalnız256 MiB bütçeyi artırmak48/50 MP'yi açmaz. Yalnız32 MP sınırını
artırmak da download/frame/iş eşzamanlılığı ve child peak RAM sorunlarını çözmez.
Bugün24 MP FD-04'te,48–200 MP ortak loader'da engellenir. Kullanıcının FD-04 native
PASS bildirimi small/fixture pipeline kabulüdür;24–50 MP ölçümü bildirilmedi.

## Glycin3.0.8 ve loader kanıtı

Client registry api_loader.rs'de specific_frame(FrameRequest::scale) ve clip vardır;
doküman loader'ın request alanlarını yok sayabileceğini açıkça söyler. Frame'de
width/height/stride/memory_format/buf_slice ve color_state okunabilir. İncelenen
FrameRequest'te decoder çıktı pixel formatını zorlayan bir seçenek görülmedi.
Bunu destekleyen başka bir API varsayılmadı.

[Önceki native kayıt](PIV18_PREVIEW_FEASIBILITY.md#arch-gnome-wayland-native-feasibility--24102026):
kullanıcı Arch'ta Glycin3.0.8/Bwrap ile807b67a üzerinde scale16x12 istediğinde
large.png1600x1200 döndü (13051µs, fresh full6366µs); JPEG64x32 için16x8 isteği
64x32 döndü (658µs, full541µs). Aynı Image'da next_frame NoMoreFrames verdi.
Bunlar kullanıcı ölçümleri, yeni Codex benchmark'ı değildir; küçük/warm-pool
ölçümlerinden genel hız sonucu çıkarılmaz. Bilinen başarısız API deneyi tekrarlanmadı.

Yerel resmi Glycin2.0.0 checkout yeniden hedefli incelendi:
glycin-image-rs::frame(_frame_request) isteği kullanmaz;
glycin-utils/image_rs.rs decoder.total_bytes() kadar SharedMemory ayırıp
read_image çağırır. Bu full pixel maliyetidir. Bu kaynak sürümü ile tüm gelecek
Arch loader sürümleri aynı kabul edilmez; runtime/version kanıtı ayrıca gerekir.

Küçük frame dönse bile child önce full decode edip küçültmüş olabilir. Gerçek
reduced decode için actual frame + parent/child peak memory + süre ayrı kanıtlanmalı.
EXIF/ICC/alpha correctness otomatik varsayılmaz; tüm orientation ve pixel kontrolleri
tekrarlanmalıdır. FD-04 full decode→resize yapar; PIV-18C'nin reddedilip kaldırılmış
libjpeg/embedded prototipi geri getirilmeyecektir.

## Bellek hesabı — ESTIMATE, benchmark değil

MP decimal milyon pixeldir; aşağıdaki MB decimal milyon byte'dır. Çıktı RGBA en
fazla4 MiB; encoder/decoder geçici buffer'ları ve file bytes bu tablonun dışındadır.

| Kaynak | RGBA8 4B/px | RGBA16 8B/px | RGBA32F16B/px | Mevcut muhasebe32B/px |
|---|---:|---:|---:|---:|
|12 MP|48 MB|96 MB|192 MB|384 MB|
|24 MP|96 MB|192 MB|384 MB|768 MB|
|48 MP|192 MB|384 MB|768 MB|1536 MB|
|50 MP|200 MB|400 MB|800 MB|1600 MB|
|108 MP|432 MB|864 MB|1728 MB|3456 MB|
|200 MP|800 MB|1600 MB|3200 MB|6400 MB|

JPEG frame RGB8 ise3B/px olabilir;4B/px tablosu her decoder'ın gerçek formatı değildir.
Parent, tam frame'e ek RGBA8 download tutar: örneğin50 MP RGBA8 frame+download
≈400 MB; float frame+download≈1000 MB, çıktı/overhead hariç. Child'da bağımsız decode
ara buffer'ları/ICC/EXIF dönüşümleri bulunabilir.50 MP float frame800 MB zaten
512 MiB cap'i aşar; PNG16 RGBA400 MB sığabilse de dönüşümün daha büyük format
üretmeyeceği varsayılamaz.32B/pixel modelini doğrudan4'e indirmek güvenli çözüm değildir.

Yeni24/48/50/108/200 MP gerçek RSS/latency ölçümü **NOT RUN / NOT MEASURED**.
Codex'te aynı secure Bwrap runtime kısıtı sürer; bu görevde büyük decode denemesi,
loader scale retry'si veya limit bypass yapılmadı. Bu sayılar destek PASS'i değildir.

## Güvenli ölçüm ve admission tasarımı

Boyutlar sandbox'lı loader.load/details üzerinden alınmalı; header bilgisi yine
untrusted'dır, integer/edge/pixel sınırları doğrulanmalıdır. Source prefix veya
compressed file size'dan güvenilir peak RAM çıkarılamaz. Metadata load'unun bile
maliyetsiz olduğu varsayılmamalı. Gerçek frame boyutu/stride/format/buffer tekrar
kontrol edilmelidir; frame sonrası kontrol pre-allocation hard limit değildir.

Önerilen ölçüm ayrı native test işidir:

- Aynı kaynak24/48/50 MP JPEG ve PNG8/16; client/loader/Bwrap sürümü, EXIF/ICC/alpha,
  frame byte'ları ve mono zaman kaydedilsin.108/200 MP ancak hesaplanan bütçe izinliyse.
- Parent PID ve tüm gerçek loader descendants pid/starttime/pidfd ile izlenmeli;
  başlangıçtan var olan idle-pool loader körlemesine dışlanmamalı.
- /proc/PID/status VmRSS/VmHWM, smaps_rollup RSS/PSS/Private/Shared değerleri birlikte
  örneklensin. Memfd sayfaları parent+child RSS toplamında iki kez sayılabilir;
  PSS veya zaten delege edilmiş cgroup memory.peak daha iyi birleşik kanıttır.
  Host cgroup/namespace güvenliği değiştirilmesin.
- Örnekleme aralığı ve eksik /proc okumaları raporlansın; tek parent VmHWM veya
  /usr/bin/time çıktısı tüm child peak RAM'i olarak sunulmasın.
- Gerçek aktif decode iptali, hızlı dosya değişimi ve tüm child cleanup ayrıca
  doğrulansın; queued/finished işi aktif decode diye sayma.

Uyarlanabilir admission için thumbnail'a özgü, test edilmiş policy gerekir:
actual/worst supported frame formatı + normalization buffer + çıktı + ölçülmüş
child/transformation reserve. Yalnız piksel başına4 byte varsayımı yetmez.
Large işlerde başlangıçta **tek active large decode**, küçük işler için mevcut
iki-worker tavanını aşmayan global weighted budget önerilir. Normal fotoğrafın
foreground önceliği korunmalı, eski istekler iptal edilip cleanup tamamlanmadan
sonsuz yeni işler başlatılmamalıdır. Yeni daemon/pool gerekmez.

8–16 GiB referans sistem ve yeterli MemAvailable üzerinde24–50 MP için aday
policy native ölçümle belirlenmeli; şu anda onaylanmış sayısal yeni limit yoktur.
Bwrap filesystem isolation tek başına OOM hard cap değildir. Resource containment
API'si veya kullanıcı-delegated limit varsa ayrıca doğrulanmalı; host güvenlik
ayarını değiştirmek veya Auto/NotSandboxed fallback çözüm değildir.

## Alternatifler

| Yol | Doğrulanmış durum / maliyet / karar |
|---|---|
| Mevcut scale isteği | JPEG/PNG native örnekte full frame; çözüm olarak kabul edilmez |
| Mevcut secure full decode + weighted admission |24 MP için ortak32 MP cap içinde araştırılabilir;50 MP için ayrıca scoped pixel/frame/download policy değişikliği ve ölçüm gerekir; en küçük uygulanabilir kısa vadeli yön |
| Gerçek upstream loader-side downsampling | Uzun vadede RAM'i düşürebilir; mevcut path'te kanıt yok. Aynı Bwrap içinde, gerçek boyut/peak kanıtıyla ayrı sürüm/loader işi |
| Güvenli thumbnail API/thumbnailer | Mevcut wrapper'ın dış sandbox varsayımı vardır; doğrudan unsandboxed executable kabul edilmez. Güvenlik ve full-decode maliyeti ayrıca kanıtlanmalı |
| clip/tiled/kademeli decode | API clip parametresi tile decode/RAM garantisi değil; incelenen still loader request'i yok sayıyor. Bugün uygulanabilir/çalışır sayılmaz |
| Host decoder, yeni libjpeg adapter veya sandbox bypass | Önerilmez; reddedilmiş prototipler geri getirilmeyecek |

108–200 MP için destek bugün **yok/kanıtlanmadı**; frame ve download sınırları da
engeldir. Gerekli RAM/format/policy sağlanmıyorsa dosyaya özgü anlaşılır limit hatası
ve mevcut placeholder korunmalı; provider değiştirilmemeli, source değiştirilmemeli,
sınırsız retry yapılmamalıdır.24–50 MP asgari ürün hedefi bu incelemeyle DONE değildir.

## Karar: FD-06 öncesi ayrı FD-05A önerisi — BAŞLATILMADI

1. Onaylı küçük native ölçüm matrisi ve referans bellek gereksinimi belirle.
2. Ortak secure boundary'de mevcut defaultları değiştirmeyen scoped thumbnail
   decode policy tasarla;32 MP,512 MiB,256 MiB ve64 MiB download etkileşimlerini
   birlikte ele al. Sadece bir constant artırma. Ana fotoğrafın32 MP davranışını
   değiştirmek ayrıca kapsam kararı gerektirir.
3. Ölçülmüş weighted admission, bir large-job sınırı, EXIF/alpha/ICC ve gerçek
   aktif cancellation/cleanup kabulünü tamamla. Native24/48/50 MP PASS olmadan
   bu ürün hedefini tamamlandı gösterme.
4. Ayrı native kabul ardından FD-06 yetkisi iste. Writer'ın no-clobber politikasında
   stale cache yenileme kararı da entegrasyon öncesi açıkça çözülmeli.

FD-04/ortak loader üretim belleği politikası bu görevde değişmedi.
FD-05A veya FD-06 için kullanıcıdan yeni yetki beklenir; otomatik devam yoktur.
