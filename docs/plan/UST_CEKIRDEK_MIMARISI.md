# 🧭 AUTOCORE ÜST ÇEKİRDEK MİMARİSİ VE DOMAIN PRİMİTİFLERİ ŞARTNAMESİ

**Tarih:** 2026-10-13  
**Durum:** Mimari Tasarım & Şartname Mühürlendi (Faz 0 Hazırlığı)  
**Doküman Tipi:** Ana Sistem Mimarisi & Alana Özgü Yetkinlik Blokları Şartnamesi  
**Kapsam:** `autocore-transform`, `autocore-vision`, `autocore-pointing`, `autocore-ground`, `autocore-aerial`, `autocore-marine`, `autocore-arbiter`

---

## 1. Problem - Bağlam - Çözüm Üçlüsü

### 1.1. Problem (Mevcut Robotik Ekosisteminin Çıkmazı)
Otonom savunma ve yarışma ekipleri (HSS, İKA, İDA, İHA), her yeni projede ve her yıl sıfırdan monolitik yazılımlar yazmaya çalışmaktadır. 
- Tekerlek kinematiği, MAVLink paket baytları, kamera distorsiyon dönüşümleri veya Kalman hedef takip filtreleri her projede teker teker baştan icat edilmekte; zamanın %80'i bu alt seviye mühendislik borçlarına harcanmaktadır.
- ROS/ROS 2 tabanlı çözümler ise aşırı düğüm hantallığı, öngörülemez serileştirme gecikmeleri (DDS overhead), `tf2` ağacının işlemciyi tüketmesi ve tekil hata noktalarında (single-point-of-failure) donanım koruması sağlayamaması nedeniyle savunma ve gerçek zamanlı görev standartlarını karşılayamamaktadır.

### 1.2. Bağlam ve Temel Dinamikler
Herhangi bir otonom robot incelendiğinde, aracın "görevi" (Mission Logic) aslında toplam kodun yalnızca %5'ini oluşturur. Geriye kalan %95; fiziksel ortamın mekaniği (kara, hava, deniz), algısal uzay dönüşümleri ($SE(3)$) ve aktüatör yönlendirmesidir. 
Bir İKA'nın hedefi takip etmesi ile bir İHA'nın hedefi takip etmesi arasındaki tek fark, hareket ettikleri fiziksel uzayın kısıtlarıdır (2D düzlem vs 3D hava sahası).

### 1.3. Çözüm: "Yetenek ve Fizik Blokları" (Capability & Domain Primitives) Modeli
Araç tipine göre monolitler yazmak yerine, **7 adet bağımsız, birbiriyle sıfır kopyalama (Zero-Copy Apache Arrow & Dora-rs) ile konuşan Lego bloğu** oluşturulur.
Bir takım yeni bir robot geliştirdiğinde:
1. `autocore` taban emniyet çekirdeğini tabana koyar (Watchdog, FSM, E-Stop).
2. İhtiyaç duyduğu alan çekirdeklerini üst üste ekler (`stack` eder).
3. En üste yalnızca 20-30 satırlık stratejik görev kodunu (Python/Rust) yazar.

---

## 2. Sistem Mimarisi Şeması

