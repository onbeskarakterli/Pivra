# PIV-18 / TH-06 — TESTING

25.10.2026 · dev · başlangıç `1b11074e6ae6b00cdd79c0c6dae985cbb6ea6dce`.
TH-04/TH-05 kullanıcı Arch GNOME kabulüyle DONE; kapanış commit'i
`0b50b7a8a0e514b2ea7743b7ff81c983b7a12acc`. PIV-18 TESTING; TH-07 TODO.
Kullanıcı native kabulü olmadan TH-06 DONE değildir.

## TH-06.1 — Alt kenara bağlı şerit / üst orta düğme

Panel ve düğme bağımsız, non-measuring/clip edilen GtkOverlay çocukları olarak
kaldı. Panelin alt/start/end margin'i0; alt köşeleri düz, arka planı%25. Panel
viewport'un alt kenarına ve iki yanına ulaşır. Panel görünürken küçük düğmenin
bottom margin'i panelin gerçek GTK natural height isteği+4 olur; düğme üst ortada
aşağı okla durur. Panel kapalıyken düğme4px bottom margin ve yukarı oktadır.
Enabled/Available/Visible, T, hide gecikmesi, scroll ve loader mantığı değişmedi.

GSK testleri açık/kapalı Fit/Fill/Actual ve zoom/pan piksellerinin aynı kaldığını,
Picture/Overlay allocation eşitliğini, panel bottom/right/left sınırlarını ve
düğmenin panelin üstünde olduğunu assert eder. Düğme satırında düğme dışındaki
bütün pikseller Picture ile aynıdır; tam genişlikte ayrı bar eklenmez. Açıkken
pick-through fotoğraf üzerindeki düğme satırında; kapalıyken alt kenarda sınanır.
Açık panelin kendisi tekerlek/fare olaylarını almaya devam eder.

TH-06.1 native fiziksel GNOME görsel kabulü NOT RUN. Tek komut:

```bash
bash tests/run_thumbnail_system_probe.sh
```

Gerçek120 JPEG/PNG/secure cache ve iki üretim fixture penceresinde panelin alt
kenarını, üst orta düğmeyi, T/hover/scroll, seçili frame, resize ve viewport'un
aynen korunmasını incele. Gerçek kullanıcının medya/config/cache dosyaları
silinmez; test özel geçici dizinleri kullanır.


TH-06.1 Codex PASS: fmt/check/test (86 Rust:62 unit+24 integration), all-target
Clippy -D warnings; X11 synthetic layout/pixel testi, Sway/Wayland aynı geometri,
X11 koyu tema+scale2 altında120 kayıtlı TH-05 gallery/production-window regresyonu.
Bunlar kontrollü GTK/GIO pikselleridir, yeni native GNOME kabulü sayılmaz.
İlk testte eski açık-panel pick koordinatı artık gerçek alt scrollbar'a denk
 geldi; fotoğraf pick-through testi üstteki düğme satırına taşındı. Panel içi
scrollbar'ın olay alması doğru davranıştır, etkileşim kapatılmadı.
