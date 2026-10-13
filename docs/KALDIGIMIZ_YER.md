# 📍 KALDIĞIMIZ YER — Autocore Anlık Durum

**Son Güncelleme:** 13.10.2026 — **Faz 0, Faz 1, Faz 2 ve Faz 3 (%100 TAMAMLANDI VE YEŞİL)**  
**Durum:** Faz 0 (Workspace & Base), Faz 1 (Safety Core), Faz 2 (HAL & SITL Simulation) ve Faz 3 (Dora-rs & Apache Arrow IPC) **bitti ve çalışıyor**.  
**Sıradaki İş:** **Faz 4: Alana Özgü Domain Kütüphaneleri (`guidance`, `ground`, `vision`)**  
**Aktif Plan:** `docs/plan/GENEL_PLAN.md`  
**Test Kanıtı:** `cargo test --workspace` → **33/33 test PASSED** (0 hata, 0 uyarı, 0 yavaşlama)  
**Git Durumu:** Faz 3 kodlandı, testleri yeşil.

---

## ▶️ NASIL DEVAM EDİLİR (Kaldığın Yerden — Kopyala/Çalıştır)

```bash
cd /home/yavuz/Belgeler/autocore

# 1) Tüm testleri çalıştır (33/33 yeşil beklenir)
cargo test --workspace --quiet

# 2) Statik kontrol
cargo check --workspace --all-targets

# 3) IPC ve Dora-rs testlerini doğrula
cargo test -p autocore-ipc -- --nocapture
```

---

## ✅ ŞU AN ÇALIŞAN SİSTEM VE BİLEŞENLER

| Crate | Görevi / Durumu | Test Kanıtı |
|---|---|---|
| **`autocore-base`** | C-ABI `#[repr(C)]` struct'lar (`KinematicCommand` 32B, `StateVector6DoF` 64B, `ActuatorFeedback` 72B), `bytemuck` zero-copy cast, mikrosaniye saat. | 3/3 test OK (`test_abi_sizes_and_alignment`, `test_zero_copy_roundtrip`, `test_safety_mode_enum`) |
| **`autocore-safety`** | 7-Durumlu Atomic FSM (`Booting`..`SafeLock`), 500ms Sentinel Watchdog, Geofence Clamping, Slew-Rate ivme yumuşatma, Hayalet Hareket Koruması (`flush_all`). | 11/11 test OK (FSM lifecycle, SafeLock trigger, Watchdog timeout, Queue flush) |
| **`autocore-hal`** | `ActuatorPort` trait, CRC-16-CCITT ikili protokol çerçeveleme (`0xAA 0x55` .. `0x0D 0x0A`), `AsyncStreamActuator`, `CanFrame` enkoder/dekoder, `GpioSafetyPin`. | 5/5 test OK (`test_packet_encode_decode_roundtrip`, `test_can_frame_encoding`, `test_gpio_pin_toggle`) |
| **`autocore-sitl`** | Linux `libc::openpty` tabanlı sanal terminal (`/dev/pts/*`), atalet ve sıcaklık modelli `MockActuator`, 50 Hz telemetrili uçtan uca köprü (`SitlSerialBridge`), `VirtualCanBus`. | 5/5 test OK (`test_virtual_pty_read_write`, `test_virtual_can_loopback`, `test_sitl_bridge_end_to_end`) |
| **`autocore-ipc`** | Dora-rs düğüm sarmalayıcısı (`DoraNodeWrapper`), Apache Arrow v59 sabit boyutlu halka havuzu (`SharedRingBufferPool`), `Drop-Old` posta kutusu/kuyruğu ve 1200 bayt MTU telemetri parçalama/birleştirme (`PacketFragmenter`, `PacketReassembler`). | 9/9 test OK (Arrow kinematic roundtrip, DropOldMailbox, DropOldLeaseQueue, Fragmenter/Reassembler, Standalone wrapper) |

---

## 🎯 SIRADAKİ İŞ: FAZ 4 (DOMAİN KÜTÜPHANELERİ)

1. `crates/autocore-guidance`: 6-DoF Durum Kestirimi, kuaterniyon tabanlı rotasyon, WGS84 $\rightarrow$ NED/ENU koordinat dönüşümü, L1/Pure Pursuit güdüm algoritması, MAVLink v2 köprüsü.
2. `crates/autocore-ground`: Diferansiyel ve Ackerman kinematiği, 2D Local Costmap (engeller + güvenlik payı şişirme katmanı), Reaktif APF (Yapay Potansiyel Alanlar) ve DWA.
3. `crates/autocore-vision`: ONNX Runtime soyutlaması, SORT / Kalman Filtresi tabanlı çoklu nesne takibi, Dinamik IFF histerezis filtresi, Angajman Önleme Geometrisi (Lead-Angle Solver).

---

## 📝 Son Log
- 2026-10-13: Faz 3 (autocore-ipc) tamamlandı; Dora-rs düğüm yaşam döngüsü, Arrow v59 sıfır-kopyalama halka havuzu, Drop-Old politikası ve 1200B MTU parçalayıcı devreye alındı, 33/33 test yeşil.
- 2026-10-13: Faz 2 (autocore-hal ve autocore-sitl) tamamlandı; PTY raw mode/cfmakeraw ile binary akış kanıtlandı, 24 test yeşil (commit b60b3b4).
- 2026-10-13: Faz 0 ve Faz 1 tamamlandı; C-ABI boyut kilitleri ve Emniyet Çekirdeği mühürlendi (commit 64154eb).
