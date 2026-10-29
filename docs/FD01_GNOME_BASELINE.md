# FD-01 — GNOME thumbnail regresyon referansı

Tarih: 25.10.2026. Dal: `dev`. İncelenen GitHub başlangıcı:
`763d5d5e3b576b8e034b4b396ee15582562e3ecf`.
Yerel temiz dal, 11 remote commit için yalnız fast-forward ile güncellendi.
Test korumaları commit'i: `2b5eea76b2920c9a091a2883e8ce08063180b1fc`.

**FD-01 denetimi, test ve belge işi tamamlandı; bu oturumdaki native kabul NOT RUN.**
Üretim kaynakları, build.rs, Cargo.toml/lock ve mevcut başarı koşulları değiştirilmedi.
Önceki kullanıcı native kabul kayıtları bu oturumda bağımsız çalıştırılmış testler
olarak sunulmaz. FD-02–FD-09 başlatılmadı.

## Kullanıcının Freedesktop şartnamesi — tasarım sınırı

Kullanıcının bu görev sırasında paylaştığı şartname esas alınır:

- İki bağımsız provider: mevcut GNOME ve gelecekte Rust içinde Freedesktop.
  Ayrı Internal Provider yok. Öncelik kesinlikle **GNOME → Freedesktop**.
- Seçim oturum/uygulama düzeyindedir. GNOME mevcut, etkin ve çalışabilirken
  Freedesktop çalışmayacak. Tek dosyanın bozukluğu, eligibility veya cache miss
  provider arızası kabul edilmeyecek.
- Gelecekte Freedesktop: XDG_CACHE_HOME/thumbnails; normal/large/x-large/xx-large;
  standartlaştırılmış kaynak URI'sinin MD5 adı; PNG Thumb::URI, Thumb::MTime,
  uygun Thumb::Size; geçerlilik ve kimlik denetimleri; güvenli izinler/geçici
  dosya/atomik taşıma. Başka uygulamaların ortak cache dosyalarına keyfî müdahale yok.
- Üretim mevcut güvenli Glycin ile; yeni JPEG/PNG decoder, Qt/KDE veya kalıcı
  servis yok. EXIF, oran, kaynak bütünlüğü, iptal, RAM politikası korunacak;
  thumbnail işleri foreground açılışı engellemeyecek.
- Modüler testler, eşzamanlı iş tekilleştirme, hata/izin/yarış yönetimi korunacak.
  Yeni bağımlılıkların bakım, lisans ve paketleme etkileri ayrıca incelenecek.
  Viewer ve şerit yeniden yazılmayacak.
- Sıra: FD-01 referans; FD-02 ortak arayüz; FD-03 cache okuma; FD-04 Glycin
  üretim adaptörü; FD-05 güvenli yazma; FD-06 Provider Manager; FD-07 regresyon;
  FD-08 native kabul; FD-09 belge/denetim. **Yalnız FD-01 yetkilidir.**

Bu maddeler gelecekteki tasarım gereksinimleridir; mevcut kodun bunların tamamını
uyguladığı iddia edilmez. Bugün Provider Manager veya Freedesktop implementasyonu yok.

## Mevcut üretim akışı ve bağımlılıklar

| Bileşen | Sorumluluk / çağrı sınırı |
|---|---|
| `build.rs` | Opsiyonel C sidecar için sırayla gnome-desktop-4, gnome-desktop-3.0 public API derleme/link kontrolü; ilk başarılı tek modül. Yoksa cache-only derleme. |
| `native/thumbnail_factory.c` | GNOME factory lookup/can_thumbnail/generate/save; kaynak stat kimliği, çıktı sınırları ve save sonrası değişim denetimi. Yeni decoder değildir. |
| `src/thumbnail.rs` | `Provider::discover`, metadata `query`/`query_on_worker`, GIO cache doğrulama; `query_or_generate` cache-first üretim koordinasyonu; helper bulma/timeout/cancel/scratch temizliği. |
| `src/thumbnail_strip.rs` | `drive` içinde worker permit → `Provider::discover(None)` → `query_or_generate` → candidate kimlik kontrolü → `image_loader::preload` → tekrar kimlik/epoch/cancel kontrolü → GTK texture. |
| `src/image_loader.rs` | Cache pikselleri dahil mevcut Glycin 3.0.8 ve zorunlu `SandboxSelector::Bwrap`; güvenilmeyen PNG metadata doğrulaması tek başına piksel güveni değildir. |
| `src/thumbnail_overlay.rs`, `src/image_view.rs` | Overlay/görünürlük ve şerit facade'ı; mevcut viewport/zoom/pan/preload/cancellation davranışı korunur. |
| `src/window.rs` | Katalog → fotoğraf indeksleri; thumbnail callback → aynı koleksiyonda select/revision/title/display_selected. Tekrar aynı seçim etkisizdir; kapanışta callback temizlenir. |

