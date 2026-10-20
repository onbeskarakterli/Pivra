# Pivra — Teknik Yetenekler ve Faz 0 Kararları

**Tarih:** 21.10.2026 · **Dal:** dev · **Kapsam:** yalnız PIV-05.
Başlangıç commit'i: `8b3947215c54dfd2515df61a50dd05c080eccf31`.
PIV-06 uygulanmadı. Production kaynakları, Cargo.toml ve Cargo.lock değiştirilmedi;
görüntü decoder'ı veya video oynatıcı yazılmadı. Proje lisansı seçilmedi.

## 1. Kanıt sınıfları ve yöntem

- **ÇALIŞTIRILDI:** bu makinede komut/probe veya regresyon sonucu; kapsamı belirtilir.
- **KAYNAK/BELGE:** sürümü belirtilen upstream kaynak, crate manifesti veya dağıtım paket metaverisi; runtime başarısı değildir.
- **NOT RUN:** bu görevde çalıştırılmamış ürün davranışı/donanım testi.
- **BLOCKED:** belirli bir deneyin somut ortam eksiği; PIV-05 araştırmasının tamamının başarısız olduğu anlamına gelmez.

Registry'de bir GStreamer factory bulunması codec'in dosyayı başarıyla oynattığını,
pkg-config kaydı runtime işlevini, Cargo çözümlemesi de derleme/ABI/runtime başarısını
kanıtlamaz. Bu ayrımlar bütün tablolarda geçerlidir.

Doğrudan docs.gtk.org, docs.rs, gstreamer.freedesktop.org ve GNOME Pages istekleri
bu ortamın proxy'sinden 403 aldı. Araştırma durmadı: Cargo'nun doğrulanmış crate
paketleri, sürümlenmiş GNOME/GStreamer GitHub mirror kaynakları ve resmi Debian
APT metaverisi okundu. Kaynaklar bölümünde sürümler sabitlenmiştir; hareketli main
README'sindeki yeni format/backend değişiklikleri eski sürümlere aktarılmadı.

## 2. Doğrulanan ortam ve mevcut kütüphaneler

| Bileşen | Gözlem ve sınır |
|---|---|
| Platform | Linux, Debian GNU/Linux 13 (trixie), x86_64 |
| Rust / Cargo | 1.99.1 / 1.99.1; proje rust-version 1.85, edition 2024; 1.85 ile ayrıca derleme yapılmadı |
| GTK / libadwaita / GLib | Native 4.18.6 / 1.7.6 / 2.84.4; mevcut geliştirme dosyaları, gerçek build ve GUI regresyonları ile doğrulandı |
| Rust GTK hattı | gtk4/gdk4 0.10.3, libadwaita 0.8.1, glib/gio 0.21.5 |
| Backend metaverisi | gtk4-wayland ve gtk4-x11 pkg-config 4.18.6; gtk4.pc targets: broadway, wayland, x11; Wayland ve X11 gerçek app smoke ile ayrı doğrulandı, Broadway NOT RUN |
| Grafik testleri | Xvfb/X11; Sway 1.10.1 / wlroots 0.18.2 headless; Weston 14.0.2 headless, software renderer |
| GStreamer runtime | gst_init_check + registry sorgusu ile 1.26.2; hiçbir pipeline/decoder/player oluşturulmadı |
| GStreamer geliştirme | gstreamer-1.0 ve gstreamer-video-1.0 .pc yok; gst-inspect-1.0 komutu yok; runtime varlığından SDK varlığı çıkarılmadı |
| Glycin | İstemci/loader kurulu değil; glycin-1/glycin-2 .pc ve loader dosyaları bulunmadı; crate kaynakları yalnız araştırma için Cargo cache'ine alındı |
| Diğer runtime'lar | libheif 1.19.8 (dav1d/libde265 plugin paketleri), libjxl 0.11.2, LibRaw 0.21.4, librsvg 2.60.0, libtiff 4.7.0, libwebp 1.5.0, lcms2 2.16 paket/runtime gözlemleri; format decode deneyleri NOT RUN |
| Diğer SDK'lar | libwebp 1.5.0 ve libtiff-4 4.7.0 .pc erişilebilir; libheif/libjxl/lcms2/vips SDK kayıtları yok. Fontconfig mevcut; libseccomp SDK kaydı yok |
| Donanım/oturum | /dev/dri ve /dev/snd yok; fiziksel GNOME/Wayland, GPU ve ses çıkışı NOT RUN |

**190 erişilebilir pkg-config modülünün tam listesi**, seçili sürüm/sorgu hataları
ve GStreamer factory kayıtları [PIV05_ENVIRONMENT.json](probes/PIV05_ENVIRONMENT.json)
içindedir. Liste aktivasyonla erişilen .pc dosyalarıdır; hepsinin sistem paket
veritabanına kurulmuş olduğu iddia edilmez. Önceki görevlerin yerel native dosyaları
ve Rust/GTK aktivasyonu korundu; bu görevde sistem paketi kurulmadı veya yükseltilmedi.

Tekrarlanabilir read-only envanter:

```sh
source /workspace/pivra-environment/gtk-activate.sh
python3 tests/probes/environment.py
```

Probe GStreamer registry'sini private geçici dizinde oluşturup siler; medya dosyası
açmaz, pipeline kurmaz ve herhangi bir factory'yi codec testi olarak etiketlemez.

## 3. GTK4 / libadwaita yetenekleri

