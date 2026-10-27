# Pivra 0.1.0 — Arch bağımlılıkları ve GNOME helper doğrulaması

Tarih: 27.10.2026 (Europe/Istanbul). İncelenen `dev` commit'i:
`148bba8b2bdf33e716a480070a359a79b085b6bd`.
Başvuru: [Arch paketleme planı](PIVRA_V0.1_ARCH_PACKAGING_PLAN.md).

**B5.1: PARTIAL. B5: NO-GO / AÇIK.** Yerel native bağlantılar, loader dosyaları
ve kurulum sözleşmesi incelendi. Gerçek Arch paket sahipliği ile GNOME4 helper
başarılı derleme/link/runtime kabulü doğrulanamadı. Bu belge uygulanmış
PKGBUILD, .SRCINFO, kurulmuş paket veya yayımlanmış release değildir.

## 1. Ortam ve kanıt sınırları

- Ortam Debian GNU/Linux 13; `pacman` yok. Dolayısıyla `pacman -Si`, `-Fl`,
  `-Ql` sonuçları **NOT RUN**; dpkg çıktısı Arch sahipliği yerine kullanılmadı.
- Arch paket JSON uçları (`gtk4`, `gnome-desktop-4`) ve repo `extra.db` /
  `extra.files` verileri denenerek erişim araştırıldı. `archlinux.org`,
  `geo.mirror.pkgbuild.com` ve `mirror.rackspace.com` istekleri
  `Tunnel connection failed: 403 Forbidden` verdi. Paket listesi indirilmedi;
  bugünkü Arch paket sürüm/ad/split ilişkileri **BLOCKED / doğrulanmadı**.
- Mevcut özel SDK etkinleştirildi; yeni sistem paketi kurulmadı, ayarlar veya
  sandbox değiştirilmedi. pkg-config kanıtı SDK/host'a aittir, Arch'a değil.
- Rust/Cargo 1.99.1, pkg-config 1.8.1, Bubblewrap 0.12.0 gözlendi.
  Arch'ın rust/Cargo sağlayıcısı veya Bubblewrap sürümü bunlardan çıkarılmadı.

Denenen veri kaynakları:

```text
https://archlinux.org/packages/extra/x86_64/gtk4/json/
https://archlinux.org/packages/extra/x86_64/gnome-desktop-4/json/
https://geo.mirror.pkgbuild.com/extra/os/x86_64/extra.db
https://geo.mirror.pkgbuild.com/extra/os/x86_64/extra.files
https://mirror.rackspace.com/archlinux/extra/os/x86_64/extra.db
```

## 2. Bağımlılık sınıflandırması

**Arch adı sütunu adaydır, Arch paket sahipliği PASS değildir.** Rol/sürüm
sözleşmesi kaynak ve yerel ELF üzerinden doğrulanabildiği ölçüde kesinleştirildi.
Bir paket runtime'da gerekiyorsa build sırasında da bulunması makedepends'e
ikinci kez eklenmesini gerektirmez. GNOME helper'lı tam ürün bu tablonun hedefidir.

