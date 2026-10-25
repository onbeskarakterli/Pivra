# Faz 2 kapanışı ve Pivra 0.1.0 teknik hazırlığı

25.10.2026 · Europe/Istanbul · yalnız `dev`.

**Karar: PIV-18 TESTING; Faz 2 henüz kapanmaya hazır değil; 0.1.0 yayın onayı yok.**
Full-only fotoğraf yolu ve fullscreen kabul edilmiş olsa da gerçek erken preview
kapsamı uygulanmadı. Ana-fotoğraf yükleme göstergesi de güncel davranışta yok.
Bunların ertelenmesi veya kabul ölçütünün değiştirilmesi kullanıcı kararıdır.
Bu denetim hiçbir özelliği kendiliğinden0.1.0 kapsamından çıkarmadı.

Denetlenen üretim/test kaynak snapshot'ı: `0d13d24449047e3d23a0e15133c47e2ca44ca86b`.
Kullanıcı kabul/status belge commit'i: `2ca8b7a895ec4a8289b3f3276c16b8f88e0c173f`.
Bu çalışma üretim kodunu, bağımlılıkları veya test assertion'larını değiştirmedi.
Aşağıdaki Codex testleri o kaynak snapshot'ına aittir; belge değişiklikleri
tekrar native decode yapılmış gibi sunulmaz.

## Native son test takibi — 25.10.2026

Bu rapordan sonra kullanıcı final native komutunda `view_modes_probe.rs:268`
actual/fit FAIL bildirdi. Secure baseline, path/URI ve preload PASS kalır;
bütün native entegrasyon paketi henüz PASS değildir. Aşağıdaki önceki Codex
matrisi tarihsel snapshot'tır, native paketin tümünü doğruladığı anlamına gelmez.

`ffe9ab5cb2a11af7e2825214d7a3a07993b735f9`: testin eski Fit-reset varsayımı ve
retained-frame ready kontrolü düzeltildi. Actual/Fill korunması önceki kabul
edilmiş üretim sözleşmesidir; manual yeni frame kabulünde Fit'e döner. Aynı
actual/fit hatası kontrollü X11'de yeniden üretildi; yeni seçim/presentation
bariyeri X11/Sway'de PASS.88 Rust+20 örnek unit ve ilgili mode/zoom/filter/
async/cache/iki pencere regresyonları PASS, gerçek secure native tekrar
BLOCKED/NOT RUN. Üretim kodu ve güvenlik değiştirilmedi.

[Native FAIL ve düzeltmenin tam kaydı](PIV18_VALIDATION.md). Aynı native komut
`bash tests/run_preview_pipeline_probe.sh` yeniden çalıştırılmalı. Bu düzeltme
PIV-18 veya Faz2 kapanışı değildir; kapsam kararları ve diğer kapılar değişmez.

## 1. Tamamlanan özellikler

PIV-01–17 DONE. TH-01/02/04/05/06/07 DONE; TH-03 bağımsız kabulü TESTING.
PIV-19 ve sonrası başlamadı. PIV-13–16'nın üst takip tablosundaki eski TESTING
etiketleri, kendi validation dosyalarında zaten var olan native/manuel DONE
kayıtlarıyla eşitlendi; eski test kayıtları silinmedi.

TH-07 için kullanıcı 25.10.2026 tarihinde gerçek Arch Linux/GNOME Wayland native
ve fiziksel klavye/fare kabulünü açıkça PASS bildirdi: secure JPEG/PNG/EXIF,
F/Esc/double-click, Fit/Fill/Actual/zoom/pan/resampling, thumbnail/klavye gezinmesi,
pencere izolasyonu ve HiDPI scale2; manuel kullanım sıkıntısız. Ayrı yeni ham log
ve native testin tam SHA'sı verilmedi; bu kullanıcı kanıtıdır, Codex native koşusu
veya clean-install kanıtı değildir. [TH-07 kabulü](TH07_FULLSCREEN_VALIDATION.md).

