# PIV-11 — canlı katalog ve kullanıcı eylemiyle yenileme

22.10.2026 · `dev` · **TESTING**, native kabul bekleniyor.
Başlangıç: `b83aa93e7b6d754c4cc2f55e7f6ff584046e49b6`.
PIV-01–10 DONE kayıtları korunur. PIV-10'un native/gerçek klavye kanıtları
kullanıcı kaynaklıdır; bu çalışmada Codex tarafından tekrar üretilmiş sayılmaz.

## Uygulama

Her dosyalı pencere kendi GIO `FileMonitor` nesnesini, kataloğunu ve iptal durumunu
sahiplenir. `WATCH_MOVES` kullanılır. Olaylar 120 ms sabit pencerede birleştirilir;
sürekli olay gelmesi zamanlayıcıyı sürekli ertelemez. Aynı anda en fazla bir
katalog worker'ı ve bir bekleyen zamanlayıcı vardır. Worker sırasında yeni olay
varsa eski snapshot uygulanmaz, yeni tarama planlanır. Dosya sistemi çalışması
worker'dadır; GTK/GIO UI güncellemeleri ana context'tedir. Doğal sıralama ve
mevcut tek-klasör/readability/uzantı politikası değişmedi.

Snapshot tamamlandığında **o andaki** seçim yolu korunur; tarama başındaki seçim
kullanılmaz. Rename eşleştirmesi GIO'nun eski/yeni yollarını kullanır. Rename
olarak eşlenemeyen taşıma/silmede başka fotoğrafa otomatik atlanmaz. Seçim yokken
Sağ ilk, Sol son komşuyu seçebilir; boş katalogda ikisi de etkisizdir. Mevcut
seçim kaybolduğunda görüntü ve bekleyen decoder iptal edilerek açık durum sunulur.
Liste güncellemesi görüntü yüklemesi başlatmaz.

Açık dosya değişirse tek bir “Dosya değiştirildi — Yenile” bileşeni gösterilir.
Düğme `win.reload-media` eylemini çalıştırır. Bu eylem mevcut güvenli Glycin/Bwrap
ve yükleme-token yolunu kullanır. Başarılı, hâlâ güncel yükleme bildirimi temizler;
hata veya yükleme sırasında yeni değişiklik bildirimi korur. Başka medya seçmek
eski bildirimi kaldırır. Tekrarlanan eylemler eski token'ı iptal eder; eski
callback görüntüyü/bildirimi değiştiremez. Yeni toolbar/ayar/video motoru yoktur.

Symlink katalogda lexical alias olarak kalır. Ayrı, iptal edilebilir GIO dosya
izleyicileri aday symlink hedeflerini de izler; başka klasördeki hedef değişikliği
seçili alias için bildirim üretir. Dangling alias katalogdan çıkarılır, hedefin
yeniden gelmesi taramayı tetikleyebilir. Klasör kaybolduğunda kontrollü durum
sunulur; yeni konuma otomatik takip/izleyici yeniden kurma sözü verilmez,
dosyayı yeniden açmak gerekir. İzleyici kurulum hataları stderr/durum alanında
bildirilir. Kapanış zamanlayıcıyı kaldırır, monitor'ları iptal eder ve worker
iptal bayrağını işaretler. Monitor callback'leri state'e yalnız weak erişir.

## Codex'te çalıştırılan sonuçlar

| Kontrol | Gerçek sonuç / kapsam |
|---|---|
| `cargo fmt --check` | PASS (`tests/check.sh`, stable) |
| `cargo check --locked` | PASS (offline, j2) |
| `cargo build --locked` | PASS (offline, j2) |
| `cargo test --locked` | **57 PASS**, 0 failed/ignored: 33 unit + 24 integration |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS (offline, j2) |
| Tüm örneklerin derlenmesi | PASS, `cargo build --locked --offline --examples -j2` |
| Python aktif-decode gözlemci regresyonu | **30 PASS**, gerçek decode sonucu değildir |
| Fixture bütünlüğü | Eski model fixture'ları ve 18 görüntü varlığının hash doğrulaması PASS |
| Eski GUI regresyonları | **12 scoped çağrı PASS**: X11 blank/files/desktop/discovery/gerçek tuş gezinmesi/controlled race; Sway blank/files/navigation; Weston blank/files; X11 navigation probe |
| Yeni folder watch probe | **2 scoped çağrı PASS**, Xvfb ve native headless Sway; gerçek GIO/dosya sistemi, görüntü hatası modu |
| Gerçek secure folder watch pixel/refresh gate | **NOT RUN, exit 77**: ilk gerçek decode engelli |
| PIV-09 `image_gui_probe` | **NOT RUN, exit 77**, aynı ortam engeli |