| Konu | Doğrulama / API | Henüz doğrulanmayan |
|---|---|---|
| Native uygulama/pencere | **ÇALIŞTIRILDI:** mevcut AdwApplicationWindow, native header/controls, Pivra başlığı, 960x640, bağımsız süreçler ve X11/Sway normal kapanışları | Fiziksel GNOME Shell davranışı, desktop entegrasyonu |
| Texture / Paintable | **ÇALIŞTIRILDI:** dosyasız 1x1 RGBA MemoryTexture -> GdkPaintable -> GtkPicture; pixel download round-trip | Gerçek dosya decode, alpha/renk profili/çok büyük texture doğruluğu |
| Snapshot/render | **ÇALIŞTIRILDI:** Paintable.snapshot, GtkSnapshot -> GSK TextureNode ve 1x1 bounds | GPU render, zero-copy, farklı driver/renderer piksel eşitliği |
| HiDPI | **ÇALIŞTIRILDI:** realized surface scale_factor ve runtime scale property, scale=1 | Fractional scale, monitör değiştirme ve ekranlar arası gerçek DPI |
| Ölçek API'si | **KAYNAK:** GdkSurface.get_scale / scale property GTK 4.12; integer scale_factor farklıdır. Mevcut Rust v4_6 feature tabanında yeni getter kapalı olduğundan probe property erişimi kullandı | Rust feature tabanı bu görevde yükseltilmedi; 1.25/1.5/2 ölçekli gerçek compositor testi yok |
| Pinch/touchpad | **ÇALIŞTIRILDI:** GestureZoom ve BOTH_AXES EventControllerScroll nesneleri. **KAYNAK:** gesture sinyalleri ve scroll olayları kullanılabilir | Gerçek pinch/gesture olayı, touchpad cihazı, zoom/pan uygulaması NOT RUN |

GtkPicture/Paintable sunumu mevcut viewport'a uygun ilk sunum adayıdır. GTK snapshot
widget durumunu render düğümlerine dönüştürür; bu düğümler renderer tarafından
işlenir. Snapshot oluşturulması nihai GL/Vulkan veya renk yönetimi başarısı değildir.
Texture fiziksel pixel boyutu ile widget'ın logical ölçüsü ayrılmalı; pencere scale
değişimi, fit ve ilerideki pointer-anchored zoom aynı koordinat dönüşümünü kullanmalıdır.
Pinch/touchpad desteği API varlığından otomatik ürün özelliği hâline gelmez.

Sentetik, uygulamadan ayrı probe (decode ve sahte Pivra pencere testi değildir):

```sh
GTK_A11Y=none GSK_RENDERER=cairo GDK_BACKEND=x11 \
  xvfb-run -a dbus-run-session -- bash tests/probes/run_gtk_api.sh
```

Geçici Cargo projesi mevcut gtk4=0.10.3 / libadwaita=0.8.1 ile derlenir; uygulamanın
manifesti/kilidi değişmez. Probe başlığı PIV-05 synthetic API probe'dur; gerçek
Pivra GUI regresyonlarının yerine geçmez. İlk probe derlemesindeki realize trait
belirsizliği açık WidgetExt çağrısıyla düzeltildi; nihai probe tamamlandı.

## 4. Glycin uygunluk değerlendirmesi

**Karar:** sandbox'lı Linux görüntü yolu için ilk aday olarak korunur; **bu makinede
decode entegrasyonuna hazır değildir**. Üretime eklenmedi. Uyumluluk ve runtime
ön koşulları ayrı kapılardır.

### 4.1 Sürüm ve dağıtım kapısı

| İncelenen Rust istemcisi | Rust/GDK/GIO gereksinimi | Mevcut projeye etkisi |
|---|---|---|
| glycin 2.1.4 | Rust 1.80; gdk4 0.9.5, gio 0.20; loader ABI 1+ | Mevcut 0.10/0.21 Rust tip hattıyla doğrudan birleştirme önerilmez; aynı native sys links ve farklı wrapper sürümleri sorunu |
| glycin **3.0.8** | Rust 1.85; gdk4 0.10 + v4_16, glib/gio 0.21; loader ABI **2+** | Mevcut wrapper hattına aday; gdk4 feature ile native GTK tabanını en az **4.16** yapar. Mevcut 4.18 bunu karşılıyor; eski 4.6 ürün tabanını karşılamaz |
| glycin 3.1.0 | Rust 1.92; gdk4 0.11, glib/gio 0.22 | Küçük bir minor güncelleme gibi alınmamalı; mevcut GTK0.10/adw0.8 hattıyla birlikte upgrade kararı gerektirir |
| glycin 4.0.3 | Rust 1.93; Linux external/builtin ayrımı; yeni binding hattı | Bu görevde seçilmedi/derlenmedi; 3.0.x format/proses davranışları buraya genellenmedi |

**ÇALIŞTIRILDI, yalnız çözümleme:** geçici manifestte gtk4=0.10.3,
adw=0.8.1, glycin=3.0.8(gdk4), gstreamer=0.24.5 ve rust-version=1.85
birlikte çözüldü. gdk4 0.10.3, glib/gio 0.21.5, glycin-common 1.0.3 ve
glycin-utils 4.0.4 seçildi. Bu **compile/link/loader testi değildir**. Gelecekte
aynı doğrudan sürümler yeni geçişli sürümler çözebilir; gerçek entegrasyonun lockfile'ı
ve MSRV testi ayrıca incelenmelidir. `^3.0` sonradan 3.1'e geçebileceğinden onaysız
minor yükseltme yapılmamalı; güncel güvenlik düzeltmeleri ve binding uyumu birlikte izlenmeli.

```sh
bash tests/probes/resolve_media.sh
```

Resmi GNOME tarball sürümü ile crates.io sürümü aynı sayı değildir: upstream
2.0.0 NEWS, Rust glycin 3.0.0 / glycin-utils 4.0.0 ve loader'lar/libglycin 2.0.0
eşleşmesini açıkça kaydeder. Mevcut Debian 13 APT metaverisi **glycin-loaders
1.2.1+ds-2 / libglycin-1-0** sunuyor; bu eski **1+** hattıdır. Sırf paket adına
bakarak 3.0.8 istemcisiyle uyum varsayılmamalı; 2+ loader/config'leri gerektiği
seçilecek dağıtımda doğrulanmalı. Loader config'leri XDG data yollarındaki
`glycin-loaders/<compat>+/conf.d` altında MIME -> executable eşlemesi sağlar.
Uyumsuz config/binary'ye yol veya isim hilesi uygulanmamalı.

