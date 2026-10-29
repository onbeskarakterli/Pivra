# FD-02 — Modüler thumbnail provider sözleşmesi

25.10.2026 — **FD-02 DONE (bağımsız sözleşme kapsamı).**
Başlangıç: `5acaf6b22479d14a6eb3237449c8e96770d29cea` (`dev`, temiz ve origin/dev ile eşit).
Kod/test commit'i: `ddebb71865d6fa2db51382f53001616e544cc2d4`.
Referans: [FD-01 GNOME baseline](FD01_GNOME_BASELINE.md) ve kullanıcının
Freedesktop Provider şartnamesi. FD-03–FD-09 başlatılmadı.

## Değişiklikler ve veri modeli

Yalnız `src/thumbnail_provider.rs`, `src/lib.rs` içindeki tek modül bildirimi,
`tests/thumbnail_provider_contract.rs` ve bu belge eklendi.
Yeni bağımlılık yok; Cargo.toml/lock değişmedi.

| Öğe | Sözleşme |
|---|---|
| `ProviderId` | Yalnız Gnome / Freedesktop; üçüncü Internal Provider yok. |
| `PROVIDER_PRIORITY` | Pasif `[Gnome, Freedesktop]` sırası; keşif veya yönlendirme fonksiyonu değildir. |
| `ProviderStatus` | Sağlayıcının kimliği, enabled durumu ve mevcut `Capability`; uygulama/oturum gözlemi. Dosya sonucu ile değiştirilmez. |
| `Eligibility` | Dosyaya özgü Unknown / Eligible / Ineligible. Eligibility sorgusunun başarısızlığı ayrıca `Error` olur. |
| `Request` | Mevcut `FileKey` referansı, GIO Cancellable ve monotonic Instant deadline. İkinci path/URI/mtime/size modeli yok. |
| `Request::check_control(now)` | Açık iptal önceliklidir; `now >= deadline` TimedOut. Sağlanan monotonic zaman ile sleepsiz sınır testleri yapılır. |
| `CacheOutcome` | **Mevcut `thumbnail::Outcome` yeniden export edilir**, kopyası yazılmadı. Hit/Missing/Stale/FailedCached/Invalid/SourceUnavailable vb. korunur. |
| `Candidate`, `Capability` | Doğrudan mevcut `thumbnail` tipleri yeniden export edilir; candidate doğrulaması ve güvenilmeyen PNG semantiği değişmez. |
| `Error` | Cancelled / TimedOut; SourceUnavailable / InvalidSource; Ineligible / Io / GenerationFailed; ayrıca açık ProviderUnavailable / ProviderTemporarilyUnavailable. Dosya hatası provider kaybı olarak yeniden sınıflandırılmaz. |

`ThumbnailProvider: Send + Sync` object-safe, **blocking worker** sözleşmesidir:

```rust,ignore
fn status(&self) -> ProviderStatus;
fn eligibility(&self, request: &Request<'_>) -> Result<Eligibility, Error>;
fn lookup(&self, request: &Request<'_>) -> Result<CacheOutcome, Error>;
fn generate(&self, request: &Request<'_>) -> Result<Candidate, Error>;
```

FD-02'de GNOME veya Freedesktop implementasyonu, default yöntemle fallback,
provider keşfi, iş kuyruğu veya manager yok. İki kimlik test-only double üzerinden
aynı `dyn ThumbnailProvider` worker arayüzünde temsil edilebilir; bu, gerçek
Freedesktop veya GNOME üretimi yapıldığı anlamına gelmez.

Cache lookup eski cache sonuçlarını korur. Üretim/uygunluk operasyonlarında
cache sonucu dönülemeyeceği için ortak `Error` ayrıca dosya/kontrol hatalarını
ifade eder. Yeni lookup implementasyonları kontrol hatalarını Err(Cancelled veya
TimedOut) döndürmelidir; ilerideki legacy adapter, eski Outcome::Cancelled'ı bu
sınırda normalize etmelidir. FD-02 mevcut Outcome'yu veya GNOME kodunu değiştirmez.

