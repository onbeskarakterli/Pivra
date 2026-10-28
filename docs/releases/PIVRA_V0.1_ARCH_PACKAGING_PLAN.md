# Pivra v0.1.0 — Arch/AUR paketleme ve kurulum planı

> **B6-F2 current status — 2026-10-29: technical GO** for accepted RC
> `7e26e780acffdb1f2db0f37e8cf6485eba0baa63`. User-reported Arch source/hash
> verification, actual `.SRCINFO` consistency, clean makepkg, installation,
> Qkk 1309 files/0 altered and GNOME acceptance PASS. Older B5 NO-GO/NOT RUN
> tables below are dated history. Codex did not independently run native Arch.
> No main/tag/publication approval; machine-local RC config/generated metadata
> remain outside the application repo. Preload risk is open and explicitly accepted
> for v0.1.0; source/license duties remain. See the
> [F2 assessment](PIVRA_V0.1_B6_READINESS_AUDIT.md).

Tarih: 27.10.2026 (Europe/Istanbul). İncelenen `dev` HEAD:
`a99bd6717699a87b405aa4b8baf221fbaf8b629f`.

**B5: NO-GO / AÇIK.** Bu belge bir tasarımdır. PKGBUILD, .SRCINFO, paket,
kurulum, ikon, tag, GitHub Release veya AUR yayını oluşturulmadı. Tek paket
hedefi `pivra`; `pivra-git`, ayrı internal provider veya yeni servis önerilmiyor.
AUR'da mevcut paket veya yayımlanmış resmî v0.1.0 olduğu iddia edilmiyor.

## 1. Kaynak incelemesi: hazır ve eksik bileşenler

| Bileşen | Kaynaktan doğrulanan durum | Paketleme için kalan iş |
|---|---|---|
| Cargo.toml | pivra 0.1.0, Rust 2024, rust-version 1.85, GPL-3.0-only; publish=false | crates.io yayını gerekmez; temiz Arch release derlemesi ve güncel MSRV kontrolü |
| Cargo.lock | 185 paket: proje + 184 dış crate; sabit checksum/sürümler | Paket tarifi bu dosyayı korumalı; --locked tek başına bit düzeyinde tekrar üretilebilirlik kanıtı değil |
| Lisans | Kök LICENSE tam GPLv3; SHA-256 `3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986`; Cargo GPL-3.0-only | Paket ve source delivery lisans/bildirim yükümlülükleri ayrıca uygulanmalı |
| README / yayın notları | JPEG/PNG full-only ve gerçek özellik sınırları güncel; eski kayıtlar korunmuş | Yayın notu yayımlanmış paket veya kurulmuş masaüstü kabulü değildir |
| CLI | `pivra`; manifest/binary/app sürümü 0.1.0 | /usr/bin kurulumu ve build ağacı olmadan --help/--version/gerçek açılış |
| Desktop | data/io.github.onbeskarakterli.Pivra.desktop; Exec=pivra %u; JPEG/PNG; DBusActivatable=false | Icon anahtarı yok; kurulu menü, gerçek Nautilus ve Open With doğrulaması |
| Uygulama ikonu | Tracked normal/sembolik uygulama ikonu yok; PNG'ler test fixture'ları | Özgün/onaylı ikon ve lisans/provenance gerekli; fixture uygulama ikonu olarak kullanılamaz |
| GNOME helper | build.rs, native/thumbnail_factory.c ve runtime sibling keşfi var | Gerçek API ile zorunlu paket build'i, deterministik helper çıktısı, kurulu binary testi |
| Glycin | Rust istemcisi =3.0.8; zorunlu Bwrap, 2+ loader ailesi, JPEG/PNG içerik kontrolü | Arch paketinin gerçekten 2+ JPEG/PNG config + executable sağladığı doğrulanmalı |
| Thumbnail | Oturumda GNOME önceliği, Freedesktop fallback; ortak cache güvenliği | Kurulu pakette iki yol ayrı kabul edilmeli; her dosya hatası fallback sebebi değil |
| Testler | CLI/desktop unit testleri, görüntü/native/thumbnail/FD probe ve geçici fixture altyapısı var | Bunlar makepkg kurulum/kaldırma betikleri değil; çoğu source-tree binary'si çalıştırıyor |
| PKGBUILD / .SRCINFO | Yok | Onaylı kaynak, bağımlılıklar, build/check/package ve üretilmiş metadata |
| Kurulum / kaldırma | Tracked installer/uninstaller/.install yok | Öncelikle pacman dosya sahipliği + sistem hook'ları; özel root betiği gerekmemeli |
| AppStream | Yok | AUR için zorunlu kapı değil; GNOME Software desteği istenirse ayrı metadata işi |

