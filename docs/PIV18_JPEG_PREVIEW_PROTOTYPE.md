# PIV-18C — bağımsız JPEG erken-preview prototipleri

24.10.2026 · dev · **PIV-18 TESTING**. Üretim ImageView, cache, preload,
cancellation, Glycin decoder yolu ve Cargo bağımlılıkları değişmedi. PIV-19 yok.
Başlangıç `807b67a402d19f81e6b2814fa6d3d644830e2786`, temiz dev; gereksiz pull yok.
Kullanıcının Arch PIV-18B gözlemi kabul edildi: JPEG/PNG scale isteği full frame,
aynı Image ikinci still frame NoMoreFrames. Bu deney tekrar edilmedi.

## Cevap ve kanıt durumu

**İki gerçek yöntem için sınırlı prototip var; gerçek erken frame bu ortamda
NOT PROVEN / native decode NOT RUN.** Full-decode-sonrası resize yapılmıyor.
Sadece API isteği değil farklı mekanizmalar deneniyor: mevcut JPEG içindeki
ayrı küçük JPEG baytları ve libjpeg-turbo scaled IDCT. Bunlar üretime önerilmiş
veya kabul edilmiş değildir; tek Bwrap denemesi namespace engelinde durdu.

| Yöntem | Mekanizma | Gerçek preview boyut/buffer | İlk frame/full süreleri | Durum |
|---|---|---|---|---|
| Embedded EXIF thumbnail | IFD1 JPEGInterchangeFormat/Length, Compression=6; çıkarılan JPEG ayrı sandbox decode | NOT RUN | NOT RUN | Varsayılan no-exif.jpg içinde NO PREVIEW; kontrollü fallback |
| Reduced JPEG | libjpeg API scale_num=1, scale_denom=8, baseline Huffman, RGB scanlines | NOT RUN | NOT RUN | Tek sandbox denemesi77; frame oluşmadı |
| Ayrı full baseline | Aynı deney yardımcısı, denom=1; preview denemelerinden sonra yalnız bir kez | NOT RUN | NOT RUN | Önceki sandbox engelinde hiç başlatılmadı |

Varsayılan fixture64×32 JPEG; native reduced output'un8×4 olması beklenir,
**bu bir ölçüm değildir**. Büyük JPEG zaman/bellek değerlendirmesi için gerçek
büyük input verilmelidir. PNG/TIFF/RAW hiçbir yola eklenmedi.

## Embedded yöntemi

`tests/jpeg_preview/probe.py` yalnız marker/EXIF alanlarını okur; host üzerinde
JPEG piksel decoder'ı çalıştırmaz. Input ≤64MiB, APP1 JPEG segment sınırları
≤65533 bayt. TIFF II/MM, magic42, iki IFD, en çok128 entry, duplicate tag,
IFD döngü/overlap, inline SHORT/LONG type/count ve thumbnail offset/length
kontrolleri vardır. İstenen JPEG thumbnail bütünüyle ilgili APP1 içinde ve IFD
tablolarından sonra olmalıdır; SOI/EOI envelope kontrol edilir. Envelope gerçek
decode başarısı sayılmaz. Thumbnail gerçekse bu dosyada önceden mevcut JPEG'in
**aynı baytları** yardımcıya gönderilir; full kaynak pixel decode yapılmaz.

Thumbnail yoksa None/NO PREVIEW; reduced/full fallback açıkça raporlanır.
Bozuk kullanılan metadata ValueError/exit1 ile reddedilir. Bu dar IFD1 profilidir,
genel EXIF parser değildir: SubIFD/MakerNote/JPEG dışı thumbnail desteklenmez;
kullanılmayan metadata alanları açılmaz. Python tamsayıları taşmaz, bütün slice
konumları payload boyutuyla kontrol edilir. Native child malformed JPEG'i ayrıca
libjpeg içinde reddeder. Üretim EXIF orientation uyarlaması yapılmaz; preview'ın
kaynakla görsel eşleşmesi/yönü ayrıca native kabul gerektirir.

## Reduced yöntemi

`tests/jpeg_preview/decoder.c`, yalnız Bwrap child executable'ıdır.
libjpeg-turbo'nun libjpeg ABI'sinde header okunur, source boyut limiti uygulanır,
scale 1/8 ayarlanır, sonra start_decompress/read_scanlines çağrılır. RGB allocation
**output_width × output_height ×3** üzerinden yapılır; kaynak boyutunda RGB buffer
oluşturulup küçültülmez. Baseline sequential Huffman dışı progressive/arithmetic
JPEG **UNSUPPORTED / exit3**. Bu sınır progressive full-image coefficient belleği
riskini prototip kapsamından çıkarır. Source≤32MP, edge≤32768; embedded source
ayrıca≤1MP. C size_t/Linux64 hesabında maksimum32MP×3≤96MB; işlem sınırları ve
header kontrolleri allocation'dan öncedir. Libjpeg warning'leri bozuk/truncated
inputta fatal sayılır. Default error handler yalnız child'ı bitirir.

Scaled IDCT API kullanımı reduced piksel üretimine aday teknik kanıttır;
internal decoder kaynak maliyeti native'de henüz ölçülmedi. 1/8 output için bile
sıkıştırılmış akışın okunması/entropy decode maliyeti vardır. Daha hızlıdır veya
full maliyeti kesin şu oranda azalır iddiası yoktur.

## İzolasyon, bellek ve iptal

