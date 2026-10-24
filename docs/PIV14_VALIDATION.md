# PIV-14 — imleç merkezli zoom ve pan

24.10.2026 · `dev` · **TESTING**, gerçek Arch/GNOME kabulü bekleniyor.
Başlangıç: `8b31944dacef310ef3df3b617a241c6f27d005ba`.
Güncel origin/dev fast-forward ile alındı. PIV-01–13 DONE kayıtları ve önceki
kullanıcı kaynaklı native sonuçlar korunur; yeni görev için genellenmez.

## Üretim davranışı

- Fotoğraf üzerinde tekerlek yukarı zoom in, aşağı zoom out. Her dikey scroll
  birimi `1.2^(-dy)` oranını kullanır; manuel ölçek **0.1–32 (%10–3200)** aralığında
  sınırlandırılır. Sıfır/NaN/sonsuz input güvenle etkisizdir; aşırı scroll delta
  üs hesaplamasından önce sınırlanır. Preset Fit/Fill başlangıç ölçeği bu aralığın
  dışında olabilir: ilk manuel giriş sınırı uygular fakat tekerlek yönünü tersine
  çevirecek bir sıçrama yapmaz. Pan tek başına preset ölçeğini değiştirmez.
- İmleç altındaki kaynak koordinatı yeni ölçekte aynı viewport koordinatına
  taşınır. Kaynak/viewport sınırları ankrajdan önce gelir: görüntü bir eksende
  sığıyorsa ortalanır, taşan eksende pan boşluk bırakmayacak aralıkta tutulur.
  Boş kenarlık üzerindeki imleç en yakın görüntü noktasına sınırlandırılır.
- Sol düğmeli GTK GestureDrag ile görünmeyen bölgelere erişilir. Başlangıca göre
  offset'ler artımlı deltalara çevrilir; her input sonrası sınır uygulanır.
  Release/cancel son geçerli konumu bırakır. GTK implicit grab dışarıdaki hareketi
  aynı gesture olarak ele alır; sınırsız delta güvenli clamp edilir. Unmapped
  fotoğraf üzerindeki geç input etkisizdir.
- Fit/Fill/%100 ve F6/F7/F8 korunur. Preset seçimi, yeni medya ve Yenile dönüşümü
  sıfırlar. Manuel durumda mevcut küçük header düğmesi **Zoom** yazar; ayrı
  toolbar veya i18n altyapısı eklenmedi. Sayısal yüzde göstergesi eklenmedi.
- Yeniden boyutta manuel ölçek korunur, viewport merkezindeki kaynak noktası
  mümkün olduğunca korunur; artık sığan eksen ortalanır. Preset Fit/Fill
  yeniden boyutta normal geometriyi hesaplamaya devam eder.

%100 tanımı PIV-13 ile aynıdır: **1 kaynak piksel = 1 GTK mantıksal birimi**.
HiDPI fiziksel ekran pikseli 1:1 iddiası yoktur. EXIF yeniden uygulanmaz;
Glycin'in zaten dönüştürdüğü texture boyutları kullanılır.

## En küçük mimari genişletme

`viewing.rs` içine pencereye özel Transform modeli eklendi; preset geometri ve
manuel dönüşüm ayrı tutulur. Modelin reset nesli, önceki yüklemede veya modda
başlamış sürüklemenin yeni fotoğrafa devam etmesini engeller.

Mevcut `actual_paintable.rs` kaynak texture'ı aynı viewport içinde manuel
geometriyle çizecek şekilde genişletildi. İlk manuel input adapter'i kurar;
sonraki input aynı paintable'ı invalidate eder. Texture byte kopyası, yeniden
decode, yeni cache/ön yükleme sistemi veya input başına yeni paintable yoktur.

Scroll/motion/drag controller'ları yalnız Picture'a bağlanır; başlık, menü ve
Yenile bildirimi fotoğraf sürüklemesi başlatmaz. İlk pointer girişindeki konum
`enter` ile, sonraki hareket `motion` ile kaydedilir. Yükleme sırasında texture
henüz yoksa zoom/pan güvenle yok sayılır. Eski worker callback token/iptal/weak
kontrolleri değişmedi. Her pencere ayrı texture referansı, Transform ve gesture
state'i taşır. Kapanmış/unmapped widget girişleri kullanılmaz.