### 4.2 Format ve backend matrisi

Aşağıdaki satırlar **3.0.8 kaynak/loader ailesi için BELGELİ yeteneklerdir**;
bu ortamda Glycin ile açılan dosya sayısı **0**. Loader build seçenekleri,
kurulu backend'ler ve gerçek fixture varyantları doğrulanmadan ürün desteği sayılmaz.

| Format | Gerekli loader/backend | Belgeli kapsam ve sürüm/ürün sınırı |
|---|---|---|
| JPEG | glycin-image-rs; Rust image decoder hattı | 2+ uyumlu loader; 3.0.8 tablosunda ICC/EXIF/XMP var. EXIF yön/corrupt/CMYK varyantları NOT RUN |
| PNG / APNG | glycin-image-rs; png/image backend | ICC/CICP/EXIF/XMP ve animasyon belgeli; gerçek 16-bit/alpha/APNG zamanlama NOT RUN |
| WebP | glycin-image-rs; image-webp/image backend | ICC/EXIF ve animasyon belgeli; XMP tabloda uygulanmamış. Kayıpsız/kayıplı ve disposal/timing NOT RUN |
| GIF | glycin-image-rs; gif/image backend | Animasyon belgeli; ICC uygulanmış kabul edilmez; palet/disposal/loop testleri gerekli |
| HEIC / AVIF | glycin-heif + libheif ve ilgili HEVC/AV1 decoder plugin'i | 3.0.8 ICC/CICP/EXIF belgeli, animasyon **uygulanmamış**. Mevcut libheif/plugin runtime varlığı Glycin loader yerine geçmez; HDR/10-bit/codec varyantları NOT RUN |
| JPEG XL | glycin-jxl + libjxl (incelenen 2+ loader hattı) | 3.0.8 tablosunda ICC/EXIF var, CICP/animasyon uygulanmamış. Yeni main dalındaki farklı JXL backend'i bu sürüme aktarılmadı; loader'ın lisans/build koşulları incelenmeli |
| TIFF | glycin-image-rs + tiff/image backend | ICC/EXIF belgeli; çok sayfa, BigTIFF, CMYK, yüksek bit derinliği ve bounded-memory decode garantisi yok |
| SVG / SVGZ | glycin-svg + librsvg, GIO/Cairo/font altyapısı | Upstream Meson 2.0.0 ayrı glycin-svg loader'ını içerir. 3.0.8 README'deki SVG decoder sütunu image-rs yazsa da kütüphane sütunu librsvg'dir; loader ailesi/build seçenekleri birlikte kontrol edilmeli. Harici kaynaklar/fontlar/pixel boyutları NOT RUN |
| RAW (DNG, CR2 vb.) | glycin-raw + **libopenraw** | Camera RAW desteği değişken; upstream 2.0.0 Meson varsayılan loader listesinde raw **yok**, ayrıca seçilmeli. NEF/ARW/CR3 ve tam RAW geliştirme desteği varsayılmaz. Makinedeki **LibRaw** libopenraw/glycin-raw değildir; embedded preview/tam decode ayrı test edilir |
| PSD | İncelenen loader format tablosunda yok | Destek iddiası yok; flattened preview/katmanlar için ileride ayrı adapter veya alternatif backend araştırılmalı |

Upstream 2.0.0 default loader listesi image-rs/heif/jxl/svg'dir; raw/jpeg2000
isteğe bağlıdır. Dağıtım build seçimleri bunu daraltabilir. Her format için MIME
config + executable + backend codec + örnek dosya başarısı birlikte kaydedilmelidir.

### 4.3 API, yön, renk, büyük görüntü ve iptal

- **KAYNAK:** async Loader.load, Image.next_frame/specific_frame, Frame.texture
  (gdk4 feature), Frame.delay ve metadata API'leri var. GTK main thread'e yalnız
  UI güncellemesi dönmeli; bloklayan decode UI callback'ine taşınmamalı.
- `apply_transformations` varsayılan true; orientation/EXIF dönüşümlerini uygulayan
  yol var. Sonradan ikinci kez döndürmek yanlış olur; decoder'ın transformation
  bilgisi ve EXIF 1/6/8 fixture'ları gerçek entegrasyonda karşılaştırılmalı.
- ICC/CICP metadata, color state ve LCMS dönüşüm kaynakları mevcut. Bu, ekranın
  ICC profiline birebir uyum veya uçtan uca HDR başarısı değildir. 3.0.8'in GTK
  texture yolu ColorState API'si nedeniyle v4_16 ister; monitor/compositor/renderer
  doğrulaması ayrıca gereklidir.
- `FrameRequest.scale/clip` ve `gio::Cancellable` var. Kaynak dokümanı loader'ın
  frame talebinin kısımlarını yok sayabileceğini açıkça söylüyor; **tile/demand decode
  ve düşük peak RAM garanti edilmez**. Küçük sunum boyutu tam çözümlemeyi önlemeyebilir.
- Animated next_frame akışı ve frame delay API'si var; scheduler, pause/disposal,
  loop politikası ve hızlı dosya geçişinde iptal ürün tarafında ayrıca tasarlanmalı.
- README'nin genel proses açıklamasının yanında 3.0.8 kaynakta Pool::global ve
  loader pool kullanımı görülüyor. Worker ömrü/startup maliyeti/peak RAM ölçülmedi;
  tam olarak her dosyada yeni proses açılacağı veya her şeyin sıfır kopya olacağı varsayılmadı.

