# PIV-18B — gerçek erken önizleme uygulanabilirliği

24.10.2026 · dev · **PIV-18 TESTING**, PIV-19 başlamadı.
Başlangıç `b8924b85778e0386f81714dea6b99487ccbaa44a`, temiz dev'de normal FF.
Mevcut full-only native/manuel kabul `PIV18_VALIDATION.md` içinde kullanıcı
kaynaklıdır. Bu araştırma onu değiştirmez ve erken preview kabulü saymaz.

## Sonuç

**Mevcut incelenen JPEG/PNG loader yolunda gerçek erken preview kanıtlanmadı;
üretime önerilmiyor.** API scale isteğini kabul eder fakat loader isteği yok
sayabilir. İncelenen loader tam buffer oluşturup tek still frame'i tüketir.
Bu, Pivra'nın mevcut güvenliğiyle hiçbir zaman yapılamayacağı anlamına gelmez:
sertifikalı reduced-decode veya embedded-preview sağlayıcısı gereklidir.
Yeni sağlayıcı bu görevde eklenmedi. Tam decode sonrası resize çözüm değildir.

## Kaynak kanıtı

- Cargo client **Glycin3.0.8**, registry `src/api_loader.rs`:
  `Image::specific_frame(FrameRequest::new().scale(w,h))` vardır; doküman loader'ın
  istek alanlarını yok sayabileceğini söyler. `next_frame` still görüntüde yalnız
  bir kez kullanılabilir. Kaynak boyutları `details`, orientation
  `transformation_orientation`; frame lifetime sahiplik/Drop üzerinden yönetilir.
- Yerel resmi GNOME **Glycin2.0.0** checkout:
  `glycin-loaders/glycin-image-rs/src/main.rs::frame(_frame_request)` isteği
  kullanmaz, still decoder'ı `take()` eder; ikinci frame `NoMoreFrames` yoluna
  düşer. JPEG/PNG/TIFF aynı dispatcher içindedir.
  `glycin-utils/src/image_rs.rs::Handler::frame` decoder.total_bytes() büyüklüğünde
  shared memory ayırıp `read_image()` çağırır. Bu tam piksel maliyetidir; API
  imzasındaki scale reduced-decode kanıtı değildir. Arch'ın farklı loader
  sürümüne bu kaynak sonucunu otomatik genellemiyoruz.
- `glycin-loaders/glycin-raw/src/bin/glycin-raw.rs`: libopenraw raw_data(false),
  rendered_image yolu; frame isteği yine `_frame_request`. İncelenen yolda
  embedded thumbnail seçimi yoktur. Config'te DNG MIME bulunması kurulu/çalışan
  loader veya Pivra format desteği anlamına gelmez.
- `glycin-thumbnailer/src/main.rs`: thumbnailer dış sandbox varsayımıyla
  `NotSandboxed` seçer; scale ister, sonra gelen frame'e CPU resize uygulayabilir.
  Bu executable'ı mevcut ortamda doğrudan güvenli alternatif olarak çalıştırmadık.
  Dış sandbox'ı tasarlayıp doğrulamak ayrı iştir; sonradan resize erken preview
  kriterini karşılamaz. Kod yalnız okundu, sandbox bypass çalıştırılmadı.

## Yöntem/format matrisi

Aşağıdaki boyut/zamanlar **gerçek ölçüm**, değilse açıkça NOT RUN'dır.

