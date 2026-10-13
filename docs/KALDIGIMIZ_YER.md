# 📍 KALDIĞIMIZ YER — Autocore Anlık Durum

**Son Güncelleme:** 13.10.2026 — **Faz 0, Faz 1, Faz 2, Faz 3 ve Faz 4 (%100 TAMAMLANDI VE YEŞİL)**  
**Durum:** Faz 0 (Workspace & Base), Faz 1 (Safety Core), Faz 2 (HAL & SITL Simulation), Faz 3 (Dora-rs & Apache Arrow IPC) ve Faz 4 (Alana Özgü Domain Kütüphaneleri) **bitti ve çalışıyor**.  
**Sıradaki İş:** **Faz 5: SDK, Takım Başlangıç Şablonları ve Siyah Kutu Teşhis Motoru (`autocore-sdk`, `templates/`, `blackbox/`, `deploy/`)**  
**Aktif Plan:** `docs/plan/GENEL_PLAN.md`  
**Test Kanıtı:** `cargo test --workspace` → **54/54 test PASSED** (0 hata, 0 uyarı, 0 yavaşlama)  
**Git Durumu:** Faz 4 kodlandı, testleri yeşil.

---

## ▶️ NASIL DEVAM EDİLİR (Kaldığın Yerden — Kopyala/Çalıştır)

```bash
cd /home/yavuz/Belgeler/autocore

# 1) Tüm testleri çalıştır (54/54 yeşil beklenir)
cargo test --workspace --quiet

# 2) Statik kontrol
cargo check --workspace --all-targets

# 3) Faz 4 domain testlerini ayrı ayrı doğrula
cargo test -p autocore-guidance
cargo test -p autocore-ground
cargo test -p autocore-vision
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
| **`autocore-guidance`** | WGS84 $\longleftrightarrow$ ECEF $\longleftrightarrow$ NED dönüşümleri, Park et al. L1 adaptif doğrusal olmayan yol takip güdümü, MAVLink v2 çerçeve ayrıştırma/kodlama, 6-DoF strapdown kuaterniyon durum kestiricisi. | 8/8 test OK (`test_geodetic_to_ecef`, `test_geodetic_to_ned`, `test_l1_guidance`, `test_mavlink_v2`, `test_state_estimator`) |
| **`autocore-ground`** | Diferansiyel sürüş ve Ackermann kinematiği (tekerlek kayma/slip faktörlü), İDA çift itici güç tahsis matrisi, 2D Local Costmap (BFS üstel şişirme katmanı), Yapay Potansiyel Alanlar (APF) ve Dinamik Pencere Yaklaşımı (DWA) engel kaçınma. | 7/7 test OK (`test_diff_drive`, `test_ackermann`, `test_ida_thruster`, `test_costmap_inflation`, `test_apf_obstacle`) |
| **`autocore-vision`** | `BoundingBox` IoU hesabı, SORT çoklu hedef takip motoru (sabit hızlı durum kestirimi, açgözlü IoU eşleme, hedef onay/tasfiye), Dinamik IFF histerezis filtresi (`min_samples` eşikli), Lead-Angle Solver (2. derece önleme geometrisi). | 6/6 test OK (`test_bbox_iou`, `test_tracker_lifecycle`, `test_iff_hysteresis`, `test_lead_angle_crossing`) |

---

## 🎯 SIRADAKİ İŞ: FAZ 5 (SDK, BAŞLANGIÇ ŞABLONLARI VE SİYAH KUTU)

1. `crates/autocore-sdk`: Rust yüksek seviye API, C++20 Header-Only C-ABI wrapper (`autocore.hpp`), Python `autocore` paketi (`ctypes` + zero-copy `numpy` view).
2. `templates/`: `starter-python` (`mission.py`), `starter-cpp` (`mission.cpp`), `starter-rust` (`mission.rs`).
3. `blackbox`: Dairesel ikili log tamponu, telemetri kaydedici ve ayrıştırıcı (Foxglove Studio / Rerun uyumlu).
4. `deploy/autocore.service`: Linux systemd servis tanımı ve `SCHED_FIFO` gerçek zamanlı CPU önceliği atamaları.
5. Uçtan Uca SITL Görev Koşusu (`python3 templates/starter-python/mission.py --sitl`).

---

## 📝 Son Log
- 2026-10-13: Faz 4 (Domain Kütüphaneleri: guidance, ground, vision) tamamlandı; WGS84/NED, L1 güdüm, MAVLink v2, diferansiyel/Ackermann/İDA kinematiği, 2D costmap, APF/DWA, SORT takipçi, Dinamik IFF ve Lead-Angle solver devreye alındı. 54/54 test yeşil.
- 2026-10-13: Faz 3 (autocore-ipc) tamamlandı; Dora-rs düğüm yaşam döngüsü, Arrow v59 sıfır-kopyalama halka havuzu, Drop-Old politikası ve 1200B MTU parçalayıcı devreye alındı, 33/33 test yeşil.
- 2026-10-13: Faz 2 (autocore-hal ve autocore-sitl) tamamlandı; PTY raw mode/cfmakeraw ile binary akış kanıtlandı, 24 test yeşil (commit b60b3b4).
- 2026-10-13: Faz 0 ve Faz 1 tamamlandı; C-ABI boyut kilitleri ve Emniyet Çekirdeği mühürlendi (commit 64154eb).