Wrapper C'yi mevcut sistem cc + libjpeg headers/library ile derler; üretime
kütüphane eklemez. JPEG decode yalnız `bwrap --unshare-all --die-with-parent
--new-session --cap-drop ALL --clearenv` içinde yapılır. /usr salt okunur,
lib symlink'leri, read-only ld cache, özel proc/dev/tmp ve read-only tek decoder
binary'si bağlanır. Home/proje/fotoğraf dizini bağlanmaz; source yalnız stdin.
Network/PID/user namespace ayrıdır. Kullanıcı/host güvenlik mekanizması değişmez.
**Glycin seccomp profili bu ayrı adapter'a taşınmadı**; onunla birebir güvenlik
ve bakım eşdeğerliği iddia edilmez. Üretim adayı olmadan syscall policy/audit ve
mevcut Glycin sınırına entegrasyon ayrıca gereklidir. Host decode fallback yok.

Parent stdout/stderr'i pipe ile sınırsız RAM'e almaz: özel TemporaryFile,
100MiB RLIMIT_FSIZE, output read bound96MB+1024, stderr read4096. Child address
space512MiB, CPU10s, wall20s; libjpeg memory target16MiB (bu toplam RSS hard cap
**değildir**). Peak child RSS gerçek getrusage ru_maxrss ile KiB yazılır; parent,
Bwrap veya sistem toplamını kapsamaz. RGB PPM byte sayısı ayrıca raporlanır.
Parent input≤64MiB ve deney full-baseline output/temporary slice kopyaları da
bellek maliyetidir. Toplam parent RSS native'de NOT MEASURED. Temporary PPM'ler
sadece deney artefaktıdır ve context çıkışında silinir; ürün disk cache'i değildir.

Timeout veya Ctrl-C yalnız dedicated sandbox process group'a TERM, gerekirse
2s sonra KILL uygular ve communicate/join yapar. Bu ayrı deney sürücüsü
politikasıdır; PIV-16 cooperative Glycin cancellation'ı değiştirmez. Gerçek
iptal/process-cleanup davranışı NOT RUN; birim testinde mock süreçle komut/grup
ve join doğrulandı. Üretime bu kill politikasını taşıma önerisi yoktur.

## Ölçüm ve native tek komut

Arch'ta mevcut cc, libjpeg-turbo headers/libjpeg ve Bwrap gereklidir; script
kurulum yapmaz. Gerçek JPEG path'i verilebilir; verilmezse no-exif.jpg kullanılır:

```bash
bash tests/run_jpeg_preview_prototype.sh /tam/yol/fotograf.jpg
```

Embedded yöntemi ölçmek için gerçekten IFD1 JPEG thumbnail içeren kaynak
seçilmelidir; varmış gibi oluşturulmaz. İki yöntemin başarılı preview frame'leri
elde edilmeden full baseline başlamaz; bir yöntem yoksa diğeri denenir. Her çıktı
source path/SHA256, yöntem, source/output boyutu, RGB buffer bytes, EXIF çıkarım
süresi, read/hash/extract dahil first_usable_total_ms, sandbox start/join dahil
süre ve child peak RSS içerir. Bu **GUI üzerinde sunum zamanı değildir**;
onscreen presentation NOT MEASURED yazılır. Zamanlar monotonic_ns kullanır.
Full baseline yalnız ayrı deneysel kıyastır; normal ürün ikinci decode yapmaz.
Warm filesystem/library cache ve tek örnekten genel hızlandırma çıkarılmaz.

Exit0 gözlemin tamamlandığı anlamındadır; UNSUPPORTED veya NO PREVIEW satırları
PASS değildir. Decoder source/pixel doğruluğu, orientation ve gerçekten küçük
frame koşulları native çıktı ile ayrıca incelenmelidir. Bwrap hatasında77 ile
ilk denemede durulur; host üzerinde yeniden decode yapılmaz.

## Gerçek Codex sonuçları

- PASS: C compile `-O2 -Wall -Wextra -Werror`; mevcut özel include/lib dizinleri
  kullanıldı. İlk link denemesinde sistem -ljpeg development symlink'i bulunmadı;
  mevcut kullanıcı prefix'inin -L yolu verilerek derlendi, yeni paket kurulmadı.
- PASS: **6 Python test**: iki endianness, gerçek JPEG baytlarının embedded IFD1
  çıkarımı, absent preview, offset/length/type/compression/count/cycle/envelope
  negatifleri ve mock timeout group/join. Metadata fixture'ı testte oluşturulur;
  gerçek kameranın erken thumbnail performans kanıtı değildir. Host decode yok.
- PASS: shell syntax, whitespace. Rust/GTK üretim kodu değişmediği için büyük
  core/GUI paketleri tekrar koşulmadı.
- NOT RUN: bir secure reduced denemesi, **exit77**:
  `bwrap: setting up uid map: Read-only file system`. Embedded başlangıcı
  no-exif.jpg için NO PREVIEW verdi; decoder başlamadı. Başka Bwrap denemesi yok.
  Native gerçek frame, piksel doğruluğu, zaman/RSS ve iptal kanıtı henüz yok.

Kod/test commit'i `f69b0edef8f1e5f33f404e234c20d49a08bae170`.
Sonuç: mekanizmalar prototiplenmiştir; **gerçek erken preview NOT PROVEN**.
Native kanıt sonrası en küçük sonraki iş güvenli adapter audit + orientation/
piksel/bozuk-input ve kaynak maliyeti doğrulamasıdır; üretim entegrasyonu ayrı
karardır. PIV-18 TESTING, PIV-19 başlamadı.
