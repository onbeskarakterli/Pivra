# PIV-13 — Fit, Fill ve %100

24.10.2026 · `dev` · **DONE**, gerçek Arch/GNOME kabulü tamamlandı.
Başlangıç: `f9d7157cb4da22bc763b7fc2bf4f0607923f4df8`.
PIV-01–12 DONE ve kullanıcı tarafından yapılan native kabul kayıtları korunur;
bunlar aşağıdaki yeni native kontrollerin Codex'te PASS olduğu anlamına gelmez.

## Davranış ve mimari

Dosyalı pencerenin başlığındaki küçük mod menüsü **Fit / Fill / 100%** seçeneklerini
sunar; düğme güncel modu yazar, menü seçimi stateful window action ile işaretlenir.
Klavye: **F6 Fit**, **F7 Fill**, **F8 %100**. Capture-phase controller mevcut
Sol/Sağ gezinmesini korur; Ctrl/Alt/Super filtrelemesi değişmedi. Yeni büyük
toolbar, serbest zoom, pan, animasyon, minimap veya video modu eklenmedi.

Her ImageView kendi `win.image-mode` action'ı ve decoded texture referansını
sahiplenir. Yeni yükleme/medya, Yenile dahil, Fit'e döner. Yükleme devam ederken
mod seçilebilir; başarılı güncel callback o anda seçili modu kullanır. Mod veya
viewport değişimi loader çağırmaz. Cancel/token/weak-reference guard'ları korunur;
eski işin callback'i yeni texture/modu değiştiremez. Video/unsupported/boş
sunumunda mod action'ı devre dışıdır. Her pencerenin modu bağımsızdır.

`src/viewing.rs` GUI bağımsız ölçek/merkez hesaplarını ve yalnız bu özelliğin
kolay değiştirilebilir kullanıcı metinlerini taşır; PIV-39 tr/en i18n işi
başlatılmadı. EXIF tekrar uygulanmaz: geometri Glycin'in dönüşümden geçmiş
**texture** boyutlarını kullanır.

| Mod | Ölçek / sunum |
|---|---|
| Fit | `min(viewport_w/image_w, viewport_h/image_h, 1)`, GTK Picture ScaleDown; küçük görüntü büyütülmez |
| Fill | `max(viewport_w/image_w, viewport_h/image_h)`, GTK Picture Cover; merkezli, viewport sınırında kırpılır |
| %100 | Ölçek 1, kaynak boyutunda merkezli çizim; büyük resmin taşan kısmı kırpılır, küçük resim büyütülmez |

Fit/Fill doğrudan aynı GdkTexture'ı kullanır. %100 için küçük bir GdkPaintable
adapter'i gerekir: GTK Picture'ın minimum pencere boyutunu kaynak görüntü kadar
büyütmeden, texture'ı kaynak boyutunda çizer ve clip uygular. Adapter'e verilen
viewport GTK tarafından boyutlandırılır; iç texture esnetilmez. %100'den Fit'e
dönüldüğünde özgün texture nesnesi kullanılır. Kaynak medya yazılmaz, tekrar decode,
ön yükleme veya yeni cache politikası yoktur.

### %100 ve HiDPI tanımı

Bu aşamada **1 kaynak piksel = 1 GTK mantıksal birimi**. Ölçek faktörü 1'de bu
normalde 1 ekran pikselidir; ölçek 2'de bir kaynak piksel nominal olarak 2×2
fiziksel piksel alanı kaplar. Compositor/fractional scaling son eşlemeyi ayrıca
etkileyebilir. %100, HiDPI'da kesin fiziksel ekran-pikseli 1:1 iddiası değildir.
Geometri scale factor'a bölünmez; tüm üç mod aynı mantıksal viewport uzayındadır.
Gerçek HiDPI monitör doğrulaması NOT RUN; aşağıdaki GDK_SCALE=2 testi sentetik
GTK render kapsamıdır.

## Gerçekleştirilen testler