| PIV-18 kapsam denetimi | Sonuç / kanıt sınırı |
| --- | --- |
| 1. JPEG/PNG secure decode | Kullanıcı native PASS; `src/image_loader.rs` Bwrap zorunlu, içerik JPEG/PNG kontrolü ve EXIF dönüşümü. Bu Codex koşusunda gerçek decode yeniden çalıştırılmadı. |
| 2. Hızlı geçiş / eski istek iptali | PIV-16 native kabulü + Codex kontrollü ters sonuç/A-B-A/1000 pending/stale callback/close PASS. Yeni secure decoder interval ölçümü yok. |
| 3. Gereksiz ana-texture reload olmaması | PIV-17/TH-07 kullanıcı kabulü; cache hit/paintable identity ve fullscreen load sayacı Codex PASS. Metadata değişimindeki geçerli reload bundan farklıdır. |
| 4. Komşu preload | PIV-17 native kabulü; next-before-previous, foreground admission/promotion ve iptal Codex PASS. |
| 5. Sınırlı cache / kaynak temizliği | Pencere başına photo cache128 MiB/3 entry/64 MiB entry; thumbnail64 texture/16 MiB hesap sınırı. LRU/pressure/worker join/weak cleanup kontrollü PASS. Sayaçlar toplam RSS/VRAM değildir. |
| 6. Büyük görsel ve modlar | Native kabul +48 GSK mode snapshot, zoom/pan/resampling Codex PASS. Input cap32 MP/64 MiB file/32768 edge; tüm cap boyutlarında FPS/latency/RSS kanıtı yok. Actual = kaynak piksel/GTK mantıksal birimi. |
| 7. Thumbnail/main seçim | TH-04/05/06/07 native kabulü;81/120 retention,120 katalog, scroll/click/selection/iki pencere Codex PASS. |
| 8. Fullscreen texture/transform | TH-07 kullanıcı native/fiziksel PASS; Codex gerçek compositor + paintable/tree/zoom-pan piksel sürekliliği PASS. |
| 9. Hata/silme/close/izolasyon | Önceki PIV-12/16 native kanıtı ve güncel kontrollü GTK/GIO testleri PASS. Kontrollü job gerçek secure child cleanup diye sayılmaz. |
| 10. Yükleme durumu göstergesi | **KAPSAM AÇIĞI.** FullLoading model state'i var; foreground status etiketi yükleme başında boş/gizli. Seçili dosya başlıkta, eski fotoğraf beklerken korunur; hata dosya adıyla açıklanır. Aktif ana-fotoğraf busy/spinner/ilerleme göstergesi yok. |

Madde10 `src/image_view.rs::load_request` ile doğrulandı: status boşaltılıp
hidden yapılır. `GtkSpinner` yalnız thumbnail hücrelerinde vardır. Başlık ve
hata mesajı, aktif foreground yükleme göstergesiyle aynı şey değildir. TH-06'da
kaldırılan Loading yazısı bu görevde geri getirilmedi. Kullanıcı mevcut sessiz
retention davranışını açıkça kabul edebilir veya ayrı minimal gösterge isteyebilir;
bu karar verilmeden maddeyi PASS diye kapatmak doğru değildir.

## 2. PASS / FAIL / BLOCKED / NOT RUN test matrisi

Mevcut ortam korundu: Rust/Cargo 1.99.1, GTK 4.18.6, libadwaita 1.7.6;
Cargo client glycin 3.0.8. Native Arch ortamının sürümleri bunlara eşit varsayılmaz.
Aşağıdaki sıralamayla önce Rust, Python, ardından görüntü/thumbnail/fullscreen
regresyonları çalıştırıldı. Yeni paket/decoder kurulmadı.