### 4.4 Sandbox ve somut ortam engeli

**KAYNAK:** native Linux yolu bubblewrap, namespace'ler, seccomp allowlist,
read-only mount'lar, peer-to-peer D-Bus/Unix socket, sealed memfd/mmap ve bellek
sınırları kullanır. Native Linux çalıştırma **Flatpak gerektirmez**. Fontconfig,
libseccomp, GLib ve renk yolu için LCMS gibi runtime/SDK'lar ile loader executable'ları
ayrı dağıtım bağımlılıklarıdır. SVG base-directory exposure varsayılan olarak
Loader API'sinde kapalıdır; gelecekte açılacaksa açık ürün/sandbox kararı gerekir.

**ÇALIŞTIRILDI:** bubblewrap 0.12.0 mevcut; yalnız `/usr/bin/true` çalıştıran
izole namespace probe'u şu hata ile başarısız:

```text
bwrap: setting up uid map: Read-only file system
```

Komut: `bwrap --unshare-all --die-with-parent --new-session --ro-bind /usr /usr
--symlink usr/lib /lib --symlink usr/lib64 /lib64 -- /usr/bin/true`.
Bu **BLOCKED sandbox probe** sonucudur; gerçek Glycin decode denenmedi.
Ek teşhis: `/usr/bin/bwrap`, Debian `bubblewrap 0.12.0-1~deb13u1` paketinden;
`ldd` gerekli libselinux/libcap/libc/libpcre kütüphanelerini çözüyor. Süreç
UID/GID 1000, `NoNewPrivs: 1`, `Seccomp: 2`, bir filtre; mountinfo'da `/proc`
`ro,nosuid,nodev,noexec` bağlı. Yüksek user namespace limiti tek başına yeterli değil.
UID map yazımı salt okunur `/proc` nedeniyle bu **Codex çalışma ortamında** engelleniyor;
bu sonuç Debian veya Glycin'in native Linux'ta genel olarak uyumsuz olduğunu göstermez.
Seccomp/NoNewPrivs gözlemleri ayrıca kaydedildi; bunların bu hatanın doğrudan nedeni
olduğu iddia edilmez. Codex dışındaki eşleşen native dağıtımda sandbox testi **NOT RUN**.

Bundan bağımsız, gerçek **paket/ABI riski** var: Debian 13'ün aday loader 1.2.1'i
1+ protokol hattında; araştırılan Glycin 3.0.8 istemcisi 2+ loader gerektiriyor.
İstemciyi eklemek eksik loader/SDK'ları veya bu paket sürüm farkını çözmez.
Sandbox kapatılmadı; `/proc` yeniden bağlanmadı, seccomp değiştirilmedi ve
unsandboxed fallback kullanılmadı. Güvenli entegrasyon önce eşleşen loader/SDK
ve namespace izinleri olan ortamda, ardından gerçek decode testleriyle doğrulanmalı.

### 4.5 Alternatifler (seçilmedi)

| Aday | Teknik yarar | Eksik/bedel |
|---|---|---|
| Rust image + özel texture dönüşümü | Yaygın temel formatlar, açık buffer yolu | Sandbox/ICC/RAW/HEIF/PSD kapsamı kendiliğinden gelmez; güvenlik sınırını uygulama üstlenir |
| GDK/GdkPixbuf yükleme yolu | Mevcut GTK ile basit sunum | Kurulu loader/GTK sürümüne bağlı; geniş format, iptal, renk ve sandbox hedeflerinin eşdeğeri sayılmaz |
| libvips | Büyük görüntü/demand-driven işleme adayı, ileride Edit için uygun araştırma yönü | İlave C/plugin zinciri; tüm decoder'larda düşük peak RAM/PSD katman garantisi yok; Edit önceliğini öne çekmek için gerekçe değil |
| libheif/libjxl/librsvg/LibRaw özel adapter'ları | Belirli formatlardaki eksikleri hedefleyebilir | Daha çok bakım, metadata/renk/iptal/sandbox entegrasyonu; yalnız ihtiyaç/fixture kanıtıyla seçilmeli |

Sandbox probe'u başarısız diye sessizce unsandboxed fallback seçilmedi. Kabul
edilen ilk aday Glycin olarak kaldı; zorunlu alternatif seçimi ayrı entegrasyon kararıdır.

## 5. GStreamer uygunluk değerlendirmesi

**Karar:** video aşaması için ilk tercih olarak korunur. Bu görevde pipeline,
player, media load, seek veya ses çıkışı oluşturulmadı. Runtime ve source araştırması
entegrasyonun çalıştığı anlamına gelmez.

### 5.1 Binding ve GTK video yüzeyi

gstreamer **0.24.5** (Rust MSRV 1.83, glib 0.21) mevcut GTK hattıyla çözülüyor.
0.25.4 Rust 1.92/glib 0.22 hattıdır; otomatik olarak karıştırılmamalı.
gst-plugin-gtk4 **0.14.5** gtk4 0.10/gstreamer 0.24 kullanır; 0.15.2 yeni hattadır.
Bu crate'ler yalnız metadata/source için incelendi, üretime eklenmedi veya derlenmedi.

`gtk4paintablesink` VideoSink + GdkPaintable sunar; GTK ana thread'inde GtkPicture
ile aynı viewport'a bağlamak önerilen araştırma yönüdür. Pipeline bus/state işleri
GTK widget thread güvenliğini ihlal etmemelidir. Bu factory **mevcut registry'de yok**.
Debian 13'ün ayrı `gstreamer1.0-gtk4` paketi 0.13.5-1; native minimumları GTK>=4.13.5,
GStreamer>=1.20 (diğer split bağımlılıklarıyla). Paket metaverisi mevcut native
sürümleri karşılıyor; binary plugin'in yüklenmesi/Paintable property/GObject uyumu
**denenmedi**. Dynamic C/GObject plugin kullanımı ile eski Rust plugin crate'ini
uyumsuz wrapper sürümleriyle statik gömmek aynı şey değildir.

