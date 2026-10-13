# 🧭 AUTOCORE GENEL MİMARİ PLANI — Dora-rs & Apache Arrow Sıfır-Kopyalama Çekirdeği

**Tarih:** 13.10.2026  
**Durum:** Faz 0, Faz 1 ve Faz 2 %100 TAMAMLANDI VE YEŞİL — Faz 3 Başlıyor  
**Kaynak:** `/home/yavuz/SecondBrain/docs/ARCHITECTURE-AUTOCORE.md` + `cekirdek.mimari.pdf`  
**Proje:** Autocore Autonomous Robotics Runtime & Safety Kernel (`autocore`)

---

## 1. Goal (Hedef)

Savunma sanayii ve otonom sistem ekiplerindeki (HSS, İKA, İDA, İHA, Robotik) tüm alt takımların tekeri yeniden icat etmeden kullanabileceği; Unix felsefesiyle izole, Apache Arrow ile sıfır kopyalama (`Zero-Copy`), bağımsız donanımsal/yazılımsal emniyet bekçili (`Sentinel Watchdog`) ve donanım bağımsız masaüstü simülasyonlu (`SITL`) endüstriyel standartta bir robotik işletim çekirdeği kurmak.

---

## 2. Success Criteria (Başarı Kriterleri)

* **Sıfır Kopyalama IPC:** Apache Arrow ve `/dev/shm` üzerinden mikrosaniyelik veri aktarımı. Python tarafında `numpy.ndarray` referansı (`view`) ile **0 ms** gecikme.
* **Fail-Safe Emniyet Çekirdeği:** 7 Durumlu CAS Atomic Safety FSM, 500 ms Sentinel Watchdog zaman aşımı, Geofence Clamping, Slew-Rate Limiter ve Hayalet Hareket Koruması (`Phantom Movement Fix`).
* **Day-1 SITL (Masaüstü Tezgâhı):** Fiziksel robot olmadan Linux Sanal Terminali (`PTY`), sanal CAN (`vcan`) ve atalet modelli `MockActuator` ile %100 uçtan uca simülasyon.
* **C-ABI Tip Güvenliği:** Tüm paylaşılan struct'larda `#[repr(C)]` ve `static_assertions` ile derleme zamanı bellek hizalama ve boyut garantisi.
* **Ölçülebilir Doğrulama (T1):** Her fazın altında net komutlarla doğrulanabilir "T1 — Tamam Tanımı" bulunması; `cargo test --workspace` ile sürekli yeşil hat.

---

## 3. Context And Current Facts (Mevcut Durum ve Olgular)

* **Workspace:** `/home/yavuz/Belgeler/autocore` altında Cargo Workspace hazır.
* **Faz 0 (Workspace & Base):** `crates/autocore-base` crate'i tamamlandı. `KinematicCommand` (32B), `StateVector6DoF` (64B), `ActuatorFeedback` (72B) C-ABI struct'ları mühürlendi.
* **Faz 1 (Safety Core):** `crates/autocore-safety` crate'i tamamlandı. 7-durumlu atomik FSM, 500ms watchdog, geofence clamping, slew-rate limiter ve lock-free queue flush kanıtlandı.
* **Faz 2 (HAL & SITL):** `crates/autocore-hal` ve `crates/autocore-sitl` tamamlandı. Linux `libc::openpty` raw mode sanal seri port, SocketCAN/vcan entegrasyonu, 50Hz telemetri loopback köprüsü ve atalet simülasyonu çalışıyor.
* **Test Durumu:** 24/24 test %100 yeşil (0 hata, 0 uyarı).

---

## 4. Constraints And Non-goals (Kısıtlar ve Kapsam Dışı)