| Kontrol | Codex bu denetimde | Veri / açıklama |
| --- | --- | --- |
| cargo fmt --check | PASS | Çıkış0 |
| cargo check --locked --offline -j2 | PASS | Mevcut stable SDK |
| cargo build --locked --offline -j2 | PASS | Debug main binary |
| cargo clippy --locked --offline --all-targets -j2 -- -D warnings | PASS | Warnings denied |
| cargo test --locked --offline -j2 | PASS | **88 test:40 lib+24 main+24 integration;0 failed** |
| cargo test --locked --offline --all-targets -j2 | PASS | **21 test harness,274 test icrası**;88 ana test ve örneklerde tekrar derlenen ortak kaynak testlerini içerir,274 benzersiz senaryo değildir |
| Python üst dizin discovery | PASS | **33** observer/native-report unit testi; gerçek secure decode değildir |
| Python thumbnail discovery | PASS | **29** GIO/cache/C sidecar limit/helper/module-selection testi; GNOME API/compiler stubları native factory başarısı değildir |
| Fixture generate.py --check, images/generate.py --check | PASS | Deterministik hash/bütünlük;18 image asset için decode iddiası yok |
| cargo build --locked --offline --examples -j2 | PASS | Mevcut tüm probe'lar |
| PIV-13 view_modes_probe --synthetic | PASS | **48** gerçek GSK snapshot; sentetik texture |
| PIV-14/15 --synthetic --zoom-pan / --resampling | PASS | Gerçek controller/GSK gradient, clipping ve filtre pikselleri; sentetik decode sınırı |
| PIV-16 async_cancel_probe --controlled | PASS | Latest pixel, pending/worker bounds, reentrancy, retained pixels, error recovery/close |
| PIV-17/18 preload_probe --controlled | PASS | Hit/miss, LRU/pressure/source identity, priority/promotion, full-only Phase |
| window_failure_race_probe / navigation_race_probe | PASS | Gerçek GTK/GIO + kontrollü error worker; monitor/reload/delete/late result/survivor |
| Thumbnail --retention-regression | PASS |81/120, visible/selected korunması,64/16 MiB, rehydrate/resize/hide/iki pencere |
| Thumbnail --catalog-regression | PASS |120 tekil başarı,0 iptal; stable scroll/cells/source/cache/production windows |
| Legacy thumbnail overlay/visibility | PASS | Kontrol/timer/resize/pick/alpha/mode/zoom-pan |
| Ayrı-süreç Enabled false→true→false reload | PASS | Özel XDG_CONFIG_HOME; yeni native kabul değildir |
| Fullscreen Xvfb+mevcut Xfwm4 + --x11-input | PASS | Gerçek WM/GDK state; XTest keyboard/photo/thumbnail/toggle/header routing |
| Fullscreen Sway headless koyu/output scale2 | PASS | Gerçek Wayland compositor/monitor allocation, scale_factor2; kontrollü fotoğraf pikselleri |
| desktop-file-validate source desktop entry | PASS | Kaynak doğrulaması, kurulum/Nautilus testi değildir |
| Yeni native secure JPEG/PNG/EXIF/full-only/aktif-close paketi | NOT RUN | Aynı kaynak için önceki Codex secure gate exit77/Bwrap ortam engeli kayıtları korunur; tekrar denenmedi |
| Gerçek GNOME factory generation bu Codex SDK'sında | NOT RUN | Önceki helper-development-API yok/exit77 kaydı; yeni native generation iddiası yok |
| Gerçek erken preview / preview→full | UNSUPPORTED / NOT PROVEN | Gerçek preview provider yok; model geçişleri ürün özelliği değildir |
| Yeni clean native install / installed AUR package | NOT RUN | Ortam sıfırlanmadı, paket kurulmadı |
| DESKTOP-06 gerçek Nautilus | NOT RUN | CLI/GIO/unit/desktop syntax yerine geçmez |
| Güncel optimized release build / source archive install | NOT RUN | Debug build ilk yayın paketi kanıtı değildir |
| Resmî Arch paket JSON metadata kontrolü | BLOCKED | Proxy403; sürüm/paket adları kesinleşmiş gibi sunulmaz |

