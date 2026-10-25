# FD-06 — Provider Manager ve thumbnail entegrasyonu

Durum: **TESTING**. FD-07 başlamadı. Başlangıç: `ffd72e31f9d1899b8a0f3d23f67436089f5774fb`.

## Üretim sınırı

`thumbnail_manager` uygulama oturumu boyunca tek sağlayıcı seçer. GNOME masaüstünde
kullanılabilir GNOME cache erişimi (CacheOnly dahil) önceliklidir; GNOME devre dışı
veya Unavailable ise Freedesktop seçilir. Dosyaya özgü Missing, Invalid, Stale,
iptal, timeout veya decode hatası sağlayıcı değiştirmez. Geçici üretim hatası
GNOME cache erişimini kaldırmaz. Sıcak/otomatik yeniden seçim uygulanmadı;
ortam/politika değişikliğinde yeni uygulama oturumu gerekir.

GNOME mevcut query_or_generate yolunu korur. Freedesktop Normal sınıfında:

- Hit: mevcut güvenli cache decode yolu.
- Missing: güvenli üretim, NOREPLACE yazma, Reader ile tekrar doğrulama.
- Stale/Invalid: yeni thumbnail bellekte sunulur; shared-cache değiştirilmez.
- Existing/yazma hatası: başarılı bellek sonucu kaybolmaz.
- Bozuk cache pikseli: aynı iş kapsamında en fazla bir güvenli yeniden üretim;
  döngü veya sağlayıcı değişimi yok.

Bellek sonucu gerçek RGBA GdkMemoryTexture kullanır; sahte Candidate oluşturulmaz.
Kaynak kimliği zorunlu, cache kimliği isteğe bağlıdır. İş sonunda kaynak,
iptal ve süre kontrolleri tekrarlanır. Strip epoch/weak-reference, görünür öğe
koruması ve 64 texture / 16 MiB sınırı korunur.

Ortak Permit en fazla iki işi tüm cache decode/üretim ömrü boyunca tutar.
Standalone generator da aynı kabul kapısını kullanır; iç generator sınırı ek
iki bağımsız iş sağlamaz. FD-05A ağırlıklı bellek ve tek large-job sınırı korunur.
Aktif large iş sırasında aynı kaynak sürümünün metadata denemesi ertelenir;
128 kaynak kimliğiyle sınırlı bu geçici kayıt ikinci görüntü cache'i değildir.
Ana fotoğraf loader, viewport, GNOME factory ve UI politikaları değiştirilmedi.

## Codex doğrulamaları

- Rust check/build/test/Clippy: PASS; `cargo test --locked --offline`: **185 test**.
- Thumbnail Python kontrollü regresyonları: **37 PASS**.
- FD-05A pidfd/partial-frame gözlemci regresyonları: **4 PASS**.
- Python derleme ve yeni shell girişinin syntax kontrolü: PASS.
- Xvfb/X11 kontrollü katalog: PASS; 120 başarılı benzersiz cache işi.
- Sway/Wayland kontrollü iptal yarışı: PASS; 120 başarı + 2 iptal.
- Bu GUI testleri gerçek GTK/GIO pencerelerini kullanır; piksel sınırı kontrollüdür.
  Gerçek GNOME factory veya secure decoder başarısı olarak raporlanmaz.
- Secure native önkoşul: **NOT RUN**; Bwrap loader exit 1, probe exit 77.
  Tek deneme yapıldı, güvenlik gevşetilmedi. Native aktif kapanış ve büyük kaynak
  entegrasyonu Codex'te doğrulanmadı.
- `cargo fmt --check`: **FAIL**, başlangıçtan kalan image_view,
  active_decode_close_probe ve application_open_probe biçim farkları. Kapsam dışı
  dosyalar değiştirilmedi. `git diff --check`: PASS.

Eski "üretime bağlı değil" statik testleri FD-06'nın yetkilendirdiği bağlantıyı
olumlu olarak doğrulayacak şekilde güncellendi. Piksel/kimlik/hata assertion'ları
kaldırılmadı. Yeni testler routing, permit ömrü, stale/invalid/yazma hatasında
bellek, kaynak değişimi, iptal/deadline ve RGBA yaşam süresini kapsar.
Önceki FD-04/05/05A native başarıları kullanıcı bildirimidir; FD-06 kabulü değildir.

