# PIV-10 — önceki/sonraki medya, 22.10.2026

**Durum: TESTING.** Kod ve Codex'teki kısa kontroller tamamlandı; yeni native
GNOME/Wayland piksel ve fiziksel tuş/focus kanıtı henüz yok. PIV-09'un kullanıcı
kaynaklı DONE kaydı korunur: `8991b78` / `c11c7b7`. Bu sonuç yeni PIV-10 kodunun
native testinin yerine geçmez. Faz 0 temiz native kurulum ve DESKTOP-06 açık kalır.

## Davranış ve sorumluluklar

Fotoğrafla açılan her pencerenin kendi MediaCollection seçimi, action'ları ve tek
ImageView'ı vardır. Sol/sağ ok doğal sıralanmış **karışık** katalogda bir öğe hareket
eder; sınırda, tek öğede ve boş katalogda seçim değişmez. Açılan alias başlangıçta
seçili kalır. Katalog keşfi bitene kadar yalnız açılan öğe bilinir; o sırada sınır
hareketi etkisizdir. Keşif tamamlandıktan sonra seçim scanner'ın açılan dosya
indeksidir. Gezinme sıralamayı değiştirmez ve yeniden tarama yapmaz.

Window capture-phase key controller odağı alt widget'ta olsa da plain Left/Right'ı
window-local previous-media/next-media action'larına iletir. Ctrl/Alt/Super okları
gezinme olarak tüketilmez. Başlık seçilen ad ve mevcut aday/uyarı sayısıyla güncellenir.
Yeni toolbar veya şerit yoktur. Video adayları atlanmaz: paintable temizlenir ve
`Video playback is not available yet.` gösterilir. Diğer henüz desteklenmeyen
fotoğraf adaylarında JPEG/PNG kapsamı açıkça belirtilir; video oynatma eklenmedi.
Doğrudan CLI/desktop video kabulü veya varsayılan oynatıcı ilişkisi değiştirilmedi.

JPEG/PNG seçimi aynı üretim image_loader.rs / zorunlu Glycin Bwrap yolunu kullanır.
ImageView eski cancellable'ı iptal eder, eski paintable'ı hemen kaldırır, ScaleDown
Fit durumunu yeniler. LoadSlot yalnız güncel ve iptal edilmemiş token'ı kabul eder.
İptal öncesinde bitmiş ama GTK callback'i sırada olan eski sonuç da yeni seçimi
veya video/hata durumunu ezemez. Close/destroy handler'ları view kurulurken bir kez
bağlanır; her tuşta yeni pencere handler'ı birikmez. GDK texture ve widget güncellemesi
GTK context'inde, decode worker context'inde kalır. Eski build/build_with_completion
PIV-09 probe girişleri korunur. Zoom/pan aracı veya yeni kalıcı ayar eklenmez.

Katalog snapshot'tır: keşiften sonra silinen öğe listede kalıp kontrollü hata verir;
kullanıcı sonraki öğeye geçebilir. Bozuk dosya hatası da gezinmeyi kapatmaz.
Kaynak medya yalnız okunur; testte silinen dosya yalnız private temp kopyadır.
PIV-11 dosya izleme/yenileme ve sonraki görevler başlatılmadı.

## Codex'te gerçekten yürütülen kontroller

Kaynak/test commit'i: `80e501dbf2500f342863f309492770c814dcb986`.
Mevcut Rust/GTK/native/Sway/Weston ortamı ve image-activate.sh kullanıldı;
kurulum sıfırlanmadı. Cargo.toml/lock, decoder, CLI, input, desktop ve fixture
byte'ları başlangıç `c11c7b79bf0dce97516c3a1f8505c5f020e9193e` ile aynıdır.

| Kontrol | Sonuç ve gerçek kapsam |
|---|---|
| cargo fmt --check | PASS |
| cargo check --locked / build --locked | PASS; mevcut helper stable/offline/-j2 kullandı |
| cargo test --locked | **54 PASS = 30 unit + 24 integration**, 0 failed/ignored, 0 doctest |
| cargo clippy --locked --all-targets -- -D warnings | PASS |
| Python PIV-09 observer regresyonu | **30 PASS**, gerçek decoder testi değildir |
| Orijinal ve 18 image fixture manifesti / CLI / desktop / katalog / doğal sıra | PASS; core helper kapsamı |
| Yeni 3 katalog unit | Boş/tek, iki sınır, açılan seçim, 1/2/10 karışık fotoğraf-video, 1000 hızlı alternatif, iki katalog bağımsızlığı PASS |
| Yeni 2 LoadSlot unit | Eski tamamlanmış callback token reddi, replacement/video/close iptali, 1000 hızlı replacement ve iki slot bağımsızlığı PASS |
| X11 navigation_smoke.py | PASS: gerçek X11 tuş olayları, Tab sonrası oklar, Ctrl filtreleme, başlık sırası/sınırlar, iki bağımsız launch, normal WM close; piksel doğrulanmadı |
| navigation_probe Xvfb --expect-image-error | Scoped PASS: gerçek üretim pencere/katalog/controller/view, mapped GDK yüzey, seçim/video/hata/rapid/bağımsız close; fotoğraf pikselleri BLOCKED |
| navigation_probe Sway --expect-image-error | Aynı scoped PASS; native Wayland, DISPLAY yok/XWayland kapalı. GTK key signal'ları kullanır, compositor klavye enjeksiyonu değildir |
| navigation_race_probe Xvfb | PASS yalnız deterministik GTK/GIO hata-worker yarışı; aşağıdaki sınırı okuyun |
| Önceki X11 boş/file/desktop/discovery testleri | 4 scoped PASS, dosya modları explicit expected-image-error |
| Önceki Sway boş/file testleri | 2 scoped PASS, native yüzey ve normal close/exit0 |
| Önceki Weston boş/file testleri | 2 scoped PASS, mapped xdg_toplevel/SHM/configure/attach/commit; normal close NOT RUN |
| navigation_probe strict, Xvfb | **BLOCKED/exit77**: ilk gerçek JPEG secure loader başlamadı. Yeni gerçek piksel/Fit ve başarılı-frame stale yolu NOT RUN |
| Python AST / shell bash -n / git diff --check | PASS |