**Bu denetimde çalıştırılıp başarısız olan test yok.** Kapsam açıkları ve NOT RUN
kontroller PASS değildir. Önceki native FAIL/exit77 kayıtları korunur; kullanıcı
kabulü onları geriye dönük değiştirmez. Toplam62 Python testi ve12 GUI probe
çağrısı çalıştırıldı; ayrı-süreç preference kontrolleri bunlara eklenir. Grafik
pikselleri kontrollüdür, gerçek GTK/GSK ve gerçek compositor kullanılması gerçek
Glycin decode veya fiziksel GNOME girişi sayılmaz. Kullanıcının native PASS
beyanları ayrı kabul sütunu/validation kayıtlarında tutulur.

Çalıştırma kayıtları: `/tmp/phase2-core.log`, `phase2-python.log`,
`phase2-python-thumbnails.log`, `phase2-fixtures.log`, `phase2-images.log`,
`phase2-thumbnails.log`, `phase2-persistence.log`, `phase2-full-x11.log`,
`phase2-full-sway.log`, `phase2-all-targets.log`. Bunlar mevcut çalışma ortamında geçici loglardır;
repo/rapor onları kalıcı dış kanıt deposu olarak varsaymaz.

## 3. Erken preview kapsam boşluğu

Üretim akışı FullLoading→FullReady/Failed'dır. PreviewLoading/PreviewReady
model variant'ları, bir preview frame'in gerçekten sunulduğu anlamına gelmez.

[PIV-18B kanıtı](PIV18_PREVIEW_FEASIBILITY.md): kullanıcı native Glycin3.0.8
`scale` denemesinde PNG1600×1200→16×12 isteğine1600×1200; JPEG64×32→16×8 isteğine
64×32 aldı. Aynı Image'da ikinci still-frame NoMoreFrames verdi. Mevcut incelenen
JPEG/PNG loader hattında reduced decode veya embedded preview kanıtı yok.
Bütün gelecekteki Glycin sürümleri için imkânsızlık iddiası yapılmaz.

PIV-18C embedded JPEG/libjpeg deneyi kullanıcı tarafından reddedilip kaldırıldı.
Beş prototip dosyasının yokluğu bu denetimde kontrol edildi. Onlar geri
getirilmedi; Pillow/özel decoder, NotSandboxed veya alternatif host decode yolu
eklenmedi. Mevcut şeridin **tam secure decode sonrası** küçük RGBA8 texture'ı
ve GNOME cache thumbnail'ı, ana büyük görüntü için erken preview sayılmaz.

Eksik kalanlar: full decode bitmeden kullanılabilir sertifikalı küçük frame,
preview→full sunum/transform sürekliliği, preview var/full hata UI yolu ve iki
kalite cache promotion. Full-only native PASS bunları tamamlamaz.0.1.0'a yetişen
onaylı güvenli yol yoksa ertelenmesi için kullanıcı kapsam onayı gerekir;
**bu rapor erteleme kararı değildir**.

## 4. Günlük kullanım açısından kritik hatalar

Bu denetimde yeni tekrar üretilebilir crash, siyah ara kare, boşalan visible
thumbnail veya stale-callback görüntü ezmesi görülmedi; testlerin sınırı içindeki
kontroller PASS. Bu ifade her dosya/donanımda hatasızlık garantisi değildir.

Somut UX açığı: kodda ilk full frame hazır olana kadar paintable yok; yavaş ilk
yüklemede boş photo alanı görülebilir. Geçişte yeni başlık altında önceki fotoğraf bekler. Kullanıcıya özel aktif yükleme
işareti yoktur. Kontrollü hata geldiğinde dosya adı ve önceki fotoğraf açıklanır;
yükleniyor ile tamamlandı ayrımı ise bu arada açık değildir. Madde10 için tasarım/
kabul kararı gerekir, bu görevde UI değiştirilmedi.