| Bileşen / aday Arch paketi | Sınıf | Yerel kanıt | Kesinleşmeyen Arch koşulu |
|---|---|---|---|
| gtk4 >=4.16 | depends | Cargo v4_16; pkg-config gtk4=4.18.6; ELF libgtk-4.so.1 | x86_64 paket sürümü, sahip dosyalar |
| libadwaita >=1.2 | depends | Cargo v1_2; pkg-config libadwaita-1=1.7.6; ELF libadwaita-1.so.0 | Paket/sürüm ve GTK bağımlılık ilişkisi |
| glib2 | depends | glib/gio/gobject pkg-config 2.84.4; ELF üç native .so | Arch glib2 dosya sahipliği |
| gdk-pixbuf2 | depends | pkg-config 2.42.12; ELF libgdk_pixbuf-2.0.so.0; helper doğrudan GdkPixbuf kullanıyor | Paket dosyaları |
| gnome-desktop-4 API sağlayan paket (aday gnome-desktop-4) | depends | build.rs ilk aday modül; native C gerçek header/API kullanıyor | Hangi paket .pc/header/DSO sağlıyor ve API/link testi; yerelde modül yok |
| JPEG/PNG 2+ loader sağlayan paket (aday glycin veya glycin-loaders) | depends | Rust glycin config kodu 2+/conf.d arıyor; SDK'da manual build config ve executable var | Arch'taki gerçek paket adı, config/Exec sahipliği ve runtime kabulü |
| bubblewrap | depends | glycin sandbox kodu bwrap çalıştırıyor; sistemde 0.12.0 | Arch sağlayıcısı/dosya modu ve gerçek namespace/seccomp çalışması |
| libseccomp | depends | pkg-config 2.6.0; ELF libseccomp.so.2 | Arch soname/paket sahipliği |
| lcms2 | depends (mevcut shared build için) | pkg-config 2.16; ELF liblcms2.so.2; lcms2-sys dynamic feature etkin | Arch release build'in shared kalması ve doğru sağlayıcı |
| fontconfig | depends adayı, doğrudan ELF bağlantısı | libfontconfig.so.1 NEEDED | Arch dosya sahibi/doğrudan bağımlılık denetimi |
| cairo | depends adayı, doğrudan ELF bağlantısı | libcairo.so.2 NEEDED | Arch sağlayıcısı |
| graphene | depends adayı, doğrudan ELF bağlantısı | libgraphene-1.0.so.0 NEEDED | Arch sağlayıcısı |
| glibc | depends adayı | libc.so.6, libm.so.6, ld-linux-x86-64.so.2 | Arch dosya sahipliği; bunlar ayrı libm/loader paketleri varsayılmaz |
| gcc-libs | depends adayı | libgcc_s.so.1 NEEDED | Arch libgcc sağlayıcı/split kararı |
| rust >=1.85 ve Cargo sağlayıcısı | makedepends | Manifest rust-version 1.85, edition2024; yerel 1.99.1 | Minimum sürümde gerçek build ve Arch rust paketinin cargo sağlaması |
| pkgconf (pkg-config komutunu sağlayan paket) | makedepends | build.rs pkg-config çağırıyor; yerelde komut sürümü 1.8.1 | Arch provider'ın adı/dosyaları |
| cc/gcc, temel build araçları | build ortamı/base-devel | build.rs ve bağımsız helper C compiler çağırıyor | Arch base-devel sözleşmesi; makepkg içinde kurulum yapılmaz |
| desktop-file-utils | checkdepends; kurulu sistem hook sağlayıcılığı ayrıca incelenecek | Desktop metadata kontrol aracı; executable'ın kendi runtime gereksinimi değil | Arch desktop cache hook/paket sahipliği; hook'un sistemde bulunması |
| python, xorg-server-xvfb, xdotool, dbus, sway | checkdepends / native test araçları | Mevcut probe/smoke testleri bunları kullanıyor | Paket adları ve native oturum; kullanıcı runtime'ına topluca eklenmez |
| hicolor-icon-theme | depends adayı, ikon entegrasyonu | Standart icon theme hierarchy planı | İkon henüz yok; gerçek dosya/hook bağımlılığı ayrı doğrulanacak |
| GTK/GLib *-dev Debian adları, rustup, ayrı uydurulmuş cargo paketi | gereksiz/yanlış varsayım | Arch bağımlılık kanıtı değiller | Debian -dev adları kopyalanmaz; rustup build sırasında çalıştırılmaz |
| Yeni decoder, Qt/KDE, sürekli servis | gereksiz / kapsam dışı | Mevcut Glycin/GNOME yolları var | Eklenmeyecek |

`gsettings-desktop-schemas`, thumbnailer kayıtları, icon/cache hook'ları ve
native transitive kütüphaneler, gerçek Arch paket grafiği görülmeden rastgele
flatten edilmemeli. GNOME generation için kurulu sistemde JPEG/PNG'yi kabul eden
thumbnailer kaydı ve gerçek Exec yolu da doğrulanmalı; yalnız helper veya yalnız
libgnome-desktop bulunması generation PASS değil. Glycin thumbnailer paketi /
`/usr/share/thumbnailers` dosyalarının hangi pakette olduğu burada doğrulanmadı.

### lcms2 için yeni somut kanıt

`cargo tree --locked --offline -e features -i lcms2-sys` çıktısında lcms2-sys
4.0.7 için `default`, `dynamic`, `parallel`, `static-fallback` etkin. Crate build
kodu pkg-config başarısızsa static-fallback'a izin veriyor. İncelenen Pivra ELF'i
`liblcms2.so.2` doğrudan NEEDED kaydediyor; bu build'de lcms2 gereksiz değil.
Arch recipe'si pkg-config lcms2 ön kontrolünü zorunlu tutmalı; silent static
fallback sonucu “aynı dağıtım/lisans/link sözleşmesi” kabul edilmemeli. Gerçek
release ELF'inin bağımlılıkları yeniden ölçülmeli; debug ELF'inden Arch artifact
veya bit eşitliği sonucu çıkarılamaz.