* **Kısıt:** Emniyet katmanı ve HAL kodları asla blocking olamaz; `async` veya atomik/kilit-serbest çalışmak zorundadır.
* **Kısıt:** Gömülü bilgisayarlarda (Raspberry Pi 5 / Jetson Orin) `SCHED_FIFO` gerçek zamanlı öncelik ve CPU Affinity (çekirdek kilitleme) uygulanmalıdır.
* **Non-goal (Başlangıç):** Fiziksel motor sürücü tasarımı (donanım tasarımı alt takımlara aittir; çekirdek standart UART/CAN protokolleri sunar).
* **Non-goal:** Ağır 3D fizik simülatörü (Gazebo vb.) çalıştırmak; bunun yerine mikro saniyelik, hafif ve CI uyumlu Linux PTY/vcan SITL tezgâhı esastır.

---

## 5. Key Decisions (Temel Mimari Kararlar)

| Karar | Seçilen | Neden | Reddedilen Alternatif |
|---|---|---|---|
| **1. Dil ve Çalışma Zamanı** | **Rust (Cargo Workspace)** | Bellek güvenliği, sıfır-maliyetli soyutlama, deterministik mikro-saniye gecikme, C-ABI uyumu | C++ (bellek sızıntısı ve segfault riski) / Saf Python (GIL ve gecikme tavanı) |
| **2. Dağıtık Middleware** | **dora-rs & Apache Arrow** | Bildirimsel DAG veri akışı, paylaşılan bellek (`/dev/shm`), Python `numpy`'a 0 ms sıfır kopyalama | ROS 2 (ağır DDS serileştirme yükü, karmaşık kurulum) |
| **3. Simülasyon Stratejisi** | **Native Linux PTY & Virtual CAN SITL** | Gazebo/ROS gerektirmeden CI ve masaüstünde 0.1 saniyede uçtan uca çalışabilirlik | Gazebo/Unity (GPU bağımlılığı, ağır kurulum) |
| **4. Emniyet Mimarisi** | **Atomic CAS FSM + Bağımsız Watchdog** | Üst katman görev yazılımı kilitlense bile donanımı 500 ms içinde nötr frene kitleme garantisi | Tek thread yazılımsal zamanlayıcı |
| **5. E-Stop Hareket Yönetimi** | **Lock-Free Queue Flush (Phantom Fix)** | E-Stop basıldığında kuyrukta bekleyen komutların temizlenmesi; kilit kalkınca fırlamayı önleme | Kuyrukta bekletme (tehlikeli sıçrama) |

---

## 6. Sistem Mimarisi ve Modüler Dağılım

```
autocore/
├── Cargo.toml                      # Root Workspace
├── crates/
│   ├── autocore-base/              # Çekirdek tipler, C-ABI struct'lar, mikrosaniye saat (FAZ 0 - OK)
│   ├── autocore-safety/            # 7-Durumlu Safety FSM, Watchdog, Slew-Rate, Clamping (FAZ 1 - OK)
│   ├── autocore-hal/               # ActuatorPort trait, CRC-16 seri framing, CanFrame (FAZ 2 - OK)
│   ├── autocore-sitl/              # Linux PTY sanal terminal, MockActuator, 50Hz köprü (FAZ 2 - OK)
│   ├── autocore-ipc/               # Dora-rs düğüm sarmalayıcı, Arrow shm, Drop-Old (FAZ 3 - SIRADA)
│   ├── autocore-guidance/          # 6-DoF Durum Kestirimi, WGS84->NED, L1/Pure Pursuit (FAZ 4)
│   ├── autocore-ground/            # Diferansiyel/Ackerman kinematiği, 2D Local Costmap (FAZ 4)
│   ├── autocore-vision/            # ONNX Runtime soyutlaması, SORT/Kalman Takip, Dinamik IFF (FAZ 4)
│   └── autocore-sdk/               # C-ABI C++ başlıkları, Python ctypes/cffi köprüleri (FAZ 5)
├── templates/                      # starter-python, starter-cpp, starter-rust (FAZ 5)
├── flows/                          # Dora veri akış çizgeleri (dataflow-sitl.yml)
└── docs/                           # HukukAI standardında tam dokümantasyon ağacı
```

---

## 7. Detaylı Faz Yol Haritası