| Yöntem/API, format | Gerçek ilk frame | İlk görüntü / full zamanı | Durum ve güvenlik | Bellek / öneri |
|---|---|---|---|---|
| specific_frame(scale) → same Image next_frame, PNG | NOT RUN | NOT RUN / NOT RUN | Tek Codex denemesi secure loader early exit1, probe77 | Buffer ölçülemedi; üretime HAYIR |
| Aynı deney, JPEG | NOT RUN | NOT RUN / NOT RUN | İlk PNG engelinde duruldu; tekrar deneme yok | Kaynak yolu tam allocation; üretime HAYIR |
| Aynı deney, TIFF | NOT RUN | NOT RUN / NOT RUN | Özel Codex config yalnız JPEG/PNG; TIFF runtime kaydı/fixture yok | Source dispatcher mevcut, Pivra TIFF desteği yok; DEFERRED PIV-40/41 |
| Aynı deney, RAW/DNG | NOT RUN | NOT RUN / NOT RUN | Özel prefix ve sistem standart dizinlerinde kurulu loader bulunmadı; fixture yok | Kaynakta full render var, üretim desteği yok; DEFERRED PIV-40/41 |
| Embedded preview erişimi, JPEG | NOT RUN | NOT RUN / NOT RUN | Glycin3.0.8'de kullanılabilir embedded-preview seçim API'si doğrulanmadı | Üretime HAYIR; ilgili PIV-18C prototipi 25.10.2026 tarihinde reddedilip kaldırıldı |
| Embedded preview erişimi, PNG | NOT RUN | NOT RUN / NOT RUN | İncelenen sağlayıcıda UNSUPPORTED; full decode → resize kabul edilmez | Üretime HAYIR |
| Embedded preview erişimi, TIFF/RAW | NOT RUN | NOT RUN / NOT RUN | Mevcut kurulu güvenli yol yok / API doğrulanmadı | Üretime HAYIR; PIV-40/41 desteği iddiası yok |
| Glycin thumbnailer, bütün formatlar | NOT RUN | NOT RUN / NOT RUN | Dış sandbox gerektirir, fallback resize yapabilir | Yeni bağımlılık/izolasyon işi; doğrudan kullanımı önerilmez |
| Yeni loader reduced decode / sınırlı adapter | NOT RUN | NOT RUN / NOT RUN | Bu görevde kurulmadı/çalıştırılmadı | Tarihsel alternatif; PIV-18C prototipi reddedilip kaldırıldı, aktif görev yok |

NOT RUN, FAIL decode sonucu veya PASS preview değildir. Kaynakta isteğin yok
sayılması loader yetenek sınırıdır; bozuk API diye raporlanmaz.

## Standalone probe

`examples/preview_feasibility_probe.rs`, yalnız Glycin API kullanır; ürün modülü
import etmez. Yeni bağımlılık yok. Her input için:

1. Regular file ≤64MiB; mandatory **Bwrap**, cancellable, transformations=true,
   base-dir exposure=false. Source metadata ≤32MP/32768 edge.
2. Monotonic Instant başlatılır; load/header süresi, MIME, kaynak boyutu ve
   orientation yazılır. Aspect korunarak max16 pixel hedefli scale isteği yapılır.
3. Dönen gerçek frame boyutu, frame_ready_us, texture download bitişi ve gerçek
   frame buffer uzunluğu yazılır. Texture indirilir, opak piksel varlığı aranır.
   Bu pencere üzerinde sunum zamanı değildir. Küçük frame dönse bile gizli full
   decode maliyeti dışlanmadığından **early-preview NOT PROVEN** yazılır.
4. İlk frame bırakılır; aynı Image üzerinde next_frame denenir. Gerçek boyut veya
   API hatası raporlanır. Süre ikinci çağrıdan itibaren ölçülür; kaynak yorumuna
   göre NoMoreFrames beklenmesi gerçek test PASS sonucu olarak yazılmaz.
5. Image bırakılır/token cancel edilir. **Yalnız deneysel kıyas için** yeni Image
   ile bir full baseline alınır; frame alanının metadata alanına eşitliği kontrol
   edilir. Baseline loader/header dahil süredir. Üretime ikinci decode eklenmez.

Aşamalar sıralı olduğundan ilk frame'in ikinci çağrıdan önce dönmesi tek başına
hız/erken preview kanıtı değildir. Warm pool, OS cache ve çok küçük varsayılan
JPEG kıyası performans sonucu üretmez. Byte başına output sınırları korunur;
frame ≤512MiB, output ≤32MP, download ≤128MB. Bunlar transient buffer sınırlarıdır,
proses toplam RSS/VRAM ölçümü değildir. Önceki frame bırakılır; en fazla4 input,
benchmark döngüsü yok. Glycin idle pool30s tutabilir; token cancel PID exit kanıtı
değildir. Gerçek worker/process sayısı ve RSS/VRAM **NOT MEASURED**.

## Native Arch komutu

Mevcut GNOME Wayland terminalinde:

```bash
bash tests/run_preview_feasibility_probe.sh
```