Yeni probun gerçek üretim pencerelerinde doğruladıkları: fotoğraf/video ekleme,
desteklenmeyen dosyayı dışlama, 1/2/10 doğal sıra, seçimi koruma, seçili olmayan
dosyaya yazma, seçili dosyaya doğrudan yazma ve atomic replace bildirimi,
başarısız Yenile'de bildirimin korunması, rename, 50 hızlı dosya ekleme ile hızlı
ileri/geri gezinme, seçili silinmede görüntünün kaldırılması, boş katalogdan yeni
medyaya kullanıcı gezinmesi, başka klasördeki symlink hedefinin değişmesi/kaybı,
klasörün taşınması, iki pencere bağımsızlığı, kapanış sonrası weak pencerenin
serbest bırakılması ve sonradan dosya olayıyla crash olmaması. Erişim reddi ve
broken-symlink uyarısı eski discovery GUI/integration testlerinde PASS.

`navigation_probe` eski sabit liste varsayımından canlı listeye uyarlandı:
silinen `12.png` artık listeden çıkar; komşu gezinmesi 5 aday üzerinden sürer.
Assertion kaldırılarak eski davranış PASS yapılmadı; yeni liste sayısı ve seçili
dosya açıkça doğrulanır. `navigation_race_probe` kontrollü test backend'idir;
Glycin piksel başarısı sayılmaz. Sway probe eylemleri GTK sinyali/action ile
çalıştırır; fiziksel Wayland klavye veya GNOME testi sayılmaz. Weston normal
kapanışı **NOT RUN**, SIGTERM cleanup kapanış PASS değildir.

Codex'in read-only `/proc` UID-map/Bwrap engeli sürer. Sandbox kapatılmadı,
`NotSandboxed`/fallback eklenmedi. Kaynak görüntüler uygulama tarafından
kaydedilmez; prob yalnız kendi geçici kopyalarını değiştirir. Cargo bağımlılıkları,
lockfile, sistem kurulumu, CI ve lisans değiştirilmedi.

## Kalan native kabul — tek komut

Gerçek Arch Linux/GNOME Wayland oturumunda güncel `dev` için:

```bash
bash tests/run_folder_watch_probe.sh
```

Komut gerçek üretim pencere/monitor/image_view/Glycin kodunu derler, commit SHA'yı
yazar, `GDK_BACKEND=wayland` ve fatal critical'larla çalışır. Varsayılan modda ilk
secure görüntü yoksa exit 77; gerçek baseline geçtikten sonraki başarısızlıklar
FAIL'dir. Atomic save öncesi frame nesnesinin kullanıcı eylemine kadar aynı
kaldığını ve eylem sonrasında yeni **32×64** frame geldiğini doğrular. Native
kabul için `--expect-image-error` kullanmayın: bu yalnız Codex'in dar GUI kapsamıdır.

Gerçek Pivra'da ayrıca yalnız şu görsel kontroller kalır:

1. Açık klasöre eklenen/silinen/yeniden adlandırılan medya gezinmeye yansır.
2. Açık fotoğraf haricen değiştiğinde Yenile bildirimi görünür.
3. Yenile düğmesine basınca yeni görüntü görünür.
4. Kullanıcı Yenile'ye basmadan görüntü kendiliğinden değişmez.

Bu native sonuç gelmeden **PIV-11 DONE değildir**. Faz 0 temiz native kurulum
INCOMPLETE ve DESKTOP-06 gerçek Nautilus NOT RUN bağımsız açık işlerdir.
PIV-12 başlatılmadı. Ağ dosya sistemi/özel monitor backend'leri ve monitor
kurulum hatası enjeksiyonu ayrıca test edilmedi; yerel Linux GIO kapsamı dışına
PASS genellenmez. Scan/decode sayılarının performans ölçümü yapılmadı; eşzamanlı
scan/token sınırları kod invariants ve mevcut yarış testleriyle sınırlıdır.