Mevcut Rust bağımlılıkları GTK4/libadwaita/Glycin; dev bağımlılığı tempfile.
Sidecar GNOME desktop + GIO/GObject/GdkPixbuf native API'sine bağlanır.
Cache gösterimi Glycin Bwrap içindedir. GNOME factory'nin kaynak üretimi GNOME
sistem thumbnail altyapısına delege edilir; C API stub testi bu altyapının gerçek
sandbox veya codec güvenliği hakkında native kanıt değildir.

## Sabitlenen mevcut davranış

- `XDG_CURRENT_DESKTOP` iki noktayla ayrılmış token'lar içinde tam GNOME eşleşmesi
  arar, büyük/küçük harfe duyarsızdır. KDE/boş/X-GNOME/GNOME-Classic tek başına
  GNOME değildir. Desteklenmeyen masaüstünde helper çağrılmaz.
- İlk capability CacheOnly (GNOME) veya Unavailable'dır. Metadata helper'ın gerçek
  IPC sonucu Eligible/Ineligible/Failed ayrıdır. CacheAndGenerate zorla atanmaz.
  Daha önce başarılı helper'ın kaybı TemporarilyFailed olabilir; capability ile
  o dosyanın generation_eligible değeri aynı kavram değildir.
- Şerit worker'ı `discover(None)` kullanır; metadata-only TH-01 probe'undaki
  PIVRA_THUMBNAIL_FACTORY_PROBE ile üretim generator keşfi karıştırılmamalıdır.
  `query_or_generate`, gerektiğinde `generation_helper()` üzerinden generator
  arar: PIVRA_THUMBNAIL_GENERATOR → binary yanındaki sidecar → build-time sidecar.
- Geçerli cache, metadata helper Ineligible/Failed olsa da kullanılabilir.
  Missing yalnız cache yokluğudur; decode veya üretim başarısı değildir.
- Gerçek GIO URI/mtime/size, FileKey ve nanosecond kimlik kontrolleri; CRC/container,
  2 MiB cache, 1024 edge, symlink ve kaynak değişimi korumaları vardır.
  Missing/Stale ve belirli bozuk PNG durumları üretilebilir; keyfî Invalid,
  SourceUnavailable, UnsupportedDesktop, Cancelled vb. otomatik üretime çevrilmez.
- Generator kaynak sınırı 64 MiB, C helper AS 1 GiB/CPU 15s/FSIZE 4 MiB;
  metadata helper AS 256 MiB/CPU 1s/FSIZE 1 MiB. Rust generation timeout ve
  process-group/cancellable/scratch temizliği ayrıca korunur.
- Pencere başına tek aktif thumbnail işi; süreç genelinde en çok iki worker ve
  path tekilleştirme. Şerit texture bütçesi **64 adet / 16 MiB**; bu toplam
  süreç RSS/VRAM sınırı değildir. Görünür/seçili kayıt koruması, LRU ve scroll
  sonrası rehydration korunur. Sabit hata invalidation olmadan hover ile denenmez.
- Tıklama katalog seçimini/başlığı değiştirir, hücreleri veya scroll'u gereksiz
  sıfırlamaz. Sol/Sağ gizli şeridi açmaz; pencereler bağımsızdır.

## Mevcut test kapıları ve eklenen eksik korumalar