Bilinen sınırlar: yalnız JPEG/PNG gerçek render; video katalogda gezinir ama
oynatılmaz, TIFF/RAW genel desteği yok.32 MP,64 MiB input ve32768 edge sınırları
kontrollü hata verir. Hesaplanan cache bütçeleri bütün süreç RSS/VRAM sınırı
olarak sunulmaz; yeni FPS, latency veya secure decoder temporal-overlap ölçümü yok.

İlk kurulumun source-only başarıyla aynı olmadığına dair somut paketleme riski:
yalnız `pivra` ikilisini kopyalamak factory helper'ı kurmaz. Build-tree helper
silinirse missing-cache generation çalışmayabilir. Bu henüz yayımlanmış paket
hatası değildir; aşağıdaki paketleme kabulüyle önlenmesi gereken eksiktir.

## 5. Kullanıcının çalıştırması gereken son native test komutları

Final kaynak snapshot'ında bütün full-only entegrasyon/active-close için:

```bash
bash tests/run_preview_pipeline_probe.sh
```

Bu komut gerçek secure baseline/main application open/preload/mode/zoom/filter/
async/partial memfd active-close aşamalarını yürütür; ilk başarısızlıkta durur.
`/tmp/pivra-preview-pipeline.json` yalnız **PASS full-only fallback** diyebilir;
erken preview'yi PASS yapmaz. Önceki kullanıcı native kanıtları geçerlidir;
final birleşik paketin bu kaynak snapshot'ında yeni sonucu ayrıca beklenir.

TH-03'ün dar kalan native restart/kalıcılık kabulü için:

```bash
bash tests/run_thumbnail_visibility_probe.sh
```

Manuel oturumda Enabled'ı değiştir, iki pencereyi kapat, aynı komutu yeniden açıp
başlangıç değerini doğrula; T/hover/auto-hide/resize listesini de açıkça onayla.
Otomatik prefix özel config kullanır, manuel bölüm normal thumbnail tercihini
okuyup değiştirebilir. TH-07 native/fiziksel kabulü zaten verildi; source kodu
bu denetimde değişmediğinden fullscreen komutunu sırf tekrar olsun diye istemiyoruz.

**Ayrı ilk-yayın kapıları:** temiz Arch/GNOME makine veya snapshot'ta native
bağımlılıklardan build/test/window/secure loader ve helper kurulumunu yeniden
üret; ardından kurulu desktop entry ile gerçek Nautilus JPEG/PNG açılışı,
Unicode/boşluklu yol, bağımsız pencereler ve video MIME varsayılanının korunmasını
kontrol et. Bunlar PIV-18'in içine eklenmiş görevler değildir. Mevcut GTK/GIO
ve temporary-desktop smoke testi DESKTOP-06 yerine geçmez.

## 6. Faz 2 kapanışı için gereken kararlar

| Kapı / karar | Şu an | Kapanış için gereken |
| --- | --- | --- |
| Gerçek erken preview | Yok; PIV-18 kapsam açığı | Onaylı güvenli provider ile gerçekleştirme veya **kullanıcının açık0.1 full-only kapsam onayı / sonraki sürüme erteleme kararı** |
| Foreground yükleme göstergesi | Bilerek gizli; madde10 tam karşılanmıyor | Ayrı onaylı minimal gösterge veya mevcut başlık+retention davranışının açık kabulü; eski Loading yazısını kendiliğinden geri getirme yok |
| TH-03 bağımsız native kabul | Kısmi birleşik UI kanıtı; restart açık | Bölüm5'teki restart/kalıcılık ve görünürlük kabulü |
| Final native full-only regression | Codex'te NOT RUN | Mevcut birleşik komutun gerçek native sonucu; early-preview yerine geçmez |

Bunlar netleşmeden PIV-18 TESTING kalır ve Faz 2 kapatılmaz. Güvenli alternative
uydurulmaz, PIV-19'a geçilmez.0.1 için zaten sonraki kapsamlarda kalan video/
geniş format/Edit özellikleri uygulanmış gibi pazarlanmaz; gerçek erken preview
ise henüz **onaylanmış ertelenmiş özellik** değildir. Faz 0/DESKTOP-06 ayrı ilk
yayın kapılarıdır; bu tabloda PIV-18 eksik özelliği olarak gösterilmez.