Request içindeki FileKey, isteğin oluşturulduğu andaki kaynak kimliğidir. Kaynak
sonradan silinebilir/değişebilir; aday yayımlanmadan yeniden doğrulanmalıdır.
Candidate hâlâ container/metadata düzeyindedir; görüntülenmeden **güvenli Glycin
çözümleme** zorunludur. Request kontrolü tek başına çalışan worker/child temizliği
sağlamaz: iptal/deadline iş boyunca ve yayımdan önce uygulanmalı, kaynak temizliği
ileride gerçek implementasyonların sorumluluğu olmalıdır. FD-02'de runtime cleanup
uygulandı veya yeni decoder sertifikalandı iddiası yoktur.

## Pasif seçim sözleşmesi — FD-06'ya kadar aktif değil

- Oturum/uygulama düzeyinde bir sağlayıcı bağlanmalı; her dosyada keşif yapılmamalı.
- Etkin ve sağlıklı GNOME önceliklidir. CacheOnly arıza değildir; Freedesktop
  yalnız GNOME kullanılamıyorsa seçilebilir. Disabled/Unavailable/TemporarilyFailed
  gözlemleri ayrıca temsil edilir; burada bunları seçime dönüştüren algoritma yoktur.
- Cache miss, bozuk dosya, Ineligible, kaynak kaybı ve kullanıcı iptali provider
  değişimi tetiklememeli. Provider-wide hata için bağımsız runtime kanıt gerekir.
- Üretim yeteneği geçici kaybolsa da geçerli cache okunabilir; CacheOnly ve
  TemporarilyFailed altında Hit temsil edilip orijinal candidate kimliği test edilir.
- Sağlayıcının geçici kaybı ileride kontrollü oturum yeniden değerlendirmesine
  konu olabilir. Tek dosyanın sonucu sessizce bu gözlemi değiştirmemeli.
- Aynı istek için iki generator paralel çalıştırılmamalı; generate yalnız o
  sağlayıcının tek operasyonudur, gizli fallback/retry yapmamalıdır.

Öncelik dizisi ve sonuçların ayrılığı bu aşamada veri düzeyinde sınanır.
Oturum keşfinin sıklığı, gerçek fallback, yeniden değerlendirme ve provider'lar
arası tekilleştirme **henüz uygulanmadı/test edilmedi**; FD-06 kabulü değildir.

## Değiştirilmeyen GNOME entegrasyon noktaları

FD-01'de kaydedilen üretim yolu aynı kalır:

`window → ImageView/overlay → ThumbnailStrip::drive → Provider::discover(None)
→ query_or_generate → GNOME sidecar/cache → candidate identity → secure Glycin preload
→ identity/epoch/cancel doğrulaması → GTK texture`.

`src/thumbnail.rs`, `src/thumbnail_strip.rs`, GNOME C sidecar, build.rs,
image_loader/image_view/window/overlay, mevcut thumbnail testleri ve native script
başlangıç commit'iyle aynıdır (`git diff --exit-code` ile doğrulandı).
GNOME token/API/sidecar keşfi, GIO cache, worker/path limiti, 64 texture/16 MiB,
scroll/seçim, foreground/preload ve pencere izolasyonu değiştirilmedi.
Üretim bu yeni trait'i henüz çağırmaz. Test double üretim loader'ına bağlanmaz.

## Test matrisi — 25.10.2026 Codex

Mevcut SDK etkinleştirildi:
`source /workspace/pivra-environment/piv09/image-activate.sh`.
Rust 1.99.1, GTK 4.18.6, libadwaita 1.7.6; kurulum tekrarı yapılmadı.