Kaynak kanıtları: `Cargo.toml`, `Cargo.lock`, `LICENSE`,
`data/io.github.onbeskarakterli.Pivra.desktop`, `build.rs`,
`native/thumbnail_factory.c`, `src/thumbnail.rs::generation_helper`,
`src/image_loader.rs`, `src/thumbnail_manager.rs`, `tests/desktop.rs`.
[README](../../README.md), [yayın notları](PIVRA_V0.1_RELEASE_NOTES.md) ve
[lisans denetimi](PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md) birlikte okunmalı.

## 2. Kaynak, sürüm ve bütünlük kararı

- Tek, sürümlü `pivra` paketi: pkgver=0.1.0, başlangıç pkgrel=1.
- Kaynak, proje sahibinin sonradan onaylayacağı main release commit'ine bağlı
  olmalı. Bugünkü dev SHA'yı yayımlanmış main/tag gibi göstermeyin; bu görev
  main'e geçmez ve merge/tag/release yapmaz.
- Tasarım tercihi: tam 40 haneli onaylı release commit'inin GitHub source
  arşivi; varsa sürüm tag'inin bu commit'e işaret ettiği ayrıca doğrulanır.
  Sadece değişebilen branch HEAD veya git+... kaynağı kullanılmaz.
- Kaynak URL taslağı:
  `https://github.com/onbeskarakterli/Pivra/archive/<APPROVED_RELEASE_COMMIT>.tar.gz`.
  Yerel kaynak adı `pivra-0.1.0.tar.gz` olabilir; çıkarılan dizin commit adını taşır.
- Gerçek arşiv indirildikten sonra SHA-256 hesaplanır, arşiv içeriği/commit,
  Cargo.lock ve lisanslar kontrol edilir; PKGBUILD ve .SRCINFO'ya aynı gerçek
  checksum yazılır. Şu anda onaylı release arşivi/checksum yok.
- `SKIP`, sahte checksum veya ağdan alınan en son kaynağı sessizce kabul etme yok.
  İmzalı tag/arşiv varsa imza ek doğrulamadır, checksum'un yerini almaz.
- Ağ erişimi prepare aşamasında `cargo fetch --locked --target ...` ile yapılır;
  build/check `--offline --locked` kullanır. cargo update, bağımlılık değiştirme,
  sistemde rustup çalıştırma veya build sırasında kullanıcıya paket kurma yok.
- Kaynak checksum'u + kilitli Cargo grafiği, tek başına bit eşitliği sağlamaz.
  Rust/linker/OS paket sürümleri, makepkg seçenekleri, SOURCE_DATE_EPOCH,
  debug/path bilgileri ve native helper derlemesi .BUILDINFO ile kaydedilmeli.
  İki ayrı temiz build/repro karşılaştırması yapılmadan “reproducible” PASS yok.

## 3. Önerilen bağımlılıklar ve kanıt düzeyi

Arch web paket JSON uçlarına yapılan salt-okunur sorgular bu Codex ağında
`Tunnel connection failed: 403 Forbidden` verdi. Aşağıdaki Arch adları **tasarım
adaylarıdır**; yerel pacman sync/veritabanı ve paket dosya listeleri ile kabulden
önce doğrulanmalı. Güncel Arch sürümleri, split-package ilişkileri veya yayımlı
paket varlığı bu görevde bağımsız doğrulanmış sayılmıyor.

