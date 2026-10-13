<div align="center">

# ⚡ AUTOCORE
### Yüksek Başarımlı Otonom Robotik Çalışma Zamanı ve Emniyet Çekirdeği
**Sıfır-Kopyalama IPC • Apache Arrow • Dora-rs • MIL-STD-882E & DO-178C Uyumlu**

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Testler](https://img.shields.io/badge/testler-70%2F70_ge%C3%A7ti-brightgreen.svg?style=flat-square)](docs/reports/SAVUNMA_SANAYII_TEST_RAPORU.md)
[![Savunma Standartları](https://img.shields.io/badge/standartlar-MIL--STD--882E_%7C_DO--178C_%7C_STANAG_4586-blue.svg?style=flat-square)](docs/reports/SAVUNMA_SANAYII_TEST_RAPORU.md)
[![IPC](https://img.shields.io/badge/IPC-Apache_Arrow_v59_%7C_Dora--rs_v1.1-purple.svg?style=flat-square)](crates/autocore-ipc)
[![Gerçek Zamanlı](https://img.shields.io/badge/ger%C3%A7ek__zaman-SCHED__FIFO_99-red.svg?style=flat-square)](deploy/autocore.service)
[![Lisans](https://img.shields.io/badge/lisans-MIT%20%2F%20Apache--2.0-lightgrey.svg?style=flat-square)](LICENSE)

<p align="center">
  <a href="README.md">English</a> • <b>Türkçe</b>
</p>

<p align="center">
  <a href="#-genel-bakış">Genel Bakış</a> •
  <a href="#-sistem-mimarisi">Sistem Mimarisi</a> •
  <a href="#-temel-mimari-sütunlar">Temel Sütunlar</a> •
  <a href="#-savunma-sanayii-standartları-ve-test-matrisi">Savunma Standartları</a> •
  <a href="#-hızlı-başlangıç">Hızlı Başlangıç</a> •
  <a href="#-crate-ve-modüler-yapı">Crate Yapısı</a> •
  <a href="#-çok-dilli-sdk">SDK Kullanımı</a> •
  <a href="#-karatutu-uçuş-veri-kaydedici-blackbox">Karatutu</a>
</p>

---

</div>

## 📌 Genel Bakış

**AutoCore**, Kara (**İKA**), Deniz Üstü (**İDA**), Hava (**İHA**) ve Hava Savunma (**HSS**) platformlarında görev yapan kritik otonom robotik sistemler için geliştirilmiş, endüstriyel standartta bir **Robotik Çalışma Zamanı (Runtime) ve Emniyet Çekirdeğidir (Safety Kernel)**.

%100 bellek güvenli (memory-safe) Rust dili ve C-ABI bellek kilitleri ile inşa edilen AutoCore; geleneksel ROS ve ROS 2 mimarilerinin getirdiği yüksek serileştirme gecikmelerini, tek hata noktası (single-point-of-failure) kırılganlıklarını ve ağır sistem yükünü ortadan kaldırır. Bünyesinde **7-durumlu kilit-serbest atomik Safety FSM**, donanımsal **500 ms Sentinel Watchdog** ve **Apache Arrow & Dora-rs** destekli sıfır kopyalama paylaşılan bellek veri yolu barındırır.

```
               ┌────────────────────────────────────────────────────────┐
               │              GÖREV VE OTONOMİ UYGULAMA KATMANI         │
               │        Python (NumPy 0-kopyalama) • C++20 • Rust       │
               └───────────────────────────▲────────────────────────────┘
                                           │ Sıfır Kopyalama Paylaşılan Bellek (/dev/shm)
               ┌───────────────────────────▼────────────────────────────┐
               │             DAĞITIK VERİ AKIŞI ÇALIŞMA ZAMANI          │
               │   Dora-rs Event Stream • Apache Arrow v59 Halka Havuz  │
               │        Drop-Old Gerçek Zaman Kuyruk • 1200B MTU        │
               └───────────────────────────▲────────────────────────────┘
                                           │ Katı Doğrulanmış C-ABI Veri Yolu
               ┌───────────────────────────▼────────────────────────────┐
               │             SERTLEŞTİRİLMİŞ EMNİYET BEKÇİ ÇEKİRDEĞİ    │
               │    7-Durumlu Atomik CAS FSM • 500ms Donanım Watchdog   │
               │   IEEE 754 NaN/Inf Filtresi • Geofence Kelepçeleme     │
               │   Slew-Rate İvme Sınırlayıcı • Hayalet Hareket Koruması│
               └───────────────────────────▲────────────────────────────┘
                                           │ Ham PTY / SocketCAN
               ┌───────────────────────────▼────────────────────────────┐
               │          DONANIM SOYUTLAMA & DAY-1 YEREL LINUX SITL    │
               │     Linux openpty Ham Seri Port • Sanal CAN (vcan0)    │
               │  Isıl ve Atalet Modelli Mock Aktüatör (τ=80ms geribesleme)│
               └────────────────────────────────────────────────────────┘
```

---

## 🚀 Temel Mimari Sütunlar

### 1. Sıfır-Kopyalama Paylaşılan Bellek IPC (Apache Arrow & Dora-rs)
- 50–500 Hz yüksek hızlı telemetri ve kinematik komutlar ağ serileştirmesine girmeden taşınır.
- `/dev/shm` üzerindeki sabit boyutlu dairesel bellek havuzları sayesinde Python uygulamaları ham belleği doğrudan `numpy.ndarray` görünümüne (`view`) bağlar (**0.0 ms kopyalama gecikmesi**).
- Gerçek zamanlı `Drop-Old` kiralama mantığı ile yavaş tüketiciler kuyrukta bayatlayan veriyi atlayarak daima en güncel robot durumunu işler, araçta faz gecikmeli tehlikeli manevraları önler.

### 2. Deterministik Fail-Safe Emniyet Çekirdeği (MIL-STD-882E)
- **7-Durumlu Atomic CAS FSM:** `Booting`, `Disarmed`, `Armed`, `Mission`, `Failsafe`, `ManualOverride`, `SafeLock`.
- **Donanımsal 500 ms Sentinel Watchdog:** Üst katmandaki yapay zekâ veya otonom görev yazılımı kilitlenirse veya tek bir besleme periyodunu kaçırırsa aracı anında kilitli `SafeLock` moduna çeker.
- **IEEE 754 NaN/$\pm\infty$ Zehirlenme Koruması:** Sensör boru hatlarından gelen bozuk/sonsuz kayan noktalı sayıların eyleyicilere sızmasını engeller; komutu anında güvenli sıfıra (`0.0 m/s`) çeker.
- **Hayalet Hareket Koruması (Anti-Phantom Movement Flush):** E-Stop basıldığında işletim sistemi kuyruklarında biriken eski komutları atomik olarak temizler; kilit açıldığında aracın aniden ileri fırlamasını engeller.

### 3. Day-1 Yerel Linux SITL (Masaüstü Simülasyonu)
- Fiziksel donanım, Gazebo veya harici simülatörler olmadan masaüstünde uçtan uca simüle edilir.
- Linux `libc::openpty` ve `libc::cfmakeraw` ile ikili bayt şeffaflığına sahip sanal seri port çifti oluşturulur.
- Eyleyici mekanik ataleti ($\tau = 80\text{ ms}$) ve motor sargı ısıl modelleri matematiksel olarak gerçeğe yakın çalışır.

### 4. Çok Alanlı Kılavuz, Seyrüsefer ve Görüntü İşleme
- **Havacılık ve Seyrüsefer:** WGS84 $\longleftrightarrow$ ECEF $\longleftrightarrow$ NED elipsoit dönüşümleri, Park et al. L1 adaptif doğrusal olmayan yol takibi, MAVLink v2 enkoder/dekoderi, 6-DoF kuaterniyon durum kestiricisi.
- **Kara ve Deniz Dinamiği:** Diferansiyel sürüş ve Ackermann kinematiği (tekerlek kayma faktörlü), İDA çift itici matrisi, 2D Yerel Maliyet Haritası (BFS üstel şişirme, NaN korumalı), APF ve DWA lokal engel kaçınma.
- **Optik ve Takip:** SORT çoklu nesne takipçisi (açgözlü IoU eşleme), çoğunluk oylamalı dinamik IFF optik karıştırma filtresi, Mach 3.5 hipersonik 2. derece kurşun açısı önleme çözücüsü (`LeadAngleSolver`).

---

## 🛡️ Savunma Sanayii Standartları ve Test Matrisi

AutoCore, bağımsız savunma test sandığı ([`crates/autocore-defense-tests`](crates/autocore-defense-tests)) tarafından test edilmiştir:

| Standart | Test Paketi | Doğrulanan Güvenilirlik & Emniyet Kriteri | Durum |
| :--- | :--- | :--- | :---: |
| **MIL-STD-882E**<br>*(Sistem Emniyeti)* | [`mil_std_882e_hazards.rs`](crates/autocore-defense-tests/tests/mil_std_882e_hazards.rs) | • IEEE 754 NaN/$\pm\infty$ komutlarının güvenli sıfırlanması<br>• 10.000 komut doygunluğunda E-Stop kuyruk temizliği ve fantom hareket bağışıklığı<br>• 500 ms Watchdog sert eşiği ve atlatılamaz SafeLock mandalı<br>• FSM yetkisiz durum sıçraması vetosu (`Booting` $\rightarrow$ `Mission` engeli) | **GEÇTİ (4/4)** |
| **DO-178C**<br>*(Uçuş Kritik Yazılım)* | [`do178c_boundary_stress.rs`](crates/autocore-defense-tests/tests/do178c_boundary_stress.rs) | • WGS84 Kutuplar ($\pm 90^\circ$), Antimeridyen ($\pm 180^\circ$), Ölü Deniz ($-430\text{m}$), Stratosfer ($50\text{km}$)<br>• Kuaterniyon $\pm 90^\circ$ pitch gimbal lock bağışıklığı ve 100 rad/s aşırı takla<br>• İçbükey geofence sınır köşe kelepçelemesi (vertex clamping)<br>• Maliyet haritası sınır dışı koordinat ve NaN bellek taşması direnci | **GEÇTİ (4/4)** |
| **STANAG 4586**<br>*(İHA Veri Bağları)* | [`stanag_integrity_fuzzing.rs`](crates/autocore-defense-tests/tests/stanag_integrity_fuzzing.rs) | • **1000 İterasyon deterministik bit-flip mutasyonu (%100 CRC-16 yakalama)**<br>• İkili çerçeveleme desenkronizasyon toparlanması (250 bayt parazit enjeksiyonu)<br>• MTU 1200 bayt paket parçası düşmesi ve sızıntısız zamanaşımı temizliği | **GEÇTİ (3/3)** |
| **EO/IR & Radar**<br>*(Optik Dayanıklılık)* | [`sort_iff_robustness.rs`](crates/autocore-defense-tests/tests/sort_iff_robustness.rs) | • Dinamik IFF optik karıştırma ve parlama histerezisi<br>• Mach 3.5 hipersonik çapraz önleme kurşun açısı çözücüsü ($t > 0$, $<1\text{m}$ hata)<br>• SORT tek karelik sahte alarm / parazit (clutter) reddi | **GEÇTİ (3/3)** |
| **MIL-STD Karatutu**<br>*(Kaza & Güç Kesintisi)* | [`blackbox_resilience.rs`](crates/autocore-defense-tests/tests/blackbox_resilience.rs) | • Ani güç kesintisi ve kırpık bayt toparlama motoru (`load_with_recovery`)<br>• 2000 karelik halka tampon doygunluğunda sıra numarası monotonluğu | **GEÇTİ (2/2)** |

**Toplam Savunma Testleri:** **16/16 Başarılı** • **Tüm Çalışma Alanı Testleri:** **70/70 Başarılı**

---

## 📦 Crate ve Modüler Yapı

| Crate | Sorumluluk | Temel Yapılar / Arayüzler | Test |
| :--- | :--- | :--- | :---: |
| [`autocore-base`](crates/autocore-base) | Sabit boyutlu C-ABI tipleri, Pod/Zeroable sıfır kopyalama modelleri | `KinematicCommand` (32B), `StateVector6DoF` (64B), `ActuatorFeedback` (72B) | 3/3 |
| [`autocore-safety`](crates/autocore-safety) | Kilit-serbest emniyet çekirdeği, bekçi, ivme/sınır kısıtlayıcılar | `CentralSafetyFsm`, `SentinelWatchdog`, `ClampingEngine`, `SlewRateLimiter` | 11/11 |
| [`autocore-hal`](crates/autocore-hal) | Asenkron donanım soyutlama, CRC-16 çerçeveleme, SocketCAN | `ActuatorPort`, `AsyncStreamActuator`, `CanFrame`, `GpioSafetyPin` | 5/5 |
| [`autocore-sitl`](crates/autocore-sitl) | Linux PTY ve sanal CAN ile masaüstü simülasyon tezgâhı | `VirtualPtyPair`, `MockActuator`, `SitlSerialBridge`, `VirtualCanBus` | 5/5 |
| [`autocore-ipc`](crates/autocore-ipc) | Dora-rs v1.1 koordinatörü, Apache Arrow v59 havuzları, Drop-Old | `DoraNodeWrapper`, `SharedRingBufferPool`, `DropOldMailbox`, `PacketFragmenter` | 9/9 |
| [`autocore-guidance`](crates/autocore-guidance) | Geodezi, L1 adaptif doğrusal olmayan yol takibi, MAVLink v2 | `GeoPoint`, `L1Controller`, `MavlinkBridge`, `StateEstimator6DoF` | 8/8 |
| [`autocore-ground`](crates/autocore-ground) | Diferansiyel/Ackermann robotiği, İDA iticileri, 2D maliyet haritası | `DiffDriveKinematics`, `AckermannKinematics`, `LocalCostmap2D`, `ApfPlanner` | 7/7 |
| [`autocore-vision`](crates/autocore-vision) | Çoklu nesne takipçisi, dinamik IFF, kuadratik önleme geometrisi | `SortTracker`, `DynamicIffFilter`, `LeadAngleSolver`, `BoundingBox` | 6/6 |
| [`autocore-sdk`](crates/autocore-sdk) | İstemci kütüphanesi, C++20 sarmalayıcısı, dairesel Karatutu | `AutocoreClient`, `BlackboxRecorder`, `BlackboxReader` | 3/3 |
| [`autocore-defense-tests`](crates/autocore-defense-tests) | MIL-STD-882E, DO-178C, STANAG 4586 doğrulama ve fuzzing paketi | `DeterministicPrng`, Kapsamlı test suitleri | 16/16 |

---

## ⚡ Hızlı Başlangıç

### Gereksinimler
- Linux İşletim Sistemi (Ubuntu 22.04+, Fedora 38+, Arch Linux)
- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Python 3.10+ (`numpy` kütüphanesi ile)
- Modern C++ derleyicisi (C++20 için `g++` 11+ veya `clang++` 14+)

### 1. Derleme ve Testleri Çalıştırma
```bash
# Depoyu klonlayın
git clone https://github.com/YavuzSB/autocore.git
cd autocore

# Tüm 70 birim, entegrasyon ve savunma testini çalıştırın
cargo test --workspace

# Statik kod kontrolü
cargo check --workspace --all-targets
```

### 2. Python ile SITL Otonom Görev Koşusu (Day-1 Simülasyonu)
Fiziksel donanıma ihtiyaç duymadan Linux sanal PTY hattı üzerinden 5 yol noktalı görevi icra eder:
```bash
python3 templates/starter-python/mission.py --sitl
```
*Beklenen Çıktı:*
```text
[INFO] AutoCore SITL köprüsü başlatıldı: /dev/pts/X <-> /dev/pts/Y
[INFO] Adım 152 | WP: 1 | Konum: (10.02, 0.05) | Hız: 2.00 m/s
[INFO] Adım 314 | WP: 2 | Konum: (19.98, 9.94) | Hız: 2.00 m/s
...
[INFO] 5/5 yol noktası başarıyla tamamlandı. (815 adım, 16.99 sn)
```

### 3. Rust Otonom Görev Düğümü
```bash
cargo run -p starter-rust
```

### 4. C++20 Görev Düğümünü Derleme ve Çalıştırma
```bash
g++ -std=c++20 -O3 -I crates/autocore-sdk/include templates/starter-cpp/mission.cpp -o templates/starter-cpp/mission
./templates/starter-cpp/mission
```

---

## 💻 Çok Dilli SDK

### Python (Sıfır Kopyalama NumPy Köprüsü)
```python
from autocore.abi import KinematicCommand, StateVector6DoF
from autocore.client import AutocoreSitlClient

# Sanal veya fiziksel seri porta bağlan
client = AutocoreSitlClient(port="/dev/pts/1", baudrate=115200)

# Sıfır kopyalama ile durum verisini al
state = client.get_latest_state()
print(f"X: {state.x:.2f} m, Y: {state.y:.2f} m, Yönelim: {state.yaw:.2f} rad")

# C-ABI ile kilitlenmiş komut gönder
cmd = KinematicCommand(v_linear_x=2.5, v_angular_z=0.25)
client.send_command(cmd)
```

### C++20 Header-Only (`autocore.hpp`)
```cpp
#include "autocore.hpp"
#include <iostream>

int main() {
    // Derleme anı C-ABI boyut doğrulaması
    static_assert(sizeof(autocore::KinematicCommand) == 32);
    static_assert(sizeof(autocore::StateVector6DoF) == 64);

    autocore::KinematicCommand cmd{};
    cmd.linear_x = 2.0f;
    cmd.angular_z = 0.15f;

    auto packet = autocore::encode_command(cmd);
    std::cout << "Paket boyutu: " << packet.size() << " bayt\n";
    return 0;
}
```

---

## 📼 Karatutu Uçuş Veri Kaydedici (Blackbox)

AutoCore, yüksek bütünlüklü dairesel bir uçuş veri kayıt motoru içerir:
- **Çerçeve Yapısı:** Çerçeve başına 184 bayt (mikrosaniye zaman damgası, 6-DoF kinematik durum, eyleyici telemetrisi, emniyet modu ve CRC-32 sağlama toplamı).
- **Güç Kesintisi Kurtarma:** Batarya kopması veya kaza anında `BlackboxReader::load_with_recovery`, dosya sonundaki kırpık son baytları güvenle ayıklar ve **sağlam uçuş çerçevelerinin %100'ünü kurtarır**.
- **Dışa Aktarma:** Uçuş sonrası analiz için ikili logları CSV veya JSON formatına dönüştürün:
```bash
cargo run -p autocore-sdk --bin autocore-blackbox-export -- telemetry.bbox telemetry.csv
```

---

## 🚀 Gömülü Linux Üretim Dağıtımı (Real-Time Linux)

AutoCore, gömülü Linux donanımlarında (**NVIDIA Jetson Orin**, **Raspberry Pi CM4**, **Radxa Rock 5B**) yerel bir `systemd` gerçek zamanlı servisi olarak çalışacak şekilde tasarlanmıştır:

```ini
[Service]
ExecStart=/usr/local/bin/autocore-daemon --config /etc/autocore/vehicle.toml
CPUSchedulingPolicy=fifo
CPUSchedulingPriority=99
CPUAffinity=2,3
LimitRTPRIO=infinity
LimitMEMLOCK=infinity
```

Gerçek zamanlı `PREEMPT_RT` çekirdek ayarları ve CPU çekirdek izolasyonu için [`deploy/deploy-guide.md`](deploy/deploy-guide.md) kılavuzuna bakın.

---

## 📄 Lisans

Yüksek güvenilirlikli otonom savunma ve robotik sistemleri için geliştirilmiştir.  
Çift lisanslıdır:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT) veya http://opensource.org/licenses/MIT)
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) veya http://www.apache.org/licenses/LICENSE-2.0)