## 7. Pivra 0.1.0 ve AUR `pivra` paketi ön koşulları

**Strateji sabit:** `dev` geliştirme/test; `main` yalnız kullanıcı onaylı kararlı
kod. Tek AUR paketi **pivra**, kaynağı main üzerindeki sürüm etiketi. `pivra-git`
yok. Yayınlandıktan sonra kurulum `yay -S pivra`, güncelleme `yay -Syu`.
Yeni dev commit'leri otomatik yayımlanmaz: her kararlı sürümde tag, pkgver/pkgrel
ve source checksum bakımının yapılması gerekir.

Bu görev merge, tag, GitHub Release, PKGBUILD veya AUR yayını yapmadı. `git
ls-remote --tags origin` denetiminde tag listesi boştu. Cargo.toml'daki0.1.0
ürün sürüm değeri yayın kanıtı değildir; `publish=false` ise yalnız crates.io
publish'i kapatır, AUR paketlemesini engellemez.

### Bağımlılıklar / sürümler

| Katman | Kaynakla doğrulanan gereksinim | Paketleme durumu |
| --- | --- | --- |
| Build | Rust/Cargo, C compiler (`cc`), pkg-config/pkgconf; GTK/Adwaita ve helper API header'ları | Arch `rust`/uygun rustup + base-devel/pkgconf adayları; resmî güncel paket çözümü clean Arch'ta doğrulanmalı |
| Test-only | Python3 ve otomatik GUI kontrolleri istenirse Xvfb/Xauth, Sway veya Xfwm, XTest/xdotool | checkdepends adları clean Arch'ta doğrulanmalı; Python/Pillow ürün görüntüleme bağımlılığı değildir. Manuel pencere bekleyen native scriptler makepkg check()/post-install içine konmamalı |
| Rust | edition2024, declared rust-version1.85, locked client glycin=3.0.8 | Linux-filtered Cargo metadata'da en yüksek ilan edilmiş dependency MSRV1.85; **gerçek1.85 build yapılmadı**, stable1.99.1 PASS |
| GTK/UI | Cargo features GTK>=4.16, libadwaita>=1.2 | Codex4.18.6/1.7.6 ve önceki kullanıcı Arch ortamı; minimum kombinasyon/native paket testi NOT RUN |
| Secure loader | `bwrap` executable (dağıtım paketi adayı bubblewrap), izinli user namespace/kernel/proc; kurulu2+ JPEG/PNG loader ve keşif config'i | Rust client3.0.8, dağıtım loader sürümünün3.0.8 olmasını gerektirmez; protokol uyumu gerekir. Native kullanıcı PASS, clean install açık |
| Doğrudan ELF NEEDED | liblcms2, libseccomp, libfontconfig, libadwaita, libgtk4, libgdk_pixbuf, libgraphene, GIO/GObject/GLib, libc/libgcc | `readelf -d target/debug/pivra` ile görüldü; distro paketleri çoğunu transitif sağlar; nihai runtime depends temiz ortamda çıkarılmalı |
| GNOME factory | Tek uyumlu gnome-desktop-4 veya fallback3.0 API, GIO/GObject/GdkPixbuf; native helper | pkg-config modül adı Arch paket adıyla karıştırılmamalı. Mevcut kullanıcı gnome-desktop-4 helper PASS; bu SDK'da helper yok/cache-only |
| MIME / tema | JPEG/PNG content sniffing için sistem MIME verisi ve GNOME/GTK ikon/tema altyapısı | shared-mime-info/hicolor/desktop-file-utils ve üst UI paketlerinin dependency/hook ilişkileri doğrulanmalı |

