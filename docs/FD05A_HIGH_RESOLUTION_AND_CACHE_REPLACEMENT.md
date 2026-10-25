# FD-05A — Karar kapısı ve mevcut kanıt

25.10.2026 — **TESTING; görev tamamlanmadı.**
Başlangıç dev:441c023f5b084f0018ecd612d3fd53f8cd51fd00.
Negatif dosya sistemi testleri:4c8b382ed07adc7e544a06d83aff1f998bddbe51.

## Koşullu yenileme: ayrı karar gerekiyor

Talimatın9. bölümü: “Güvenli koşullu replace kanıtlanamıyorsa üretim koduna
güvensiz replacement ekleme; desteklenen alternatifleri ve sınırlamayı belgele.”
11. bölüm ayrıca ayrı tasarım kararı ve kullanıcı onayı gerektirir.

İncelenen standart Linux rename arayüzlerinde beklenen hedef inode/device/ctime
parametresi yoktur. Reader Stale/Invalid sonucu ve FileKey snapshot'ı, sonraki
atomic replace'e bağlanamaz. Son stat'tan hemen sonra başka üretici geçerli bir
kayıt koyabilir; rename bu yeni kaydı ezer. Gerçek temporary filesystem testi
bunu deterministik olarak yeniden üretti. Bu tüm olası filesystem mekanizmalarının
imkânsızlık ispatı değil, stat+rename protokolünün güvenli olmadığının kanıtıdır.

RENAME_NOREPLACE mevcut dosyayı değiştirmez. RENAME_EXCHANGE iki dosyayı koşulsuz
değiştirir; kontrolü sonradan yapmak geçerli kaydın önceden kaldırılmasını önlemez.
Rollback de başka üreticinin araya girebileceği ikinci bir koşulsuz işlemdir.
Bu exchange argümanı API analizi olup yeni bir kernel exchange testi değildir.
Lock dosyası diğer uygulamalara zorunlu değildir: ikinci test, lock-file varken
normal üreticinin rename yapabildiğini doğrular. flock tüm üreticiler katılmadıkça
bu protokolün eksiğini gidermez; flock syscall testi yapıldığı iddia edilmez.

Üretime conditional replacement eklenmedi; mevcut no-clobber korunur.

**Önerilen karar:** paylaşılan cache'te no-clobber sürsün; yenileme için Pivra'ya
ait özel namespace/cache üzerinde kendi üreticilerinin birlikte kilitlendiği yol
ayrıca yetkilendirilsin. Bu ortak stale girdiyi yerinde yenilemez ve yeni tasarım
kararıdır. Alternatif koşulsuz atomik replace standard cache görünürlüğünü korur
ama son anda gelen başka üreticinin geçerli kaydını koruma şartını sağlayamaz;
bu nedenle mevcut talimat altında uygulanamaz. Sil-sonra-yaz önerilmez.

## Yüksek çözünürlük — henüz uygulanmadı

[FD-05 analizi](FD05_LARGE_IMAGE_DESIGN.md) hâlâ geçerli:256 MiB/32B=8.39MP;
ortak32MP/32768-edge,512MiB frame,64MiB download ve64MiB source sınırları ayrı
engellerdir.24MP RGBA8 tek buffer96MB;50MP200MB; child/transformation/download
bunlara eklenir. Yalnız bütçe artırmak50MP desteği değildir.

Mevcut scale gözlemleri full frame döndüğünü gösterir; API deneyi tekrarlanmadı.
Ana görüntüleme veya thumbnail limitleri değiştirilmedi.108/200MP destekleniyor
iddiası yoktur. Kullanıcı bu talimatta FD-04/05 native başarı bildirdi; bunlar
kullanıcı kanıtıdır, yeni Codex native doğrulaması değildir.

**Açık işler:** thumbnail'a özel ölçülebilir policy; tek large-job/weighted admission;
izole24/48/50MP ölçüm aracı; parent/child RSS/PSS ve cleanup kanıtı; gerçek aktif
iptal; yeni policy'nin kontrollü/native testleri. Bunlar yapılmış gibi gösterilmez.
Bu aşamada yeni benchmark/native ölçüm aracı hazırlanmadı. Büyük fotoğraf için
üretim desteği veya kabul edilmiş yeni sayısal sınır bulunmuyor.

## Çalıştırılan kontroller

- İki hedefli negatif race testi PASS; conditional renewal PASS'i değildir.
- cargo check --locked --offline PASS.
- cargo test --locked --offline:171 PASS (169 mevcut+2 negatif protokol testi).
- cargo clippy --locked --offline --all-targets -- -D warnings PASS.
- Thumbnail Python suite:37 PASS.
- cargo fmt --check FAIL: önceden mevcut image_view.rs447,
  active_decode_close_probe.rs211, application_open_probe.rs65/83/97/135.
- Yeni test rustfmt ve git diff --check PASS.
- Yeni24/48/50MP native ölçüm, aktif decoder cleanup ve replacement pixel
  roundtrip: NOT RUN. Bilinen Codex Bwrap engeli tekrar denenmedi/bypass edilmedi.

Tüm üretim kaynakları, FD-03/04/05 davranışları ve dependencies başlangıçla aynı.
FD-06, paketleme, yayın ve main değişikliği yok. Koşullu yenileme için yukarıdaki
somut politika kararı alınmadan güvenli olmayan bir replace uygulanmayacak.