## Arch GNOME native kabul

Mevcut GNOME referansı (iki özel TEST penceresini yönergeye göre kapatın):

```sh
bash tests/run_thumbnail_system_probe.sh
```

Freedesktop entegrasyonu ve aktif decode sırasında gerçek pencere kapanışı:

```sh
bash tests/run_fd06_freedesktop_probe.sh
```

İkinci komut yalnız test sürecinde farklı desktop kimliği ve özel XDG cache/config
kullanır; sistem GNOME ayarlarını değiştirmez. Önce native katalog/pencere kabulü,
ardından 50 MP kaynakla aktif kapanış çalışır. İlk manuel iki pencere kapatılmalıdır.
Yeterli bellek (yaklaşık 2.3 GiB kullanılabilir), pidfd ve /proc erişimi gerektirir.
Kaynaklar testin geçici dizinindedir; kullanıcı fotoğrafları kullanılmaz.

Gözlemci loader PID/starttime/pidfd kimliğini, doğru boyut ve kısmi frame
prefix/tail koşullarını, STOP sonrası mapping device/inode sahipliğini ister.
Sertifika olmadan PASS vermez. İlk gerçek pencere kapatılıp referansları bırakılır;
loader'ın çıktığı, permit/bellek rezervasyonunun sıfırlandığı, diğer pencerenin
başlığı ve render piksellerinin korunduğu doğrulanır. Sertifika kaçırılırsa 77
NOT RUN'dır; sadece kuyruğa giren veya bitmiş iş aktif decode sayılmaz.

Native GNOME/Freedesktop sonuçları ve gerçek aktif kapanış bekleniyor. FD-06
native kabul gelmeden DONE olmayacak. FD-07 kapsamı başlatılmadı.

## Commit kayıtları

- `be3deadf1752126b606db15197696ba9211e732b`: session routing, ortak admission,
  cache/bellek entegrasyonu ve kontrollü regresyonlar.
- `2c3c35842c489a55fed2e4b0829876a97f271cd6`: native FD katalog ve gerçek aktif
  decode sırasında iki pencere izolasyonu sürücüsü.

GNOME native girişinin önkoşulu ayrıca exit 77 verdi: bu ortamda API kontrolünden
geçmiş GNOME generation sidecar yok. Bu sonuç Bwrap engelinden ayrı NOT RUN'dır.

## Native T/şerit hatası — 2026-10-26

Kullanıcı Arch GNOME native referansının exit 0 ile geçtiğini bildirdi.
Freedesktop denemeleri yalnız session-selection mesajından sonra ilerlemedi;
kullanıcı T ile şeridi açamadı ve iki denemeyi Ctrl-C ile bitirdi (exit 130).
Bunlar Freedesktop üretim başarısı veya decoder hatası kanıtı değildir.

**Kök neden üretim UI admission kontrolüdür**, yalnız test stub'ı değildir.
`ImageView::new` eski `thumbnail::Provider::discover(None)` GNOME capability
kontrolünü kullanıyordu. FD native girişinin process-only desktop kimliğinde bu
Unavailable döner. `thumbnail_overlay::install` T handler'ını kurar, ancak
`available=false` nedeniyle tuşu işlemeden geri döner; fare/düğme de aynı kapıya
takılır. Buna karşılık strip worker doğru biçimde Freedesktop Manager'a bağlıydı.
Manager'ın session seçimi başarılı olduğu halde UI hiç iş başlatamıyordu.

Düzeltme `2e5759a261f7589e4ff9a16d4cb2e3117f5c46ce`: UI kullanılabilirliği
session provider'ına göre belirlenir. GNOME dalındaki eski capability kontrolü
aynen korunur; Freedesktop GNOME bulunmasını gerektirmez. Factory, Glycin,
preload/cache/bellek sınırları veya görünürlük politikası değiştirilmedi.