| Rol | Önerilen Arch adı / koşul | Kanıt ve doğrulama |
|---|---|---|
| Runtime | gtk4>=4.16 | Cargo gtk4 v4_16 ve glycin gdk4 entegrasyonu; pkg-config gtk4 + gerçek binary link incelemesi |
| Runtime | libadwaita>=1.2 | Cargo libadwaita v1_2; pkg-config libadwaita-1; GTK minimumunu düşürmez |
| Runtime | glib2, gdk-pixbuf2 | GLib/GIO/GObject ve GNOME helper doğrudan API kullanıyor; paket isim/soname sahipliği doğrulanmalı |
| Runtime | gnome-desktop-4 (aday, zorunlu tam GNOME build hedefi) | build.rs önce gnome-desktop-4 sonra gnome-desktop-3.0 dener. Tek Arch paketi tasarımında gerçek gnome-desktop-4 API/link koşulu sabitlenmeli; uyumsuz modül sessizce seçilmemeli |
| Runtime | glycin **veya** glycin-loaders sağlayan gerçek Arch paketi — karar açık | Crate glycin3.0.8 ile native paket sürümünü eşitlemeyin. /usr/share/glycin-loaders/2+/conf.d altında JPEG/PNG ve ilgili executable şart; libglycin.so tek başına loader değil |
| Runtime | bubblewrap | glycin Bwrap kodu bwrap executable'ını çalıştırıyor; sadece “yüklü” olmak user namespace/seccomp çalışıyor demek değil |
| Runtime/build | libseccomp | glycin sandbox yolu ve libseccomp-sys; paket link/runtime soname kontrolü |
| Runtime/build adayı | lcms2 | lcms2-sys dynamic/static-fallback seçenekleri var. Temiz build'de gerçekten shared mı bundled mı ölçülmeli; kullanılmayan runtime bağımlılığı körlemesine eklenmemeli |
| Runtime | glibc, gcc-libs | Doğrudan native bağlantılar ldd/readelf/namcap ve dosya sahipliğiyle doğrulanmalı |
| Runtime/masaüstü | hicolor-icon-theme | Standart icon hierarchy/hook altyapısı; gerçek paket gereksinimi doğrulanmalı |
| Makedepends | rust>=1.85 | Manifest alt sınırı; edition2024; mevcut lockfile'ın Arch derlemesi bu minimumda ayrıca kanıtlanmadı. Arch'ın rust paketi Cargo sağlıyorsa ayrı cargo paketi varsayılmaz |
| Makedepends/base-devel | cargo sağlayıcısı, cc/gcc, pkgconf, makepkg/pacman, temel araçlar | build.rs cc ve pkg-config çağırıyor. base-devel grup varsayımı Arch ilkelerine göre belirtilir; kendi kendine kurulum yapılmaz |
| Checkdepends | desktop-file-utils; GUI testleri için python, xorg-server-xvfb, xdotool, dbus; Sway için sway | Paket adları doğrulanmalı. Gerçek GNOME testleri clean chroot check() içine zorla sokulmaz |
| Audit/developer araçları | namcap, devtools, diffoscope/repro araçları, binutils | Kullanıcı runtime'ına sırf paket denetimi için eklenmez |

Önce `pacman -Si <aday>` ve `pacman -Fl <paket>` / `pacman -Ql <kurulu-paket>`
ile sahiplik kontrolü yapılmalı. `pkg-config --modversion gtk4 libadwaita-1
libseccomp` ve `pkg-config --exists gnome-desktop-4` API/build ön kontrolleridir.
Loader config'in gerçek Exec hedefi, sahip paketi ve JPEG/PNG MIME kayıtları
incelenmeli; ardından gerçek secure decode yapılmalı. Tüm aday adları birden
kurup başarı varsaymak veya glycin/glycin-loaders ikisini gereksiz yere
zorunlu kılmak doğru değil. Uyumlu loader çalışmadan paket kabul edilmez.

## 4. PKGBUILD ve .SRCINFO tasarımı (çalıştırılabilir teslimat değil)

Aşağıdaki alanlar/akış sonraki uygulama görevinin şartnamesidir; gerçek PKGBUILD
ve .SRCINFO dosyası bu görevde eklenmez. Açık alanlar geçerli makepkg değerleri
gibi sunulmaz.