| Kontrol | Sonuç / kanıt sınıfı |
|---|---|
| İlk hedefli `cargo test --locked --offline --test thumbnail_provider_contract -j2` | PASS: 9 bağımsız sözleşme testi; son test değişiklikleri tam regresyonda da geçti. |
| `cargo check --locked --offline -j2` | PASS |
| `cargo test --locked --offline -j2` | **97 PASS**, 0 fail/ignored: 88 mevcut + 9 sözleşme testi |
| `cargo clippy --locked --offline --all-targets -j2 -- -D warnings` | PASS |
| `cargo build --locked --offline --example thumbnail_provider_probe --example thumbnail_overlay_probe -j2` | PASS |
| `python3 -m unittest discover -s tests/thumbnail -p 'test_*.py' -v` | **37 PASS**, test kaynakları değiştirilmedi; GIO/C IPC ve GNOME API stub ayrımı FD-01'deki gibi |
| Xvfb/X11 katalog | PASS: 120 girişim, 120 tekil başarı, 0 iptal; click/scroll/seçim, kaynak/cache hata senaryoları, üretim penceresi izolasyonu |
| Xvfb/X11 retention | PASS: 81/120 kayıt, görünür/seçili pikseller, 64 texture/16 MiB bütçesi, iki pencere |
| Headless Sway/Wayland katalog + cancellation race | PASS: 122 girişim, 120 tekil başarı + 2 iptal; açıklanmayan ek decode yok |
| `--require-generation-helper` | **NOT RUN / exit 77**: API-checked GNOME sidecar mevcut değil |
| Gerçek GNOME üretimi, secure Glycin cache/main decode ve fiziksel Arch GNOME kabulü | **NOT RUN**: native helper ön koşulu yok; bilinen Codex Bwrap/proc engeli aşılmadı, decode tekrar döngüsü yok |
| `cargo fmt --check` | **FAIL, önceden mevcut**; FD-01 ile aynı image_view.rs:447, active_decode_close_probe.rs:211 ve application_open_probe.rs:65/83/97/135. Kapsam dışı dosyalara format uygulanmadı. |
| `rustfmt --edition 2024 --check src/thumbnail_provider.rs tests/thumbnail_provider_contract.rs` | PASS; lib.rs'ye yalnız modül bildirimi eklendi |
| `git diff --check` | PASS |

9 sözleşme testi: iki kimlik/öncelik; availability–eligibility ayrımı;
Ineligible–Failed ayrımı; cache miss/üretim hatasında durumun korunması;
bozuk kaynak–kayıp kaynak–provider kaybı ayrımı; iptal/timeout ile iş kabul etmeme;
monotonic deadline/iptal önceliği; CacheOnly/geçici üretim kaybında aynı eski
Candidate ile Hit/kimlik; object-safe iki-provider kimliği.

GTK çalıştırmalarında özel geçici config/cache, XDG_CURRENT_DESKTOP=GNOME,
G_DEBUG=fatal-criticals ve cairo renderer kullanıldı. Bunlar gerçek GTK/GIO
nesneleri ve kontrollü/sentetik piksellerdir; gerçek GNOME üretimi veya secure
decode PASS'i değildir. Wayland headless Sway testi fiziksel GNOME input kabulü değildir.

Mevcut native giriş değiştirilmedi:

```bash
bash tests/run_thumbnail_system_probe.sh
```

Eksik sidecar/secure decode için 77, NOT RUN'dır. FD-02 pasif sözleşme testlerinin
başarısı bu native sonuçların yerine geçmez.

## FD-03 için bağlantı sınırları — yalnız öneri

- Bağımsız Freedesktop cache okuyucusu ileride lookup sözleşmesini kullanabilir;
  Candidate/FileKey/Outcome yeniden kopyalanmamalı. URI/mtime/size, cache kimliği,
  corrupt/stale ayrımı ve iptal/deadline korumaları bağımsız test edilmelidir.
- Boyut sınıfları, ortak cache okuma güvenliği ve shared-cache dosyalarına müdahale
  etmeme ayrı kabul konusudur. Bu görevde cache okuyucu/yazıcısı geliştirilmedi.
- Yeni GNOME adapter ve production call-site geçişi ayrı yetki gerektirir.
  Manager FD-06, Glycin üretim adaptörü FD-04, atomik yazma FD-05 kapsamındadır.
- Runtime güvenliği, worker limitleri ve session-level tek-provider seçimi
  gelecekte gerçek implementasyon/yöneticide kanıtlanmalıdır; bu pasif arayüz
  bunları kendiliğinden uyguluyor gibi sunulmamalıdır.

FD-02 burada durur; FD-03 ve sonraki aşamalar için yeni kullanıcı yetkisi gerekir.
