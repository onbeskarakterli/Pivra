# FD-05A — Yüksek çözünürlüklü thumbnail / kabul kaydı

25.10.2026 — **TESTING. Native yüksek çözünürlük ve aktif cleanup kabulü beklenir.**
Başlangıç dev:c7339b9f611536bcb010492f9bda6000d6739be4.
- Thumbnail policy/shared opt-in API/test:082eaa5ceb32b2339bf5c46c56a0f03e1a1429e9.
- Native benchmark/observer:579e47e552bba8290f865c9a9cecf27852a616d9.

## Kullanıcı kararı — shared-cache değişmeyecek

Koşullu shared-cache replacement yarışı kullanıcı tarafından değerlendirildi.
RENAME_NOREPLACE korunur; koşulsuz replace, sil-sonra-yaz ve ikinci Pivra kalıcı
cache'i **uygulanmayacak**. Önceki özel-cache önerisi reddedildi. Eski negatif
protokol testleri4c8b382 hâlâ geçerlidir; bir yenileme algoritması PASS'i değildir.

FD-06 için onaylanan yön: Stale/Invalid durumda secure generator'ın güncel sonucunu
bellekte sun, shared-cache dosyasına dokunma. Bu routing/arayüz entegrasyonu burada
yapılmadı. Writer ve Reader davranışları aynı kaldı.

## Yeni thumbnail'a özgü opt-in politika

`image_loader::load_thumbnail` mevcut tek Bwrap/Glycin boundary'sine küçük opt-in
API ekler. `load` ve `preload` eski defaultlarıyla çalışır:32MP/32768 edge,
64MiB source/512MiB frame, preload'un eski32byte/pixel muhasebesi korunur.
Yeni API50,000,000 pixel deneysel thumbnail tavanını ve ayrı admission callback'ini
uygular. JPEG/PNG MIME, Bwrap zorunluluğu, EXIF dönüşümü ve expose_base_dir(false)
değişmez. Ana fotoğraf politikası yükseltilmedi.

`freedesktop_generator` full-decode→resize yapar; reduced decode varsayımı veya
PIV-18C/libjpeg prototipi yoktur.12/24/48/50MP artık eski8.39MP gate'inden dolayı
reddedilmez; aşağıdaki kontrollerden geçmelidir:

- Secure image.details sonrası, next_frame öncesi toplam rezervasyon
  `pixel*32 +128MiB` (checked arithmetic).32B muhafazakâr iki worst-case temsil,
 128MiB ek decoder/dönüşüm reserve'idir; native peak RAM ölçümü yerine geçmez.
- Süreç genelindeki standalone thumbnail rezervasyon toplamı en çok2GiB.
 8,388,608pixel üzeri **en çok bir large-job**; mevcut iki-job/dev+inode dedup
 tavanı korunur. Busy fail-fast; yeni queue/thread pool yoktur.
- MemAvailable ve görünür finite cgroup-v2 atalarının memory.max-current değerinin
 minimumu, toplam rezervasyon+512MiB sistem headroom'unu karşılamalıdır. Eksik/
 okunamayan zorunlu bellek verisi veya yetersiz headroom kontrollü hata verir.
 `memory_headroom()` yalnız gözlemlenmiş headroom'dur; RAM garantisi değildir.
- Büyük rezervasyon decode, download, resize ve publication boyunca RAII ile
 tutulur; hata/iptal sonrası bırakılır. Gerçek frame pixel sayısı rezervasyonu
 aşamaz. Edge/stride/buffer tekrar doğrulanır.
- Frame512MiB; RGBA download224MiB; çıktı hâlâ sınıfa göre <=4MiB.50MP RGBA8
 download200MB'dır. Float50MP frame800MB512MiB'yi aşar ve LimitExceeded olur;
 tüm format/bit depth'ler için koşulsuz50MP sözü verilmez.

Bu sayılar **deneysel admission ceiling**'dir; ölçülmüş Supported etiketi değildir.
24MP rezervasyon≈902MB,50MP≈1734MB; buna512MiB free headroom gerekir.50MP+1MP
small job global bütçeye sığabilir, iki large job sığsa bile yasaktır.
Native child peak'i128MiB reserve'den büyük olabilir. Bwrap tek başına RAM hard
cap değildir; başka uygulamalar memory snapshot sonrası RAM tüketebilir. Post-frame
512MiB kontrolü de frame'in hiç allocate edilmediği anlamına gelmez.
108/200MP desteklenmez. Daha büyük sınır bu görevde verilmedi.

Main foreground bu admission'a bağlanmadı; şerit/GNOME/foreground priority davranışı
etkilenmez. FD-06 üretime bağlarken mevcut foreground önceliği ve global worker
permit'i birlikte korunmalı; bağımsız limitleri toplayarak yeni worker açılmamalı.

## Native tek komut ve ölçüm anlamı

```bash
bash tests/run_fd05a_native.sh
```