```text
pkgname=pivra
pkgver=0.1.0
pkgrel=1
arch=(x86_64)                       # ilk kabul hedefi; aarch64 NOT RUN
url=https://github.com/onbeskarakterli/Pivra
license=(GPL-3.0-only)
source=(pivra-0.1.0.tar.gz::<onaylı tam commit URL'si>)
sha256sums=(<gerçek arşiv SHA-256>)  # SKIP yok
makedepends=(<doğrulanmış rust/Cargo ve pkgconf araçları>)
depends=(<yukarıdaki tablodan doğrulanmış runtime listesi>)
checkdepends=(<ayrık build-time doğrulama araçları>)
```

Önerilen fonksiyon sorumlulukları:

1. **prepare():** arşiv köküne geç; Cargo.lock'un beklenen hash'ini kontrol et;
   release için sabit Rust target kaydı; cargo fetch --locked ile önceden indir.
   Source-tree patch, cargo update veya GLYCIN_DATA_DIR SDK yönlendirmesi yok.
2. **build():** makepkg'in toolchain/hardening ortamını koru;
   `cargo build --release --locked --offline --bin pivra` çalıştır.
   `--target` kullanılıyorsa çıktı yolunu target'a göre sabitle.
   Zorunlu GNOME helper derlemesi aşağıdaki bölüme göre ayrı doğrulanır.
3. **check():** `CARGO_INCREMENTAL=0 cargo test --locked --offline`,
   `cargo check --locked --offline`, desktop-file-validate ve gerektiğinde
   lisans metadata kontrolleri. Test başarısızlığı `|| true` ile yutulmaz.
   Clippy/rustfmt yayın kaynak kapılarıdır; makepkg için araç paketlerinin
   mevcut olduğu ayrıca doğrulanır. GNOME/secure decode gate'i ayrı native kabul.
4. **package():** yalnız `$pkgdir` altına install -Dm755/-Dm644; dosya listesi
   aşağıdaki gibi. Kullanıcının HOME, XDG config/cache veya varsayılan MIME
   ilişkilerine erişim yok. package() içinde cargo veya ağ çalıştırılmaz.
5. **.SRCINFO:** tamamlanan PKGBUILD'den `makepkg --printsrcinfo` ile üretilir;
   elle farklı bir depends/source/checksum listesi tutulmaz. AUR gönderimi
   öncesi üretilmiş içerikle karşılaştırılır. Henüz gerçek .SRCINFO yok.

Arch varsayılan repo sağlamlık seçeneklerini gereksiz yere devre dışı
bırakmayın. strip/debug/split-debug ve LTO kararları ölçülmeli; `!strip`,
`!lto` veya native CPU optimizasyonu test olmadan eklenmez. `-C target-cpu=native`
ile taşınamaz artifact üretilmez. İlk hedef x86_64; başka mimariler test edilmeden
arch=('any') veya destek listesine eklenmez.

## 5. Kurulacak dosyalar ve sahiplik

```text
/usr/bin/pivra
/usr/bin/pivra-thumbnail-factory
/usr/share/applications/io.github.onbeskarakterli.Pivra.desktop
/usr/share/icons/hicolor/scalable/apps/io.github.onbeskarakterli.Pivra.svg
/usr/share/icons/hicolor/symbolic/apps/io.github.onbeskarakterli.Pivra-symbolic.svg
/usr/share/licenses/pivra/LICENSE
/usr/share/licenses/pivra/third-party/...         # gerçek upstream metinleri
/usr/share/doc/pivra/PIVRA_V0.1_RELEASE_NOTES.md
/usr/share/doc/pivra/PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md
```

Executable'lar root:root 0755; desktop/icon/lisans/belgeler root:root 0644.
Setuid, capabilities, writable /usr dosyaları ve kullanıcıya ait paket dosyaları
yok. `pivra` PATH'ten çalışır. Uygulama kimliği
`io.github.onbeskarakterli.Pivra` ile ikon/desktop adları tutarlı kalmalı.

