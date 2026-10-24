# PIV-18 — preview/full pipeline — TESTING

24.10.2026 · dev. Başlangıç `07f47b5fb21200d8337933893128b700e5b8329d`;
origin/dev fetch ve fast-forward kontrolü: zaten güncel, çalışma dizini temizdi.
PIV-01–17 DONE, son kullanıcının görev talimatındaki durum beyanına göre korunur.
İlk yerel tracking ref PIV-17 TESTING gösteriyordu. Push öncesi explicit dev
fetch ile `5046419` dahil PIV-17 kullanıcı native/manuel kabul kayıtları ve kenar
navigasyon tasarımı normal merge ile korundu. Bunlar Codex native sonucu değildir.

## API kanıtı ve uygulanabilir yol

İncelenen client: Cargo registry **glycin-3.0.8/src/api_loader.rs**:
`Image::specific_frame` dokümanı loader'ın istek alanlarını yok sayabileceğini
söyler; `FrameRequest::scale` yalnız protokol isteğini ayarlar. `next_frame`
sabit görüntü için yalnız bir kez çağrılabilir. Boyutlar `Image::details`,
orientation `transformation_orientation` ile erişilebilir. Mevcut loader
`apply_transformations(true)` kullanır; ikinci EXIF uygulaması eklenmedi.
Cancellable istek, frame lifetime ve global pool politikaları değiştirilmedi.

Yerel resmi GNOME Glycin 2.0.0 kaynak checkout'unda
`glycin-loaders/glycin-image-rs/src/main.rs` içindeki
`fn frame(&mut self, _frame_request: FrameRequest)` isteği kullanmaz ve decoder'ı
`take()` eder. Böyle bir yolda scale→full iki istek güvenilir erken-preview
sağlamaz; ikinci full decode üretmek de hedefe aykırıdır. Başka loader veya
Arch sürümünün davranışı bu kaynak incelemesinden doğrulanmış sayılmaz.
Client'ta embedded JPEG/RAW/TIFF preview seçen sertifikalı bir erişim yolu
bulunmadı. **Üretimde erken preview etkinleştirilmedi.** Full frame'den küçük
texture üretip erken preview diye raporlanmadı; decode gecikmesi iyileşmesi
iddiası yoktur.

| Format | Embedded preview | Üretimde full decode | Yeni native sonuç |
|---|---|---|---|
| JPEG | Kullanılan yolda destek doğrulanmadı | Güvenli mevcut yol | NOT RUN |
| PNG | Kullanılan yolda destek doğrulanmadı | Güvenli mevcut yol | NOT RUN |
| TIFF | UNSUPPORTED / DEFERRED PIV-40/41 | UNSUPPORTED / DEFERRED PIV-40/41 | NOT RUN |
| RAW/DNG | UNSUPPORTED / DEFERRED PIV-40/41 | UNSUPPORTED / DEFERRED PIV-40/41 | NOT RUN |

## Uygulanan fallback ve sınırlar

`image_pipeline::Phase`: Requested, PreviewLoading, PreviewReady, FullLoading,
FullReady, Failed, Cancelled. Gerçek ImageView yolu doğrudan FullLoading →
FullReady/Failed, close/message iptalinde Cancelled kullanır. Eski callback
önce mevcut token/weak/visible kontrollerinden geçer, sonra phase günceller.
Preview geçiş fonksiyonları yalnız model testleriyle doğrulanır; bunlar gerçek
preview sunumu veya decode kanıtı değildir. Üretimde preview sağlayıcısı yoktur.

Tek decode, tek kalite (full), mevcut loading mesajı ve cache-hit texture reuse
korunur. Preview/full için çift texture sahipliği, ek worker veya ikinci decode
yoktur. Mevcut cache yalnız full kalite taşır; preview cache'e yazılmaz.
3 admitted + 1 pending, 5s admission, aktif seçim > tek background, cache
128 MiB/3 girdi/64 MiB girdi sınırları, monitor/metadata/Refresh/pressure,
Fit/Fill/Actual, zoom/pan/filter ve pencere izolasyonu değişmedi. Hesaplanan
cache byte'ları RSS/VRAM değildir. Kaynak medya, Cargo bağımlılıkları ve Bwrap
zorunluluğu korunur.

