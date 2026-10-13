# 🔍 AUTOCORE SİSTEM DURUM RAPORU

**Tarih:** 13.10.2026 (Faz 0, Faz 1, Faz 2, Faz 3 ve Faz 4 tamamlanması sonrası)  
**Hazırlayan:** AI Mimari ve Çekirdek Sistemler Ajanı  
**Proje:** Autocore Autonomous Robotics Runtime & Safety Kernel (`autocore`)  
**Repo:** `/home/yavuz/Belgeler/autocore`  

---

## 1. SİSTEM ÖZETİ

Autocore; savunma sanayii, insansız kara/deniz/hava araçları ve robotik yarışma takımları (HSS, İKA, İDA, İHA) için geliştirilen endüstriyel standartta, sıfır-kopyalama (`Zero-Copy`) paylaşılan bellek modeline sahip, bağımsız emniyet bekçili ve donanım bağımsız masaüstü simülasyonlu (`SITL`) bir robotik işletim çekirdeğidir.

---

## 2. MİMARİ BİLEŞENLER

### 2.1 Paket (Crate) Envanteri

| Crate | Rol / Katman | Durum | Test Sayısı | Boyut / Tip Garantisi |
|---|---|---|---|---|
| **`autocore-base`** | C-ABI Struct'lar, Mikrosaniye Saat, Ortak Hatalar | ✅ Aktif (Faz 0) | 3 passed | `KinematicCommand` (32B), `StateVector6DoF` (64B), `ActuatorFeedback` (72B) |
| **`autocore-safety`** | 7-Durumlu Safety FSM, 500ms Watchdog, Clamping, Slew-Rate | ✅ Aktif (Faz 1) | 11 passed | CAS Atomic FSM, Lock-Free Queue Flush (Hayalet Hareket Koruması) |
| **`autocore-hal`** | `ActuatorPort` Trait, CRC-16 Çerçeveleme, CAN Bus, GPIO | ✅ Aktif (Faz 2) | 5 passed | CRC-16-CCITT ikili paket (38B), SocketCAN / `CanFrame` |
| **`autocore-sitl`** | Linux PTY Sanal Port, Atalet Modelli `MockActuator`, 50Hz Köprü | ✅ Aktif (Faz 2) | 5 passed | `libc::openpty` raw mode, $\tau=80\text{ms}$ atalet filtresi, sanal telemetri |
| **`autocore-ipc`** | Dora-rs Aktör Sarmalayıcısı, Apache Arrow `/dev/shm`, Drop-Old, 1200B Parçalayıcı | ✅ Aktif (Faz 3) | 9 passed | Arrow v59, Pre-allocated Shared Ring Buffer, Drop-Old Mailbox/Queue |
| **`autocore-guidance`**| 6-DoF Durum Kestirimi, WGS84 $\leftrightarrow$ NED, L1/Pure Pursuit, MAVLink v2 | ✅ Aktif (Faz 4) | 8 passed | Kuaterniyon matematiği, Park et al. L1 yol takibi, MAVLink v2 çerçevesi |
| **`autocore-ground`**  | Diferansiyel & Ackerman Kinematiği, 2D Costmap, APF/DWA | ✅ Aktif (Faz 4) | 7 passed | Slip-faktörlü kinematiği, İDA çift itici tahsis matrisi, BFS üstel şişirme |
| **`autocore-vision`**  | BBox IoU, SORT Takipçi, Dinamik IFF, Lead-Angle Solver | ✅ Aktif (Faz 4) | 6 passed | Açgözlü IoU eşleme, histerezis dost/düşman filtresi, 2. derece önleme kökü |
| **`autocore-sdk`**     | C-ABI C++ Header, Python ctypes/numpy view köprüsü, Blackbox | ⏳ Sırada (Faz 5) | Planlandı | Takım SDK ve şablonları, systemd SCHED_FIFO servisi |

---

## 3. MEVCUT KONTROL VE VERİ AKIŞI