## Test sonuçları — kanıt sınıfları ayrı

| Kontrol | Gerçek sonuç |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo check --locked`, `cargo build --locked` | PASS, mevcut stable/offline/-j2 helper |
| `cargo test --locked` | **63 PASS**, 39 unit + 24 integration, 0 failed/ignored |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| Tüm örneklerin locked/offline derlenmesi | PASS |
| Python aktif-decode gözlemci unit testleri | **30 PASS**, gerçek decoder testi değildir |
| Fixture/manifest bütünlüğü | Eski fixture'lar ve 18 image varlığı PASS |
| GUI/helper çağrıları | **24 scoped PASS**: X11 14, Sway 8, Weston 2 |
| Gerçek decoder kapıları | **9 NOT RUN / exit77** |
| Shell/Python syntax ve whitespace | PASS |

### 1. Model/unit

Dört yeni unit testi: zoom in/out kaynak noktası ankrajı; hızlı girişler ve iki
zoom limiti; dört pan köşesi; küçük görüntüde pan olmaması; resize sırasında
merkez kaynak noktasını koruma; reset nesli; geçersiz değerler ve preset ölçeği
manuel aralık dışındayken tekerlek yönünün tersine dönmemesi. Önceki geometri,
CLI, katalog, doğal sıra, input, desktop ve lifecycle testleri korunur.

### 2. Gerçek GTK controller/render, sentetik texture

Mevcut view_modes_probe ortak yardımcıları kullanıldı; `--synthetic --zoom-pan`
için küçük bir test modülü eklendi. Üretim install_interaction ve paintable
kodu, gerçek GTK motion-enter/scroll/drag/cancel sinyalleriyle çalışır.

Üç çağrı: Xvfb, Xvfb `GDK_SCALE=2`, native headless Sway. Her çağrıda yatay
1600×1200, dikey 1200×1600, küçük 32×64 kaynak ve üç preset kullanılır.
Ankraj koordinatı, zoom out, dört köşe, limitler, her preset için 1000 hızlı
sürükleme update'i ve iki yönde 100 scroll olayı, reset sırasında stale gesture,
release/cancel, resize, iki ayrı pencere ve aktif drag sırasında close sınanır.
İkinci pencerenin state/paintable'ı değişmez ve ilk kapanınca input çalışır.

**72 snapshot/çağrı, toplam 216** yeni zoom/pan snapshot'ı: üretim GSK render
node clip sınırı ve iki eksenli sentetik degrade kaynak renk örnekleri okunur.
Kaynak texture ve manuel paintable kimliğinin korunması ayrıca doğrulanır.
Önceki üç Fit/Fill/%100 sentetik çağrısının **144 snapshot**'ı da PASS;
birlikte **360 snapshot**. Bunlar JPEG/PNG/EXIF veya gerçek decoder PASS'ı
**değildir**. Sentetik texture yalnız testte oluşturulur; uygulamaya decoder
fallback'i konmadı. Fiziksel fare girişi enjekte edilmiş sayılmaz.

Önceki PIV-09–13 GUI çağrılarının 21'i korunur; yeni üç zoom çağrısıyla 24 olur.
Mevcut kontrollü window/worker yarışında texture yokken scroll/drag sinyallerinin
görünür frame veya ek worker üretmediği ve seçilmiş modun korunması da test edildi.
F6/F7/F8, Sol/Sağ, Yenile gerçek GTK button click/pencere kimliği, monitor, bozuk
medyadan devam ve close izolasyonu regresyonları PASS. Kontrollü worker testleri
Glycin etkinliği/iptali değildir.

Sway'de DISPLAY yok/XWayland kapalıdır. Controller sinyalleri fiziksel GNOME
mouse/klavye sayılmaz. GDK_SCALE=2 sentetik render testi gerçek HiDPI monitör
kabulü değildir. Weston mapped surface testleri PASS; normal close **NOT RUN**
kalır, SIGTERM cleanup onun yerine geçmez. Input gecikmesi/FPS benchmark'ı
ölçülmedi; akıcılık iddiası fiziksel native kabulde değerlendirilecektir.

### 3. Gerçek Glycin/Bwrap

Ayrı girişlerle image_probe PNG/JPEG-first, image_gui_probe, strict navigation,
strict folder watch, strict medya hatası, active_decode_close, strict view_modes
ve yeni `view_modes_probe --zoom-pan` denendi: **9 exit77 / NOT RUN**. İlk secure
baseline oluşmadı; bilinen Codex Bwrap/read-only proc UID-map kısıtı sürer.
Native zoom/EXIF/two-real-texture pozitif yollar derlendi fakat çalışmış gibi
raporlanmaz. Önceki kullanıcı PIV-09–13 kabulü yeni PIV-14 PASS'ı değildir.
Bwrap zorunluluğu, decoder ve güvenlik politikası değişmedi; NotSandboxed veya
bypass/fallback yoktur. Gerçek decoder zaman aralığı/overlap ölçülmedi.

## 4. Native Arch/GNOME kabulü — tek komut

Güncel `dev` üzerinde gerçek Arch/GNOME Wayland oturumunda:

```bash
bash tests/run_zoom_pan_probe.sh
```

HEAD yazdırılır, locked build yapılır, GDK_BACKEND=wayland kullanılır. Önce
mevcut image_probe gerçek JPEG/PNG/EXIF 1–8 piksel kontrolü; sonra üretim
view_modes_probe'nin `--zoom-pan` senaryosu çalışır. Yalnız özel temp kopyaları
kullanılır; kullanıcının fotoğrafları değiştirilmez. İlk baseline engeli exit77;
baseline sonrasındaki sağlam görüntü ve assertion hataları FAIL'dir. Native
kabulde `--synthetic` kullanmayın.

Yeni native pozitif yol henüz **NOT RUN**: EXIF sonrası kaynaklar üzerinde
üretim scroll/drag sinyalleri, Actual pan, manuel state, Fit reset, aynı decoded
texture kimliği, eski Fit/Fill/%100 render/resize kontrolleri, iki gerçek
pencere ve kaynak byte bütünlüğü. Gerçek native fotoğrafta mouse ankrajı/
kenarlar/akıcılık için şu kısa manuel kabul ayrıca gerekir:

1. Büyük fotoğrafı F8 ile %100 açıp dört kenar/köşesine sürükleyin.
2. Fotoğrafın farklı noktalarında tekerlek yukarı/aşağı kullanın; sınır ve küçük
   görüntü merkezlemesi dışında altındaki nokta yerini korumalı.
3. Hızlı tekerlek/sürükleme, resize ve F6/F7/F8 reset'i deneyin; yeni fotoğraf
   Fit açılmalı, diğer pencere etkilenmemeli.
4. Yenile/menü tıklaması sürükleme başlamamalı; drag sırasında bırakma,
   pencere dışına çıkış ve bir pencereyi kapatma güvenli kalmalı.

Kullanıcı test ettiği SHA ile sonucu bildirene kadar **PIV-14 TESTING**.
PIV-15 interpolasyon, touchpad/pinch özelleştirmesi, cache, minimap, video zoom,
edit, paketleme ve i18n uygulanmadı. PIV-01–13 DONE korunur; Faz 0 bağımsız temiz
kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN ve PIV-39 tr/en planı açık kalır.

## Gerçek kaynak commit'leri

| SHA | İçerik |
|---|---|
| `0bacddbd8cd4fd2e266ed63c1886e33f35468408` | Ankrajlı manuel zoom, bounded pan/resize, nesilli gesture reset; mevcut paintable/texture ve header indicator entegrasyonu |
| `e1d977124e939897b129e759293df866682a0de3` | Sentetik render/controller/izolasyon ve pending-input regresyonları; native zoom-pan girişi, Sway argümanı |