## 3. Glycin runtime dosyaları ve sahiplik

Rust crate **glycin 3.0.8**, SDK loader **glycin-image-rs 2.0.0** ve kullanıcı
geçmişindeki Arch Glycin 2.2.3 farklı sürüm alanlarıdır. Birbirine eşitlenmedi.
İstemci `glycin-loaders/{COMPAT_VERSION}+/conf.d/*.conf` altında kayıt arıyor;
bu istemci ailesinin sözleşmesi **2+**. Paket adı/sürümü tek başına kanıt değil.

Yerelde gerçekten okunan dosya:

```text
/workspace/pivra-environment/piv09/loader-prefix/share/glycin-loaders/2+/conf.d/pivra-jpeg-png.conf
SHA256 bd0f6241c80bd5929d32adb098f746421463dc9a22a2c2eef7981baf79fd543c

[loader:image/jpeg]
Exec=/workspace/pivra-environment/piv09/loader-prefix/libexec/glycin-image-rs

[loader:image/png]
Exec=/workspace/pivra-environment/piv09/loader-prefix/libexec/glycin-image-rs
```

Config GLib key-file/INI biçiminde MIME başına loader grubu ve Exec içeriyor;
2+ dizini uyumluluk ailesini belirliyor. Bu manual SDK config dosyası bir Arch
paketinden çıkarılmış değildir; pakete kopyalanması önerilmiyor.

Gerçek Exec dosyası mevcut: ELF x86-64, mode 0755, SHA-256
`9b93831143bfe83e9a9d87aea4c2d94dab4f657bb0c853a60517c295172a416b`.
SDK loader-build.json kaydı upstream GNOME tag 2.0.0, commit
`724f18c5f3519885995428a7e2e6e5df9ca2ab27`, JPEG/PNG kayıtları gösteriyor.
Dosya varlığı/konfigürasyon kontrolü **PASS (SDK kapsamı)**; bu görevde gerçek
istemci keşif probe'u, JPEG/PNG pixel decode ve aktif cancellation **NOT RUN**.

**Arch config/Exec paket sahipliği: BLOCKED.** Gerçek sistemde her ikisi için
`pacman -Qo /gerçek/dosya` ve sahip paket için `pacman -Ql <paket>` çıktısı
saklanmalı; repo `pacman -Fl` listesiyle de karşılaştırılmalı. Sadece dizin adı,
SDK yerel build kaydı veya Debian paket adı, Arch sahipliği sayılmaz.

Runtime gate: environment SDK yönlendirmeleri olmadan sistem XDG data yollarında
uyumlu kayıt, gerçek executable, bwrap, libseccomp, gereken namespace/mount/proc
izinleri ve başarılı sandbox'lı piksel çözümleme birlikte kanıtlanmalı.
`SandboxSelector::Bwrap` ve `use_expose_base_dir(false)` mevcut üretim kodunda
korunuyor. NotSandboxed, host fallback, root'ta decoder veya sysctl/seccomp
gevşetmesi önerilmiyor. Codex'in daha önce kaydedilmiş namespace/proc engeli
normal Arch uyumsuzluğu diye raporlanmıyor; bu görevde yeniden decode denenmedi.

## 4. GNOME4 bağımsız C derleme sonucu

Mevcut SDK pkg-config sonuçları:

```text
gtk4             4.18.6
libadwaita-1     1.7.6
glib/gio/gobject 2.84.4
gdk-pixbuf-2.0   2.42.12
libseccomp       2.6.0
lcms2            2.16
gnome-desktop-4  bulunamadı
gnome-desktop-3.0 bulunamadı
```

`pkg-config --cflags --libs gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0`
**exit 1** verdi. Mevcut GLib/GIO/GObject/GdkPixbuf bayraklarıyla, gerçek native
kaynağa bağımsız `cc -O2 -Wall -Wextra -Werror ...` denemesi yapıldı; hiçbir
header/API stub'ı oluşturulmadı. Sonuç **exit 1**:

```text
native/thumbnail_factory.c:5:10: fatal error: libgnome-desktop/gnome-desktop-thumbnail.h: No such file or directory
    5 | #include <libgnome-desktop/gnome-desktop-thumbnail.h>
      |          ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
compilation terminated.
```

