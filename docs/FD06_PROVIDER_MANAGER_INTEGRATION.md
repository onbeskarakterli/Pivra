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