| Sunum yolu | Değerlendirme |
|---|---|
| gtk4paintablesink | Tercih edilen aynı GTK viewport adayı. 0.14.5 manifesti GTK v4_6 ister, default gtk_v4_10'u açar; daha düşük upstream genel minimum anlatımı yerine gerçekten seçilen build feature'ları esas alınmalı |
| GL texture yolu | Upstream sink README'sinde Linux için waylandegl / x11egl / x11glx build feature'ları gerekir; driver/display/GL sharing ayrıca denenir |
| DMABuf | Sink `dmabuf` feature'ı GTK>=4.14, GStreamer video>=1.24 gerektirir; native sürüm yeterli olsa bile format/modifier/driver anlaşması ve gerçek zero-copy NOT RUN |
| waylandsink | Registry'de var; Wayland yüzey/video overlay entegrasyonu özel protokol/thread/surface sahipliği ister. GTK pencere handle'ı X11 XID gibi ele alınamaz; sink'in ayrı pencere açması ürün hedefini karşılamaz |
| appsink + özel frame sunumu | Registry'de var; kontrol sağlar ancak kopya, clock, caps/stride, renk ve GPU readback maliyeti getirir. Hazır Paintable sink yerine gereksiz ilk tercih yapılmadı |
| GtkVideo/GtkMediaFile | Basit oynatma alternatifi; özel pipeline/sink/kare kontrolü için daha az uygun, GTK'nın ayrıca kurulabilen media backend'ine bağımlı olabilir; mevcut boş GTK penceresi bu backend'i doğrulamaz |

### 5.2 Runtime factory ve dağıtım paketleri

**ÇALIŞTIRILDI:** gst_init_check ve factory metadata sorguları. Aşağıdaki
"var" ifadesi factory kayıtlı demektir; pipeline negotiate/decode/device testi değildir.

| Grup / örnek | Bu ortam | Dağıtım notu |
|---|---|---|
| Core runtime | 1.26.2 | Core sürümü codec listesini belirlemez |
| Base | playback/decodebin3/playbin3/uridecodebin3, appsink, videoconvert, volume var | Base paket 1.26.2; split paketler nedeniyle her Base element var sayılmaz; textoverlay **yok** |
| Good | jpegdec/pngdec/vp9dec/autoaudiosink/pulsesink var | Paket 1.26.2; libvpx vb. runtime bağımlılıkları ve gerçek dosya varyantları ayrıca test edilmeli |
| Bad | openh264dec/libde265dec/webpdec/assrender/waylandsink var | Paket 1.26.2; isim kalite hükmü değildir, optional backend/build seçimleri önemlidir |
| Ugly | Paket 1.26.3 kurulu | Tüm Ugly codec'ler veya patent/dağıtım koşulları onaylandı denmez; gereksiz encoder paketi istenmedi |
| Libav | Paket kurulu değil; avdec_h264 ve avdec_h265 **yok** | Libav ayrı `gstreamer1.0-libav` paketi olabilir; mevcut diğer H.264/H.265 factory'leri bunun eşdeğer test kanıtı değildir |
| AV1 | av1dec (aom) var, dav1ddec yok | Sistemde libheif dav1d plugin'i olması GStreamer dav1d factory'si demek değildir |
| GTK/GL/X11 sinks | gtk4paintablesink, glimagesink, ximagesink yok | Ayrı gtk4/gl/x paketleri ve build özellikleri incelenmeli; GTK X11/Wayland smoke bunları doğrulamaz |
| Donanım örnekleri | vah264dec/v4l2h264dec/nvh264dec yok; /dev/dri yok | Bu makinede donanım decoder/renderer testi NOT RUN; başka GPU/driver/oturum için yeniden envanter gerekir |

Plugin/element minimumları, kurulu Core sürümüne uymalıdır. Runtime'da kullanılacak
elementlerin factory/caps/negotiation durumu denetlenmeli; Core/Base/Good/Bad/Ugly/Libav
grup adlarından evrensel codec desteği çıkarılmamalıdır. Native paket adları dağıtıma
göre değişir; burada yalnız Debian 13 metaverisi denetlendi. Rust SDK için
libgstreamer1.0-dev ve ilgili plugins-base development dosyaları gibi split SDK'lar
ileride gerekecek; şimdi kurulmadı.

### 5.3 Diğer video yetenekleri: belgeli, oynatılarak doğrulanmadı

| Başlık | Destek yönü / doğrulanması gereken davranış |
|---|---|
| Ses / seviye | Playbin3 audio-sink, volume/mute/StreamVolume API'leri ve autoaudiosink/pulsesink yönü var; çalışan ses sunucusu/çıkış cihazı, perceptual volume ve ses kesilmesi NOT RUN |
| Çoklu ses / altyazı | Playbin3 stream collection + GST_EVENT_SELECT_STREAMS; dış altyazı suburi yolu; subparse/assrender registry'de. ASS font/style, gecikme, çoklu stream değişimi ve textoverlay eksikliğinin etkisi gerçek fixture'larla denenmeli |
| Kare adımı | gst_event_new_step, GST_FORMAT_BUFFERS ve STEP_DONE belgeli. İleri tek-kare, decoder/sink akışına bağlı; geriye kare adımı/keyframe/VFR için decode/preroll stratejisi ve PTS kanıtı gerekir; evrensel frame-accurate vaat yok |
| Hassas seek | Seek/query API ve FLUSH/ACCURATE/KEY_UNIT tercihleri belgeli; container index/GOP/codec/VFR performansı ve gerçek hedef PTS NOT RUN |
| Kaynak kare dışa aktarma | Playbin current-sample/convert-sample veya appsink/VideoFrame yolu aday; negotiated source caps/stride/color ve GPU map/readback doğru ele alınmalı; viewport screenshot'ı kaynak çözünürlük kanıtı değildir |
| Eksik codec/hata | Bus ERROR ve missing-plugin element mesajları/pbutils installer detail ayrıştırılmalı; eksik decoder, dosya I/O, bozuk stream ve negotiation/device hataları karıştırılmamalı; mevcut factory sorgusu bu hata yollarını test etmez |
| State/ömür | File değişiminde pipeline state, bus kaynakları, clock ve ses temizliği yönetilmeli; başka medyaya geçişte eski sonuç/frame/ses taşmamalı. Bu davranışlar implemente/test edilmedi |