Helper binary'si oluşmadı. **GNOME4 API/link kabulü BLOCKED (eksik ön koşul)**;
bu sonuç kaynak API'sinin uyumsuz olduğunun kanıtı değil. Link aşamasına
ulaşılmadığı için DSO/GObject link başarısı veya factory runtime PASS yok.
Başarılı Cargo build'i bu durumu değiştirmez: build.rs optional helper derlemesi
başarısız olsa da cache-only uyarısıyla uygulama derlemesine izin verir.

## 5. Kesin helper build ve kurulum sözleşmesi

Kaynaklar: `build.rs`, `native/thumbnail_factory.c`,
`src/thumbnail.rs::generation_helper`. Mevcut runtime arama sırası:

1. PIVRA_THUMBNAIL_GENERATOR environment override;
2. current_exe ile aynı dizinde `pivra-thumbnail-factory`;
3. varsa gömülü PIVRA_BUILT_THUMBNAIL_FACTORY absolute OUT_DIR yolu.

**Kurulum hedefi `/usr/bin/pivra-thumbnail-factory`, uygulama `/usr/bin/pivra`.**
Paket küresel override yazmaz. Libexec'e taşıyıp mevcut kodun bulmasını beklemek
geçersiz; böyle bir değişiklik bu görevde yetkili değil.

Sonraki paketleme uygulaması için tercih: build() aşamasında helper'ı aynı
native C kaynağından, zorunlu GNOME4 pkg-config ön kontrolüyle deterministik bir
staging çıktısına ayrıca derle. Cargo'nun optional OUT_DIR glob'unu seçme.
Aşağıdaki bağımsız komut ancak gerçek Arch ön koşulları sağlandığında uygulanır:

```bash
pkg-config --exists gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0
# Başarısızsa paket build'ini durdur; cache-only ile geçme.
# makepkg CC/CPPFLAGS/CFLAGS/LDFLAGS korunmalı. Kütüphaneler kaynak/objeden sonra:
cc -O2 -Wall -Wextra -Werror native/thumbnail_factory.c \
  -o "$builddir/pivra-thumbnail-factory" \
  $(pkg-config --cflags --libs gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0)
```

Bu bir PKGBUILD değildir; örnek komut makepkg hardening bayraklarının yerine
geçmez ve bu ortamda başarılı çalışmış gibi sunulmaz. Paket recipe'si makepkg
compiler/linker bayraklarını koruyarak uygular; shell bayrak bölme kuralları
yol/argümanları bozmamalı. GObject bağımlılığı açıkça linklenir, yalnız DSO'nun
transitive bağımlılığına güvenilmez. Seçilen header ve DSO aynı gerçek API
sağlayıcısından gelmeli; uyumsuz ABI'yi bastıran cast/patch yok.

Paket check gate'i helper'ın mevcut/executable olduğunu, ELF native bağlantılarını
ve kurulu sibling keşfini doğrulamalı. Build ağacı kaldırıldıktan ve environment
override'ları temizlendikten sonra gerçek GNOME eligible/ineligible/failed API
sonucu ve actual missing-cache generation ayrı kanıtlanmalı. Sadece helper'ın
başlaması veya CacheOnly/Missing sonucu generation PASS değil. CPU/bellek/output,
source identity ve timeout/child cleanup sınırları korunur.

## 6. OUT_DIR ve tekrar üretilebilirlik

build.rs helper başarıyla üretildiğinde mutlak OUT_DIR yolunu rustc-env üzerinden
binary'ye gömer; src/thumbnail.rs bunu option_env! ile fallback olarak kullanır.
Farklı build kökleri aynı kaynak için farklı literal string üretebilir.
`--remap-path-prefix` debug path'lerini düzenlese de bu environment literal'ını
yeniden yazdığı varsayılmaz. Sibling kurulumu runtime keşfini çözer; binary byte
eşitliği problemini kendiliğinden çözmez.

Yerel SDK'da GNOME helper bulunmadığından build.rs boş fallback üretir. Bu
görevde iki başarılı GNOME4 build'i arasında byte karşılaştırması yapılamadı;
**tekrar üretilebilirlik NOT RUN**. Kaynak düzeyinde path bağımlılığı gösterildi,
ölçülmüş release binary farkı iddia edilmiyor. Sabit build path veya ayrı
minimal build-script çözümü gerekiyorsa sonraki görevin açık kararı olmalı;
üretim/build kodu burada değiştirilmedi.

## 7. Dosya yerleşimi, kaldırma ve lisans