Varsayılan large.png1600×1200 ve no-exif.jpg64×32 fixture'larıdır. Boyutlar
fixture bilgisi, **yeni decode ölçümü değildir**. İsteğe bağlı1–4 gerçek dosya
path'i aynı betiğe verilebilir; büyük JPEG ve mevcut TIFF/DNG loader'ını sınamak
için bu gereklidir. Probe uzantıdan MIME atamaz ve dosyalara yazmaz. TIFF/RAW
kabulü yalnız standalone araştırmadır, Pivra format desteği değildir.
Script locked build sonrası Wayland/fatal-criticals altında90s deadline uygular.
Exit0 yalnız gözlemlerin tamamlandığını söyler; preview PASS değildir. İlk
loader/sandbox/unknown-format engelinde77, diğer hata1, assertion101, timeout124.
Küçük frame + süreler gelirse loader içindeki işlem maliyeti ayrıca kanıtlanmalı;
gerçek window presentation ölçümü bu probe'un kapsamı dışındadır.

## Codex doğrulaması

Debian13, GTK4.18.6/adw1.7.6, Bwrap0.12.0; mevcut özel loader prefix'i, 2+
protokol JPEG/PNG config. Rust/GTK kurulumu değiştirilmedi.

- **PASS:** hedefli probe build ve1 Rust boyut/overflow testi; fmt --check,
  probe Clippy -D warnings, shell syntax ve git diff --check.
- **NOT RUN:** tek secure PNG denemesi exit77; loader/Bwrap early exit1,
  frame/header ölçümü oluşmadı. Bilinen Codex namespace/proc engeli aşılmadı ve
  tekrar denenmedi. JPEG/TIFF/RAW için ilave secure denemeler yapılmadı.
- Üretim kodu değişmediğinden mevcut büyük core/GUI paketleri tekrar çalıştırılmadı.
  Eski native/manuel kabul yeni preview PASS sayılmadı.

## Tarihsel alternatif planı — artık aktif değil

Bu araştırmada ayrı güvenli JPEG embedded-thumbnail/reduced-IDCT prototipi,
sonrasında kaynak maliyeti ve üretim entegrasyonu değerlendirmesi önerilmişti.
PIV-18C bu alternatifleri denedi; kullanıcı 25.10.2026 tarihinde reddetti ve
prototip dosyaları kaldırıldı. Bu planın native testi, devam geliştirmesi veya
ürün entegrasyonu beklenmiyor. PIV-18B Glycin API/source ve native gözlemleri
korunur; mevcut full-only pipeline değişmez. **PIV-18 TESTING; PIV-19 başlamadı.**

Kod/test commit'i: `b0caa4f99e30031271b870d381b7fafee1d53fd0`.

## Arch GNOME Wayland native feasibility — 24.10.2026

Kullanıcı `807b67a402d19f81e6b2814fa6d3d644830e2786` sürümünde `bash tests/run_preview_feasibility_probe.sh` çalıştırdı: **exit 0 (observation complete, early preview NOT PROVEN)**. Zorunlu Bwrap etkin, Glycin3.0.8.

- `large.png`: source 1600×1200; request 16×12; returned 1600×1200 (**REDUCED_FRAME=false**), frame_ready 13051µs, fresh full baseline 6366µs; same-image `next_frame` NoMoreFrames.
- `no-exif.jpg`: source 64×32; request 16×8; returned 64×32 (**REDUCED_FRAME=false**), frame_ready 658µs, fresh full baseline 541µs; same-image `next_frame` NoMoreFrames.

**Karar:** Bu runtime JPEG/PNG scale isteğini küçültülmüş çözümleme olarak uygulamıyor. Aynı Image üzerinden ikinci still frame elde edilemiyor. Ölçümler küçük fixture ve seri warm-pool etkileriyle genel performans kıyası değildir. Tam decode→resize üretim erken preview seçeneği olarak reddedilir. O tarihte önerilen ayrı JPEG embedded-thumbnail/reduced-IDCT deneyi PIV-18C olarak yapıldı; 25.10.2026 kullanıcı kararıyla reddedildi ve kaldırıldı. Bu prototip için aktif test veya entegrasyon beklentisi yoktur. PIV-18 TESTING.
