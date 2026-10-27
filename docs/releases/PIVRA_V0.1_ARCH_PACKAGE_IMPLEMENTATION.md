# Pivra 0.1.0 — Arch paketleme taslağı uygulaması

Tarih: 28.10.2026 (Europe/Istanbul). Kaynak/ikon baseline:
`07ef59795bb154a57e419d8efff1008a9610e326` (`dev`).

**B5: NO-GO / AÇIK.** Paketleme kodu ve kontrollü doğrulaması hazırlandı;
gerçek Arch makepkg/build/install/remove kabulü yapılmadı. Onaylı main release
commit'i, tag ve arşiv checksum'u henüz yok. Bu bir yayıma hazır AUR paketi,
oluşturulmuş binary paket veya gerçekleştirilmiş kurulum değildir.

Başvurular: [paketleme planı](PIVRA_V0.1_ARCH_PACKAGING_PLAN.md),
[bağımlılık denetimi](PIVRA_V0.1_ARCH_DEPENDENCIES.md),
[lisans denetimi](PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md).

## 1. Eklenen dosyalar

| Dosya | İşlev |
|---|---|
| packaging/arch/PKGBUILD | pivra 0.1.0-1 / x86_64 / GPL-3.0-only taslağı; zorunlu helper, locked/offline release build ve stage kurulumu |
| packaging/arch/release-source.conf.example | Bilinçli boş onaylı commit/URL/SHA-256 alanları; gerçek release-source.conf oluşturulmadı |
| packaging/arch/collect-licenses.py | Standard-library-only, offline, checksum doğrulayan tam crate kaynak/bildirim toplayıcısı |
| packaging/arch/README.md | Kaynak onayı, bağımlılık belirsizlikleri, kurulum/lisans ve sonraki kabul adımları |
| Bu belge | Gerçek test sonuçları ve açık engeller |

Üretim Rust kodu, mevcut test/probe'lar, Cargo.toml/Cargo.lock, ikon SVG'leri ve
.desktop bu görevde değiştirilmedi. Yeni servise, decoder'a, sandbox fallback'ine,
config/cache temizleme veya kullanıcı MIME varsayılanlarını değiştirme koduna
başvurulmadı. main/tag/release/AUR üzerinde işlem yapılmadı.

## 2. İkon ve desktop doğrulaması

Mevcut iki SVG XML olarak parse edildi. SVG namespace/viewBox doğrulandı;
script, foreignObject veya dış image öğesi yok. Dosya adları ve hicolor yolları
uygulama kimliğiyle eşleşiyor:

```text
data/icons/hicolor/scalable/apps/io.github.onbeskarakterli.Pivra.svg
data/icons/hicolor/symbolic/apps/io.github.onbeskarakterli.Pivra-symbolic.svg
Icon=io.github.onbeskarakterli.Pivra
```

Bu görevde /tmp altında bağımsız bir GTK C render aracı derlendi. Gerçek
GtkIconTheme hicolor lookup, GtkIconPaintable, GtkSymbolicPaintable ve GSK Cairo
renderer kullanıldı. SVG kaynakları yalnız geçici theme ağacına kopyalandı;
sistem hicolor index.theme ile çözüldü. Lookup'ın fallback ikon değil gerçekten
bu dosyaları kullandığı ve symbolic bayrağı ayrı doğrulandı.

**20 render PASS:** 16, 32, 48, 128, 256 px × normal/sembolik × açık/koyu.
Sembolik siyah SVG, GTK symbolic snapshot yolu üzerinden açık temada koyu,
koyu temada açık foreground ile çizildi. Kaynak rengini/geometrisini değiştirme
yok; dosyaya beyaz renk yazarak temaya uyuyormuş gibi gösterilmedi.