`--require-freedesktop` şimdi decode öncesi üretim ImageView/overlay üzerinde T
kontrolünü de çalıştırır: kapalı başlangıç, işlenen T, gerçekten mapped strip,
tekrar kapatma, ikinci pencereye bağımsız reveal, aynı ana görüntü pikseli ve
viewport ölçüleri. Bu kısa bölüm TEST pikselleridir; generation PASS değildir.
Native gerçek cache-piksel kontrolü ayrıca mapped strip zorunluluğu taşır.
`--fd06-overlay-regression` yalnız bu kısa bölüm için test girişidir.

Düzeltme sonrası Codex sonuçları:

- Xvfb/X11 ve izole headless Sway/Wayland T/overlay regresyonu: PASS.
  Gerçek GTK controller sinyali ve widget mapping; fiziksel klavye testi değil.
- GNOME kontrollü katalog/iki üretim penceresi regresyonu: PASS; 120 benzersiz
  başarılı cache işi, sıfır iptal; gerçek decoder iddiası yok.
- cargo check, example build, cargo test (185), Clippy all-targets: PASS.
- Thumbnail Python 37 ve gözlemci Python 4: PASS; shell syntax/diff-check PASS.
- fmt --check: önceki kapsam dışı üç dosyanın biçim farkları nedeniyle FAIL.
  Yeni değişikliklere ait format farkı yok.
- Secure native gate: tek denemede Bwrap loader exit 1 / probe exit 77:
  NOT RUN. Yeni T kontrolü bu denemede PASS; Freedesktop üretimi ve aktif decode
  kapanışı native kabulü henüz yok. Güvenlik gevşetilmedi.

Kullanıcının tek yeniden doğrulama komutu:

```sh
bash tests/run_fd06_freedesktop_probe.sh; printf 'FREEDESKTOP EXIT CODE: %s\n' "$?"
```

Önce otomatik T/overlay ve katalog adımları çalışır; bu sırada test pencerelerini
kapatmayın. `MANUAL TH-05` mesajını bekleyin. Ardından iki üretim penceresinde fareyi
alta götürün, T ile kapatıp/açın, thumbnail piksellerini, kaydırma/tıklama ve
Sol/Sağ gezinmesini kontrol edin. İki manuel pencereyi kapatınca otomatik aktif
decode kapanış testi başlar. Nihai exit 0 ve gerçek piksel/aktif kapanış çıktıları
olmadan Freedesktop native kabulü verilmez. FD-06 TESTING; FD-07 başlamadı.

## Aktif native kapanış teşhisi — 2026-10-26

Kullanıcının `1ea0948` Arch/GNOME logu: Freedesktop seçim/T, 120 benzersiz secure
cache decode, değiştirilmiş kaynak pikselleri, gezinme ve iki pencere PASS.
Aktif kapanış bölümü `READY` ve `CANCEL_ISSUED` sonrasında 95 saniyelik deadline'a
ulaşmış, GLib callback içindeki assertion nedeniyle non-unwinding panic ile
sonlanmıştır (driver exit 1). Bu başarısızlık kaydı korunur.

`/tmp/pivra-fd06-close-report-f2vseflx.json` bu Codex ortamında yoktur; paylaşılan
metin yalnız logdur, JSON değildir. PID/starttime, partial certificate ve loader
çıkışı alanları bağımsız okunamadı. Mevcut driver'da cancel komutu ancak STOP
sonrası doğrulanmış partial-frame checkpoint'inden sonra gönderilir; log bu
protokol aşamasına ulaşıldığıyla tutarlıdır, fakat eksik JSON alanları tahmin
edilmez. İki 64x32 FD05A_FRAME kaydı büyük işin tamamlandığını kanıtlamaz.