| Test | Koruma / kanıt sınıfı |
|---|---|
| `tests/thumbnail/test_cache.py` | Gerçek GIO cache metadata; Unicode URI, stale/replace/rename/delete, failure marker, CRC, symlink, boyut sınırı. Yeni 6 test: GNOME token/IPC URI-MIME-mtime, GNOME olmayan oturumda çağrı yok, helper hata/ineligible sırasında cache hit, bozuk IPC ve eksik helper. |
| `tests/thumbnail/test_module_selection.py` | Fake pkg-config/cc; 4→3 derlenebilir tek modül, link sırası, SIGPIPE büyük liste, kontrollü fallback. Gerçek ABI/native generation kanıtı değildir. |
| `tests/thumbnail/test_factory_limits.py` | Gerçek GObject ve C metadata helper + GNOME API stub; eski SIGXFSZ negatif kanıtı, limitler ve gerçek helper hata sonucu. |
| `tests/thumbnail/test_generation_helper.py` | Gerçek üretim C helper + GLib/GIO/GdkPixbuf, GNOME API stub. cached/stale/generate/save/kaynak yarışları ve limitler. Yeni 2 test: 1025px çıktı save öncesi ret; save hatası exit 6, sahte generated yok. |
| Rust thumbnail/strip unit testleri | Dört capability, busy/cancel/protocol, kimlik, process-group/scratch temizliği, global worker limiti, texture hesabı, eviction retry döngüsünü önleme. |
| `examples/support/thumbnail_catalog.rs` | Mevcut 120 öğe GTK/GIO testleri: ilk/79/119 hizalama, stable click/scroll, bozuk/eksik cache/source, kimlik yarışı, invalidation, iki üretim penceresinde seçim/başlık/Sol-Sağ/kapanış. |
| `--catalog-cancel-race` | Her ek decode'un dosya bazlı iptal ile açıklanması: 122 girişim = 120 tekil başarı + 2 iptal; koşul gevşetilmez. |
| `--retention-regression` | 81/120 büyük sentetik cache; görünür/seçili pikseller, scroll dönüşü, gizleme/resize, 64/16 MiB ve iki pencere. |
| `tests/run_thumbnail_provider_probe.sh` | TH-01 metadata-only native modül/API ön kontrolü; thumbnail üretmez. Exit 0 + Missing bir üretim PASS'i değildir. |
| `tests/run_thumbnail_system_probe.sh` | Native helper ön koşulu → retention → gerçek secure Glycin cache ve GNOME üretimi; özel config/cache/medya, iki manuel üretim penceresi. |

Önceki native kabul kriterleri ve tarihsel kayıtlar:
[TH-01](TH01_THUMBNAIL_VALIDATION.md),
[TH-05](TH05_THUMBNAIL_SYSTEM_VALIDATION.md),
[TH-06](TH06_VALIDATION.md), [thumbnail planı](PIV18_THUMBNAIL_PLAN.md).
Bu belgelerdeki eski sonuçlar korunur; yeni oturum sonuçları aşağıdadır.

## 25.10.2026 Codex sonuçları

Ortam: Rust 1.99.1, özel GTK 4.18.6, libadwaita 1.7.6, GIO 2.84.4,
GdkPixbuf 2.42.12; mevcut `/workspace/pivra-environment/piv09/image-activate.sh`
kullanıldı. Kurulum/dependency değişikliği yapılmadı.