## 6. Wayland first, X11 compatible

| Test kategorisi | Bu görevde tekrar çalıştırılan gerçek sonuç |
|---|---|
| X11/Xvfb | PASS: iki gerçek Pivra pencere, title/960x640, normal WM_DELETE_WINDOW, exit 0, boş stdout/stderr |
| Sway headless Wayland | PASS: DISPLAY yok, XWayland kapalı, GDK_BACKEND=wayland; compositor'da iki mapped xdg_shell yüzey/PID/app ID/title/960x640; normal close exit 0 ve bağımsızlık |
| Weston headless | Yüzey/protokol PASS: gerçek xdg_toplevel, SHM buffer, title/window geometry/configure/attach/commit. **Normal kapanış NOT RUN**; test cleanup SIGTERM kanıt sayılmaz |
| Fiziksel GNOME/Wayland | NOT RUN: Shell/Wayland desktop entegrasyonu, fractional HiDPI, touchpad, çoklu monitör, accessibility, GPU ve gerçek ses |

Wayland istemcileri başka uygulamaların pencerelerini X11'deki gibi global olarak
kontrol edemez; xdotool testi Wayland'a taşınamaz. Sway IPC ve Weston debug yalnız
test backend'ine özeldir, production pencere yönetimi API'si değildir. GTK/GIO
application/window ömrü korunur; file başına bağımsız pencere kararı değişmedi.

Video sink'inin iki backend'de aynı GTK viewport'a gerçekten gömülmesi **NOT RUN**;
boş pencere başarısı video başarısı değildir. Scale logical/pixel dönüşümü,
fractional scale yuvarlama/texture keskinliği, DMABuf modifier'ları ve GL/Vulkan/Cairo
farkları gerçek donanım/compositor matrisi ister. XWayland fallback'i backend testini
yanlışlıkla geçiremez; mevcut smoke'lar bunu açıkça engeller.

## 7. Minimum gereksinimler ve açık sınırlar

| Katman | Mevcut taban / gelecekteki koşul |
|---|---|
| Bugünkü shell | Manifest Rust>=1.85, GTK API feature>=4.6, libadwaita>=1.2; seçilen native dağıtım daha yüksek alt bağımlılıklar isteyebilir. Gerçek test edilen sürümler 2. bölümde |
| Glycin 3.0.8 gdk4 adayı | Rust>=1.85, aynı 0.10/0.21 binding hattı, native GTK>=4.16 ve glib feature>=2.68; 2+ loader'lar, bwrap, namespace/seccomp/memfd/socket desteği ve gerekli SDK/runtime'lar. Bunlar henüz yeni ürün minimumu ilan edilmedi |
| Gst/Paintable adayı | gstreamer0.24 hattı, uygun Core/plugin/development paketleri; seçilen sink default feature'ları GTK>=4.10, native Debian sink paketi >=4.13.5; GL ve DMABuf için ek koşullar |
| DMABuf optimizasyonu | GTK>=4.14 + Gst>=1.24 feature ve uygun GPU/driver/caps/modifier; bugünkü GTK/Gst sürüm yeterliliği başarı garantisi değil |
| Bellek / hız | Ölçülmüş minimum RAM, FPS veya gecikme eşiği yok; uydurulmadı. Bu makinenin 33 GiB RAM'i ürün minimumu değildir |

Bugünkü proje tabanı, aday kütüphanelerin gelecekte gerektireceği tabanla
karıştırılmamalı. Bu görev minimumları zorlamak için manifest/feature yükseltmedi.

## 8. Mimari, performans ve güvenlik riskleri

