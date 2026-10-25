# FD-05 — Freedesktop cache writer

25.10.2026 — **TESTING: kod/kontrollü kabul tamam, native writer roundtrip bekleniyor.**
Başlangıç: `0462c7a23b833867c433ff93ced45626a0009424`, temiz dev.
Kod/test: `d6cc7d07845841060787113bc4ec522596016ba7`.
Native giriş: `1e9f27b439c20153ad750ab38899ceb0a8c7f417`.
Referanslar: [FD-03](FD03_FREEDESKTOP_CACHE_READER.md),
[FD-04](FD04_GLYCIN_GENERATOR.md), [büyük fotoğraf kararı](FD05_LARGE_IMAGE_DESIGN.md).

## Tasarım ve PNG profili

`freedesktop_writer::CacheWriter::write(&GeneratedThumbnail, &Request)` bağımsız
blocking worker çağrısıdır. Decoder, provider seçimi, thread/pool/queue ve UI bağlantısı
yoktur. FD-04 çıktısını alır; yeniden kaynak decode etmez veya resize yapmaz.

Mevcut GdkPixbuf PNG encoder, GIO WriteOutputStream üzerinden sınırlı dosya sink'ine
akışla yazar. RGBA Bytes referansı kullanılır, tam RGBA kopyası/PNG Vec'i oluşturulmaz.
Width/height pozitif ve sınıf içinde, stride=width*4 ve buffer tam stride*height
olmalıdır. Maksimum giriş4 MiB'dir. Encoder'ın satır/zlib tamponları ayrıca vardır;
toplam süreç RSS'i için4 MiB garantisi verilmez.

Metadata yalnız standart `tEXt` ASCII alanlarıdır: Thumb::URI, Thumb::MTime,
Thumb::Size, Software=Pivra. URI GIO'dan gelir; sayılar decimal u64 profiline uyar.
Kaynak FileKey ve mtime/size snapshot'ı doğrulanır; eski pikseller yeni kaynağın
metadata'sıyla yazılamaz. PNG signature/IHDR/IDAT/IEND/CRC mevcut encoder'dandır;
dört sınıfta gerçek çıktının FD-03 Reader tarafından Hit kabulü test edildi.
Alfa için RGBA IHDR doğrulandı; **gerçek piksel roundtrip native testte ayrıca gerekir**.

XDG konumu, sınıf isimleri ve GIO URI→MD5 adı FD-03 yardımcılarını kullanır.
Reader'da yalnız5 mevcut sembol pub(crate) yapıldı: MAX_FILE, fd_path,
open_component, open_directory, same. Algoritma/güvenlik kontrolleri değişmedi.
Writer ek olarak parent traversal içeren açık cache home'u reddeder.

## Atomik yayın ve yarış politikası

- Dizinler `/` kökünden pinned fd üzerinden, FD-03 O_NOFOLLOW/O_NONBLOCK
  yürüyüşüyle açılır; symlink takip edilmez. mkdirat ile yeni dizinler0700'dür.
- Cache alt dizinleri etkin UID'ye ait, owner write/execute açık ve group/world
  writable olmamalıdır. Sticky olmayan writable atalar reddedilir. Var olan
  dizin izinleri değiştirilmez. Restrictive umask erişimi engellerse fail-closed.
- 128-bit ad entropisi `/dev/urandom`'dan alınır; rastgele temp adı aynı hedef
  dizinde create_new ile açılır. Kendi yeni fd'sine0600 uygulanır.
- Source/control, temp inode/len/mtime/ctime/mode/link count ve görünür hedef
  dizin kimliği yayın öncesi yeniden kontrol edilir. Temp bütünlüğü değişirse ret.
- Dosya fsync sonrası **renameat2(RENAME_NOREPLACE)** ile atomik yayımlanır.
  Linux/filesystem bunu desteklemiyorsa overwrite fallback yoktur; Io döner.
- Politika **no-clobber**: aynı anda hedefi oluşturan üretici kazanır. Kaybeden
  Existing döndürür; hiçbir mevcut regular kayıt, bozuk/eski olsa bile ezilmez.
  Existing bir Hit/başarılı yenileme iddiası değildir. Symlink/directory hedef hata olur.
- Kendi geçici dosyası RAII ile yalnız fd/name inode eşleşmesi sürüyorsa silinir.
  Yerine konmuş başka inode korunur. Başarısız işlem yeni boş dizin bırakabilir;
  global temizlik veya başka uygulama kaydı silme yoktur.

Bu konservatif politika **stale/corrupt mevcut kaydı yenilemez**. FD-06 öncesi
cache-first akışı ve gerekirse ayrı, onaylı conditional replacement politikası
kararlaştırılmalıdır. Şu an sil-üstüne-yaz veya koşulsuz atomic replace eklenmedi.

Başarılı rename commit noktasıdır: daha sonra gelen iptal geçerli kaydı silmez.
Directory fsync ve namespace/source kontrolü sonrasında hata olursa yayınlanmış
kayıt geri silinmez; caller Error alabilir ve Reader ile tekrar sorgulamalıdır.
Source/cache iki farklı dosyadır, ortak filesystem transaction yoktur. Son kontrolden
sonra kaynak değişirse kaydedilen metadata eski snapshot'tır; post-check/Reader
bunu reddeder. Directory rename sonrası namespace kaybı da açık hata olur.