```
[Algılama & Hedef Tespit (autocore-vision)]
        │ SORT Takip & Lead-Angle Solver
        ▼
[Yol & Güdüm Planlama (autocore-guidance / ground)]
        │ WGS84->NED, L1 Yol Takibi, APF/DWA Kaçınma
        ▼
[Kinematik Komut Üretici (Görev Düğümü)]
        │
        ▼ (Dora EventStream / Apache Arrow Shared Buffer)
┌─────────────────────────────────────────────────────────────┐
│ autocore-ipc                                                │
│  ├─ DoraNodeWrapper (DoraCoordinator / Standalone SITL)     │
│  ├─ SharedRingBufferPool (Önceden tahsisli Arrow havuzu)    │
│  ├─ DropOldMailbox / DropOldLeaseQueue (Sıfır kuyruk gecik.)│
│  └─ PacketFragmenter / Reassembler (1200 bayt MTU)          │
└─────────────────────────────────────────────────────────────┘
        │ KinematicCommand (Arrow UInt8Array / #[repr(C)] 32B)
        ▼
┌─────────────────────────────────────────────────────────────┐
│ autocore-safety                                            │
│  ├─ SentinelWatchdog (500ms heartbeat kontrolü)             │
│  ├─ CentralSafetyFsm (Booting -> Disarmed -> Armed/Mission) │
│  ├─ ClampingEngine (Geofence & Sınır Kırpma)                │
│  ├─ SlewRateLimiter (İvme Yumuşatma Delta_v / Delta_t)      │
│  └─ LockFreeFlushQueue (E-Stop anında kuyruk boşaltma)      │
└─────────────────────────────────────────────────────────────┘
        │ KinematicCommand (32 bayt #[repr(C)])
        ▼
┌─────────────────────────────────────────────────────────────┐
│ autocore-hal / autocore-sitl                                │
│  ├─ CRC-16 İkili Çerçeveleyici (0xAA 0x55 .. 0x0D 0x0A)     │
│  ├─ VirtualPtyPair (/dev/pts/* raw mode terminal)           │
│  ├─ SitlSerialBridge (50 Hz telemetri köprüsü)              │
│  └─ MockActuator (tau=80ms atalet ve sıcaklık modeli)       │
└─────────────────────────────────────────────────────────────┘
        │ ActuatorFeedback (72 bayt #[repr(C)])
        ▼
[Telemetri Geri Beslemesi (50 Hz Arrow Stream / MAVLink)]
```

---

## 4. TEST VE DOĞRULAMA METRİKLERİ

- **Toplam Birim ve Entegrasyon Testi:** 54
- **Başarılı Test (Passed):** 54 (%100 Başarı)
- **Başarısız Test (Failed):** 0
- **Çalışma Süresi:** ~0.15 saniye
- **Clippy / Derleyici Uyarıları:** 0 hata, 0 uyarı (`cargo check --workspace --all-targets` temiz).

---

## 5. FAZ İLERLEME DURUMU

| Faz | Tanım | Tamamlanma | Notlar |
|---|---|:---:|---|
| **Faz 0** | Workspace & C-ABI Temelleri | %100 | C-ABI struct boyutları ve padding `static_assertions` ile kilitlendi (3 test) |
| **Faz 1** | Emniyet Sentineli & Safety FSM | %100 | 7-durumlu CAS atomik FSM, watchdog, geofence, slew-rate ve flush queue (11 test) |
| **Faz 2** | HAL & Masaüstü SITL Simülasyonu | %100 | Linux openpty raw mode, MockActuator atalet modeli, 50Hz telemetri köprüsü (10 test) |
| **Faz 3** | Dora-rs IPC & Apache Arrow `/dev/shm`| %100 | DoraNodeWrapper, Arrow pool, Drop-Old kuyruğu, 1200B telemetri parçalayıcı (9 test) |
| **Faz 4** | Domain Kütüphaneleri (Guidance, Ground, Vision) | %100 | WGS84/NED, L1, MAVLink, Ackermann/Diff/İDA, Costmap, APF/DWA, SORT, IFF, Lead-Angle (21 test) |
| **Faz 5** | SDK, Takım Şablonları & Siyah Kutu | %0 | Sırada |

---

## 6. AÇIK GÖREVLER VE SIRADAKİ ADIMLAR

1. **Faz 5 Geliştirmesi:**
   - `crates/autocore-sdk`: C-ABI C++ Header (`autocore.hpp`), Python `autocore` paketi (`ctypes`/`numpy view`), Rust SDK.
   - `templates/`: Python, C++, Rust takım başlangıç şablonları.
   - `blackbox`: Dairesel ikili telemetri kayıt cihazı ve görselleştirme dışa aktarıcısı.
   - `deploy/autocore.service`: Linux systemd `SCHED_FIFO` 99 gerçek zamanlı servis dosyası.
   - Uçtan uca SITL görev simülasyonu doğrulaması.

---
*Kaynak: `crates/autocore-*`, `docs/plan/GENEL_PLAN.md`, `[[AGENTS.md]]` §7 & §8*