| Risk | Teknik karşılık / sonraki doğrulama kapısı |
|---|---|
| Çok büyük fotoğraf | 100 MP RGBA8 tek buffer yaklaşık 400 MB (381 MiB); decode ara verisi, CPU/GPU kopyası ve komşular bunu katlar. Bu aritmetik tahmin, ölçüm değil. Header/pixel/byte limitleri ve ölçekli/tile taleplerin gerçekten peak RAM'i azaltması ölçülmeli |
| Hızlı dosya geçişi / iptal | Cancellable tek başına stale result yarışını çözmez; generation/selection kimliğiyle geç dönen sonuç reddi, bounded iş kuyruğu ve process cleanup gerekir; PIV-13–18 kapsamına önceden kod eklenmedi |
| Bellek / cache | Uygulama genelinde byte/concurrency bütçesi; sadece worker'ın rlimit'i veya sistemde boş RAM olması yeterli değil. Yalnız RAM thumbnail kararı korunur; kalıcı medya cache'i eklenmedi |
| CPU/GPU render | Cairo/GL/Vulkan renk, premultiplication, yüksek bit derinliği, sync ve readback maliyetleri farklı; headless software sonuç GPU benchmark'ı sayılmaz |
| Animasyon | Tam kare koleksiyonu yüklemek yerine ölçülmüş frame talebi/scheduler; delay/disposal/loop/pause ve dosya değişiminde iptal testleri gerekli |
| Hardware video | Factory/caps ve gerçek cihaz erişimi, decoder seçimi, fallback, DMABuf transferi/modifier ve güç tüketimi; hiçbiri test edilmedi |
| Codec eksikliği | Dosya bazlı anlaşılır hata ve korunmuş gezinme; dependency grup adıyla destek sözü verilmez; iki H.264 decoder'ın profilleri eşdeğer varsayılmaz |
| Loader çökmesi / eksikliği | Missing loader/config/sandbox, malformed image ve process crash ayrı hatalar olmalı; bounded retry ve kaynak serbestliği test edilmeli; sandbox devre dışı bırakılarak başarı üretilmez |
| Wayland / X11 | Same-window video yüzeyi, focus/input/scale ve bağımsız pencere semantiği backend bazında denenmeli; compositor-specific debug/IPC production'a sızmamalı |
| Dağıtım paketleri | Glycin client/loader ABI 1+/2+, Rust binding hatları, GTK/Gst feature minimumları ve codec split paketleri manifest/version matrisiyle doğrulanmalı; native paketleme hâlâ sonraki faz |
| Renk / metadata | Orientation tek kez uygulanmalı; ICC/CICP metadata varlığı ekran ICC/HDR doğruluğu değildir; fixture referansları ve SDR fallback ayrıca gerekli |
| Güvenlik / lisans sınırları | Sadece gerekli loader/plugin, içerik boyut limitleri ve native sandbox. Upstream crate/codec/loader lisansları farklıdır (Glycin MPL/LGPL, bazı loader bağımlılıkları GPL/LGPL, sink MPL vb.); proje lisansı seçilmedi, yeniden dağıtım kararı verilmedi |

Performans kapıları ileride aynı örnekler üzerinde cold/warm load, peak RSS,
ilk kullanılabilir frame, hızlı geçiş iptali ve uzun oturum kaynak ölçümleriyle
tanımlanmalı. Kaynak dokümanı, geçmiş tecrübe veya anlık boş RAM ölçüm yerine geçmez.

## 9. Kabul edilen teknik kararlar ve henüz doğrulanmayanlar

1. Rust + GTK4/libadwaita, native Linux, **Wayland first / X11 compatible** korunur.
2. UI/app/CLI ile GUI bağımsız model sınırları korunur; bu görev production modül eklemez.
3. Görsel decoder için Glycin ilk aday; 3.0.8 aynı wrapper hattında metadata düzeyinde
   uygun. 2+ loader/sandbox/SDK ve gerçek decode kapıları çözülmeden hazır sayılmaz.
4. Video için GStreamer ilk tercih; gtk4paintablesink aynı viewport adayıdır.
   Registry'de yokluğu, SDK eksikleri ve gerçek pipeline testinin yapılmadığı açık kalır.
5. Güvenlik doğrulaması kapatılmaz; gereksiz decoder/player, production Cargo
   bağımlılığı, CI, Flatpak/AppImage veya lisans dosyası eklenmez.
6. Headless iki backend sonucu fiziksel GNOME/Wayland yerine geçmez. Weston normal
   kapanış alanı sonraki gerçek test işi olarak **NOT RUN** kalır.

**Açık deneyler:** bütün gerçek format decode/metadata/animation/RAW/PSD örnekleri;
media crate compile/link; loader/config ABI ve sandbox; video sink yükleme/embedding;
ses, subtitle/stream switch, accurate seek/frame-step, source frame export; GPU,
fractional HiDPI ve gerçek giriş cihazları. Belge bu alanları PASS yapmaz.

## 10. Sonraki görevler için önerilen sıra (uygulama yapılmadı)

1. **PIV-06:** kullanıcı ayrıca yetkilendirdiğinde explicit dosya/URI/GIO açılış
   sözleşmesini ve bağımsız pencere semantiğini ele al; decoder gerektirmeyen sınırları koru.
2. **PIV-07/08:** koleksiyon/filtre/doğal sıra modelini görevlerine göre uygula;
   fixture düzenini koru, uzantıyı decode başarısı sayma.
3. **PIV-09 öncesi ortam kapısı:** desteklenen native dağıtımda 2+ loader/SDK ve
   çalışan sandbox; Glycin/GTK wrapper sürüm kilidi ve gerçek MSRV derlemesi.
   Sonra yalnız JPEG/PNG, orientation ve kontrollü bozuk dosya testleriyle görüntü yolunu doğrula.
4. **PIV-10–18:** plan sırasıyla gezinme, watcher, bounded iptal/cache ve büyük
   görüntü/zoom; çözümlenmemiş peak RAM/scale varsayımlarını ölçmeden genişletme.
5. **Video fazı:** Gst SDK + gtk4paintablesink availability/paintable embedding;
   Wayland ve X11, software baseline, sonra codec/audio/subtitle/seek/frame-step/export,
   en son ölçülmüş hardware hızlandırma. Bunların hiçbiri PIV-05'te başlatılmadı.

### 10.1 Açık ön koşullar ve kapsam sınırları

| Görev / kapı | Açık ön koşul ve mevcut durum |
|---|---|
| PIV-06 dosya/URI sözleşmesi | Decoder gerektirmeyen argüman/GIO/bağımsız pencere işi ile gerçek görüntü açılışı ayrılmalı. Gerçek fotoğraf açıldı kabulü için aşağıdaki görüntü kapısı gerekir; PIV-06 burada başlatılmadı |
| PIV-09 görüntü entegrasyonu | Glycin 3.0.8'e eşleşen 2+ loader/config, native SDK, sandbox'ı destekleyen oturum ve JPEG/PNG/orientation/bozuk dosya deneyleri. Şu an loader/SDK eksik; Codex sandbox probe BLOCKED; gerçek decode NOT RUN |
| PIV-19 GTK video entegrasyonu | Gst SDK ve gerçekten yüklenen gtk4paintablesink; aynı viewport'a bağlanma, Wayland/X11 gerçek pipeline, hata/cleanup testleri. Sink ve SDK eksik; embedding/playback NOT RUN |
| PIV-19 codec matrisi | Libav paketi ve avdec_h264/avdec_h265 yok. Libav kullanan fixture'lar için paket ve gerçek decode testi ön koşul; alternatif openh264/libde265 factory varlığı dosyanın oynadığı anlamına gelmez. Libav tüm codec'ler için evrensel zorunluluk ilan edilmedi |