Resmî Arch JSON uçlarına gtk4/libadwaita/glycin/glycin-loaders/gnome-desktop
paket sorguları proxy403 nedeniyle okunamadı. Aday isimlerin hepsinin var olduğu
veya doğrudan depends olması gerektiği varsayılmadı. Paketleme sırasında yerel
`pacman -Si`, kurulu loader/helper için `pacman -Qo` ve gerçek pkg-config API
kontrolleriyle provider eşlemesi yapılmalı. Bu ağ engeli Pivra'nın Arch'ta
çalışmadığını göstermez; kullanıcı native başarısı ayrıdır. GTK'nin transitif
medya bağımlılığı Pivra'da video motoru uygulanmış anlamına gelmez.

### Kurulu dosya yerleşimi ve somut eksikler

| Dosya / iş | Mevcut durum / hedef |
| --- | --- |
| `/usr/bin/pivra` | Release binary, `cargo build --release --locked` ve clean staging/install testi henüz yapılmadı |
| `/usr/bin/pivra-thumbnail-factory` | Mevcut kodun current_exe sibling keşfiyle uyumlu kurulum; helper doğru aktif OUT_DIR'den alınmalı. Yalnız build-tree absolute fallback'e güvenilmez. Libexec tercih edilirse keşif uyarlaması ve testi ayrıca gerekir |
| `/usr/share/applications/io.github.onbeskarakterli.Pivra.desktop` | Kaynak var, desktop-file-validate ve Rust desktop testleri PASS; kurulu gerçek Nautilus kabulü NOT RUN |
| `/usr/share/icons/hicolor/.../apps/io.github.onbeskarakterli.Pivra.*` | Repo'da uygulama ikonu yok; desktop dosyasında Icon alanı da yok. GNOME ürün/masaüstü hazırlığı eksiği, derleme/çökme hatası değildir |
| MIME/hook | Mevcut Exec=`pivra %u`, yalnız image/jpeg + image/png, DBusActivatable=false. Yeni MIME XML gerekmez; desktop/icon cache hook'ları doğrulanmalı. Video varsayılanı veya kullanıcı varsayılanları otomatik değiştirilmez |
| Lisans / sahip kararı | LICENSE/COPYING ve Cargo license metadata yok; proje sahibi yayın/lisans kararını netleştirmeli. Bağımlılık lisansı proje lisansı yerine konmaz; bu görev lisans seçmedi |
| AppStream metainfo | Yok; GNOME Software bütünleşmesi için sonraki metadata işi, AUR build/runtime için zorunluymuş gibi sunulmaz |
| Release source / paket | PKGBUILD/install recipe yok, onaylı main etiketi ve immutable checksum yok; clean makepkg/install/remove/upgrade testi yapılmadı |

Helper mevcut build.rs'de **optional**: yalnız Cargo build PASS olması GNOME
üretim yeteneği PASS değildir. Tam özellikli paket helper API/link build'ini ve
kurulu sibling'den runtime keşfini ayrıca doğrulamalı; build dizini kaldırıldıktan
sonra missing-cache generation tekrar çalışmalıdır. Paket mimarisi listesi de
kanıta dayanmalı; bu denetimin derleme/metadata hedefi x86_64 Linux'tur.

**Sonuç:**0.1.0 bugün yayımlanmaya hazır değildir. Engeller: PIV-18 kapsam/UX
kararları ve TH-03/final native kabulü; bunlardan ayrı Faz 0/DESKTOP-06; lisans,
helper/ikon/desktop ve clean release paketleme hazırlığı; onaylı main sürüm
etiketi/paket artefaktı. Yeni kanıtlanmış bir crash veya secure decode kod hatası
bu denetimde bulunmadı. İkon/metainfo eksikliği runtime crash diye, Codex sandbox
kısıtı ise genel Arch uyumsuzluğu diye raporlanmaz. Önce açık kararlar ve ayrı
yayın kapıları tamamlanmalı; hiçbir native kurulum, merge veya yayın yapılmadı.