Gelecekte `.desktop` dosyasına `Icon=io.github.onbeskarakterli.Pivra` eklenmesi
gerekir; bu görevde eklenmedi. Exec=`pivra %u`, JPEG/PNG MIME listesi ve
DBusActivatable=false korunur; sistemin varsayılan video oynatıcısı yapılmaz.
Paket kurulumunda xdg-mime default, kullanıcı adına gio mime yazımı veya sudo
ile kullanıcı oturumunu başlatma yok. Open With ilişkisi ayrı kabul edilir.

Desktop/icon cache güncellemesi için öncelikle Arch'ın mevcut pacman hook'ları
kullanılmalı; hook sahipliği ve komut kullanılabilirliği gerçek sistemde
kontrol edilmeli. Aynı işlemi iki kere yapan .install betiği eklenmemeli.
AppStream bu aşamanın zorunlu bağımlılığı değil.

## 6. GNOME thumbnail helper: build, keşif ve kaldırma

Mevcut build.rs gerçek cc/API/link testinden geçen tek modülü kullanır; üretim
binary'si sırasıyla environment override, current_exe sibling ve build-time
absolute OUT_DIR fallback'ini arar. İlk yoldaki kullanıcı override'ını paket
küresel olarak ayarlamamalı. Sibling `/usr/bin/pivra-thumbnail-factory` mevcut
üretim koduyla çalışır; `/usr/lib/pivra` veya libexec'e taşıma bugünkü keşif
koduyla otomatik çalışmaz. Bu nedenle mevcut kodu değiştirmeyen tasarım sibling'dir.

**Paket build'i isteğe bağlı helper başarısını zorunlu gate'e çevirmeli:**

- `gnome-desktop-4`, `gio-2.0`, `gobject-2.0`, `gdk-pixbuf-2.0` pkg-config
  cflags/libs ile aynı native/thumbnail_factory.c kaynak dosyasını derle.
  Kaynak/objeden sonra link flags ve kütüphaneler gelsin; GObject'a transitive
  DSO bağlantısı varsayılmasın. Gerçek C API prototipleri ve -Werror kontrolü var.
- Güvenilir paketleme seçeneği: aynı C kaynağını zorunlu ve deterministik
  `$srcdir/build/pivra-thumbnail-factory` çıktısına paket build() içinde derle;
  makepkg CC/CPPFLAGS/CFLAGS/LDFLAGS ortamını koru. Cargo build.rs'nin optional
  kopyası ayrıca var olabilir; paketlenen helper açıkça bu başarılı C build'dir.
  Üretim kodunda değişiklik gerekmez. Önce header/library/sürüm uyumu kontrol edilir.
- Alternatif: Cargo `build-script-executed` JSON kaydındaki **bu build'in**
  pivra out_dir'sini yakalayıp doğrulanmış helper'ı al. Eski target altında glob,
  `find | head`, ilk/son OUT_DIR veya başka sürüm helper'ı seçmek yasak.
- Helper yoksa build/check FAIL; yalnız uygulamanın derlenmesi GNOME generation
  PASS değil. gnome-desktop-4 çalışmıyorsa bağımlılık/API sorunu raporlanır;
  uyumsuz ABI veya “CacheOnly'yi zorla Generate” çözümü yok.
- Kurulu binary ve helper readelf/ldd/namcap ile incelenir. Build/source/SDK
  yolları runtime çözümü olmamalı. Build ağacı kaldırılıp env override'ları
  olmadan, kurulu sibling üzerinden gerçek GNOME generation yapılır.
- native kodun bellek, CPU, çıktı boyutu, timeout, kaynak kimliği ve child cleanup
  sınırları değişmez; GNOME işi başarısızsa sağlayıcı önceliği değiştirilmez.
- Kaldırma pacman'ın bu iki executable'ı ve diğer sahip olduğu /usr dosyalarını
  kaldırmasıdır; helper bir servis değildir, systemd unit/daemon eklenmez.
  uninstall sırasında kullanıcı cache/config temizliği veya süreçleri topluca
  öldüren betik yok. Açık uygulama süreçleri ve kaldırılan paket etkisi ayrı test.

