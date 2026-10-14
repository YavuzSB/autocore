<div align="center">

# ⚡ AUTOCORE
### Yüksek Başarımlı Otonom Robotik Çalışma Zamanı & Yetkinlik Blokları Çekirdeği
**Sıfır-Kopyalama IPC • Apache Arrow • Dora-rs • MIL-STD-882E & DO-178C Uyumlu**

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![CI](https://github.com/YavuzSB/autocore/actions/workflows/ci.yml/badge.svg)](https://github.com/YavuzSB/autocore/actions)
[![Testler](https://img.shields.io/badge/testler-88%2F88_ge%C3%A7ti-brightgreen.svg?style=flat-square)](crates/autocore-defense-tests)
[![Savunma Standartları](https://img.shields.io/badge/standartlar-MIL--STD--882E_%7C_DO--178C_%7C_STANAG_4586-blue.svg?style=flat-square)](crates/autocore-defense-tests)
[![IPC](https://img.shields.io/badge/IPC-Apache_Arrow_v59_%7C_Dora--rs_v1.1-purple.svg?style=flat-square)](crates/autocore-ipc)
[![Gerçek Zamanlı](https://img.shields.io/badge/ger%C3%A7ek__zaman-SCHED__FIFO_99-red.svg?style=flat-square)](deploy/autocore.service)
[![Lisans](https://img.shields.io/badge/lisans-MIT-lightgrey.svg?style=flat-square)](LICENSE)
[![Güvenlik Politikası](https://img.shields.io/badge/g%C3%BCvenlik-politikas%C4%B1-green.svg?style=flat-square)](SECURITY.md)

<p align="center">
  <a href="README.md">English</a> | <a href="README.tr.md">Türkçe</a>
</p>

<p align="center">
  <a href="#-genel-bakış">Genel Bakış</a> •
  <a href="#-sistem-mimarisi">Sistem Mimarisi</a> •
  <a href="#-yetkinlik-ve-fizik-blokları-lego-matrisi">Yetkinlik Blokları</a> •
  <a href="#-savunma-sanayii-standartları-ve-emniyet-matrisi">Savunma Standartları</a> •
  <a href="#-hızlı-başlangıç">Hızlı Başlangıç</a> •
  <a href="#-crate-ve-modüler-yapı">Crate Yapısı</a> •
  <a href="#-çok-dilli-sdk">SDK Kullanımı</a> •
  <a href="#-karatutu-uçuş-veri-kaydedici-blackbox">Karatutu</a>
</p>

---

</div>

## 📌 Genel Bakış

**AutoCore**, Kara (**İKA**), Deniz Üstü (**İDA**), Hava (**İHA**) ve Hava Savunma / Taret (**HSS**) platformlarında görev yapan kritik otonom sistemler için geliştirilmiş, endüstriyel standartta bir **Robotik Çalışma Zamanı (Runtime), Emniyet Çekirdeği (Safety Kernel) ve Modüler Yetkinlik Blokları (Domain Primitives)** ekosistemidir.

%100 bellek güvenli (memory-safe) Rust dili ve deterministik C-ABI bellek kilitleri ile inşa edilen AutoCore; geleneksel ROS ve ROS 2 mimarilerinin getirdiği yüksek serileştirme gecikmelerini, tek hata noktası (single-point-of-failure) kırılganlıklarını ve ağır sistem yükünü ortadan kaldırır.

Araç tipine özel monolitik yazılımlar yazmak yerine takımlar, kinematik, $SE(3)$ dönüşüm ağacı, balistik çözücüler, MAVLink köprüsü ve maliyet haritalarından oluşan **Yetkinlik ve Fizik Bloklarını** taban emniyet çekirdeğinin üzerine Lego gibi ekler; yalnızca en üstteki stratejik görev karar kodunu (Mission Logic) yazar.

---

## 🏛️ Sistem Mimarisi

```
┌────────────────────────────────────────────────────────────────────────┐
│               GÖREV KATMANI (Takımın Yazdığı Tek Kod)                  │
│       "Hedefi ara -> Kilitlen -> Yaklaş -> Hakem Onayıyla Eyleme Geç"   │
│             Python (NumPy 0-kopyalama) • C++20 • Rust Düğümü           │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Arrow Sıfır-Kopyalama Veri Akışı
┌───────────────────────────────────┴────────────────────────────────────┐
│              ALANA ÖZGÜ ÜST ÇEKİRDEKLER (Seç & Üst Üste Ekle)           │
│                                                                        │
│  ┌───────────────────┐ ┌───────────────────┐ ┌──────────────────────┐  │
│  │  autocore-vision  │ │  autocore-ground  │ │   autocore-aerial    │  │
│  │ (ByteTrack/Coast/ │ │ (Ackermann/Diff/  │ │ (MAVLink Offboard/   │  │
│  │  Derinlik İzdüşüm)│ │  Mecanum/Costmap) │ │  L1 Güdüm/Geofence)  │  │
│  └─────────┬─────────┘ └─────────┬─────────┘ └──────────┬───────────┘  │
│            │                     │                      │              │
│  ┌─────────▼─────────┐ ┌─────────▼─────────┐ ┌──────────▼───────────┐  │
│  │ autocore-pointing │ │  autocore-marine  │ │  autocore-transform  │  │
│  │ (Lead-Angle / IK /│ │ (İtki Tahsisi /   │ │ (SE(3) Dönüşüm Ağacı│  │
│  │  Gecikme Telafisi)│ │  Yengeç Açısı)    │ │  WGS84<->NED Geodezi)│  │
│  └─────────┬─────────┘ └─────────┬─────────┘ └──────────┬───────────┘  │
│            │                     │                      │              │
│            └─────────────────────┼──────────────────────┘              │
│                                  ▼                                     │
│                     ┌──────────────────────────┐                       │
│                     │     autocore-arbiter     │                       │
│                     │ (3 Seviyeli Komut Hakemi)│                       │
│                     └────────────┬─────────────┘                       │
└──────────────────────────────────┼─────────────────────────────────────┘
                                   │ Onaylanmış KinematicCommand
┌──────────────────────────────────▼─────────────────────────────────────┐
│                 SERTLEŞTİRİLMİŞ EMNİYET VE IPC ÇEKİRDEĞİ               │
│    7-Durumlu Atomik CAS FSM • 500ms Donanım Sentinel Watchdog          │
│    IEEE 754 NaN/Inf Filtresi • Geofence Kelepçeleme Motoru             │
│    Slew-Rate İvme Sınırlayıcı • Hayalet Hareket Önleyici E-Stop Flush  │
│    Dora-rs Event Stream • Apache Arrow Paylaşılan Bellek (/dev/shm)    │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │ Ham PTY / SocketCAN
┌──────────────────────────────────▼─────────────────────────────────────┐
│                 DONANIM SOYUTLAMA & YEREL LINUX SITL                   │
│    Linux openpty Seri Port • Sanal CAN (vcan0) • Atalet Modelli Eyleyici│
└────────────────────────────────────────────────────────────────────────┘
```

---

## 🧩 Yetkinlik ve Fizik Blokları (Lego Matrisi)

| Üst Çekirdek | Görev ve Yetkinlik Alanı | Matematik Motoru | Sıfır-Kopyalama Çıktı |
| :--- | :--- | :--- | :--- |
| **`autocore-transform`** | $SE(3)$ Kinematik & Geodezi | Birim Kuaterniyonlar, Bowring WGS84 $\longleftrightarrow$ ECEF $\longleftrightarrow$ NED ($<1\text{ mm}$ hata), Kamera optik $\to$ mekanik eksen izdüşümü | `LocalPose` (48B C-ABI) |
| **`autocore-arbiter`** | Katı Öncelikli Komut Hakemi | 3 Seviyeli Ezme Hiyerarşisi: Tier 1 (Veto/E-Stop) > Tier 2 (Reaktif Engel Kaçınma) > Tier 3 (Görev Kodu), `SlewRateLimiter` | `ArbitratedCommand` (40B C-ABI) |
| **`autocore-pointing`** | Taret, Gimbal & Balistik Angajman | Ölü hesaplama gecikme telafisi, 2. derece kuadratik balistik önleme ve yerçekimi düşüş çözücüsü, 2-DoF ters kinematik, `ActionDiscipline` 4 şartlı atış mandalı | `PointingCommand` (20B C-ABI) |
| **`autocore-ground`** | İnsansız Kara Araçları (İKA) | Diferansiyel, Ackermann ve Mecanum çoklu şasi kinematiği, BFS üstel sönümlü Costmap2D, APF & DWA reaktif kaçınma, Pure Pursuit & Stanley rota takipçileri | `KinematicCommand` (32B C-ABI) |
| **`autocore-vision`** | Algılama, Takip & İzdüşüm | Hedef kayıplarında yörüngeyi sürdüren ByteTrack / SORT çoklu nesne takipçisi (Coasting), İğne deliği 2D $\to$ 3B `DepthProjector`, Dinamik IFF çoğunluk oylama süzgeci | Arrow RecordBatch / Tracks |
| **`autocore-aerial`** | İnsansız Hava Araçları (İHA) | MAVLink v2 `SET_POSITION_TARGET_LOCAL_NED` köprüsü, Park et al. L1 doğrusal olmayan yol güdümü, `FlightEnvelopeGuard` (stall & $45^\circ$ bank limiti), 3B `AirGeofence` RTL, VTOL geçiş durum makinesi | `AutopilotSetpoint` (28B C-ABI) |
| **`autocore-marine`** | İnsansız Deniz Araçları (İDA) | Hidrodinamik İtki Dağıtım Matrisi ($B^\dagger \boldsymbol{\tau}$), Akıntı kaynaklı sürüklenmede yengeç açısı (Crab Angle) kompanzatörü, `MarineNavigator` kavitasyon ve alabora önleme | `ThrusterSignals` (16B C-ABI) |

---

## 🛡️ Savunma Sanayii Standartları ve Emniyet Matrisi

AutoCore, savunma sanayii ve havacılık standartlarında sertleştirilmiş mimari güvenceler sunar:

| Standart | Tehlike / Gereksinim | AutoCore Mimari Çözümü | Test Kanıtı |
| :--- | :--- | :--- | :--- |
| **MIL-STD-882E** | Sistemik Kaza Risklerinin Önlenmesi | 500 ms Sentinel Watchdog donanım aşımında geri dönüşsüz kilitlenen `SafeLock` modu | `test_mil882e_watchdog_hard_deadline_and_safelock_latch` |
| **MIL-STD-882E** | Sensör Zehirlenmesi / İstenmeyen İtki | IEEE 754 NaN/$\pm\infty$ tespitinde komutların nötr fren durumuna kelepçelenmesi | `test_mil882e_nan_inf_poisoning_mitigation` |
| **MIL-STD-882E** | E-Stop Sonrası Hayalet Hareket | E-Stop anında kuyrukta bekleyen tüm bayat komutların atomik olarak temizlenmesi | `test_mil882e_estop_under_saturation_and_phantom_immunity` |
| **DO-178C (DAL-A)** | Uç Değer ve Sınır Koşulu Dayanıklılığı | Kutuplarda ($\pm 90^\circ$), antimeridyende ($180^\circ$) ve yüksek irtifada mikron geodezi kararlılığı | `test_do178c_wgs84_poles_antimeridian_and_stratosphere` |
| **DO-178C (DAL-A)** | Tekillik & Gimbal Lock Önleme | Euler yerine saf birim kuaterniyon dönüşümleriyle kilitlenme riskinin sıfırlanması | `test_do178c_quaternion_gimbal_lock_and_high_tumbling` |
| **STANAG 4586** | Telemetri Protokol Bütünlüğü | 1000 iterasyonluk rastgele bit-çevirme (bit-flip fuzzing) stresinde sıfır bellek sızıntısı | `test_stanag_1000_bit_flip_mutations_zero_leakage` |
| **DO-178C / FDR** | Ani Güç Kesintisi ve Veri Bütünlüğü | Karatutu kaydedicisinde eksik baytların ayıklanarak sağlam karelerin %100 kurtarılması | `test_blackbox_power_loss_truncation_recovery` |

---

## ⚡ Hızlı Başlangıç

### Gereksinimler
- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Python 3.10+ (`numpy` kütüphanesi ile)
- Modern C++ derleyicisi (C++20 destekli `g++` 11+ veya `clang++` 14+)

### 1. Derleme ve Testlerin Çalıştırılması (88/88 Test)
```bash
# Depoyu klonlayın
git clone https://github.com/YavuzSB/autocore.git
cd autocore

# Tüm birim, entegrasyon ve savunma testlerini çalıştırın
cargo test --workspace

# Statik doğrulama
cargo check --workspace --all-targets
```

### 2. Python ile Otonom SITL Görevi (Masaüstü Simülasyonu)
Linux sanal PTY üzerinden fiziksel donanım olmadan 5 yol noktalı otonom görevi simüle eder:
```bash
python3 templates/starter-python/mission.py --sitl
```

### 3. Rust Görev Düğümü Koşusu
```bash
cargo run -p starter-rust
```

### 4. C++20 Görev Düğümünü Derleme ve Çalıştırma
```bash
g++ -std=c++20 -O3 -I crates/autocore-sdk/include templates/starter-cpp/mission.cpp -o templates/starter-cpp/mission
./templates/starter-cpp/mission
```

---

## 💻 Çok Dilli SDK Kullanımı

### Python (NumPy Sıfır Kopyalama Köprüsü)
```python
from autocore.abi import KinematicCommand, StateVector6DoF
from autocore.client import AutocoreSitlClient

client = AutocoreSitlClient()
client.connect()

# Aracı sür: 2.5 m/s ileri, 0.1 rad/s açısal dönüş
cmd = KinematicCommand(linear=[2.5, 0.0, 0.0], angular=[0.0, 0.0, 0.1])
client.send_command(cmd)

# 50 Hz telemetriyi kopyalamasız oku
telemetry = client.read_telemetry()
print(f"Konum: {telemetry.position}, Emniyet Modu: {telemetry.safety_mode}")
```

### C++20 Header-Only
```cpp
#include "autocore.hpp"
#include <iostream>

int main() {
    static_assert(sizeof(autocore::KinematicCommand) == 32);
    static_assert(sizeof(autocore::StateVector6DoF) == 64);

    autocore::KinematicCommand cmd{};
    cmd.linear_x = 2.0f;
    cmd.angular_z = 0.15f;

    auto packet = autocore::encode_command(cmd);
    std::cout << "Kodlanmış paket boyutu: " << packet.size() << " bayt\n";
    return 0;
}
```

---

## 📼 Karatutu Uçuş Veri Kaydedici (Blackbox)

AutoCore, yüksek bütünlüklü dairesel bir ikili uçuş veri kaydedicisi barındırır:
- **Kare Yapısı:** Mikrosaniye zaman damgası, 6-DoF durum, eyleyici telemetrisi ve CRC-32 sağlama toplamı içeren 184 baytlık sabit C-ABI kareleri.
- **Güç Kesintisi Kurtarma:** `BlackboxReader::load_with_recovery` motoru, elektrik kesintisinde yarım kalmış son kareyi izole ederek geriye kalan **sağlam karelerin %100'ünü kurtarır**.
- **Dışa Aktarım:** İkili `.bbox` dosyalarını analiz için CSV veya JSON formatına dönüştürün:
```bash
cargo run -p autocore-sdk --bin autocore-blackbox-export -- ucus.bbox telemetri.csv
```

---

## 🚀 Sahada Dağıtım (Gerçek Zamanlı Linux)

AutoCore, gömülü Linux kartlarında (**NVIDIA Jetson Orin**, **Raspberry Pi CM4**, **Radxa Rock 5B**) yerel bir `systemd` servisi olarak en yüksek öncelikle çalıştırılmak üzere tasarlanmıştır:

```ini
[Service]
ExecStart=/usr/local/bin/autocore-daemon --config /etc/autocore/vehicle.toml
CPUSchedulingPolicy=fifo
CPUSchedulingPriority=99
CPUAffinity=2,3
LimitRTPRIO=infinity
LimitMEMLOCK=infinity
```

---

## 📄 Lisans

Savunma sanayii ve otonom robotik sistemleri için tasarlanmış ve geliştirilmiştir.  
**MIT Lisansı** ([LICENSE](LICENSE)) altında sunulmaktadır.
