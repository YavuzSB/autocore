<div align="center">

# ⚡ AUTOCORE
### High-Performance Autonomous Robotics Runtime & Safety Kernel
**Zero-Copy IPC • Apache Arrow • Dora-rs • MIL-STD-882E & DO-178C Hardened**

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-70%2F70_passed-brightgreen.svg?style=flat-square)](docs/reports/SAVUNMA_SANAYII_TEST_RAPORU.md)
[![Safety Standards](https://img.shields.io/badge/standards-MIL--STD--882E_%7C_DO--178C_%7C_STANAG_4586-blue.svg?style=flat-square)](docs/reports/SAVUNMA_SANAYII_TEST_RAPORU.md)
[![IPC](https://img.shields.io/badge/IPC-Apache_Arrow_v59_%7C_Dora--rs_v1.1-purple.svg?style=flat-square)](crates/autocore-ipc)
[![Real-Time](https://img.shields.io/badge/realtime-SCHED__FIFO_99-red.svg?style=flat-square)](deploy/autocore.service)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-lightgrey.svg?style=flat-square)](LICENSE)

<p align="center">
  <b>English</b> • <a href="README.tr.md">Türkçe</a>
</p>

<p align="center">
  <a href="#-overview">Overview</a> •
  <a href="#-system-architecture">Architecture</a> •
  <a href="#-defense-standards--safety-matrix">Defense Standards</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-crates--modular-breakdown">Crates</a> •
  <a href="#-multi-language-sdk">SDKs</a> •
  <a href="#-flight-data-recorder-blackbox">Blackbox</a> •
  <a href="README.tr.md">🇹🇷 Türkçe Dokümantasyon</a>
</p>

---

</div>

## 📌 Overview

**AutoCore** is an industrial-grade, mission-critical autonomous robotics runtime and safety kernel engineered for mission platforms operating across Land (**İKA**), Maritime Surface (**İDA**), Aerial (**İHA**), and Air Defense (**HSS**) domains.

Built in 100% memory-safe Rust with deterministic C-ABI memory layouts, AutoCore eliminates the single-point-of-failure vulnerabilities, high serialization latencies, and bloated runtime overheads of legacy ROS/ROS 2 stacks. It integrates a **7-state lock-free atomic Safety FSM**, a hardware-like **500 ms Sentinel Watchdog**, and zero-copy shared memory IPC powered by **Apache Arrow** and **Dora-rs**.

```
               ┌────────────────────────────────────────────────────────┐
               │           MISSION & AUTONOMY APPLICATION LAYER          │
               │        Python (NumPy 0-copy) • C++20 • Rust SDK        │
               └───────────────────────────▲────────────────────────────┘
                                           │ Zero-Copy Shared Memory (/dev/shm)
               ┌───────────────────────────▼────────────────────────────┐
               │            AUTOCORE DISTRIBUTED DATA RUNTIME           │
               │   Dora-rs Event Stream • Apache Arrow v59 Ring Buffer   │
               │       Drop-Old Realtime Queues • 1200B MTU Packetizer   │
               └───────────────────────────▲────────────────────────────┘
                                           │ Strictly Verified C-ABI Bus
               ┌───────────────────────────▼────────────────────────────┐
               │            HARDENED SAFETY SENTINEL KERNEL             │
               │    7-State Atomic CAS FSM • 500ms Hardware Watchdog    │
               │   IEEE 754 NaN/Inf Filter • Geofence Clamping Engine   │
               │   Slew-Rate Acceleration Limiter • Anti-Phantom Flush   │
               └───────────────────────────▲────────────────────────────┘
                                           │ Raw PTY / SocketCAN
               ┌───────────────────────────▼────────────────────────────┐
               │              HAL & DAY-1 NATIVE LINUX SITL             │
               │     Linux openpty Raw Serial • Virtual CAN (vcan0)     │
               │   Thermal & Inertial Mock Actuator (τ=80ms feedback)   │
               └────────────────────────────────────────────────────────┘
```

---

## 🚀 Key Architectural Pillars

### 1. Zero-Copy Shared Memory IPC (Apache Arrow & Dora-rs)
- High-rate 50–500 Hz telemetry and kinematic commands bypass network serialization.
- Fixed-size shared memory ring buffers in `/dev/shm` allow Python applications to map raw buffers directly into `numpy.ndarray` via zero-copy strided views (**0.0 ms transfer latency**).
- Real-time `Drop-Old` lease mailboxes guarantee that slow consumers process only the freshest state, preventing phase-lag maneuvering instability.

### 2. Deterministic Fail-Safe Safety Kernel (MIL-STD-882E)
- **7-State Atomic CAS FSM:** `Booting`, `Disarmed`, `Armed`, `Mission`, `Failsafe`, `ManualOverride`, `SafeLock`.
- **Hardware-Isolated 500 ms Sentinel Watchdog:** Trips into non-volatile `SafeLock` if upper-level autonomy locks up or skips a single deadline.
- **IEEE 754 NaN/$\pm\infty$ Poisoning Protection:** Prevents actuator runaway when sensor pipelines produce unvalidated floating-point anomalies.
- **Anti-Phantom Movement Flush:** Atomically flushes pending command queues when E-Stop triggers, preventing sudden vehicle leaps upon safety release.

### 3. Day-1 Native Linux SITL (Zero Simulator Dependencies)
- Simulates complete physical robot hardware without Gazebo, ROS, or GPU dependencies.
- Native Linux `openpty` configured with `libc::cfmakeraw` provides byte-transparent binary serial communication.
- Virtual actuator dynamics simulate mechanical inertia ($\tau = 80\text{ ms}$) and motor winding temperature models.

### 4. Cross-Domain Guidance, Navigation & Vision
- **Aviation Guidance:** Full WGS84 $\longleftrightarrow$ ECEF $\longleftrightarrow$ NED conversions, Park et al. L1 adaptive nonlinear crosstrack guidance, MAVLink v2 encoder/decoder, 6-DoF strapdown quaternion state estimator.
- **Ground & Naval Dynamics:** Differential drive and Ackermann kinematics with wheel slip compensation, dual-thruster USV allocation, 2D Local Costmap with BFS exponential inflation, APF & DWA local obstacle avoidance.
- **Optics & Tracking:** SORT multi-target tracker with greedy IoU matching, dynamic IFF hysteresis filter with majority voting, and 2nd-order quadratic Mach 3.5 lead-angle intercept solver.

---

## 🛡️ Defense Standards & Safety Matrix

AutoCore is rigorously verified by a dedicated defense test harness ([`crates/autocore-defense-tests`](crates/autocore-defense-tests)):

| Standard | Test Suite | Verified Engineering Properties | Status |
| :--- | :--- | :--- | :---: |
| **MIL-STD-882E**<br>*(System Safety)* | [`mil_std_882e_hazards.rs`](crates/autocore-defense-tests/tests/mil_std_882e_hazards.rs) | • IEEE 754 NaN/$\pm\infty$ command zeroing<br>• E-Stop queue flush & phantom movement immunity under 10k saturation<br>• Hard 500 ms Watchdog deadline & unbypassable SafeLock latch<br>• FSM illegal state transition veto (`Booting` $\rightarrow$ `Mission` blocked) | **PASSED (4/4)** |
| **DO-178C**<br>*(Airborne Software)* | [`do178c_boundary_stress.rs`](crates/autocore-defense-tests/tests/do178c_boundary_stress.rs) | • WGS84 Poles ($\pm 90^\circ$), Antimeridian ($\pm 180^\circ$), Dead Sea ($-430\text{m}$), Stratosphere ($50\text{km}$)<br>• Quaternion $\pm 90^\circ$ pitch gimbal lock invariance & 100 rad/s high tumbling<br>• Concave geofence vertex boundary clamping<br>• Costmap out-of-bounds & NaN memory resilience | **PASSED (4/4)** |
| **STANAG 4586**<br>*(UAV Data Links)* | [`stanag_integrity_fuzzing.rs`](crates/autocore-defense-tests/tests/stanag_integrity_fuzzing.rs) | • **1000-Iteration deterministic bit-flip fuzzing with 100% CRC-16 rejection**<br>• Binary framing stream desynchronization recovery (250-byte noise injection)<br>• MTU 1200 fragment drop and leak-free timeout reclamation | **PASSED (3/3)** |
| **EO/IR & Tracking**<br>*(Sensor Robustness)* | [`sort_iff_robustness.rs`](crates/autocore-defense-tests/tests/sort_iff_robustness.rs) | • Dynamic IFF optical jamming & glare chattering hysteresis<br>• Mach 3.5 hypersonic crossing lead-angle intercept solver ($t > 0$, $<1\text{m}$ miss distance)<br>• SORT single-frame false alarm / clutter rejection | **PASSED (3/3)** |
| **MIL-STD FDR**<br>*(Crash Survivability)* | [`blackbox_resilience.rs`](crates/autocore-defense-tests/tests/blackbox_resilience.rs) | • Mid-write sudden electrical power loss truncation recovery (`load_with_recovery`)<br>• 2000-frame sustained saturation ring buffer monotonicity | **PASSED (2/2)** |

**Total Defense Verification:** **16/16 Passed** • **Full Workspace Test Suite:** **70/70 Passed**

---

## 📦 Crates & Modular Breakdown

| Crate | Responsibility | Key Structs / Traits | Tests |
| :--- | :--- | :--- | :---: |
| [`autocore-base`](crates/autocore-base) | Fixed-size C-ABI primitives, Pod/Zeroable zero-copy memory layouts | `KinematicCommand` (32B), `StateVector6DoF` (64B), `ActuatorFeedback` (72B) | 3/3 |
| [`autocore-safety`](crates/autocore-safety) | Non-blocking safety kernel, hardware watchdog, rate/boundary clamps | `CentralSafetyFsm`, `SentinelWatchdog`, `ClampingEngine`, `SlewRateLimiter` | 11/11 |
| [`autocore-hal`](crates/autocore-hal) | Async hardware abstraction, CRC-16 protocol framing, SocketCAN | `ActuatorPort`, `AsyncStreamActuator`, `CanFrame`, `GpioSafetyPin` | 5/5 |
| [`autocore-sitl`](crates/autocore-sitl) | Hardware-in-the-loop desktop simulator using native Linux PTY & CAN | `VirtualPtyPair`, `MockActuator`, `SitlSerialBridge`, `VirtualCanBus` | 5/5 |
| [`autocore-ipc`](crates/autocore-ipc) | Dora-rs v1.1 event coordinator, Apache Arrow v59 ring buffers, Drop-Old | `DoraNodeWrapper`, `SharedRingBufferPool`, `DropOldMailbox`, `PacketFragmenter` | 9/9 |
| [`autocore-guidance`](crates/autocore-guidance) | Geodesy, L1 adaptive nonlinear crosstrack guidance, MAVLink v2 bridge | `GeoPoint`, `L1Controller`, `MavlinkBridge`, `StateEstimator6DoF` | 8/8 |
| [`autocore-ground`](crates/autocore-ground) | Differential/Ackermann robotics, USV thrusters, 2D local costmap | `DiffDriveKinematics`, `AckermannKinematics`, `LocalCostmap2D`, `ApfPlanner` | 7/7 |
| [`autocore-vision`](crates/autocore-vision) | High-speed multi-object tracker, dynamic IFF, quadratic intercept solver | `SortTracker`, `DynamicIffFilter`, `LeadAngleSolver`, `BoundingBox` | 6/6 |
| [`autocore-sdk`](crates/autocore-sdk) | Client library, C++20 header wrapper, Circular Blackbox recorder | `AutocoreClient`, `BlackboxRecorder`, `BlackboxReader` | 3/3 |
| [`autocore-defense-tests`](crates/autocore-defense-tests) | MIL-STD-882E, DO-178C, STANAG 4586 verification & fuzzing suites | `DeterministicPrng`, Comprehensive test suites | 16/16 |

---

## ⚡ Quick Start

### Prerequisites
- Linux OS (Ubuntu 22.04+, Fedora 38+, Arch Linux)
- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Python 3.10+ (with `numpy`)
- Modern C++ compiler (`g++` 11+ or `clang++` 14+ for C++20)

### 1. Build and Run All Tests
```bash
# Clone the repository
git clone https://github.com/YavuzSB/autocore.git
cd autocore

# Run all 70 unit, integration, and defense tests
cargo test --workspace

# Static verification across all targets
cargo check --workspace --all-targets
```

### 2. Run Autonomous Python SITL Mission (Day-1 Desktop Simulation)
Simulates a 5-waypoint autonomous mission over Linux virtual PTY without physical hardware:
```bash
python3 templates/starter-python/mission.py --sitl
```
*Expected Output:*
```text
[INFO] AutoCore SITL köprüsü başlatıldı: /dev/pts/X <-> /dev/pts/Y
[INFO] Adım 152 | WP: 1 | Konum: (10.02, 0.05) | Hız: 2.00 m/s
[INFO] Adım 314 | WP: 2 | Konum: (19.98, 9.94) | Hız: 2.00 m/s
...
[INFO] 5/5 yol noktası başarıyla tamamlandı. (815 adım, 16.99 sn)
```

### 3. Run Native Rust Autonomy Mission
```bash
cargo run -p starter-rust
```

### 4. Compile and Run C++20 Mission Node
```bash
g++ -std=c++20 -O3 -I crates/autocore-sdk/include templates/starter-cpp/mission.cpp -o templates/starter-cpp/mission
./templates/starter-cpp/mission
```

---

## 💻 Multi-Language SDK

### Python (Zero-Copy NumPy Bridge)
```python
from autocore.abi import KinematicCommand, StateVector6DoF
from autocore.client import AutocoreSitlClient

# Connect to virtual or physical serial port
client = AutocoreSitlClient(port="/dev/pts/1", baudrate=115200)

# Zero-copy state feedback unpacking
state = client.get_latest_state()
print(f"X: {state.x:.2f} m, Y: {state.y:.2f} m, Heading: {state.yaw:.2f} rad")

# Issue command verified by C-ABI
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
    std::cout << "Encoded packet size: " << packet.size() << " bytes\n";
    return 0;
}
```

---

## 📼 Flight Data Recorder (Blackbox)

AutoCore includes a high-integrity, cyclic binary flight data recorder:
- **Frame Layout:** 184 bytes per frame with microsecond timestamp, 6-DoF state, actuator feedback, safety mode, and CRC-32 integrity checksum.
- **Power-Loss Recovery:** In the event of catastrophic battery disconnect or crash, `BlackboxReader::load_with_recovery` safely isolates and discards the trailing truncated bytes while recovering **100% of intact flight frames**.
- **Exporting Logs:** Convert binary logs to CSV/JSON for post-flight analysis:
```bash
# Export blackbox logs to CSV
cargo run -p autocore-sdk --bin autocore-blackbox-export -- input.bbox telemetry.csv
```

---

## 🚀 Production Deployment (Real-Time Linux)

AutoCore is engineered to run as a native `systemd` real-time service on embedded Linux hardware (e.g., **NVIDIA Jetson Orin**, **Raspberry Pi CM4**, **Radxa Rock 5B**):

```ini
[Service]
ExecStart=/usr/local/bin/autocore-daemon --config /etc/autocore/vehicle.toml
CPUSchedulingPolicy=fifo
CPUSchedulingPriority=99
CPUAffinity=2,3
LimitRTPRIO=infinity
LimitMEMLOCK=infinity
```

Refer to [`deploy/deploy-guide.md`](deploy/deploy-guide.md) for PREEMPT_RT kernel tuning, CPU isolcpus, and CAN interface configuration.

---

## 🇹🇷 Türkçe Özet (Turkish Summary)

**AutoCore**, otonom savunma platformları (HSS, İKA, İDA, İHA) ve robotik sistemleri için geliştirilmiş yüksek başarımlı bir **Robotik Çalışma Zamanı ve Emniyet Çekirdeğidir**.

- **Sıfır Kopyalama IPC:** Apache Arrow v59 ve Dora-rs ile `/dev/shm` üzerinden mikrosaniyelik veri aktarımı; Python tarafında `numpy view` ile sıfır kopyalama (0 ms gecikme).
- **Askeri Emniyet Çekirdeği:** 7-Durumlu Atomic CAS Safety FSM, 500 ms bağımsız donanımsal Sentinel Watchdog, IEEE 754 NaN koruması, Geofence Clamping ve Hayalet Hareket Önleyici Lock-Free Queue Flush.
- **Day-1 SITL (Masaüstü Tezgâhı):** Gazebo/ROS gerektirmeden Linux `openpty` ve `vcan` ile masaüstünde uçtan uca simülasyon.
- **Savunma Sanayii Uyumluluğu:** MIL-STD-882E, DO-178C, STANAG 4586 (1000 iterasyon bit-flip fuzzing) ve Karatutu elektrik kesintisi toparlama testleriyle mühürlü **70/70 yeşil test kanıtı**.

---

## 📄 License & Attribution

Designed and developed for high-reliability autonomous defense systems.  
Licensed under either of:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
