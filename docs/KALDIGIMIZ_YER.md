# 📍 KALDIĞIMIZ YER — Autocore Anlık Durum

**Son Güncelleme:** 13.10.2026 — **Faz 0, Faz 1 ve Faz 2 (%100 TAMAMLANDI VE YEŞİL)**  
**Durum:** Faz 0 (Workspace & Base), Faz 1 (Safety Core), Faz 2 (HAL & SITL Simulation) **bitti ve çalışıyor**.  
**Sıradaki İş:** **Faz 3: Dora-rs Veri Akış Çizgesi & Apache Arrow Sıfır Kopyalama IPC (`autocore-ipc`)**  
**Aktif Plan:** `docs/plan/GENEL_PLAN.md`  
**Test Kanıtı:** `cargo test --workspace` → **24/24 test PASSED** (0 hata, 0 uyarı)  
**Git Durumu:** Commit `b60b3b4` (`feat(hal,sitl): Faz 2 (autocore-hal ve autocore-sitl) tamamlandi`)

---

## ▶️ NASIL DEVAM EDİLİR (Kaldığın Yerden — Kopyala/Çalıştır)

```bash
cd /home/yavuz/Belgeler/autocore

# 1) Tüm testleri çalıştır (24/24 yeşil beklenir)
cargo test --workspace --quiet

# 2) Statik kontrol
cargo check --workspace --all-targets

# 3) SITL Sanal Donanım ve PTY uçtan uca testini doğrula
cargo test -p autocore-sitl -- --nocapture
```

---

## ✅ ŞU AN ÇALIŞAN SİSTEM VE BİLEŞENLER

| Crate | Görevi / Durumu | Test Kanıtı |
|---|---|---|
| **`autocore-base`** | C-ABI `#[repr(C)]` struct'lar (`KinematicCommand` 32B, `StateVector6DoF` 64B, `ActuatorFeedback` 72B), `bytemuck` zero-copy cast, mikrosaniye saat. | 3/3 test OK (`test_abi_sizes_and_alignment`, `test_zero_copy_roundtrip`, `test_safety_mode_enum`) |
| **`autocore-safety`** | 7-Durumlu Atomic FSM (`Booting`..`SafeLock`), 500ms Sentinel Watchdog, Geofence Clamping, Slew-Rate ivme yumuşatma, Hayalet Hareket Koruması (`flush_all`). | 11/11 test OK (FSM lifecycle, SafeLock trigger, Watchdog timeout, Queue flush) |
| **`autocore-hal`** | `ActuatorPort` trait, CRC-16-CCITT ikili protokol çerçeveleme (`0xAA 0x55` .. `0x0D 0x0A`), `AsyncStreamActuator`, `CanFrame` enkoder/dekoder, `GpioSafetyPin`. | 5/5 test OK (`test_packet_encode_decode_roundtrip`, `test_can_frame_encoding`, `test_gpio_pin_toggle`) |
| **`autocore-sitl`** | Linux `libc::openpty` tabanlı sanal terminal (`/dev/pts/*`), atalet ve sıcaklık modelli `MockActuator`, 50 Hz telemetrili uçtan uca köprü (`SitlSerialBridge`), `VirtualCanBus`. | 5/5 test OK (`test_virtual_pty_read_write`, `test_virtual_can_loopback`, `test_sitl_bridge_end_to_end`) |

---

## 🎯 SIRADAKİ İŞ: FAZ 3 (DORA-RS & ARROW IPC)

1. `crates/autocore-ipc` paketinin açılması.
2. `dora-node-api` entegrasyonu: `DoraNodeWrapper` ve asenkron `EventStream` (`Event::Input`, `Event::Stop`) yaşam döngüsü.
3. `/dev/shm` üzerinde Apache Arrow sabit boyutlu halka tampon havuzları (`Pre-allocated Shared Ring Buffers`).
4. Drop-Old bellek kiralama politikası (yavaş tüketicilerin eski verilerini RAM'de tutmadan anında düşürme).
5. 1200 baytlık GCS telemetri parçalama ve sıfır kopyalama ile yeniden birleştirme (`Reassembly Buffer`).
6. `flows/dataflow-sitl.yml` örnek veri akış çizgesi.

---

## 📝 Son Log
- 2026-10-13: Faz 2 (autocore-hal ve autocore-sitl) tamamlandı; PTY raw mode/cfmakeraw ile binary akış kanıtlandı, 24 test yeşil (commit b60b3b4).
- 2026-10-13: Faz 0 ve Faz 1 tamamlandı; C-ABI boyut kilitleri ve Emniyet Çekirdeği mühürlendi (commit 64154eb).
- 2026-10-13: Workspace kuruldu; `docs/ARCHITECTURE-AUTOCORE.md` şartnamesi mühürlendi.