**Kanıtlanmış harness kusuru:** eski hazır koşulu pencere ve fotoğraf weak
referanslarının None olmasını bekliyordu. Bu, native pencere kapanışı ve decoder
kaynak temizliğiyle aynı olay değildir. Gerçek GTK kontrollü testinde referanslar
bilerek tutulduğunda `close-request` ve `unmap` geldi; native GDK surface
`is_destroyed=true`, görünür/mapped false ve worker/rezervasyon sıfır olduğu
halde iki GObject referansı da yaşıyordu. Widget `destroy` sinyali ancak eldeki
referanslar bırakılırken geldi. Aynı davranış X11 ve Sway/Wayland'da doğrulandı.
Bu bulgu eski bekleme koşulunun yanlışlığını kanıtlar; kullanıcının son
çalışmasında worker'ın gerçekten takılıp takılmadığı eksik JSON nedeniyle
belirlenemez. Üretim cleanup kusuru kanıtlanmadı; `src/` değiştirilmedi.

Düzeltme şunları ayrı ayrı zorunlu tutar ve loglar:

- `close-request`, unmap, görünmez/unmapped pencere ve **gerçek native GDK
  yüzeyinin yok edilmesi**; Widget destroy/ref yaşam süresi ayrıca tanı verisidir.
- Ortak Permit sayısı 0, reserved bytes 0 ve large reservation false.
- Dış gözlemcinin değişmeyen PID/starttime/pidfd STOP+partial sertifikası,
  CANCEL_ISSUED onayı ve diğer pencere açıkken gerçek loader process çıkışı.
- Ancak bu koşullardan sonra finish el sıkışması; ana GTK döngüsü bitince diğer
  pencerenin başlığı/render pikselleri ve kaynak kimliği yeniden doğrulanır.

95 saniye ve dış loader çıkışı için 5 saniye sınırları artırılmadı. Callback artık
assert/panic veya render beklemesi yapmaz: sonucu kaydedip MainLoop'u sonlandırır;
FAIL/exit 1 ve NOT RUN/77 ana akışta verilir. Erken ikinci pencere kapanışı,
yinelenen/erken komut ve observer kopması açık hatadır. Observer timeout hatası
artık JSON'a da kaydedilir; cleanup doğrulanmadıysa NOT VERIFIED kalır.

Doğrulama: Rust check/example build/Clippy PASS; normal Rust 185 ve example
26 test PASS (2 yeni kapanış koşulu testi dahil). Thumbnail Python 37 ve observer
Python 5 PASS (eksik sertifika/ack/loader exit/finish negatifleri dahil).
X11/Sway gerçek GTK kapanış regresyonu PASS, **decoder mock'u veya gerçek aktif
decode PASS değildir**. Shell syntax, Python compile, diff-check PASS.
fmt --check önceki image_view/active_decode_close_probe/application_open_probe
format farkları nedeniyle FAIL; yeni değişikliklerde format farkı yok.
Bu düzeltmenin native secure aktif kapanış kabulü **NOT RUN**: bilinen Codex
Bwrap engeli nedeniyle native benchmark tekrarlanmadı.

Tek yeniden test komutu değişmedi:

```sh
bash tests/run_fd06_freedesktop_probe.sh; printf 'EXIT CODE: %s\n' "$?"
```

`MANUAL TH-05` bölümündeki iki pencereyi kontrol edip kapatın. Ardından açılan
aktif decode bölümündeki pencereleri **elle kapatmayın**; observer kapatma ve
loader çıkışını kendisi doğrular. `CLOSE_EVENT`, `CLOSE_STATE`, certificate,
`window_finish_sent` ve cleanup alanlarını içeren yeni raporu saklayın.
FD-06 TESTING kalır; FD-07 başlamadı.

Kapanış harness/observer düzeltme commit'i:
`84b9b4d5c621555bf2782213bf65f4b2a260f47a`.

## Son native JSON incelemesi — loader pool çıkış aşaması

Kullanıcının yüklediği `pivra-fd06-close-report-_jj25n_c.json` gerçekten okundu.
Kesin hata Rust render/title assertion'ı değildir:
`observer_failure = "loader still alive while other window open"`, exit 1.
`window_finish_sent=false`, `cleanup=NOT VERIFIED`. Dış observer, pencere/worker
hazır bildiriminden sonra hedef pidfd'nin 5 saniyede çıkmasını bekleyip durmuştur.
Dolayısıyla Rust'ın son başlık/render/kaynak kontrolüne bu native koşuda
**ulaşılmadı**; onlara PASS verilmez.

Rapordaki gerçek kanıtlar:

- PID 282739, starttime 18110476, 150000000 byte frame;
  `partial_after_stop=true`, `cancel_acknowledged=true`: aktif large decode
  checkpoint'i ve iptal komutu onayı vardır. `/proc` errors 0.
- close-request/unmap/destroy, native surface destroyed, pencere artık referanssız;
  workers=0, reserved=0, large=false, WINDOW_CLOSED_OTHER_ALIVE görülmüştür.
- Bunlar decode await'inin/iş kabul kapsamının bırakıldığını destekler: generator
  decode await'inden çıktıktan sonra permit ve rezervasyon bırakılır. Ancak
  Glycin Image::drop içindeki ayrı async done/pool temizliği ve loader process
  çıkışı bunlarla eşdeğer değildir. O process'in çıktığı **doğrulanmadı**.
- Son örnekte hedef loader RSS yaklaşık 8.7 MB'a düşmüştür; düşük RSS veya boş
  permit bir process-exit kanıtı değildir.
- photo_referenced=true tek başına aktif decoder veya sızıntı sertifikası değildir.
  GTK nesnesinin sahip referansı bu JSON'dan bulunamaz; kalıcı referans sızıntısı
  olmadığı da iddia edilmez. Pencere/yüzey kapanışı ayrı olarak kanıtlanmıştır.

Glycin **sabitlenmiş 3.0.8** kaynakları incelendi: `pool.rs` PoolConfig::default
loader_retention_time=30s; UsageTracker::drop bu gecikmeyle clean_loaders planlar;
clean_loaders ancak kullanıcı sayısı sıfır ve idle>30s ise girdiyi bırakır.
`api_loader.rs` iptal işleminde process'i doğrudan öldürmek yerine image-path
`done` isteği gönderir; Image::drop da async done sonrası usage tracker'ı bırakır.
Bu nedenle önceki 5s process-exit beklentisi bağımlılığın idle pool politikasıyla
uyumsuzdur. Bu kaynak kanıtı, native PID'nin daha sonra mutlaka çıkacağını
kanıtlamaz; yeni native koşu bunu ölçmelidir.

`344c2ebc7a32d0721c8cb3d43be55239fef837c9` düzeltmesi yalnız probe/test tarafındadır:

- Belgeli 30s pool bekleme aşaması + mevcut 5s gerçek exit grace aşaması.
  Üst native 95s / observer 100s sınırları korunur. Rastgele timeout seçilmedi.
- Aynı PID/starttime'a ait gerçek pidfd çıkışı görülmeden finish gönderilmez.
  35s sonunda hâlâ canlıysa açık FAIL; RSS/idle/permit sayısı exit yerine geçmez.
- LOADER_EXIT fazı, PID/starttime ve ölçülen süre terminale yazılır.
- Observer hatası artık terminalde `FAIL FD06 active close: <neden>` olarak çıkar;
  JSON hata ayrıntıları korunur. Rust son title/render/source kontrolleri ayrı
  başarı mesajları verir; assertion ve güvenlik kontrolleri kaldırılmadı.

Codex: 8 observer testi PASS (pool/grace sınırları, eksik sertifika/ack/exit/finish,
canlı gerçek child ve sonrasında gerçek pidfd exit dahil); thumbnail Python 37
PASS; Rust check/test (185)/example build/Clippy PASS. Kontrollü GNOME X11 katalog,
120 tekil cache işi ve iki pencere regresyonu PASS; secure decoder başarısı değildir.
Python compile ve diff-check PASS. Önceki fmt farkları kapsam dışı kalır.
Yeni native decode/loader exit ve son render kabulü **NOT RUN**; bilinen Codex
sandbox engelinde tekrar deneme yapılmadı. Önceki native FAIL kaydı korunur.

Tek komut: `bash tests/run_fd06_freedesktop_probe.sh; printf 'EXIT CODE: %s\n' "$?"`.
Son otomatik decode pencerelerini elle kapatmayın. Loader retention aşaması
loglandığında bekleyin; gerçek PID çıkışı ve tüm final kontrolleri olmadan
kabul verilmez. **FD-06 TESTING; FD-07 başlamadı.**