Render contact sheet görsel olarak incelendi: 32–256 px biçimler seçilebilir;
16 px normal ikon çizgileri ince, koyu zeminde koyu dış sınır düşük kontrastlı.
16 px sembolik siluet daha okunur; küçük boyutta ayrıntı kaybı bir görsel risk,
ölçülmüş XML/render başarısızlığı değil. İkonun kullanıcı görsel/hak kabulü ve
fiziksel GNOME Shell menü/HiDPI görünümü **NOT RUN**. Açık/koyu sonuçları
kontrollü GTK foreground/background doğrulamasıdır, tüm Adwaita/GNOME oturumunun
otomatik kabulü değildir. Mevcut SVG'lerin checksum'ları değişmedi:

```text
normal   88a317c859023e00604112f04b9d5f85d2a1fa240c8b0e613405fee4a92628ae
symbolic 0d39394d47c034eaea2b7fc1420f0f40109bfd309bd4af951417b87e940cf2c3
```

`desktop-file-validate data/io.github.onbeskarakterli.Pivra.desktop`: **PASS**.
Desktop Exec=pivra %u, JPEG/PNG ve DBusActivatable=false kapsamı korunuyor.

## 3. Kaynak bütünlüğü: taslak bilerek çalışmayı reddediyor

PKGBUILD, gerçek `release-source.conf` yoksa **exit 1** ile durur:

```text
BLOCKED: approved release-source.conf is missing; this is not a publishable AUR recipe.
```

Bu fail-closed davranış test edildi. makepkg source/metadata/build akışı kaynak
onayı olmadan başlamamalı. Örnekte release_commit, release_url ve release_sha256
boş; sahte digest veya SKIP yok. Gerçek dosya daha sonra onaylı tam 40 hex
commit, o commit'in tam GitHub archive URL'si ve indirilen gerçek arşivin 64 hex
SHA-256'sını içermeli. Hareketli dev veya yalnız tag adı kaynak değil.

Yerel collect-licenses.py için PKGBUILD'deki SHA-256 gerçek dosyadan hesaplandı
ve yeniden doğrulandı; bu, eksik release checksum'u yerine geçen bir değer değil.
Kaynak arşivi/Cargo.lock/LICENSE ve tag-to-commit ilişkisi sonraki onayda ayrıca
incelenmeli. makepkg yok ve kaynak onayı eksik olduğundan gerçek `.SRCINFO`
üretilmedi. Elle geçersiz .SRCINFO veya yayıma hazırmış gibi bir şablon eklenmedi;
sonraki tam tariften `makepkg --printsrcinfo` ile üretilmesi gerekiyor.

## 4. Build/check sözleşmesi ve bağımlılık durumu

- pkgname=pivra, pkgver=0.1.0, pkgrel=1, arch=x86_64, GPL-3.0-only.
- CARGO_HOME ve CARGO_TARGET_DIR source staging alanında; kullanıcının mevcut
  ~/.cargo, ~/.config veya ~/.cache dizinlerine kurulum yazımı yok.
- prepare(): cargo fetch --locked. Tüm kilitli grafik alınır; sadece aktif Linux
  crate'leri değil yabancı target/build/dev bildirimleri de korunur.
- build(): GTK>=4.16, libadwaita>=1.2 ve GNOME4/GIO/GObject/GdkPixbuf/lcms2/
  libseccomp pkg-config ön koşulları. Aynı native/thumbnail_factory.c kaynağı
  gerçek GNOME4 API'ye zorunlu C derlenir; CFLAGS/CPPFLAGS/LDFLAGS/CC korunur.
  Kaynak/objeden sonra açık GObject dahil link bayrakları kullanılır.
- Cargo build --release --locked --offline --target x86_64-unknown-linux-gnu.
  Oluşan release ELF'te lcms2/seccomp shared NEEDED sözleşmesi kontrol edilir;
  LCMS override ile sessiz static alternatif kabul edilmez.
- check(): CARGO_INCREMENTAL=0 cargo test --locked --offline, cargo check
  --locked --offline ve desktop-file-validate. Her komutun başarısızlığı taşınır;
  test/assertion kaldırma veya || true yok. check() gerçek GNOME gate'i değildir.
