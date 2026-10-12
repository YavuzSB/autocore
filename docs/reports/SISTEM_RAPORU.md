# 🔍 AUTOCORE SİSTEM DURUM RAPORU

**Tarih:** 13.10.2026 (Faz 0, Faz 1 ve Faz 2 tamamlanması sonrası)  
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
| **`autocore-ipc`** | Dora-rs Aktör Sarmalayıcısı, Apache Arrow `/dev/shm`, Drop-Old | ⏳ Sırada (Faz 3) | Planlandı | Sıfır kopyalama bellek aktarımı |
| **`autocore-guidance`**| 6-DoF Durum Kestirimi, WGS84 $\rightarrow$ NED, L1/Pure Pursuit | ⏳ Bekliyor (Faz 4)| Planlandı | Kuaterniyon matematiği, MAVLink köprüsü |
| **`autocore-ground`**  | Diferansiyel & Ackerman Kinematiği, 2D Costmap, APF/DWA | ⏳ Bekliyor (Faz 4)| Planlandı | Engelden kaçınma, yerel yol planlama |
| **`autocore-vision`**  | ONNX Runtime, SORT Tracker, Dinamik IFF, Lead-Angle | ⏳ Bekliyor (Faz 4)| Planlandı | Hedef angajmanı |
| **`autocore-sdk`**     | C-ABI C++ Header, Python ctypes/numpy view köprüsü | ⏳ Bekliyor (Faz 5)| Planlandı | Takım SDK ve şablonları |

---

## 3. MEVCUT KONTROL VE VERİ AKIŞI

```
[Görev/Kullanıcı Komutu]
        │
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
[Telemetri Geri Beslemesi (50 Hz)]
```

---

## 4. TEST VE DOĞRULAMA METRİKLERİ

- **Toplam Birim ve Entegrasyon Testi:** 24
- **Başarılı Test (Passed):** 24 (%100 Başarı)
- **Başarısız Test (Failed):** 0
- **Çalışma Süresi:** 0.10 saniye
- **Clippy / Derleyici Uyarıları:** 0 hata, 0 uyarı (`cargo check --workspace --all-targets` temiz).

---

## 5. FAZ İLERLEME DURUMU

| Faz | Tanım | Tamamlanma | Notlar |
|---|---|:---:|---|
| **Faz 0** | Workspace & C-ABI Temelleri | %100 | C-ABI struct boyutları ve padding `static_assertions` ile kilitlendi |
| **Faz 1** | Emniyet Sentineli & Safety FSM | %100 | 7-durumlu CAS atomik FSM, watchdog, geofence, slew-rate ve flush queue |
| **Faz 2** | HAL & Masaüstü SITL Simülasyonu | %100 | Linux openpty raw mode, MockActuator atalet modeli, 50Hz telemetri köprüsü |
| **Faz 3** | Dora-rs IPC & Apache Arrow `/dev/shm`| %0 | Sıradaki iş: `autocore-ipc` paketi, Arrow shm, Drop-Old politikası |
| **Faz 4** | Domain Kütüphaneleri (Guidance, Ground, Vision) | %0 | Planlandı |
| **Faz 5** | SDK, Takım Şablonları & Siyah Kutu | %0 | Planlandı |

---

## 6. AÇIK GÖREVLER VE SIRADAKİ ADIMLAR

1. **Faz 3 Başlangıcı:**
   - `crates/autocore-ipc/Cargo.toml` ve `src/lib.rs` oluşturulması.
   - `dora-node-api` entegrasyonu ile asenkron `DoraNodeWrapper` yazımı.
   - `/dev/shm` üzerinde Apache Arrow RecordBatch tahsisatçısı (`Shared Ring Buffer`).
   - `Drop-Old` kiralama mekanizmasının entegrasyonu.
   - GCS telemetri için 1200 baytlık paket parçalayıcı ve birleştirici (`Reassembly Buffer`).
   - `flows/dataflow-sitl.yml` akış çizgesinin yazımı.

---
*Kaynak: `crates/autocore-*`, `docs/plan/GENEL_PLAN.md`, `[[AGENTS.md]]` §7 & §8*