Dosya fsync, yeni directory entry'lerinde parent fsync, yayın sonrası hedef dizin
fsync uygulanır. Atomic görünürlük, depolama donanımının güç kesintisi dayanıklılığı
ile aynı garanti değildir; gerçek power-loss testi yapılmadı.

Güvenlik modeli cooperating same-user cache üreticileri ve filesystem izinleridir.
Kötücül aynı UID/root tüm kullanıcı dosyalarını değiştirebildiğinden inode-check ile
unlink arasına karşı eksiksiz bir security boundary iddia edilmez. Symlink/directory
swap ve temp substitution senaryoları deterministik olarak sınanır; başka inode'un
temp adıyla değiştirilmesi halinde silinmez.

## Kontrol ve kaynak sınırları

Request iptal/deadline önceliği korunur. Encoder öncesi/sonrası, sink write bloklarında,
fsync sonrası ve yayın öncesi kontrol vardır. Sink Error::Cancelled/TimedOut/
SourceChanged/SourceUnavailable/LimitExceeded ayrıntısını korur. Tek kernel syscall
veya encoder C çağrısı preempt edilmez; kooperatif kontroldür.

PNG akışı FD-03 ile aynı **2 MiB** üst sınırında kesilir ve temp temizlenir. Özellikle
noise içeren1024px PNG2 MiB'yi geçebilir; LimitExceeded döner, Reader sınırı gevşetilmez.
Bu4 MiB RGBA girişiyle çelişmez: ayrı encoded-file güvenlik sınırıdır.
Writer yeni worker üretmez; FD-06 mevcut global admission'ı generate+write boyunca
paylaşmalı ve64 texture/16 MiB politikasını korumalıdır. Henüz bu entegrasyon yoktur.

## Doğrulama

| Kontrol | Gerçek sonuç |
|---|---|
| Hedefli `cargo test --locked --offline --lib freedesktop_writer::tests -j2` | **24 PASS** |
| cargo check --locked --offline | PASS |
| cargo test --locked --offline | **169 PASS**,145 mevcut+24 yeni,0 fail/ignored |
| cargo clippy --locked --offline --all-targets -- -D warnings | PASS |
| Thumbnail Python suite | **37 PASS**, değiştirilmedi |
| Native probe build / shell syntax / git diff --check | PASS |
| Writer→Reader dört sınıf/metadata/MD5/source identity | PASS, gerçek encoder+container okuyucu; decode başarısı değil |
| Atomic görünürlük, iki eşzamanlı üretici, no-clobber, private temp/mode/cleanup/fd | PASS |
| Symlink/temp/directory/source değişimi, izin, iptal/timeout, encoded-size sınırı | PASS |
| Gerçek secure Glycin written-PNG alfa/piksel roundtrip | **NOT RUN**, bilinen aynı Codex Bwrap engeli; yeni tekrar yapılmadı |
| cargo fmt --check | **FAIL — önceden mevcut**: image_view.rs447, active_decode_close_probe.rs211, application_open_probe.rs65/83/97/135 |
| Yeni/değişen dosyaların hedefli rustfmt kontrolü | PASS |

Kullanıcı bu görevin talimatında FD-04 native üretim PASS bildirdi. Bu kullanıcı
kanıtıdır; Codex bağımsız decode PASS'i veya FD-05 written-PNG kabulü değildir.
FD-04 aktif native iptal/cleanup ve entegrasyon kontrolleri açık kalır.

## Native komut ve kapsam sonu

```bash
bash tests/run_freedesktop_writer_probe.sh
```

Özel source/cache/config dizinlerinde secure kaynak üretimi → writer → Reader Hit
→ yazılan PNG'nin gerçek secure Glycin decode'u ve RGBA karşılaştırması yapılır.
Dört sınıf, JPEG/PNG kaynakları, açıkça controlled RGBA alpha girdisi ve kaynak
invalidation kapsanır. İlk secure gate engelinde77/NOT RUN, assertion hatasında
başarısızlık olur; retry veya bypass yoktur. Kullanıcı cache'ine yazılmaz.

GNOME thumbnail.rs/strip, image_loader/view/window, native sidecar/build.rs,
FD-04 generator ve Cargo.toml/lock aynı kaldı (git diff --exit-code). Reader'ın
beş crate içi görünürlük değişimi dışında davranış farkı yoktur. Yeni crate veya
image framework eklenmedi; mevcut GdkPixbuf/GIO encoder ve küçük Linux libc API
bildirimleri kullanıldı. GPL/LGPL uygulama kodu kopyalanmadı, lisans seçilmedi.

FD-05 **TESTING** kalır; native writer roundtrip sonucu beklenir. FD-06 ve sonrası
başlatılmadı. Büyük fotoğraf için önerilen FD-05A yalnız tasarımdır; yeni yetki gerekir.