**Tekrar üretilebilirlik riski:** build.rs, PIVRA_BUILT_THUMBNAIL_FACTORY içine
absolute OUT_DIR metnini gömer. Kurulu sibling önceliği runtime'ı çözer fakat
farklı build dizinlerinde binary'nin byte eşitliğini garanti etmez.
Rust remap-path-prefix'in env! ile üretilmiş bu string'i temizlediği varsayılmaz.
İki build karşılaştırması gerekli; sorun doğrulanırsa gelecekte ayrı yetkiyle
minimal build/packaging çözümü tasarlanmalı. Bu görevde kod değiştirilmez.

## 7. İkon şartnamesi — henüz tasarım/asset yok

- Normal ikon: GNOME/HIG ile uyumlu, özgün ve hakları açık SVG master;
  freedesktop hicolor scalable/apps altında uygulama kimliğiyle aynı ad.
  128/256 referans ızgarası ve küçük görünüm okunurluğu değerlendirilir;
  raster gerekiyorsa hicolor 16/24/32/48/64/128/256/512 sınıflarına uygun export.
  HiDPI için SVG veya doğrulanmış uygun ölçekler; haricî kaynak/font bağımlılığı yok.
- Sembolik ikon: `...-symbolic.svg`; GNOME symbolic kurallarına uygun basit
  tek renkli şekil, tema rengiyle boyanabilir. Normal ikonun yalnız siyah-beyaz
  kopyası otomatik kabul edilmez; açık/koyu ve yüksek kontrastta test edilir.
- SVG içinde script, uzak resim, dış entity veya lisansı belirsiz logo yok.
  Özgün kaynak/provenance, lisans ve gerekiyorsa sanatçı bildirimi kayıt edilir.
  GPL-3.0-only proje kararı asset'in sahiplik geçmişini kendiliğinden kanıtlamaz.
- İkon seçimi ve görsel kabul kullanıcıya ait; bu görevde ikon üretilmedi.

## 8. Lisans ve üçüncü taraf dağıtım yükümlülükleri

[B4 denetimi](PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md) korunur. PKGBUILD license
alanı GPL-3.0-only; GPL-3.0-or-later'a çevrilmez. LICENSE pakette tam olarak
korunur. Source archive'ın yanında GPL corresponding source ve gerekli
build/install tariflerine erişim açık olmalı; Cargo.lock tek başına tüm kaynak
ve GPL/LGPL teslim yükümlülüklerinin yerine geçtiği iddia edilmez.

Statik Rust bağımlılıkları için MIT/BSD/ISC/Unicode/Apache metinleri, varsa
NOTICE ve orijinal upstream copyright bildirimleri binary paketinde de
korunmalı. Glycin'in seçilen LGPL seçeneği için kaynak ve yeniden bağlama/
replace edebilme koşulları değerlendirilir; sadece bir LICENSE koymak yetmez.
OS paketlerine depend etmek ile native library/loader binary'sini pakete
kopyalamak ayrı durumlardır. SDK'dan .so veya loader kopyalama yapılmamalı.

Glycin loader'ın IJG kaynaklı bölümü varsa dağıtım belgelerinde gerekli
Independent JPEG Group bildirimi korunur. Varsayılan source/loader sürümleri
B4'ten farklıysa yeni sürüm/paketin copyright envanteri yeniden incelenir.
İkonlar ayrıca hak/provenance denetiminden geçer. B4 kaynak lisansı kapanışı,
gelecekteki her bağımlılık paketi veya binary artifact'ı önceden onaylamaz.

## 9. Gerçek Arch kabul planı — tamamı bu görevde NOT RUN

Bu komutlar hazırlık/test planıdır; bu görevde root kurulumu, makepkg veya
kullanıcı dosyalarına işlem yapılmadı. Özel VM/test hesabı ve yalnız geçici
fixture kopyaları kullanın. Sistem paket yönetimi için root yalnız pacman/
chroot kurulumunda; cargo/makepkg/probe normal kullanıcı olarak çalışmalı.