| Kontrol | Sonuç / sınır |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo check --locked`, `cargo build --locked` | PASS, mevcut helper stable/offline/-j2 |
| `cargo test --locked` | **59 PASS**: 35 unit + 24 integration, 0 failed/ignored |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| Tüm örneklerin locked/offline derlenmesi | PASS |
| Python aktif decoder gözlemci unit testleri | **30 PASS**, decoder sonucu değildir |
| Fixture bütünlüğü | Eski fixture manifestleri ve 18 görüntü varlığı PASS |
| GTK/GIO/helper regresyonları | **21 scoped çağrı PASS**, dağılım aşağıda |
| Gerçek secure decoder kapıları | **8 NOT RUN / exit77** |
| Shell/Python syntax ve `git diff --check` | PASS |

Yeni iki geometri unit testi, yatay/dikey/küçük/büyük kaynakları yatay/dikey/
küçülen viewport ile sınar: en-boy oranı, merkez, Fit sınırları, Fill kaplama,
%100 kaynak boyutu, geçersiz boyut/mod reddi. Önceki testler korunur.

**21 GUI/helper çağrısı:** X11 12, headless native Sway 7, Weston 2.
Bunların **18'i önceki** blank/files/desktop/discovery, gerçek X11 tuş gezinmesi,
PIV-10 navigation, PIV-11 folder/Yenile ve PIV-12 medya hatası/worker yarış
regresyonlarıdır. Yeni **3** çağrı `view_modes_probe --synthetic`:
Xvfb (varsayılan ölçek), Xvfb `GDK_SCALE=2`, Sway. Her çağrıda 48, toplam **144
snapshot** gerçek GTK Picture, üretim `present`/actual paintable ve GSK renderer
üzerinden okunur. 1600×1200, 1200×1600, 32×64, 64×32 kaynak; üç pencere boyutu;
Fit→Fill→Actual→Fit sınanır. Test gerçek atanmış widget boyutunu kullanır,
istenen pencere boyutunun birebir uygulanmasını varsaymaz.

Sentetik görüntü iki eksenli renk degrade desenidir. Alpha sınırları,
merkezleme, viewport dışına taşmayan render node ve geometriye göre kaynak
renk örnekleri doğrulanır. Böylece yalnız dolu bir dikdörtgen çizilmesi oran veya
merkez PASS'ı olarak kabul edilmez. **Bu texture'lar test üretimidir: JPEG/PNG,
EXIF veya güvenli decoder PASS'ı değildir.** Üretim sunum kodu değiştirilmeden
kullanılır, test texture'ları decoder yerine uygulamaya fallback yapılmaz.

Mevcut kontrollü ImageView yarışı: yükleme sırasında seçilen Fill'in geç callback
ile Fit'e dönmemesi, sonraki yüklemenin Fit'e dönmesi, mod değişimiyle kapanış.
Kontrollü üretim window/GIO yarışı: gerçek capture key signal'ları ile F6/F7/F8,
menü etiketi/action eşleşmesi, iki pencerenin farklı modu, yeni medyada yalnız o
pencerenin Fit'e dönmesi, kapanış ve aynı 7 worker başlangıç sayısı. Bunlar
**gerçek decoder etkinliği/piksel kanıtı değildir**.

Sway'de DISPLAY/XWayland kapalıdır; gerçek Wayland yüzeyi kullanılır. Test
sinyalleri fiziksel GNOME klavyesi veya fiziksel menü seçimi sayılmaz. Weston
normal close **NOT RUN**; SIGTERM cleanup bu kabulün yerine geçmez.

### Çalıştırılamayan pozitif yollar

Ayrı Xvfb girişleriyle image_probe PNG-first/JPEG-first, image_gui_probe, strict
navigation, strict folder watch, strict PIV-12 medya hatası, active_decode_close
ve yeni strict view_modes_probe **exit77** verdi. İlk güvenli baseline decode
başlamadı; Codex'in bilinen Bwrap/read-only `/proc` UID-map engeli sürer.
Yeni native mod/EXIF/iki gerçek texture penceresi pozitif yolu yalnız derlendi,
**PASS sayılmadı**. Bwrap zorunluluğu korunur; NotSandboxed veya güvenlik bypass
fallback'i eklenmedi. Önceki kullanıcı native başarıları kaynağı belirtilerek
korunur; yeni görev için genellenmez.

## Tek native komut ve kalan kabul

Güncel `dev` üzerinde gerçek Arch Linux / GNOME Wayland oturumunda:

```bash
bash tests/run_view_modes_probe.sh
```

Betik HEAD'i yazar, locked build yapar, `GDK_BACKEND=wayland` ile önce mevcut
image_probe JPEG/PNG/EXIF 1–8 **piksel** kontrolünü çalıştırır; ardından yeni
üretim pencere/mod/render probunu çalıştırır. Varsayılan güvenli yol zorunludur.
İlk baseline engeli exit77/NOT RUN; baseline geçtikten sonraki sağlam görüntü
ve render hataları FAIL'dir. Native kabulde `--synthetic` kullanmayın.

Yeni native pozitif yolun kapsamı (şu anda NOT RUN): büyük/küçük/dikey PNG,
EXIF 1–8 sonrası boyutlar, tüm modlarda gerçek snapshot, resize, aynı texture
kimliğinin korunması, kaynak byte bütünlüğü; iki gerçek görüntü penceresinde
Actual/Fill izolasyonu; yeni medya Fit reset; bozuk PNG'den sağlam görüntüye
dönüş ve bir pencere mod değiştirilerek kapanırken diğerinin Fill'de kalması.
Üretim load isteği sırasında mod seçilir; decoder'ın tam zamansal çalışma
aralığı/overlap bu testte sertifikalandırılmaz.

Yalnız fiziksel GNOME kabulü kalır: menü ve F6/F7/F8 ile mod geçişi/etiket;
Fit'te bütün görüntü, Fill'de oranı bozmayan merkezli kırpma, %100'de kaynak
boyutu; resize; başka fotoğrafta Fit reset; iki pencerenin bağımsızlığı. Gerçek
HiDPI fiziksel eşlemesi ayrıca NOT RUN olarak belirtilmelidir.

**PIV-13 TESTING; native sonuç gelmeden DONE değil.** PIV-14 başlamadı.
PIV-01–12 DONE, Faz 0 bağımsız temiz kurulum INCOMPLETE, DESKTOP-06 Nautilus
NOT RUN ve PIV-39 i18n planı korunur. Sistem/Rust/GTK kurulumu, Cargo.toml/lock,
loader, kaynak fixture'lar, CI ve lisans değiştirilmedi.

## Kaynak SHA'ları

| Commit | İçerik |
|---|---|
| `b576c35f30e0fb42b1e971c8162afe9fa7f37873` | Mode/geometri modeli, mantıksal %100 paintable, texture üzerinden mod sunumu, pencere menüsü ve F6/F7/F8 |
| `5ad7b98bf0f020c8fa7bb21c54165a2148b662f7` | Snapshot/native gate, mod/worker/klavye izolasyon regresyonları, tek native giriş ve Sway sentetik argümanı |


## PIV-13 native kabul ve kapanış — 24.10.2026 — DONE

Kullanıcı Arch Linux / GNOME Wayland üzerinde `d60c33078c08720524413d204f7131b34df2ff5b` ile `bash tests/run_view_modes_probe.sh` çalıştırdı: **exit0/PASS**; gerçek Glycin JPEG/PNG/EXIF 1–8 decode ve Fit/Fill/Actual render geometrisi, kırpma, merkez, resize, iki bağımsız pencere, yeni medyada Fit sıfırlama, bozuk medyadan toparlanma ve kaynak bütünlüğü doğrulandı. Kullanıcı normal GNOME uygulamasında modları manuel kontrol ettiğini bildirdi. %100 modunda büyük fotoğrafın sadece kesitinin görünmesini ve fareyle sürükleyerek gezinemediğini gözledi; ilki Actual davranışı, ikincisi henüz uygulanmamış **PIV-14 pan** kapsamıdır, PIV-13 hatası değildir. Gerçek HiDPI fiziksel 1:1 ölçek testi NOT RUN; Actual = bir kaynak piksel / GTK mantıksal birimi. Önceki Codex ortamı exit77 sonuçları geriye dönük PASS değildir. **PIV-13 DONE; PIV-14 TODO**. Faz 0 temiz native kurulum INCOMPLETE, DESKTOP-06 Nautilus NOT RUN, PIV-39 i18n planı açık.