### 🔹 Faz 0: Çalışma Alanı, Cargo Workspace ve C-ABI Temelleri (TAMAMLANDI)
- `autocore-base` paketi kuruldu.
- `KinematicCommand` (32 bayt), `StateVector6DoF` (64 bayt), `ActuatorFeedback` (72 bayt) struct'ları `#[repr(C)]`, `bytemuck::Pod` ve `bytemuck::Zeroable` olarak yazıldı.
- `static_assertions` ile derleme zamanında boyut ve hizalamalar mühürlendi.
- **T1 Kanıtı:** `cargo test -p autocore-base` → 3/3 test PASSED.

### 🔹 Faz 1: Emniyet Sentineli, FSM ve Bekçi Çekirdeği (TAMAMLANDI)
- `autocore-safety` paketi kuruldu.
- 7 Durumlu CAS Atomic FSM (`Booting`..`SafeLock`).
- 500 ms zaman aşımlı bağımsız `SentinelWatchdog`.
- `ClampingEngine`: `geo` poligon geofence sınır denetimi ve eksen kelepçeleme.
- `SlewRateLimiter`: $\Delta v/\Delta t$ ivme sınırlandırması ve ani yönelim yumuşatması.
- `LockFreeFlushQueue`: Hayalet hareket önleyici atomik kuyruk temizleme.
- **T1 Kanıtı:** `cargo test -p autocore-safety` → 11/11 test PASSED.

### 🔹 Faz 2: HAL ve Masaüstü SITL Simülasyon Tezgâhı (TAMAMLANDI)
- `autocore-hal` ve `autocore-sitl` paketleri kuruldu.
- `ActuatorPort` asenkron trait'i ve `0xAA 0x55` .. CRC16 .. `0x0D 0x0A` ikili protokol çerçeveleyici.
- Linux `libc::openpty` tabanlı, `cfmakeraw` ile ikili uyumlu sanal seri terminal çifti (`VirtualPtyPair`).
- $\tau = 80\text{ ms}$ fiziksel atalet ve ısıl modelli `MockActuator`.
- 50 Hz telemetri geri besleme döngülü `SitlSerialBridge`.
- `VirtualCanBus` ve `CanFrame` desteği.
- **T1 Kanıtı:** `cargo test -p autocore-hal -p autocore-sitl` → 10/10 test PASSED. (Toplam: 24/24).

### 🔹 Faz 3: Dora-rs Veri Akış Çizgesi & Apache Arrow Sıfır Kopyalama IPC (TAMAMLANDI)
- `autocore-ipc` paketinin açılması.
- `dora-node-api v1.1.0` entegrasyonu: `DoraNodeWrapper` (`DoraCoordinator` ve `Standalone SITL` modları) ve asenkron `EventStream` (`Event::Input`, `Event::Stop`) yaşam döngüsü.
- Apache Arrow v59 sabit boyutlu halka havuzu (`SharedRingBufferPool`).
- Drop-Old bellek kiralama politikası (`DropOldMailbox`, `DropOldLeaseQueue`).
- 1200 baytlık GCS telemetri parçalama ve sıfır kopyalama ile yeniden birleştirme (`PacketFragmenter`, `PacketReassembler`).
- `flows/dataflow-sitl.yml` örnek veri akış çizgesi.
- **T1 Kanıtı:** `cargo test -p autocore-ipc` → 9/9 test PASSED.

### 🔹 Faz 4: Alana Özgü Domain Kütüphaneleri (TAMAMLANDI)
- `autocore-guidance`: 6-DoF Durum Kestirimi, WGS84 $\longleftrightarrow$ ECEF $\longleftrightarrow$ NED dönüşümleri, Park et al. L1 adaptif yol takibi, MAVLink v2 köprüsü.
- `autocore-ground`: Diferansiyel ve Ackerman kinematiği (slip faktörlü), İDA çift itici güç tahsis matrisi, 2D Local Costmap (BFS üstel şişirme), Yapay Potansiyel Alanlar (APF) ve DWA engel kaçınma.
- `autocore-vision`: `BoundingBox` IoU hesabı, SORT Kalman çoklu hedef takipçisi, Dinamik IFF (`min_samples` eşikli histerezis) ve Lead-Angle Solver (2. derece önleme geometrisi).
- **T1 Kanıtı:** `cargo test -p autocore-guidance -p autocore-ground -p autocore-vision` → 21/21 test PASSED.