| Yol | Sözleşme |
|---|---|
| /usr/bin/pivra | CLI, 0755, paket sahibi root |
| /usr/bin/pivra-thumbnail-factory | Zorunlu gerçek API'li helper, 0755, sibling keşfi |
| /usr/share/applications/io.github.onbeskarakterli.Pivra.desktop | Exec=pivra %u; yalnız JPEG/PNG; Icon alanı gelecekte gerçek asset ile tamamlanmalı |
| /usr/share/icons/hicolor/scalable/apps/io.github.onbeskarakterli.Pivra.svg | Henüz olmayan normal ikon; bu görevde üretilmedi |
| /usr/share/icons/hicolor/symbolic/apps/io.github.onbeskarakterli.Pivra-symbolic.svg | Henüz olmayan sembolik ikon |
| /usr/share/licenses/pivra/LICENSE ve third-party/ | Tam GPL-3.0 metni ve gerçek upstream bildirimleri |
| /usr/share/doc/pivra/ | Yayın notları, kaynak/lisans envanteri |

Dosyalar yalnız package staging alanında root:root, executable 0755 ve veri
0644 olarak hazırlanır. Setuid/capabilities yok. Pacman yalnız kendi /usr
belgelerini/binary'lerini kaldırmalı; uninstall hook'u kaynak fotoğrafları,
XDG_CONFIG_HOME/pivra veya XDG_CACHE_HOME/thumbnails dosyalarını silmez. Kullanıcı
varsayılan MIME ilişkileri ve diğer uygulamaların ortak cache'i korunur.
Desktop/icon cache için gerçek Arch sistem hook'ları doğrulanmalı; kullanıcı
ayarına yazan servis veya temizleme betiği eklenmez.

[B4 lisans denetimi](PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md) geçerli:
GPL-3.0-only, bağımlılıkların MIT/BSD/ISC/Apache/Unicode/IJG bildirimleri ve
Glycin LGPL kaynak/relinking koşulları korunmalı. Kopyalanan native loader/DSO
varsa onun gerçek sürümünün kaynak/bildirimleri ayrıca gerekir; SDK binary'sini
pakete kopyalamak önerilmez. AUR source-build recipe'si ve Cargo.lock tek başına
her binary conveyance yükümlülüğünü yerine getirmiş sayılmaz. İkonların
hak/provenance kabulü de henüz yok.

## 8. Sonraki uygulama için zorunlu Arch kanıt paketi

1. Gerçek Arch x86_64 VM/chroot'ta pacman -Si/-Fl ve kurulu pacman -Ql/-Qo
   çıktıları; yukarıdaki adayların gerçek provider/split/sürüm listesi.
2. JPEG/PNG config ve Exec dosyalarının sahipliği; override'sız sistem keşfi,
   uyumlu 2+ protokol ve gerçek secure JPEG/PNG pixel decode.
3. GNOME4 .pc/header/DSO sahipliği; bağımsız helper compile+link PASS;
   build ağacı olmadan kurulu /usr/bin sibling üzerinden gerçek factory sonucu.
4. Sistem thumbnailer kayıtları/Exec ve gerçek GNOME generation; provider
   önceliği korunarak ayrı non-GNOME oturumunda gerçek Freedesktop fallback.
   Cache hit veya tek dosya hatası provider'ın bozulduğu anlamına gelmez.
5. Arch release ELF NEEDED/RPATH/build-path incelemesi; lcms2 shared bağımlılığı,
   hardening ve third-party source/notices teslimi.
6. İki temiz build/repro ölçümü, install/upgrade/remove ve kullanıcı verileri
   checksum koruması; gerçek Nautilus/menü kabulü. Bunlar bu görevde NOT RUN.

Bu kanıtlar olmadan tam doğrulanmış runtime depends dizisi veya çalışır
PKGBUILD/.SRCINFO üretildiği söylenemez. B5.1 **PARTIAL**: yerel link/runtime
sözleşmesi kanıtlı, Arch paket sahipliği ve GNOME4 derleme gate'i **BLOCKED**.
B5 **NO-GO** kalır. Bu görevde Rust/GUI regresyonları yeniden çalıştırılmadı;
önceki PASS sonuçları yeni test diye sunulmadı. Yalnız bu belge değiştirildi;
Cargo.toml/Cargo.lock, üretim/test kodu, güvenlik politikası, main/tag/release ve
kullanıcı cache/config dosyaları değiştirilmedi.