- Python>=3.11 lisans toplayıcısı için makedepends; rust>=1.85, pkgconf build
  araçları. CC/binutils ve temel araçlar Arch base-devel ön koşuludur.
  desktop-file-utils checkdepends. GUI araçları kullanıcı runtime'ına eklenmedi.

Depends listesi önceki belgeye göre **aday Arch adları** taşır: gtk4,
libadwaita, glib2, gdk-pixbuf2, gnome-desktop-4, glycin, bubblewrap, libseccomp,
lcms2, fontconfig, cairo, graphene, glibc, gcc-libs, hicolor-icon-theme.
Özellikle gerçek gnome-desktop-4 API ve 2+ JPEG/PNG loader sahibi paketler
pacman -Si/-Fl/-Ql/-Qo ile henüz doğrulanmadı. Glycin Rust crate sürümü 3.0.8
ile OS paket sürümünü eşitleme yok. Bu adayların tarifte bulunması sahiplik veya
kurulabilirlik PASS değildir; önceki B5.1 PARTIAL durumu devam eder.

## 5. Dosya yerleşimi ve kaynak/bildirim teslimi

package() yalnız pkgdir altına kurar:

```text
/usr/bin/pivra                                      0755
/usr/bin/pivra-thumbnail-factory                    0755
/usr/share/applications/io.github.onbeskarakterli.Pivra.desktop 0644
/usr/share/icons/hicolor/scalable/apps/io.github.onbeskarakterli.Pivra.svg
/usr/share/icons/hicolor/symbolic/apps/io.github.onbeskarakterli.Pivra-symbolic.svg
/usr/share/licenses/pivra/LICENSE
/usr/share/licenses/pivra/third-party/<crate>/...
/usr/share/licenses/pivra/third-party/inventory.json
/usr/share/doc/pivra/PIVRA_V0.1_RELEASE_NOTES.md
/usr/share/doc/pivra/PIVRA_V0.1_DEPENDENCY_LICENSE_AUDIT.md
/usr/share/doc/pivra/corresponding-source/crates/*.crate
/usr/share/doc/pivra/corresponding-source/pivra-0.1.0.tar.gz
/usr/share/doc/pivra/corresponding-source/{PKGBUILD,release-source.conf,collect-licenses.py}
```

Veri dosyaları 0644; gerçek paket root sahipliği fakeroot/makepkg/pacman
kabulünde ayrıca ölçülür. Kontrollü stage testi host kullanıcısıyla çalıştı;
root:root package ownership PASS iddiası yok. Mevcut runtime sibling keşfi
helper'ın /usr/bin'de bulunmasını sağlar; environment override veya libexec
workaround kurulmaz. Kurulu binary ile helper generation **NOT RUN**.

Collector gerçek Cargo metadata + unchanged lockfile üzerinden her `.crate`
arşivini SHA-256 ile doğrular, named LICENSE/COPYING/NOTICE/AUTHORS/readme ve
manifestleri toplar. Inline bildirimleri kaybetmemek için tam kaynak arşivleri
de pakete konur. 184 arşiv checksum ile doğrulandı; toplam **18,910,822 bayt**
(ek paketleme sıkıştırmasından önce). Bu muhafazakâr kaynak teslimi paket boyutunu
artırır; dış native loader/SDK binary'leri kopyalanmaz. Pivra arşivi ve gerçek
tarif/config/collector da teslim edilir; source/relinking/inventory incelemesi
son artifact üzerinde tekrar gerekir. Bu çalışma her gelecek loader sürümünü
veya paketleyicinin kaynak/notice eksiltmesini önceden onaylamaz.

Collector bir mevcut crate çıktı dizininin üstüne yazmaz; kabul prosedürü temiz
build ağacıdır. Bozuk archive hash'i negatif testte reddedildi. Archive notice
çıkarımı absolute/.. yollarını ve büyük notice girdilerini reddeder; linkleri
takip etmez. Komple source arşivi değiştirilmeden kopyalanır.