```
┌────────────────────────────────────────────────────────────────────────┐
│               GÖREV KATMANI (Takımın Yazdığı Tek Kod)                  │
│       "Hedefi ara -> Kilitlen -> Yaklaş -> Hakem Onayıyla Eyleme Geç"   │
│           Python / C++20 / Rust State Machine (Dora Node)              │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Arrow IPC Event Stream (shm)
┌───────────────────────────────────┴────────────────────────────────────┐
│                    ALANA ÖZGÜ ÜST ÇEKİRDEKLER                          │
│                                                                        │
│  ┌───────────────────┐ ┌───────────────────┐ ┌──────────────────────┐  │
│  │  autocore-vision  │ │  autocore-ground  │ │   autocore-aerial    │  │
│  │ (YOLO/ByteTrack/  │ │ (Ackermann/Diff/  │ │ (MAVLink Offboard/   │  │
│  │  Depth Projection)│ │  Costmap2D/DWA)   │ │  3D Envelope/VTOL)   │  │
│  └─────────┬─────────┘ └─────────┬─────────┘ └──────────┬───────────┘  │
│            │                     │                      │              │
│  ┌─────────▼─────────┐ ┌─────────▼─────────┐ ┌──────────▼───────────┐  │
│  │ autocore-pointing │ │  autocore-marine  │ │  autocore-transform  │  │
│  │ (Lead-Angle / IK /│ │ (Thruster Alloc / │ │ (SE(3) Kinematik Ağaç│  │
│  │  Latency Comp)    │ │  Akıntı Dengeleme)│ │  WGS84<->NED Motoru) │  │
│  └─────────┬─────────┘ └─────────┬─────────┘ └──────────┬───────────┘  │
│            │                     │                      │              │
│            └─────────────────────┼──────────────────────┘              │
│                                  ▼                                     │
│                     ┌──────────────────────────┐                       │
│                     │     autocore-arbiter     │                       │
│                     │ (3 Seviyeli Komut Hakemi)│                       │
│                     └────────────┬─────────────┘                       │
└──────────────────────────────────┼─────────────────────────────────────┘
                                   │ Arbitrated Twist & Control Packet
┌──────────────────────────────────▼─────────────────────────────────────┐
│                    autocore (Taban Emniyet & HAL)                      │
│     dora-rs / Arrow Zero-Copy / 500ms Watchdog / 7-State CAS FSM       │
│     IEEE 754 NaN Süzme / E-Stop Anti-Phantom Flush / PTY-CAN SITL      │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Alanlarına Göre 7 Üst Çekirdek Matrisi

### 3.1. `autocore-transform` (Koordinat Ağacı & Küresel Seyrüsefer)
- **Rolü:** Sensörler, kameralar, eklemler, taretler ve küresel GNSS koordinatları arasındaki uzaysal bağıntıları ($SE(3)$ Lie Grubu) mikrosaniye hassasiyetinde çözen omurga.
- **Bileşenler:**
  - `TransformTree`: Kilit-serbest `ArcSwap` tabanlı $4\times 4$ homojen dönüşüm ağacı (`base_link`, `camera_link`, `turret_link`, `odom`, `map`).
  - `GeoConverter`: WGS84 $\longleftrightarrow$ ECEF $\longleftrightarrow$ NED/ENU elipsoit dönüşümleri ($<1\text{ mm}$ hassasiyet).
  - `FrameProjector`: Kamera optik çerçevesini ($Z$ ileri, $X$ sağ, $Y$ aşağı) robot mekanik çerçevesine ($X$ ileri, $Y$ sol, $Z$ yukarı) kopyalamasız dönüştürme.
- **SITL İkizi:** `mock_transform_broadcaster` (`transforms.toml` dosyasından okunan sanal kinematik zincir).

### 3.2. `autocore-vision` (Algılama, Çıkarım & Çoklu Hedef Takip)
- **Rolü:** Platformdan bağımsız yapay zekâ nesne tespiti, zamansal takip ve 3B uzaysal konum kestirimi.
- **Bileşenler:**
  - `InferenceEngine`: ONNX Runtime / TensorRT C-API soyutlaması, GPU/NPU çıkarımı, sıfır-yığın (zero-heap) bellek.
  - `SpatialTracker`: ByteTrack / SORT türevi 8-DoF Kalman durum kestiricisi ($x, y, a, h, \dot{x}, \dot{y}, \dot{a}, \dot{h}$) ve hedef kaybolduğunda yörüngeyi sürdürme (Coasting).
  - `DepthProjector`: Kamera iç matrisi ($K$) ve derinlik/stereo verisiyle 2D kutudan 3B metre vektörüne ($\vec{P}_c = [x, y, z]$) izdüşüm.
  - `HypothesisFilter`: HSV bantları ve çoğunluk oylaması (Majority Voting) histerezisiyle optik parlamaları ve yanlış pozitifleri eleme.
- **SITL İkizi:** `synthetic_camera_feed` (Video dosyasından veya sentetik BBox üretecinden 50 FPS veri akışı).

### 3.3. `autocore-pointing` (Yönlendirme, Taret, Gimbal & Angajman)
- **Rolü:** Taret, gimbal, yönlendirilebilir anten ve robot kollarının 2-DoF / 3-DoF açısal yönelim matematiği ve atış disiplini.
- **Bileşenler:**
  - `LatencyCompensator`: Sensör gecikmesini hedefin hız vektörüyle kompanse eden ölü hesaplama:
    $$\vec{P}_{\text{gerçek}} = \vec{P}_{\text{tespit}} + \vec{V}_{\text{hedef}} \cdot (t_{\text{eylem}} - t_{\text{kamera}})$$
  - `LeadAngleSolver`: Hedef mesafesi ($d$), mermi hızı ($v_p$) ve yerçekimi düşüşü ile namlunun bakması gereken gelecekteki açıyı çözen 2. derece kuadratik balistik motor.
  - `InverseKinematics2DoF`: Kartezyen hedeften ($X,Y,Z$) Azimut ve Elevasyon açı/hızlarına dönüşüm.
  - `ActionDiscipline`: Hedef tolerans penceresine girmeden ($\le \epsilon$), mekanik salınım sönümlenmeden ve soğuma dolmadan eylem izni vermeyen güvenlik mandalı (`fire_gate`).
- **SITL İkizi:** `mock_pan_tilt_rig` (Linux PTY üzerinden 2-DoF servo atalet simülasyonu ve vuruş doğrulama raporu).

### 3.4. `autocore-ground` (Zemin Seyrüsefer & Şasi Kinematiği)
- **Rolü:** Tekerlekli, paletli, omni veya Ackermann kara araçlarının engelsiz seyrüseferi.
- **Bileşenler:**
  - `ChassisKinematics`: Twist komutunu diferansiyel, Ackermann, Mecanum veya Skid-Steer tekerlek hızlarına çevirme.
  - `Costmap2D`: LiDAR ve ultrasonik engelleri içeren BFS üstel sönümlü yerel ızgara haritası.
  - `DwaPlanner & ApfSolver`: Dinamik Pencere Yaklaşımı ve Yapay Potansiyel Alanlar ile reaktif engel kaçınma.
  - `PathTracker`: Pure Pursuit ve Stanley yol takip kontrolcüleri.
  - `GroundEstimator`: Tekerlek enkoderleri ve 9-eksenli IMU füzyonlu EKF zemin odometrisi.
- **SITL İkizi:** `mock_ground_chassis` (Sanal CAN `vcan0` ve PTY üzerinden kayma dinamikli odometri).

### 3.5. `autocore-aerial` (Uçuş Dinamiği, Otopilot & 3D Güdüm)
- **Rolü:** Döner kanat, sabit kanat ve VTOL İHA'ların 3D seyrüseferi ve otopilot entegrasyonu.
- **Bileşenler:**
  - `MavlinkBridge`: Pixhawk/PX4/ArduPilot otopilotlarıyla asenkron MAVLink v2 `SET_POSITION_TARGET_LOCAL_NED` iletişimi.
  - `L1Guidance`: Park et al. doğrusal olmayan L1 yanal ivme kılavuzu.
  - `FlightEnvelopeGuard`: Stall önleme, hücum açısı (AoA), maksimum yatış açısı ve tırmanma limitleri koruyucusu.
  - `AirGeofence`: 3D irtifa ve poligon sınır ihlalinde otopilota otomatik RTL/Loiter emri.
  - `VtolTransitionController`: Dikey kalkıştan yatay uçuşa aerodinamik itki/tilt transfer durum makinesi.
- **SITL İkizi:** `mock_mavlink_vehicle` (TCP `127.0.0.1:5760` portundan sanal MAVLink telemetrisi ve kalp atışı).

### 3.6. `autocore-marine` (Hidrodinamik Seyrüsefer & İtki Dağıtımı)
- **Rolü:** İDA (su üstü) ve UUV (su altı) platformlarının akıntıya dirençli seyrüseferi.
- **Bileşenler:**
  - `ThrusterAllocationMatrix`: Genel $[F_x, F_y, M_z]^T$ kuvvet talebini çoklu pervane/dümen PWM sinyallerine dönüştürme.
  - `DriftCompensator`: Akıntı ve dalga sapmalarını integral süzgeciyle kompanse eden pruva yengeç açısı (Crab Angle) kontrolü.
  - `MarineNavigator`: Şamandıra dönüşleri ve koridor takibinde kavitasyon ve devrilme önleyici yumuşak profiller.
- **SITL İkizi:** `mock_marine_simulator` (Dalga gürültüsü ve hidrodinamik dirençli sanal döngü).

### 3.7. `autocore-arbiter` (Komut Hakemi & Öncelik Yöneticisi)
- **Rolü:** Tüm üst çekirdeklerden ve kullanıcı görev kodundan aktüatörlere giden komutları toplayıp öncelik çakışmalarını çözen merkezi hakem.
- **Öncelik Seviyeleri (Strict Priority Tiers):**
  - **Tier 1 (Mutlak Veto):** `autocore` Taban Emniyeti (Donanımsal E-Stop, 500ms Watchdog SafeLock, Geofence Inhibit) $\implies$ Anında nötr fren (`0.0 m/s`).
  - **Tier 2 (Reaktif Güvenlik):** `autocore-ground` / `autocore-aerial` Engel Kaçınma $\implies$ Kullanıcı hedefini maskeler, kaçınma manevrası yapar.
  - **Tier 3 (Görev Katmanı):** Takımın Python/Rust görev kodu $\implies$ Sadece ortam güvenli ve engelsizse yürütülür.

---

## 4. Çekirdekler Arası Apache Arrow & C-ABI Veri Sözleşmesi

Tüm çekirdekler bellek üzerinde kopyalamasız (zero-copy) çalışır ve C-ABI uyumludur:

| Çekirdek | Ürettiği Veri Akışı | Veri Tipi / Şeması | Boyut / Protokol |
| :--- | :--- | :--- | :--- |
| `autocore-transform` | `local_pose` | `[x, y, z, qw, qx, qy, qz, vx, vy, vz, timestamp_us]` | 48B C-ABI Struct |
| `autocore-vision` | `detected_tracks` | `Arrow RecordBatch: [id, class, conf, x, y, z, vx, vy, vz, t_us]` | Arrow Zero-Copy |
| `autocore-pointing` | `actuator_setpoints`| `[pan_rad: f32, tilt_rad: f32, pan_rate: f32, tilt_rate: f32, fire_gate: u8]` | 20B C-ABI Struct |
| `autocore-ground` | `drive_twist` | `KinematicCommand [linear_x, linear_y, linear_z, angular_x..]` | 32B C-ABI Struct |
| `autocore-aerial` | `autopilot_setpoints`| `[ned_vx, ned_vy, ned_vz, yaw_rate, z_accel, mode_flags]` | 28B MAVLink Arrow |
| `autocore-marine` | `thruster_signals` | `[t1_pct: f32, t2_pct: f32, rudder_rad: f32, mode: u8]` | 16B C-ABI Struct |
| `autocore-arbiter` | `arbitrated_cmd` | `KinematicCommand + active_tier: u8 + reason_code: u8` | 36B C-ABI Struct |

---

## 5. Doğrulama Kontrol Listesi

- [ ] `autocore-transform`: WGS84 $\longleftrightarrow$ NED dönüşüm doğruluğu ($<1\text{ mm}$ tolerans) — `cargo test -p autocore-transform test_geo_conversions`
- [ ] `autocore-transform`: $SE(3)$ homojen dönüşüm ve kamera optik $\to$ mekanik paralaks doğrulaması — `cargo test -p autocore-transform test_optical_to_mechanical_projection`
- [ ] `autocore-vision`: Çoklu nesne takibi ID sürekliliği ve Coasting yörünge testi — `cargo test -p autocore-vision test_spatial_tracker_coasting`
- [ ] `autocore-vision`: Kamera iç matrisi ile 2D pikselden 3B metre derinlik izdüşümü — `cargo test -p autocore-vision test_depth_projection_accuracy`
- [ ] `autocore-pointing`: Pipeline gecikme telafisi (Dead Reckoning) ekstrapolasyon doğruluğu — `cargo test -p autocore-pointing test_latency_compensation`
- [ ] `autocore-pointing`: Mach 3.5 hipersonik ve yerçekimi balistik önleme çözümü — `cargo test -p autocore-pointing test_lead_angle_ballistics`
- [ ] `autocore-pointing`: Salınım sönümlenmeden ve tolerans dışındayken `fire_gate` engeli — `cargo test -p autocore-pointing test_action_discipline_latch`
- [ ] `autocore-ground`: Ackermann ve Mecanum çoklu şasi hız ayrıştırma doğruluğu — `cargo test -p autocore-ground test_chassis_kinematics_multimode`
- [ ] `autocore-ground`: DWA ve APF reaktif engel kaçınma manevrası — `cargo test -p autocore-ground test_obstacle_avoidance_evasion`
- [ ] `autocore-aerial`: MAVLink v2 `SET_POSITION_TARGET_LOCAL_NED` paket kodlama/çözme — `cargo test -p autocore-aerial test_mavlink_offboard_stream`
- [ ] `autocore-aerial`: 3D hava sahası kafesi ihlalinde otomatik RTL geçişi — `cargo test -p autocore-aerial test_air_geofence_rtl_trigger`
- [ ] `autocore-marine`: Çift pervane ve dümen hidrodinamik itki tahsis matrisi — `cargo test -p autocore-marine test_thruster_allocation_matrix`
- [ ] `autocore-marine`: Akıntı kaynaklı sapmada yengeç açısı (Crab Angle) kompanzasyonu — `cargo test -p autocore-marine test_drift_compensation_crabbing`
- [ ] `autocore-arbiter`: Tier 1 (E-Stop/SafeLock) anında Tier 3 kullanıcı komutunun kesilmesi — `cargo test -p autocore-arbiter test_tier1_veto_preemption`
- [ ] `autocore-arbiter`: Tier 2 (Engel Kaçınma) anında Tier 3 kullanıcı hedefinin maskelenmesi — `cargo test -p autocore-arbiter test_tier2_safety_override`