### 🔹 Faz 5: SDK, Takım Başlangıç Şablonları ve Siyah Kutu Teşhis Motoru (TAMAMLANDI)
- `autocore-sdk`: Rust SDK (`AutocoreClient`), C++20 Header-Only C-ABI sarmalayıcısı (`autocore.hpp`), Python `autocore` paketi (`ctypes`/`numpy view` 0 ms).
- `templates/`: `starter-python`, `starter-cpp`, `starter-rust` şablonları.
- `blackbox`: Dairesel ikili log tamponu (184B `BlackboxFrame`, 40B `BlackboxHeader`, CRC-16 korumalı), CSV ve JSON dışa aktarımı.
- Linux systemd servis tanımı (`deploy/autocore.service`) ve `SCHED_FIFO` gerçek zamanlı önceliklendirme rehberi (`deploy/deploy-guide.md`).
- **T1 Kanıtı:** `python3 templates/starter-python/mission.py --sitl` ile 5 yol noktası uçtan uca simüle edildi (815 adım, 16.99s, %100 başarı). `cargo run -p starter-rust` ile 100 adımlık görev ve siyah kutu mühürlendi.

---

## 8. Doğrulama Kontrol Listesi

- [x] Workspace derleme doğrulaması — `cargo check --workspace --all-targets` (0 hata)
- [x] C-ABI struct bellek boyutu ve hizalama testi — `cargo test -p autocore-base test_abi_sizes`
- [x] 500 ms Watchdog zaman aşımı ve SafeLock tetikleme — `cargo test -p autocore-safety test_watchdog_timeout`
- [x] Geofence çokgen dışı koordinat kırpma (Clamping) — `cargo test -p autocore-safety test_geofence_clamping`
- [x] Slew-rate ivme yumuşatma doğrulaması — `cargo test -p autocore-safety test_slew_rate_limiting`
- [x] E-Stop lock-free kuyruk temizleme (Phantom Fix) — `cargo test -p autocore-safety test_queue_flush_on_estop`
- [x] Sanal Seri Port (PTY) loopback iletişimi — `cargo test -p autocore-sitl test_virtual_pty_read_write`
- [x] Virtual CAN paket aktarımı — `cargo test -p autocore-sitl test_virtual_can_loopback`
- [x] Mock aktüatör kinematik geri besleme ve atalet modeli — `cargo test -p autocore-sitl test_sitl_bridge_end_to_end`
- [x] Apache Arrow `/dev/shm` sıfır kopyalama bellek aktarımı — `cargo test -p autocore-ipc test_arrow_kinematic_roundtrip`
- [x] Drop-Old yavaş tüketici tampon politikası — `cargo test -p autocore-ipc test_mailbox_drop_old`
- [x] MTU 1200 bayt telemetri parçalama ve birleştirme — `cargo test -p autocore-ipc test_packet_fragmentation_and_reassembly`
- [x] Diferansiyel sürüş sağ/sol tekerlek kinematik hesabı — `cargo test -p autocore-ground test_diff_drive_roundtrip`
- [x] WGS84 $\rightarrow$ NED koordinat dönüşüm doğruluğu — `cargo test -p autocore-guidance test_geodetic_to_ned_and_inverse`
- [x] SORT Tracker çoklu nesne eşleme doğruluğu — `cargo test -p autocore-vision test_tracker_lifecycle_and_confirmation`
- [x] Python şablonu ile SITL otonom görev koşusu — `python3 templates/starter-python/mission.py --sitl`

---
*Kaynak: `/home/yavuz/SecondBrain/docs/ARCHITECTURE-AUTOCORE.md`, `AGENTS.md` §7 & §8*