Önce mevcut secure JPEG/PNG/EXIF1–8/alpha generator ve writer roundtrip regresyonları;
ardından izole12/24/48/50MP RGB8 PNG **ve JPEG** benchmark'ı çalışır. PNG fixture'ları
row-streamed zlib ile, JPEG fixture'ları mevcut encoder ile güvenilir RGB dizisinden
oluşur (en çok150MB, headroom kontrolü; tek producer, decode öncesi producer çıkar).
Yeni kaynak decoder'ı yoktur. Kullanıcının fotoğraf/cache'i kullanılmaz. Fixture'lar
geçicidir; JSON rapor varsayılan olarak0600 rastgele tempfile'da kalır ve yolu yazılır.
`--report /yeni/dosya.json` verilirse mevcut dosya ezilmez.

Rapor: source format/dimensions; monotonic decode/resize; actual frame boyut,
stride/format/bytes (`FD05A_FRAME` trace); output/download bytes; parent PID/starttime
ve VmRSS/VmHWM; loader PID/starttime ile ayrı RSS/PSS örnekleri. Eksik ölçümler
NOT MEASURED'dır. Memfd shared sayfalar RSS toplamında iki kez sayılabilir; parent
RSS toplam iş RAM'i değildir. Varsa cgroup memory.peak before/after kaydedilir,
**shared existing cgroup**, reset edilmemiş historical peak olarak etiketlenir;
izole iş peak'i gibi sunulmaz. Örnekleme5ms hedeflidir, /proc tarama maliyeti eklenir;
kısa peak'ler kaçabilir. Loader process sayısı gerçek decode concurrency sayısı değildir.

Son aşama50MP PNG'de aktif iptal sertifikası arar: own descendant/starttime+pidfd;
hedef RGB8 frame için tam3*pixel memfd boyutu, writable mapping/inode, doğru nonzero
prefix ve zero tail. pidfd SIGSTOP sonrası gerçekten stopped durum ve aynı partial
kanıt yeniden doğrulanır. Parent'a iptal iletilip CANCEL_ISSUED alınır; SIGCONT finally
ile garanti edilir. Queue/bitmiş frame bu ölçütü karşılamaz. Future Cancelled,
kaynak kimliği değişmezliği ve gözlemlenmiş descendants'ın exit/zombie durumu gerekir.
Eksik checkpoint77/NOT RUN; cleanup/future hatası FAIL olur. RGB8 layout değişirse
bu checkpoint NOT RUN kalabilir; başarı kriteri gevşetilmez.

Bu native benchmark bir GTK pencere kapanış entegrasyonu değildir. Bağımsız iki
aktif kontrollü isteğin iptal izolasyonu mevcut testte korunur; eşzamanlı iki gerçek
native isteğin temporal overlap'i ayrıca ölçülmüş kabul edilmez. Kamera JPEG/PNG16,
ICC/HDR ve büyük EXIF'li gerçek kaynaklarla ek kabul gerekir; küçük mevcut native
EXIF/alpha kapıları büyük formatların tamamının kanıtı değildir.

## Gerçek Codex sonuçları

| Kontrol | Sonuç |
|---|---|
| cargo check --locked --offline | PASS |
| cargo test --locked --offline | **174 PASS**,0fail/ignored;171 baseline+1weighted policy+ortak opt-in policy testinin iki yerde çalışması |
| cargo clippy --locked --offline --all-targets -- -D warnings | PASS |
| Hedefli generator testleri | PASS;12/24/48/50 geometry, low-memory rejection, tek large, small+large bütçesi ve RAII dahil |
| Thumbnail Python testleri | **37 PASS** |
| Yeni observer testleri | **4 PASS**, gerçek own memfd/mapping ve parser/fixture; native decoder PASS'i değil |
| Native probe build / Python syntax / shell syntax / diff check | PASS |
| Native tek giriş | **NOT RUN / exit77**, ilk secure baseline PNG'de Bwrap loader exit1; tekrar yok |
| Büyük native JPEG/PNG ölçümleri ve actual active decoder cleanup | **NOT RUN / NOT MEASURED**; fixture benchmark aşamasına geçilmedi |
| cargo fmt --check | **FAIL, önceden mevcut**: image_view.rs447, active_decode_close_probe.rs211, application_open_probe.rs65/83/97/135; yeni dosyalar formatlı |

Eski24MP-reject geometry beklentisi yeni opt-in policy için108MP-reject ile
karşılandı;24MP için pozitif geometry/admission ve low-headroom negatif test eklendi.
Ana loader'ın48MP'yi reddetmesi ayrıca test edilir. Assertions atlanmadı.

## Değişiklik ve açık kabul

Değişenler: generator, minimum shared-loader opt-in API/test, kontrollü generator
testleri, yeni native Rust/Python probe/script ve bu belge. GNOME thumbnail.rs,
thumbnail_strip.rs, image_view/window, native sidecar/build.rs, Reader/Writer,
Cargo.toml/lock değişmedi (git diff --exit-code). Yeni bağımlılık yoktur.

FD-04/05 native PASS kullanıcı kanıtı olarak korunur; yeni high-resolution policy'nin
native kabulü değildir. **FD-05A TESTING**: en az24MP gerçek PASS,48/50MP format/RAM
sonuçları ve aktif native cleanup gelmeden DONE yapılamaz. Shared-cache kararı
kapanmıştır; özel kalıcı cache veya replacement algoritması yoktur. FD-06'ya geçilmedi.