### 10.2 PIV-05 ve Faz 0 kapanışını ayrı değerlendirme

**PIV-05: DONE:** sürümlü araştırma, küçük izole probe'lar, açık risk/NOT RUN
kayıtları ve mevcut regresyonların doğrulanması bu belgeleme görevinin koşullarını
karşılar. Bu karar gelecekteki decoder veya player entegrasyonuna PASS vermez.

**Faz 0: INCOMPLETE.** Yol haritasının literal çıkış kapısı "Temiz kurulumda
cargo build, cargo test, uygulama penceresi ve açık test raporu". Mevcut native
ortam korunarak boş, ayrı Cargo target dizininde locked/offline build ve 16 test
başarıyla tamamlandı; aynı temiz derlemenin ikilisiyle X11/Sway/Weston smoke'ları
tekrar geçti (Weston normal kapanış hariç). Bu, **temiz Cargo derlemesi** kanıtıdır;
yeni kurulmuş makine/ortam veya snapshot'tan yeniden kurulum testi değildir.
Temiz native kurulumun bağımsız yeniden üretimi **NOT RUN** olduğundan faz kapısı
kapanmış sayılmadı. PIV-01–05 görev sayacı 5/5 olabilirken bu ayrı kapı açık kalır.
Fiziksel GNOME/Wayland ve Weston normal kapanış da NOT RUN olarak korunur;
bunlar yapılan headless testlerle tamamlandı sayılmaz. Ortam sıfırlanmadı;
PIV-06 veya sonraki geliştirme başlatılmadı.

## 11. Kaynaklar ve yeniden üretim

Upstream kaynaklar **21.10.2026** tarihinde incelendi. Crate kaynakları Cargo info
ile indirilen doğrulanmış yayın paketlerinden okundu; aşağıdaki source bağlantıları
aynı sürümü gösterir. Linkin verilmesi o web sitesinin bu ortamdan açıldığı iddiası değildir.

- [Glycin 3.0.8 README, format/ABI/sandbox](https://docs.rs/crate/glycin/3.0.8/source/README.md)
- [Glycin 3.0.8 manifest](https://docs.rs/crate/glycin/3.0.8/source/Cargo.toml)
- [Glycin 3.0.8 Loader/Frame/Request API](https://docs.rs/crate/glycin/3.0.8/source/src/api_loader.rs)
- [Glycin 3.0.8 ICC](https://docs.rs/crate/glycin/3.0.8/source/src/icc.rs) ve [pool](https://docs.rs/crate/glycin/3.0.8/source/src/pool.rs)
- [GNOME Glycin 2.0.0 NEWS: tarball/crate eşleşmeleri](https://raw.githubusercontent.com/GNOME/glycin/2.0.0/NEWS)
- [GNOME Glycin 2.0.0 Meson loader seçimleri](https://raw.githubusercontent.com/GNOME/glycin/2.0.0/meson_options.txt)
- [Glycin 3.1.0 manifest](https://docs.rs/crate/glycin/3.1.0/source/Cargo.toml) ve [4.0.3 manifest](https://docs.rs/crate/glycin/4.0.3/source/Cargo.toml)
- [GTK 4.18.6 GtkPicture kaynak/doküman](https://raw.githubusercontent.com/GNOME/gtk/4.18.6/gtk/gtkpicture.c)
- [GTK 4.18.6 GdkSurface scale kaynak/doküman](https://raw.githubusercontent.com/GNOME/gtk/4.18.6/gdk/gdksurface.c)
- [gst-plugin-gtk4 0.14.5 README](https://docs.rs/crate/gst-plugin-gtk4/0.14.5/source/README.md) ve [manifest](https://docs.rs/crate/gst-plugin-gtk4/0.14.5/source/Cargo.toml)
- [GStreamer Rust 0.24.5 manifest](https://docs.rs/crate/gstreamer/0.24.5/source/Cargo.toml)
- [GStreamer 1.26.2 step event kaynak/doküman](https://raw.githubusercontent.com/GStreamer/gstreamer/1.26.2/subprojects/gstreamer/gst/gstevent.c)
- [GStreamer 1.26.2 Playbin3 kaynak/doküman](https://raw.githubusercontent.com/GStreamer/gstreamer/1.26.2/subprojects/gst-plugins-base/gst/playback/gstplaybin3.c)
- Resmi Debian 13 APT show/query: glycin-loaders 1.2.1+ds-2, libglycin-1-0,
  gstreamer1.0-gtk4 0.13.5-1, mevcut Core/Base/Good/Bad/Ugly paketleri. Bunlar kurulu
  durum ile aday paket durumunun ayrıldığı metaveri gözlemleridir; yeni kurulum yapılmadı.

Regresyon komutları README/PIV-04 ile aynıdır: `bash tests/check.sh`, Xvfb
`tests/gui_smoke.py`, Sway `tests/wayland_smoke.py`, Weston `tests/weston_smoke.py`.
Probe kaynakları `tests/probes/` altında; ortam snapshot'ı `docs/probes/` altında.
Gerçek komut sonuçları, temiz target doğrulaması ve kapanış SHA'ları commit takip
tablosunun PIV-05 bölümünde kaydedilir.