| Adım | Uygulama ve kabul kanıtı | Bu görev |
|---|---|---|
| 1. Temiz bağımlılıklar | Güncel Arch clean chroot/VM; gerçek pacman -Si/-Fl sonuçları, paket sürümleri ve user namespace/seccomp ön kontrolleri kaydedilir. Bwrap testi kontrol edilen dosya/mount izolasyonunu kanıtlar; başarısızsa güvenlik ayarı gevşetilmez | NOT RUN |
| 2. Kaynak ve build | makepkg --verifysource; normal kullanıcıyla makepkg --cleanbuild --syncdeps --check veya devtools clean-chroot eşdeğeri; gerçek SHA, lock hash, .BUILDINFO ve build log saklanır | NOT RUN |
| 3. Artifact | makepkg --packagelist; pacman -Qip/-Qlp; bsdtar ile uid/gid/mode, .PKGINFO/.BUILDINFO/.MTREE ve dosya listesi; namcap/readelf/ldd incelemesi. Beklenmeyen RPATH/build-path, HOME dosyası veya setuid yok | NOT RUN |
| 4. Kurulum/menü | VM'de pacman -U gerçek artifact; pacman -Qkk pivra; command -v pivra; pivra --version; desktop-file-validate kurulu desktop; menüden normal GNOME oturumunda açılış ve kapanış | NOT RUN |
| 5. Gerçek medya | Kurulu /usr/bin/pivra ile özel JPEG/PNG, Unicode path/URI, EXIF1–8; Fit/Fill/100%, zoom/pan, filtre, resize, iki pencere, bozuk/silinmiş dosyadan sağlam medyaya geçiş; piksel/geometry ve exit kanıtı | NOT RUN |
| 6. Sandbox | Kurulu gerçek 2+ loader config/Exec ve paket sahipliği; Bwrap child + başarılı gerçek JPEG/PNG pixels; aktif iptal ve kaynak temizliği. Sadece PID/loader keşfi veya sürecin başlaması PASS değil | NOT RUN |
| 7. Thumbnail | GNOME oturumunda gerçek helper generation, cache decode ve tıklama/scroll/pin/resize; bağımsız pencereler. Ayrı izole non-GNOME oturumunda fallback seçimi ve gerçek FD pixels. Dosya hatası provider switch değil | NOT RUN |
| 8. Nautilus | Kullanıcı varsayılanlarını değiştirmeden Open With Pivra, tek JPEG/PNG, Unicode/URI ve pencere/başlık; gerekiyorsa varsayılanlık yalnız test hesabında açık kullanıcı seçimiyle; video varsayılanı yok | NOT RUN |
| 9. Kaldırma/upgrade | Özel hesapta önce/sonra checksum/sahiplik envanteriyle pacman -R pivra; yalnız paket dosyaları gider; kaynak fotoğraflar, XDG_CONFIG_HOME/pivra ve XDG_CACHE_HOME/thumbnails değişmeden kalır. Yeniden kurulum ve upgrade ayrıca test | NOT RUN |
| 10. Repro | Aynı kaynak/toolchain ile iki ayrı temiz build; artifact farkları diffoscope ile incelenir; checksum/bit eşitliği ölçülmeden tekrar üretilebilirlik PASS verilmez | NOT RUN |

Kaldırma anında uygulama kapalı olmalı; böylece uygulamanın kendi yasal config
kaydı/thumbnail işi ile pacman'ın kaldırma etkisi karıştırılmaz. `pacman -Rns`
kullanıp başka uygulamalara gerekli runtime paketlerini kaldırmak bu gate'in
varsayılanı değildir; paylaşılan cache için rm -rf/özel temizleme hook'u yok.

### Mevcut probe'ların rolü ve kurulu binary testi farkı

- `tests/run_preview_pipeline_probe.sh`: kaynak ağacında gerçek JPEG/PNG/EXIF,
  view/cache/cancellation birleşik gate; kurulu binary'yi kendiliğinden seçmez.
- `tests/run_thumbnail_system_probe.sh`: GNOME helper, retention ve native
  katalog/piksel kabulü; private cache/config kullanır. X11 koşulunda başlangıç
  imleci test başlamadan `(1279,1023)` konumuna taşınmalı, gerçek koordinat
  ve tüm test pencerelerinin dışında kaldığı doğrulanmalı; physical GNOME ayrı
  gözlenmeli. X11 bağlantısı açık tutulmalı; boş Xvfb bağlantısının kapanıp
  ekranı/imleci sıfırlaması önlenmeli.