**Uygulanmayan/NOT RUN:** erken-preview piksel sunumu, preview→full sırasında
viewport/zoom/pan koordinat dönüşümü, preview üzerinde hafif full-loading
göstergesi, gerçek preview-success/full-failure UI yolu ve iki kalite arasında
cache promotion. Mevcut full-only fallback bu özellikleri yapmış sayılmaz.
Sertifikalı bir erken-preview sağlayıcısı olmadan bu işlemler etkin değildir.
PIV-18 TESTING kalır; native fallback başarısı tek başına erken-preview kabulü
anlamına gelmez.

## Test kanıtı

- PASS: 2 yeni Rust model testi; doğrudan full, preview/full başarı/hata
  kombinasyonları, late-preview suppression ve cancel terminal durumu.
- PASS: kontrollü gerçek GTK/GIO preload probe; FullLoading/FullReady ve close
  Cancelled assertion'ları eklendi. Synthetic pixel verisi gerçek decode değildir.
- PASS: `bash tests/check.sh` mevcut activation ile, stable/locked/offline/-j2:
  fmt/check/build/test/clippy all-targets -D warnings; **74 Rust** (50 unit +24
  integration), **30 Python observer**, fixture/hash bütünlüğü.
- PASS: 3 yeni Python rapor sürücüsü testi; full-only etiketleme, exit77'de
  tekrar etmeme, timeout'u FAIL kaydetme. Subprocess mock'ları decode değildir.
- BLOCKED/NOT RUN: strict secure preload probe Xvfb'de bir kez **exit77**;
  ilk görüntü oluşmadı, loader erken çıktı. Önceki Codex Bwrap/read-only proc
  engeli bağlamı korunur. Aynı başarısız native kapı tekrar çalıştırılmadı.

- PASS: 20 scoped grafik çağrısı (14 X11, 4 Sway, 2 Weston): navigation,
  watch/failures/race, async/timeout, mode/zoom/filter ve surface kontrolleri.
  Gerçek decode PASS değildir; Weston normal close NOT RUN. Tüm örneklerin
  locked/offline derlemesi, shell syntax ve whitespace PASS.

## Tek native komut

```bash
bash tests/run_preview_pipeline_probe.sh
```

Mevcut Arch/GNOME Wayland terminalinde güncel dev checkout ile çalıştırılır.
Secure JPEG/PNG/EXIF baseline → phase/cache/preload → gerçek zoom/pan/filter →
async cancel/race/two-window → mevcut partial memfd/pidfd aktif-close checkpoint.
Kaynaklar testin geçici kopyalarıdır. Her alt komutun exit durumu ve full-only
etiketi `/tmp/pivra-preview-pipeline.json` içine yazılır; active-close detayları
`/tmp/pivra-preview-active-close.json`. İlk hata/77'de durur; tamamlanmayan aşamalar
PASS değildir. Hepsi exit0 olsa bile rapor yalnız **PASS full-only fallback**
der; early-preview ve preview/full continuity UNSUPPORTED/NOT RUN kalır.
Gerçek GNOME manuel kabulü ve desteklenebilir preview sağlayıcısı değerlendirmesi
açıktır. Faz 0 temiz kurulum, DESKTOP-06, PIV-39 ve kenar navigasyon tasarım kararı
korunur; PIV-19 başlamadı.

## Commit'ler

- `99d49ddbdfb73c88d1c638f5d1d0d07ae4c85c3d`: kalite yaşam döngüsü, full-only ImageView entegrasyonu.
- `29059174e800ed067414651c99b02c2e2a9c779e`: phase assertion'ları, tek native komut/JSON ve rapor testleri.

Uzak belge değişiklikleri kodu değiştirmedi; kod testleri tekrar edilmedi.