Toplam **12 başarılı scoped GUI/helper çağrısı**: 8 eski + 4 yeni; bunlar 12 gerçek
piksel testi olarak sayılamaz. Native strict gate bu sayıya dahil değildir.

`navigation_race_probe` üretim ImageView callback kodunu ve gerçek GIO blocking
worker/MainContext join'ini kullanır; test-only backend yalnız kapılı **Err** üretir.
İkinci worker birinciden önce bitirilir; sonra eski sonuç döndürülür. Güncel hata
mesajı korunmalı. Diğer vakalarda geç sonuç video durumunu ezmemeli; pending close
sonrası job join/weak widget bırakılması/iptal token'ı ve diğer pencere korunmalı.
Bu, gerçek Glycin decode veya sandbox cancellation başarısı değildir; fake frame,
mock piksel, NotSandboxed ya da uygulama fallback'i yoktur. Test worker gecikmesi
yalnız ayrı example binary'sindedir; üretim decoder'ı değiştirilmez/yavaşlatılmaz.

Broken private fixture'da Codex'in aldığı hata sandbox başlangıç hatasıdır;
gerçek bozuk JPEG/PNG decode recovery'si PASS sayılmadı. Silinmiş private dosya
ise gerçek metadata NotFound hatasıyla kontrol edildi. Strict native testte geçerli
görüntü baseline'ı geçmeden bozuk decode kontrolleri başarı sayılmaz.

## Tek komutlu native GNOME/Wayland kapısı

Güncel `dev` checkout'unda, zaten çalışan güvenli Glycin ortamında:

```sh
bash tests/run_navigation_probe.sh
```

Betik HEAD'i yazar, matching example'ı locked build eder ve GDK_BACKEND=wayland
ile çalıştırır. GTK critical hataları fatal'dır. **Native doğrulamada
--expect-image-error kullanılmaz.** İlk gerçek baseline decode başarısızsa exit77/BLOCKED'dır; baseline geçtikten
sonraki geçerli görüntü hatası FAIL olur, ortam skip'i sayılmaz.
Codex aktivasyon betiği, yeni sistem kurulumu veya güvenlik ayarı gerekmez.

Probe private temp kopyalarla iki gerçek üretim penceresi açar; seçilen ilk
10.jpg'yi, 1.png/2.mp4/10.jpg doğal sırasını, video mesajını, iki sınırı, tek öğeyi,
rapid sağ/solu, bozuk ve silinmiş görüntüden çıkışı kontrol eder. Başarılı frame'lerde
gerçek texture boyutları ve quadrant RGB/alpha byte'ları doğrulanır; actual
Picture WidgetPaintable/GSK snapshot'ında Fit/center alpha sınırları ölçülür.
Farklı boyutlu yeni seçimler önceki ölçek/konumu taşımamalı. Rapid sonrası son
selection/pixels tekrar doğrulanır; ikinci pencere ve normal kapanış kontrol edilir.
Bu snapshot, compositor ekran fotoğrafı veya fiziksel HiDPI testi değildir.
Tuş yolu gerçek controller key-pressed sinyaliyle otomatik çalışır; fiziksel
Wayland tuş teslimi için aşağıdaki kısa manuel kontrol ayrıca gereklidir.

Yalnız native kullanıcıya kalanlar:

1. Bu komutun gerçek secure görüntü/piksel/Fit, rapid sonrası son görüntü ve
   bozuk görüntüden sonraki başarılı decode sonucunu, tam SHA ve exit code ile kaydetmek.
2. Normal Pivra'yı kendi fotoğraf klasöründe açmak; fotoğraf alanına ve başlık
   düğmelerine tıklayıp/Tab ile odak değiştirerek fiziksel sol/sağ tuşlarını denemek.
   İlk/sonda durmalı; fotoğraf→video mesajı→fotoğraf geçmeli; ikinci pencere
   bağımsız kalmalı. Bu manuel GNOME/Wayland kontrolü Codex'te yapılmış değildir.

Her iki native kanıt gelmeden **PIV-10 TESTING** kalır. PIV-09 DONE kaydı,
Faz 0 INCOMPLETE ve DESKTOP-06 NOT RUN bağımsız durumları korunur.


## Sonraki geçerli durum

PIV-10, kullanıcı kaynaklı native probe ve fiziksel ok tuşu kanıtıyla DONE oldu;
[22.10.2026 kapanış kaydı](PIVRA_COMMIT_TRACKER.md#piv-10-arch-native-kabul-ve-kapanış--22102026--done)
önceki TESTING ifadelerini tarihsel olarak tamamlar. PIV-11 regresyonunda
`navigation_probe` silinen komşunun canlı listeden çıkmasını bekler (6 → 5 aday);
PIV-10 doğal sıra, sınırlar, gerçek tuşlar ve iptal/stale-result testleri korunur.