| Kontrol | Sonuç |
|---|---|
| İki thumbnail probe'u `cargo build --locked --offline` | PASS |
| `python3 -m unittest discover -s tests/thumbnail -p 'test_*.py' -v` | **37 PASS** (29 mevcut + 8 yeni); GNOME API stub ayrımı yukarıda |
| `cargo test --locked --offline -j2` | **88 PASS**, 0 fail/ignored (40 lib + 24 bin + 24 integration) |
| Xvfb/X11 `--catalog-regression` | PASS; 120 girişim, 120 tekil başarı, 0 iptal; tıklama ve üretim penceresi izolasyonu dahil, kontrollü pixel boundary |
| Headless Sway/Wayland `--catalog-regression --catalog-cancel-race` | PASS; gerçek GTK/Wayland, 122 girişim, 2 iptal, 120 tekil başarı; kontrollü pixel boundary, fiziksel GNOME input testi değil |
| Xvfb/X11 `--retention-regression` | PASS; 81/120, texture <=64 ve <=16 MiB, gerçek GIO/GTK ve sentetik pikseller |
| `--require-generation-helper` | **NOT RUN / exit 77**: API-checked GNOME sidecar yok. Native test buradan ileri çalıştırılmadı. |
| Gerçek GNOME generate/save, gerçek secure Glycin cache/main pikselleri, fiziksel Arch GNOME davranışı | **NOT RUN**; mevcut Bwrap uid-map/proc engeli de korunur; bypass veya tekrar decode denemesi yapılmadı |
| `cargo fmt --check` | **FAIL / exit 1, başlangıçtan mevcut**: image_view.rs:447, active_decode_close_probe.rs:211, application_open_probe.rs:65/83/97/135. Bu dosyalar FD-01'de değiştirilmedi; kapsam dışı format düzeltmesi yapılmadı. |
| `git diff --check` | PASS |

Kontrollü GTK çalıştırmaları özel XDG_CONFIG_HOME/XDG_CACHE_HOME altında,
XDG_CURRENT_DESKTOP=GNOME, G_DEBUG=fatal-criticals ve cairo renderer ile yapıldı.
Yeni testler gerçek decoder başarısı iddia etmez; assertion kaldırılmadı,
üretime stub/fallback taşınmadı.

### Native Arch GNOME yeniden doğrulaması

Mevcut tek giriş korunur:

```bash
bash tests/run_thumbnail_system_probe.sh
```

Önce API-checked generation sidecar bulunmalı ve secure decode ön koşulu geçmeli.
Gerçek cached PNG pikselleri, eksik/bozuk cache üretimi ve source replacement
sonrası yeni piksel/portrait doğrulanır. Son iki özel TEST penceresinde scroll→click,
ana fotoğraf/başlık, Sol/Sağ, T/hover, resize ve bağımsız kapanış kontrol edilir.
Normal fotoğraflara veya kullanıcının cache/config'ine müdahale edilmez.
Eksik helper/secure decoder ile exit 77, **NOT RUN**'dır; kabul PASS'i değildir.

## FD-02 için önerilen entegrasyon sınırları — uygulanmadı

1. Önce mevcut Provider/Outcome/Candidate, iptal ve kimlik semantiğini kapsayan
   küçük ortak arayüzü tanımla; mevcut GNOME adapter/generation helper korunmalı.
   Mevcut oturum GNOME tespiti ve dosyaya özgü eligibility ayrı tutulmalı.
2. İleride Manager'ın uygulama/oturum seçimini, dosya isteğinin Missing/Stale/
   FailedCached/Invalid/SourceUnavailable sonuçlarından ayrı modelle. Geçerli
   cache helper hatasında kullanılmaya devam edebilir; tek dosyanın hatası
   fallback tetiklememeli. CacheOnly ve geçici provider kaybının seçim anlamı
   native yetenek kanıtıyla açıkça tanımlanmalı; manager FD-06 kapsamındadır.
3. Şerit worker'ındaki `discover(None) → query_or_generate` sınırı gelecekteki
   enjeksiyon noktasıdır. GTK hücreleri, callback indeksleri, viewport ve
   foreground/preload yolu aynı kalmalı. Cache verisi her provider için hâlâ
   güvenilmeyen girdidir; secure decode/kimlik/epoch/limit denetimleri korunmalı.
4. FD-03–05'te cache okuyucu/yazar ve Glycin adaptörü bağımsız test edilecek;
   ortak URI/metadata sözleşmesi, atomik yazma ve başka uygulamanın cache'ine
   müdahale etmeme koşulları ayrıca kanıtlanacak. Mevcut kodda bu yeni
   Freedesktop yetenekleri varmış gibi raporlanmamalı.

FD-01 burada durur. Sonraki aşamalar için yeni kullanıcı yetkisi gerekir.
