# 📜 AUTOCORE — KURALLAR (AI Prompt & Geliştirici Kılavuzu)

> **Bu dosyayı yeni bir AI oturumuna prompt olarak ver.** AI bu dosyadaki kuralları ve okuma sırasını takip ederek hiçbir açıklama istemeden projenin kaldığı yerden devam eder.
> Ardından sırasıyla `docs/KALDIGIMIZ_YER.md` ve `docs/plan/GENEL_PLAN.md` okunur.

---

## 1. Proje Amacı ve Felsefesi

**Autocore** — Otonom robotik ekosistemi ve savunma sanayii yarışma ekipleri (HSS, İKA, İDA, İHA, Robotik) için geliştirilen endüstriyel standartta, sıfır-kopyalama (`Zero-Copy`) ve emniyet odaklı işletim çekirdeğidir.
- **Sıfır Kopyalama:** Apache Arrow ve `/dev/shm` üzerinden mikrosaniyelik veri aktarımı. Python tarafında `numpy.ndarray` referansı (`view`) ile **0 ms** gecikme.
- **Fail-Safe Emniyet:** 7 Durumlu Central Safety FSM, 500 ms Sentinel Watchdog zaman aşımı, Geofence Clamping, Slew-Rate Limiter ve Hayalet Hareket Koruması (`Phantom Movement Fix`).
- **Day-1 SITL:** Fiziksel robot olmadan masaüstünde Linux Sanal Terminali (`PTY`), sanal CAN (`vcan`) ve atalet modelli `MockActuator` ile %100 uçtan uca simülasyon.

---

## 2. Geliştirici AI İçin Temel İlkeler

1. **Okuma Sırası Kutsaldır:**
   `KURALLAR.md` → `GENEL_PLAN.md` → `KALDIGIMIZ_YER.md` → `BRAIN-KOPRU.md` → `docs/ajan-hafizasi/DURUM.md` → `docs/reports/SISTEM_RAPORU.md` → kod. Sırayı bozma, tahminle ilerleme.
2. **Kullanıcıya "Nerede Kaldık?" Sorma:**
   `docs/KALDIGIMIZ_YER.md` içindeki `Durum` ve `Sıradaki` başlıklarından doğrudan bağlamı al ve devam et.
3. **Tek Aktif Plan:**
   `docs/plan/GENEL_PLAN.md` tek nihai ve geçerli plandır.
4. **İskelet ≠ Bitti (AGENTS.md §7):**
   Yalnızca boş klasör veya trait tanımlayıp "bitti" demek yasaktır. Her faz gerçek çalışan kodlar, birim ve entegrasyon testleriyle tamamlanır. Mock veri kesinlikle yasaktır (SITL katmanı hariç, o da gerçek donanım fiziğini emüle eden test tezgâhıdır).
5. **Kaldığımız Yeri Canlı Tut:**
   Her faz ve alt görev başında/sonunda `docs/KALDIGIMIZ_YER.md` güncellenir (`Son Güncelleme`, `Tamamlananlar`, `Şu An`, `Sıradaki`, `Test Kanıtı`).
6. **Test Kanıtı Olmadan İddia Yasaktır (AGENTS.md §8):**
   "Çalışıyor / bitti" demeden önce `cargo test --workspace` ve `cargo check --workspace --all-targets` komutları çalıştırılır; kaç testin geçtiği kanıt olarak rapora işlenir.
7. **Raporla Birlikte Kapat:**
   Kodu güncelleyip raporları eski bırakma. İlgili `docs/reports/` dosyasını ve `docs/ajan-hafizasi/` notlarını eşzamanlı güncelle.
8. **C-ABI Güvencesi:**
   Tüm ortak wire/IPC struct'ları `#[repr(C)]`, `bytemuck::Pod` ve `bytemuck::Zeroable` olmak zorundadır. Struct boyutları `static_assertions` ile kilitlenmelidir.
9. **Kayıt Yenileme:**
   Her oturum başında raporları oku, oturum sonunda kayıtları güncelle. `docs/KALDIGIMIZ_YER.md` `Son Log` en fazla 3 satır tutulur.

---

## 3. Okuma Sırası (Yeni AI İçin)

1. **Bu dosya** (`docs/KURALLAR.md`)
2. `docs/plan/GENEL_PLAN.md` (Mimari şartname ve faz yol haritası)
3. `docs/KALDIGIMIZ_YER.md` (Anlık durum, bitti/kalan, sıradaki komutlar)
4. `docs/BRAIN-KOPRU.md` (SecondBrain köprüsü)
5. `docs/ajan-hafizasi/DURUM.md` ve `docs/ajan-hafizasi/DERSLER.md`
6. `docs/reports/SISTEM_RAPORU.md`
7. Kodlar (`crates/`)

---

## 4. Çalışma Akışı

- **Başlarken:** `KALDIGIMIZ_YER.md` oku → `Sıradaki` işi al → ek soru sormadan geliştir.
- **Kodlarken:** Crate'i ve ilgili test modüllerini geliştir.
- **Bitirirken:**
  1. `cargo test --workspace` çalıştır.
  2. `docs/KALDIGIMIZ_YER.md` güncelle.
  3. `docs/reports/` altına faz raporunu ekle.
  4. Git commit at.

---
*Kaynak: `[[docs/ARCHITECTURE-AUTOCORE.md]]`, `[[AGENTS.md]]` §7 & §8*