Kaldırma için özel .install/servis/hook eklenmedi. Pacman yalnız paket sahipli
/usr dosyalarını kaldırmalı; kullanıcı fotoğrafları, config/cache/shared
thumbnail dosyaları ve MIME varsayılanları korunmalı. Arch sistem desktop/icon
cache hook'ları ve gerçek remove/upgrade kabulü henüz yok.

## 6. Gerçek doğrulama sonuçları

| Kontrol | Sonuç ve sınır |
|---|---|
| SVG/XML namespace/viewBox ve aktif/dış içerik kontrolü | PASS — iki SVG |
| GTK hicolor lookup + symbolic ve GSK render | PASS — 20 gerçek render, X11/Xvfb |
| 5 boyutta contact sheet görsel inceleme | Tamamlandı; 16 px normal ikon ince/düşük sınır kontrastı riski, kullanıcı GNOME kabulü açık |
| desktop-file-validate | PASS, exit 0 |
| bash -n PKGBUILD ve source-config örneği | PASS |
| Eksik kaynak-onay gate'i | PASS — beklenen BLOCKED/exit 1, build çalışmadı |
| ShellCheck | NOT RUN — araç yok, kurulum yapılmadı |
| Collector / 184 kaynak ve bildirim envanteri | PASS — gerçek checksum'lar; source digest eşleşmesi |
| Collector bozuk arşiv negatif testi | PASS — checksum mismatch reddedildi |
| package() kontrollü dosya yolu/mode staging | PASS — /tmp içinde; gerçek makepkg artifact değil |
| cargo fmt --check | PASS |
| CARGO_INCREMENTAL=0 cargo test --locked | PASS — 186 test, sıfır başarısız |
| cargo check --locked | PASS — mevcut load_stats kullanılmıyor uyarısıyla |
| Gerçek makepkg/.SRCINFO/Arch paket sahipliği | NOT RUN/BLOCKED — araçlar yok, kaynak onayı yok |
| Native GNOME4 helper C build/link | BLOCKED — mevcut SDK'da API/header paketi yok; önceki denetimdeki koşul değişmedi |
| Gerçek Arch kurulum/kaldırma/upgrade, Nautilus | NOT RUN |
| Gerçek GNOME / secure Glycin pixels / provider fallback | NOT RUN — bu görev GUI decode kabulü değil |

Kontrollü package() testi, kaynak onay gate'ini gerçek release varmış gibi
geçmedi: yalnız package fonksiyon gövdesi /tmp fixture ortamında çalıştırıldı.
Uygulama dosyası için mevcut debug binary, helper/arşiv için açıkça stage-only
dosya kopyalama fixture'ları kullanıldı. Bunlar helper başarısı, gerçek source
checksum'u veya release build olarak sayılmadı. Kurulum yolları, modları ve
ikonun birebir kopyalandığı kontrol edildi; HOME/etc yazımı olmadı.

## 7. Açık engeller ve sonraki adım

1. Proje sahibinin onaylı main release commit'i/tag/arşivi ve gerçek checksum;
   kaynak config tamamlanmadan taslak bilinçli olarak paket üretemez.
2. Arch paket/provider sahipliği; clean build ve mandatory GNOME helper.
3. Gerçek .SRCINFO üretimi ve paket bağımlılık/artifact incelemesi.
4. build.rs'nin gömdüğü absolute OUT_DIR fallback'i için iki temiz build'de
   bit karşılaştırması. Sibling kurulumu runtime keşfini çözer, byte eşitliğini
   garanti etmez; bu görev build.rs değiştirmedi.
5. Native GNOME/secure decode/provider kabulü; kurulu artifact ile menü/Nautilus.
6. Gerçek pacman install/remove/upgrade, dosya sahipliği ve kullanıcı verilerinin
   korunması. Üçüncü taraf kaynak/notice ve ikon hakları son artifact'ta incelenmeli.

**B5 kapanmadı.** Bu taslaklar yalnız dev'e commit/push edilecek; AUR yayını,
main/tag/release veya yetkisiz native güvenlik ayarı değişikliği yapılmayacak.
