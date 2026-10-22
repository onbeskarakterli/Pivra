# PIV-12 — medya hataları ve pencere izolasyonu

22.10.2026 · `dev` · **TESTING**, native kabul bekleniyor.
Başlangıç: `1ceaa04e202e94a320d2dd3f632f48aa1554a7bd` (güncel origin/dev'e fast-forward).
PIV-01–11 DONE kayıtları ve kullanıcı kaynaklı Arch/GNOME kabul kanıtları korunur.
Bu eski native sonuçlar yeni testlerin Codex'te çalıştırıldığı anlamına gelmez.

## Kapsam incelemesi ve küçük düzeltme

PIV-09 image_probe/image_gui_probe ve aktif-decode gözlemcisi gerçek piksel,
EXIF, geometri ve aktif secure-decode kapanışı için mevcut kapılardır. PIV-10
navigation_probe/gerçek X11 tuş testi ve LoadSlot unit testleri doğal sıra,
sınırlar, bozuk/silinmiş dosyadan devam ve eski token reddini kapsar. PIV-11
folder_watch_probe gerçek GIO olaylarıyla ekle/sil/rename/atomic save, symlink,
klasör kaybı ve bağımsız izleyicileri sınar. Bunlar yeniden yazılmadı.

İncelemede `reload-media` eyleminin video ve henüz desteklenmeyen görüntü
adaylarını da Glycin'e gönderdiği bulundu. Yalnız bu yol daraltıldı: video veya
JPEG/PNG dışı öğelerde mevcut açıklayıcı desteklenmiyor durumu korunur, decoder
başlatılmaz. Aynı senaryo WebP ve video için üretim pencere probuna eklendi.
Görüntü decoder'ı, token/cancellable/weak-reference mekanizması değiştirilmedi.
Yeni toolbar, zoom/pan, video motoru, i18n, bağımlılık veya CI yoktur.

## Eklenen eksik kontroller

### Üretim pencereleri ve gerçek dosya sistemi

Mevcut `folder_watch_probe` içine `--media-failures` senaryosu eklendi;
ortak GTK/GIO yardımcıları tekrar kopyalanmadı. Ayrıntılar
`examples/support/media_failure_cases.rs` içindedir. İki gerçek ApplicationWindow,
üretim window/image_view/input/image_loader ve GIO FileMonitor kullanılır.
Yalnız testin kendi geçici kopyaları değiştirilir.

- Bozuk JPEG/PNG, ilk 20 bayta kesilmiş PNG, metin içerikli JPEG uzantısı, boş
  JPEG, henüz desteklenmeyen WebP ve video adayları.
- Her hatadan sonra gezinme; hatalı öğe seçiliyken katalog değişse de seçimin ve
  hata durumunun korunması; başka pencerede seçim/frame/bildirimin değişmemesi.
- Tekrarlanan Yenile'den başka medyaya geçiş; eski hatanın video/unsupported
  durumunu ezmemesi; bozuklardan sağlam PNG'ye dönüş.
- Gerçek üretim yüklemesi sıraya konduktan hemen sonra dosyayı silme, monitor
  tarafından görüntünün kaldırılması ve gezinmenin yeniden çalışması.
- Hızlı gezinme ve monitor olayı sırasında pencere kapatma; weak pencerenin
  serbest bırakılması; diğer pencerenin aktif kalması ve kendi Yenile işlemi.
- Gerçek GTK Yenile button `clicked` sinyali; işlem öncesi/sonrası aynı
  GtkWindow nesnelerinin korunması, yeni pencere oluşmaması.

Varsayılan native mod ilk gerçek secure baseline yoksa **exit77** verir.
Baseline geçtikten sonraki sağlam resim başarısızlıkları FAIL'dir. Sağlam
frame'lerde 4×2 boyut doğrulanır; yanlış hata sınıflandırmasını önlemek için
native bozuk dosya kontrolleri loader sürecinin erken çıkışını PASS saymaz.
`--expect-image-error` modu gerçek Glycin çağrısını korur fakat yalnız kontrollü
UI hata durumunu doğrular; **bozuk format decoder PASS'ı veya piksel PASS'ı değildir**.

### Kontrollü worker yarışları — gerçek decoder değildir

`navigation_race_probe` genişletildi: A→B→C, C/B/A tamamlanması; video durumuna
geçiş; bir pencere kapanırken diğerinde gerçekten bekleyen kontrollü worker;
tekrarlanan yükleme ve yalnız güncel istek için result callback'i. Sekiz worker
join edilir; eski token'ların iptali ve weak widget'ların bırakılması doğrulanır.
Bu probe yalnız test backend'inden hata döndürür, piksel üretmez.

`window_failure_race_probe` aynı kontrollü hata tekniğini **üretim window.rs,
image_view, gerçek GIO monitor ve özel dosyalarla** birleştirir. Bekleyen yükleme
sırasında harici değişiklik, otomatik decode başlamaması, iki Yenile, seçili
dosyanın silinmesi, ters sırada gelen eski hatalar, bekleyen worker/monitor ile
kapanış ve hâlâ bekleyen diğer pencerenin bağımsız tamamlanması deterministiktir.
Tam **7** worker başlangıcı sayılarak olayların otomatik/sınırsız yeniden deneme
başlatmadığı kontrol edilir. Bu, gerçek Glycin decoder etkinliği/iptali değildir.

## Gerçek test sonuçları

Kaynak commit'leri:

| SHA | İçerik |
|---|---|
| `7b48e9492c9ad063b831836fb1c7c6afcfa9215a` | Minimal Yenile düzeltmesi, üretim medya hatası senaryosu ve native giriş |
| `2b76543c969cb3b96565a958c9ee5e6220416e65` | Kontrollü A/B/C + pencere/monitor yarışları, Sway probe argümanı |

Mevcut `/workspace/pivra-environment/piv09/image-activate.sh` etkinleştirildi;
Rust/GTK/loader/Xvfb/Sway/Weston kurulumu değiştirilmedi.

| Kontrol | Sonuç |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo check --locked`, `cargo build --locked` | PASS, mevcut helper stable/offline/-j2 |
| `cargo test --locked` | **57 PASS**, 33 unit + 24 integration, 0 failed/ignored; mevcut test sayısı korunur |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS |
| Tüm örneklerin locked/offline derlenmesi | PASS |
| Python aktif-decode gözlemcisi | **30 PASS**, gözlemci unit testleri; decoder PASS değildir |
| Fixture/manifest bütünlüğü | Model fixture'ları ve 18 görüntü varlığı PASS |
| GUI/helper çağrıları | **18 scoped PASS**, aşağıdaki dağılım |
| Yeni/önceki gerçek decoder kapıları | **7 NOT RUN / exit77**, aşağıdaki liste |
| Python syntax, shell syntax, `git diff --check` | PASS |

18 çağrı: **X11 10** (blank, files, desktop, discovery erişim/bozuk symlink,
gerçek xdotool gezinmesi, genişletilmiş ImageView yarış probu, navigation,
folder watch, medya hatası, window/monitor yarış probu); **Sway 6** (blank,
files, navigation, folder watch, medya hatası, window/monitor yarışı);
**Weston 2** (blank/files). Sway testlerinde DISPLAY/XWayland yoktur ve gerçek
Wayland yüzeyi kullanılır; probe action/sinyalleri fiziksel klavye/buton testi
sayılmaz. Weston normal kapanış **NOT RUN**; SIGTERM cleanup bunun yerine geçmez.

7 başarısız ortam kapısı: `image_probe` PNG-first, JPEG-first;
`image_gui_probe`; strict navigation; strict folder watch; yeni strict medya
hatası senaryosu; `active_decode_close.py`. Hepsi **exit77 / NOT RUN**.
Bilinen Codex read-only `/proc` UID-map/Bwrap kısıtında ilk secure baseline
oluşmadı. Aktif kapanış raporu `first_uncompleted_stage=baseline_decoded`,
`decoder_time_overlap=NOT MEASURED`, loader kimliği `not_certified` verdi.
Bu görevde gerçek decoder hatası sınıflandırması, sağlam piksellere dönüş ve
aktif secure decode iptali **PASS değildir**. Sandbox gevşetilmedi.

Dosya yüklemeden önce yoksa mevcut loader unit testi kontrollü hata yolunu;
listing erişim reddi mevcut discovery GUI testini korur. Özel ACL/ağ dosya
sistemi hataları ayrıca test edilmedi. Üretim probundaki silme/kapanışın tam
**aktif secure decode** aralığında gerçekleştiği sertifikalandırılmaz. Bu
zamanlama kontrollü worker testlerinde deterministiktir; native secure checkpoint
ve gerçek süreç overlap ölçümüyle aynı kanıt değildir.

## Native kabul

Güncel `dev` üzerinde gerçek Arch Linux/GNOME Wayland oturumunda tek komut:

```bash
bash tests/run_media_failure_probe.sh
```

Özel geçici dizinleri, aynı uygulamadaki iki gerçek pencereyi ve zorunlu güvenli
Glycin yolunu kullanır. SHA yazdırılır; exit0 yeni native senaryo PASS,
exit77 başlangıç engeli/NOT RUN, diğer sıfır dışı sonuç FAIL'dir. Native kabulde
`--expect-image-error` eklemeyin. Bu komut yeni senaryoyu çalıştırır; önceki
PIV-09 aktif decode checkpoint testi yerine geçmez.

Gerçek uygulamada kısa manuel kabul: bozuk öğeden sağlam fotoğrafa oklarla
geçilebilmesi; bir pencerede hata/gezinme/Yenile/kapanışın diğer pencerenin
fotoğrafını/seçimini/bildirimini bozmaması. Kullanıcı sonucu ve test edilen SHA
ayrıca kaydedilmeden **PIV-12 DONE yapılmaz**.

PIV-01–11 DONE kalır. PIV-13 başlamadı. PIV-39 Türkçe/İngilizce i18n kararı
korunur. Faz 0 bağımsız temiz kurulum INCOMPLETE ve DESKTOP-06 gerçek Nautilus
NOT RUN ayrı açık işlerdir. Kaynak medya, Cargo.toml/lock, sistem güvenliği,
main ve otomatik CI değiştirilmedi.
