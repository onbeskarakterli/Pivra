# FD-03 — Freedesktop thumbnail cache reader

25.10.2026 — **FD-03 DONE: bağımsız, salt okunur cache denetimi.**
Başlangıç: `70b27191ff7f7a74611894c4dcc38907b4fd5f3d`, temiz ve güncel `dev`.
FD-02 kod referansı `ddebb71865d6fa2db51382f53001616e544cc2d4` korunur.
Kod/test commit'i: `4f5c7d8af687d77ff15735f937340a97e4073309`.
Referanslar: [FD-01](FD01_GNOME_BASELINE.md), [FD-02](FD02_PROVIDER_INTERFACE.md),
[kullanıcının esas aldığı Freedesktop standardı](https://specifications.freedesktop.org/thumbnail-spec/latest/).
Standart sayfasına bu ortamdan bağımsız indirme denemesi proxy 403 ile engellendi;
uyumluluk kuralları kullanıcı şartnamesi ve aşağıdaki açık test/politikalarla kaydedilir.

## Kapsam ve API

- `src/freedesktop_thumbnail.rs`: `CacheReader`, `SizeClass`, `cache_home`, `cache_name`.
- `src/lib.rs`: tek modül bildirimi.
- `tests/freedesktop/cache_reader.rs`: yeni modülün 28 bağımsız testi; lib testlerine
  dahil edilir, ayrı hedefli filtreyle de çalışır.
- Bu belge. Başka dosya değiştirilmedi; dependency eklenmedi.

`CacheReader::lookup(&Request, SizeClass) -> Result<CacheOutcome, Error>` blocking
worker API'sidir. GTK initialization veya ana döngü gerektirmez. Yeni thread havuzu,
servis, provider implementasyonu, keşif, manager veya üretim entegrasyonu içermez.
FD-02 trait'i ve mevcut GNOME modelleri değiştirilmedi; `FileKey`, `Request`,
`Candidate`, `CacheOutcome` ve `Error` aynı tiplerdir. ThumbnailProvider implementasyonu
yapmak için generation metodu uydurulmadı; bu aşama yalnız okuyucudur.

**Cache okuması hiçbir dosya/dizin oluşturmaz, kaydetmez, temizlemez veya decode etmez.**
Başarılı Candidate untrusted container/metadata verisidir; piksel başarısı değildir.

## Dizin, sınıf ve URI/MD5

`from_environment()` XDG_CACHE_HOME'u her reader oluşturulurken okur. Geçerli
mutlak ve NUL içermeyen yol kullanılır; ayarsız/boş/göreli/NUL içeren değer için
GLib'in kullanıcı home dizini altında `.cache` seçilir. Geçerli home yoksa Io
hatası verilir; cwd'ye veya başka global cache'e sessiz geçiş yapılmaz.
`cache_home` fonksiyonunun açık girdileri env değiştirmeden bağımsız test sağlar.
`CacheReader::new` açıkça verilen cache home'un mutlak/NUL-free olmasını ister.

| İstenen maksimum kenar | Sınıf | Cache yolu |
|---|---|---|
| 1–128 | Normal | thumbnails/normal |
| 129–256 | Large | thumbnails/large |
| 257–512 | XLarge | thumbnails/x-large |
| 513–1024 | XxLarge | thumbnails/xx-large |

`SizeClass::for_edge` en küçük yeterli sınıfı seçer; 0 veya >1024 için None döner.
Lookup yalnız istenen sınıfı okur; daha büyük sınıfa otomatik fallback yapmaz.
O sınıftaki PNG'nin her iki kenarı da sınıf sınırına uymalıdır. Görüntü oranı veya
pikselleri değiştirilmez; resize/decode yoktur. Farklı sınıfları birleştiren gelecekteki
politika bu aşamada uygulanmaz.

Kaynak yolu `gio::File::for_path(path).uri()` ile URI olur. GLib Checksum::Md5
URI'nin UTF-8 baytlarından küçük harfli `<digest>.png` üretir. Kaynak içeriği okunup
hash'lenmez; MD5 bütünlük/güvenlik kontrolü değildir. Ek canonicalize/percent-decode
uygulanmaz. Göreli yollar ve `.` GIO davranışını izler; Unicode, boşluk, #/%/?,
non-UTF8 Linux adları percent-encoded URI ile test edilmiştir. Yerel URI'nin GIO
path roundtrip'i de kontrol edilir. Kaynak symlink'i desteklenir: link yolunun URI'si
ve o path'e ait mevcut FileKey korunur, hedefe canonicalize edilmez.

Sabit dış referans hash testi:
`file:///tmp/pivra%20test%23%25%3F.png` → `9b25b56f1e46c9e06354b6b814e2bde7.png`.

## PNG/container ve metadata denetimi

IDAT **inflate edilmez**, renk/piksel doğrulaması yapılmaz. Sınırlı okuyucu şunları
kontrol eder: signature, IHDR'nin yeri/tekliği ve alanları, class boyut sınırları,
chunk tip/uzunluk/CRC, palette kuralları, IDAT sırası, IEND/son bayt, kritik bilinmeyen
chunk'ların reddi. IDAT içindeki zlib/piksel hatalarını ileride güvenli Glycin bulmalıdır.

Standart Thumb alanları `tEXt` üzerinden okunur:

- Thumb::URI zorunlu, boş/NUL/geçersiz UTF-8/yanlış URI reddedilir.
- Thumb::MTime zorunlu, decimal u64; kaynak filesystem mtime saniyesiyle eşleşmeli.
- Thumb::Size isteğe bağlı, varsa decimal u64 ve kaynak dosya boyutuyla eşleşmeli.
- Aynı alanın tekrarına, taşan/signed/whitespace içeren sayıya güvenilmez.
- Eksik/bozuk/çelişkili metadata Invalid; eski mtime/size Stale'dir.
- Kaynağın nanosecond ctime'ı cache mtime'ından yeniyse saniye metadata'sı eşleşse de
  Stale döner. Bu, mevcut GNOME kimlik politikasındaki gibi konservatif bir korumadır;
  eski mtime korunarak kopyalanmış bazı cache'leri de reddedebilir.
- Thumb alanlarını zTXt/iTXt içinde sunan veya bunlarla gizleyen girdiler **Invalid**
  döner. Sıkıştırılmış metadata açılmaz; bu okuyucunun desteklediği profil standart
  tEXt Thumb alanlarıdır. Bu uyumluluk sınırı sessiz bir Hit olarak gizlenmez.

Önce container/metadata akışla denetlenir. Hatalı/stale dosya için bütün PNG'yi bir
Vec'e almak gerekmez. Hit'in mevcut Candidate.png alanı için doğrulamadan sonra
aynı açık dosya başa alınır ve sınırlar içinde okunur; son kimlik/kontrol bariyeri
geçilmeden Candidate yayımlanmaz. Bu nedenle geçerli Hit iki okuma geçişi kullanır.

## Sonuçlar ve güvenlik sınırları

| Sonuç | Anlamı |
|---|---|
| Hit | Metadata/container ve anlık kimlik geçerli; decoder/piksel doğrulaması hâlâ gerekli |
| Missing | İstenen sınıfta dosya ve güncel failure marker yok; provider arızası değil |
| Stale | Mtime/size/ctime veya kaynak/cache FileKey değişimi; Hit değil |
| Invalid | Yanlış URI, bozuk/eksik metadata/container, unsafe cache yolu veya boyut sınırı |
| SourceUnavailable | Başlangıçta kaynak yok, regular değil veya okunabilir mode biti yok |
| FailedCached | fail/producer/MD5.png altında güncel, doğrulanmış failure marker; görüntü adayı değil |
| Err(Cancelled) / Err(TimedOut) | FD-02 kontrol hataları ayrı kalır; legacy Outcome::Cancelled kullanılmaz |
| Err(Io) | Dosya/dizin erişimi veya I/O hatası; provider değiştirme komutu değil |

Failure marker'lar üretici alt dizinlerinde aranır, eski marker yok sayılır, bozuk
marker Invalid olur. Geçerli ana cache adayı marker'dan önce gelir. Marker hiçbir
zaman dışarı Hit/Candidate olarak verilmez; başka uygulamanın cache'i silinmez.

Sınırlar: **dosya 2 MiB; toplam text 64 KiB; URI 16 KiB; 2048 PNG chunk;
64 failure producer girdisi; I/O/CRC payload blokları 4096 bayt**.
Hit sırasında PNG + metadata/buffer belleği sınırlıdır; bu toplam uygulama RSS
veya mevcut şerit 16 MiB bütçesinin değiştiği anlamına gelmez.

Cache dosyası regular olmalı. Linux O_NONBLOCK FIFO gibi özel girdilerde bloklamayı,
O_NOFOLLOW symlink'i engeller. Cache dizinleri `/` kökünden açık directory fd'leri
üzerinden `/proc/self/fd` ile yürünür; tüm cache atalarındaki symlink'ler konservatif
olarak reddedilir. Cache root'u symlink olan meşru kurulumlar da reddedilebilir.
Kaynak symlink politikası bundan ayrıdır. `/proc/self/fd` çalışmazsa unsafe fallback yok.
Yeni libc bağımlılığı yerine doğrulanmış Linux x86/x86_64/aarch64 open flag değerleri
kullanılır; başka hedefler fail-closed Unsupported verir. Runtime test platformu
Linux x86_64'tür; diğer mimarilerde native kabul yapılmadı.

Açık fd metadata'sı ve path'in FileKey/dev/inode/len/mtime/ctime kimliği başlangıçta
ve yayın öncesi yeniden karşılaştırılır; cache yeniden güvenli açılışla da doğrulanır.
Kimlik doğrulanamazsa Hit yoktur. İptal/deadline her bounded okuma öncesi/sonrası,
chunk akışında ve yayın öncesi kontrol edilir; explicit cancellation önceliklidir.
Kontroller kooperatiftir: tek bir kernel/filesystem syscall'ını preempt etmezler;
bloklayan uzak filesystem için mutlak wall-clock garantisi iddia edilmez.

Okuyucu command/decoder çalıştırmaz; chmod, cache yazımı, dizin oluşturma ve global
temizlik yoktur. İçerik/mtime/ctime/source kimliği değişmeme testleri vardır.
Filesystem normal read atime güncellemesi yapabilir; atime değişmezliği iddia edilmez.

## Test matrisi — Codex

Mevcut image-activate.sh etkinleştirilerek Rust 1.99.1 / GTK 4.18.6 / adw 1.7.6
ortamı korundu; gereksiz kurulum yapılmadı.

| Kontrol | Sonuç |
|---|---|
| Bağımsız `cargo test --locked --offline --lib freedesktop_thumbnail::tests -j2` | **28 PASS** |
| `cargo check --locked --offline -j2` | PASS |
| `cargo test --locked --offline -j2` | **125 PASS**, 0 fail/ignored: 97 mevcut + 28 FD-03 |
| `cargo clippy --locked --offline --all-targets -j2 -- -D warnings` | PASS |
| İki mevcut thumbnail probe örneğinin locked/offline build'i | PASS |
| `python3 -m unittest discover -s tests/thumbnail -p 'test_*.py'` | **37 PASS**, mevcut testler değişmeden |
| Xvfb/X11 katalog regresyonu | PASS: 120 girişim/120 tekil başarı/0 iptal; click/scroll/iki üretim penceresi dahil, kontrollü pikseller |
| Sway/Wayland katalog + cancellation race | PASS: 122 girişim = 120 tekil başarı + 2 iptal; gerçek GTK/Wayland, kontrollü pikseller |
| `--require-generation-helper` | **NOT RUN / exit 77**: API-checked GNOME sidecar yok |
| Gerçek GNOME üretimi, secure Glycin decode ve fiziksel GNOME/Wayland kabulü | **NOT RUN**; bilinen Codex Bwrap/proc engeli aşılmadı/tekrar denenmedi |
| `cargo fmt --check` | **FAIL — önceden mevcut**, aşağıda; yeni dosyalarda format hatası yok |
| Yeni iki dosya için rustfmt check; `git diff --check` | PASS |

Genel fmt hataları FD-01/02 ile aynı korunmuş dosyalardadır:
image_view.rs:447, active_decode_close_probe.rs:211,
application_open_probe.rs:65/83/97/135. Kapsam dışı format değişikliği yapılmadı.

28 test XDG fallback/sınıflar; gerçek fixture PNG'ye eklenmiş metadata; URI/MD5;
Hit/Missing/Stale/Invalid/FailedCached/SourceUnavailable; CRC/metadata/chunk/file
limitleri; izin, source/cache symlink ve FIFO; source replacement/deletion;
iptal ve deadline ayrımı; I/O sırasında iptal/hata; içerik/kimlik değişmezliğini kapsar.
Kaynak/cache değiştirme yarışları, üretimin kullandığı son kimlik bariyerinde
aynı fd açıkken deterministik replacement/symlink ile sınanır; fiziksel GNOME
veya gerçek concurrent decoder testi olarak sunulmaz.

Üretim graph'ının okuyucuya referans vermediği statik guard ve korunmuş dosyalar
üzerinde `git diff --exit-code` kontrolü PASS. GNOME mevcutken yeni okuyucu aktif
olmaz: **henüz hiçbir üretim oturumunda bağlı değildir**. Bu, FD-06 otomatik
provider seçim mekanizmasının test edildiği anlamına gelmez.

## Korunan üretim ve FD-04 sınırı

thumbnail.rs, thumbnail_strip.rs, image_loader.rs, image_view.rs, window.rs,
GNOME C sidecar, build.rs, FD-02 sözleşmesi ve Cargo.toml/lock değişmedi.
Worker/path tekilleştirme, 64 texture/16 MiB, cache/preload/cancellation, zoom/pan,
thumbnail seçim/scroll ve pencere izolasyonu mevcut akışta korunur.

FD-04'te ileride Candidate mevcut secure Glycin yoluna verilmeden önce ve decode
sonrası Candidate::still_current + Request kontrolü uygulanmalı; decoder başarısı
ayrıca ölçülmelidir. Okuyucunun verdiği boyut sınıfı container sınırıdır, gerçek
frame sonucu değildir. Yeni üretim adaptörü bu görevde uygulanmadı. Önbellek
kaydetme FD-05, aktif provider entegrasyonu FD-06 kapsamındadır.

Mevcut GNOME native regresyon girişi değiştirilmedi:
`bash tests/run_thumbnail_system_probe.sh`. Bu, yeni okuyucuyu üretime bağlamaz;
GNOME/Glycin için eksik native kontrol hâlâ ayrıca çalıştırılmalıdır.

FD-03 burada durur. FD-04–FD-09 ve paketleme/yayın/AUR işlemleri başlatılmadı.