- `tests/run_fd06_freedesktop_probe.sh`: private XDG dizinleri ve süreçle sınırlı
  test desktop snapshot'ı ile FD seçimi; sonra gerçek secure thumbnail ve aktif
  kapanış gözlemcisi. Bu test üretimde GNOME önceliğini değiştirmez. Aktif
  loader-pool gözleminde exit77/inconclusive PASS'a çevrilmez.
- `tests/run_thumbnail_provider_probe.sh`: capability/cache metadata; tek başına
  gerçek thumbnail üretimi veya pixels PASS değil.
- `tests/desktop_smoke.py`: kaynak desktop/GIO akışı; gerçek Nautilus kurulum
  kabulünün yerine geçmez. --expect-image-error decode başarısı değildir.
- `tests/run_active_decode_close.sh` ve FD-05A native probe'ları: yalnız gerekli
  release kapsamı/ön koşullarıyla ayrı secure aktif-iş ve bellek kontrolleri.

Bu betiklerin çoğu dev branch'i ve target/debug ağacını varsayıyor. Release
arşivinde .git olmayabilir; makepkg check() bunları “dev dalını taklit ederek”
çalıştırmamalı. Mevcut source-tree probe kabulü ile kurulu paketin /usr/bin
manuel/native kabulü ayrı tutulur. Kurulu artifact'a otomatik probe gerekiyorsa
ayrı yetkiyle harness adaptörü hazırlanır; bu görev test dosyalarını değiştirmez.

## 10. Güvenlik ve sağlayıcı davranışı korunacak

SandboxSelector::Bwrap ve expose_base_dir(false), mevcut dosya/boyut/piksel
sınırları, cancellation ve cleanup değişmez. NotSandboxed, root'ta decoder,
host fallback, namespace/seccomp/sysctl veya izin gevşetmesi yok. Loader'ı
kurmak veya terminalde çalıştırmak güvenli decode PASS değildir.

GNOME mevcut/etkin/kullanılabilir olduğunda GNOME seçimi korunur; seçimin
kaynağı tek dosyanın corrupt/missing sonucu değil oturum yeteneğidir. Çalışan
GNOME'u kapatıp FD üretimine zorlayarak ana paket kabulü verilemez. Fallback
ayrı test oturumunda kanıtlanır. Cache Stale/Invalid için onaylı güvenli
üretim/bellek davranışı ve shared-cache RENAME_NOREPLACE politikası değişmez.

Paket kurulum/kaldırma adımları fotoğraf/config/cache silmez, kullanıcı
varsayılanlarını değiştirmez; testler private kopyalarda çalışır. Bubblewrap'ın
paket yöneticisi tarafından sağlanan dosya modu ve güvenlik modeli korunur;
pivra/helper'a setuid veya capabilities eklenmez.

## 11. Açık kararlar ve B5 kapanış şartı

1. Onaylı release commit/tag/arşiv, gerçek checksum ve sürümleme yetkisi.
2. Arch runtime paket adları, glycin 2+ loader sahibi ve tek GNOME API paketi.
3. Onaylı normal/sembolik ikon ve hakları; .desktop Icon alanı.
4. Uygulanmış PKGBUILD/.SRCINFO, helper install ve tam üçüncü taraf bildirimleri.
5. Kurulu helper'ın build ağacı olmadan bulunması; absolute OUT_DIR kaynaklı
   tekrar üretilebilirlik farklarının ölçümü ve gerekiyorsa ayrı çözüm kararı.
6. Gerçek Arch clean build/install/remove/upgrade, GNOME/secure decode,
   provider fallback ve Nautilus kabulü; test sonuçları ve artifact SHA'ları.

**NO-GO:** B5 bu belgeyle kapanmaz. Sonraki uygun aşama yalnız ayrıca
yetkilendirilecek paketleme uygulaması/ikon kararı ve gerçek Arch kabulüdür.
B1/B2/B3 kontrollü regresyon ve B4 kaynak lisansı sonuçları korunur; bunlar
kurulu Arch paketi PASS'ı değildir. `main`, tag, release ve AUR değişmez.
