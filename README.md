<div align="center">

# ⚡ AUTOCORE
### High-Performance Autonomous Robotics Runtime & Capability Primitives Kernel
**Zero-Copy IPC • Apache Arrow • Dora-rs • MIL-STD-882E & DO-178C Hardened**

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![CI](https://github.com/YavuzSB/autocore/actions/workflows/ci.yml/badge.svg)](https://github.com/YavuzSB/autocore/actions)
[![Tests](https://img.shields.io/badge/tests-88%2F88_passed-brightgreen.svg?style=flat-square)](crates/autocore-defense-tests)
[![Safety Standards](https://img.shields.io/badge/standards-MIL--STD--882E_%7C_DO--178C_%7C_STANAG_4586-blue.svg?style=flat-square)](crates/autocore-defense-tests)
[![IPC](https://img.shields.io/badge/IPC-Apache_Arrow_v59_%7C_Dora--rs_v1.1-purple.svg?style=flat-square)](crates/autocore-ipc)
[![Real-Time](https://img.shields.io/badge/realtime-SCHED__FIFO_99-red.svg?style=flat-square)](deploy/autocore.service)
[![License](https://img.shields.io/badge/license-MIT-lightgrey.svg?style=flat-square)](LICENSE)
[![Security Policy](https://img.shields.io/badge/security-policy-green.svg?style=flat-square)](SECURITY.md)

<p align="center">
  <a href="README.md">English</a> | <a href="README.tr.md">Türkçe</a>
</p>

<p align="center">
  <a href="#-overview">Overview</a> •
  <a href="#-system-architecture">Architecture</a> •
  <a href="#-capability--domain-primitives-matrix">Domain Primitives</a> •
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

**AutoCore** is an industrial-grade, mission-critical autonomous robotics runtime, safety kernel, and modular domain primitive stack engineered for unmanned platforms operating across Land (**İKA**), Maritime Surface (**İDA**), Aerial (**İHA**), and Air Defense / Turret (**HSS**) domains.

Built in 100% memory-safe Rust with deterministic C-ABI memory layouts, AutoCore eliminates the single-point-of-failure vulnerabilities, high serialization latencies, and bloated runtime overheads of legacy ROS/ROS 2 stacks. 

Instead of writing monolithic, platform-specific code from scratch, engineering teams stack **Capability & Domain Primitives** (kinematics, SE(3) transform trees, ballistic solvers, MAVLink bridges, costmaps) on top of the hardened **AutoCore Safety & IPC Kernel**, writing only high-level mission logic.

---

## 🏛️ System Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│               MISSION APPLICATION LAYER (Your Only Code)                │
│       "Search Target -> Track -> Intercept -> Engage on Safe Auth"     │
│             Python (NumPy 0-copy) • C++20 • Rust Mission Node          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Arrow Zero-Copy Event Stream
┌───────────────────────────────────┴────────────────────────────────────┐
│              CAPABILITY & DOMAIN PRIMITIVES (Stackable Lego)            │
│                                                                        │
│  ┌───────────────────┐ ┌───────────────────┐ ┌──────────────────────┐  │
│  │  autocore-vision  │ │  autocore-ground  │ │   autocore-aerial    │  │
│  │ (ByteTrack/Coast/ │ │ (Ackermann/Diff/  │ │ (MAVLink Offboard/   │  │
│  │  Depth Projection)│ │  Mecanum/Costmap) │ │  L1 Guidance/Geofence│  │
│  └─────────┬─────────┘ └─────────┬─────────┘ └──────────┬───────────┘  │
│            │                     │                      │              │
│  ┌─────────▼─────────┐ ┌─────────▼─────────┐ ┌──────────▼───────────┐  │
│  │ autocore-pointing │ │  autocore-marine  │ │  autocore-transform  │  │
│  │ (Lead-Angle / IK /│ │ (Thruster Alloc / │ │ (SE(3) Transform Tree│  │
│  │  Latency Comp)    │ │  Crab Angle Comp) │ │  WGS84<->NED Geodesy)│  │
│  └─────────┬─────────┘ └─────────┬─────────┘ └──────────┬───────────┘  │
│            │                     │                      │              │
│            └─────────────────────┼──────────────────────┘              │
│                                  ▼                                     │
│                     ┌──────────────────────────┐                       │
│                     │     autocore-arbiter     │                       │
│                     │ (3-Tier Priority Arbiter)│                       │
│                     └────────────┬─────────────┘                       │
└──────────────────────────────────┼─────────────────────────────────────┘
                                   │ Arbitrated KinematicCommand
┌──────────────────────────────────▼─────────────────────────────────────┐
│                 HARDENED SAFETY SENTINEL & IPC KERNEL                  │
│    7-State Atomic CAS FSM • 500ms Hardware Sentinel Watchdog           │
│    IEEE 754 NaN/Inf Filter • Geofence Clamping Engine                  │
│    Slew-Rate Acceleration Limiter • Anti-Phantom E-Stop Flush          │
│    Dora-rs Event Stream • Apache Arrow Shared Memory (/dev/shm)        │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │ Raw PTY / SocketCAN
┌──────────────────────────────────▼─────────────────────────────────────┐
│                     HAL & NATIVE LINUX SITL TWIN                       │
│    Linux openpty Serial • Virtual CAN (vcan0) • Mock Inertial Actuator │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 🧩 Capability & Domain Primitives Matrix

| Domain Crate | Responsibility | Mathematical Core | Zero-Copy Output |
| :--- | :--- | :--- | :--- |
| **`autocore-transform`** | $SE(3)$ Kinematics & Geodesy | Unit Quaternions, Bowring WGS84 $\longleftrightarrow$ ECEF $\longleftrightarrow$ NED ($<1\text{ mm}$ error), Optical $\to$ Mechanical axis projection | `LocalPose` (48B C-ABI) |
| **`autocore-arbiter`** | Strict Priority Command Arbitration | 3-Tier Preemption: Tier 1 (Veto/E-Stop) > Tier 2 (Reactive Obstacle Avoidance) > Tier 3 (Mission Logic), `SlewRateLimiter` | `ArbitratedCommand` (40B C-ABI) |
| **`autocore-pointing`** | Turret, Gimbal & Engagement | Dead Reckoning latency extrapolation, 2nd-order quadratic ballistic lead-angle solver with gravity drop compensation, 2-DoF IK, `ActionDiscipline` 4-rule latch | `PointingCommand` (20B C-ABI) |
| **`autocore-ground`** | Unmanned Ground Vehicles (UGV) | Differential, Ackermann, and Mecanum multi-chassis kinematics, BFS exponential decay Costmap2D, APF & DWA avoidance, Pure Pursuit & Stanley path trackers | `KinematicCommand` (32B C-ABI) |
| **`autocore-vision`** | Perception, Tracking & Projection | ByteTrack / SORT multi-object tracker with target coasting during occlusions, Pinhole 2D $\to$ 3D `DepthProjector`, Dynamic majority-voting IFF filter | Arrow RecordBatch / Tracks |
| **`autocore-aerial`** | Unmanned Aerial Vehicles (UAV) | MAVLink v2 `SET_POSITION_TARGET_LOCAL_NED` bridge, Park et al. L1 nonlinear crosstrack guidance, `FlightEnvelopeGuard` (stall & $45^\circ$ bank limit), 3D `AirGeofence` RTL, VTOL transition | `AutopilotSetpoint` (28B C-ABI) |
| **`autocore-marine`** | Unmanned Surface / Underwater (USV) | Thruster Allocation Matrix ($B^\dagger \boldsymbol{\tau}$), DriftCompensator integral crab angle for ocean currents, `MarineNavigator` anti-cavitation and anti-rollover | `ThrusterSignals` (16B C-ABI) |

---

## 🛡️ Defense Standards & Safety Matrix

AutoCore is engineered to meet strict military and aerospace verification standards:

| Standard | Hazard / Requirement | AutoCore Architectural Guarantee | Test Verification |
| :--- | :--- | :--- | :--- |
| **MIL-STD-882E** | System Hazard Risk Elimination | 500 ms Sentinel Watchdog hardware timeout forced into latching `SafeLock` mode | `test_mil882e_watchdog_hard_deadline_and_safelock_latch` |
| **MIL-STD-882E** | Sensor Poisoning / Actuator Runaway | IEEE 754 NaN/$\pm\infty$ mitigation clamps corrupted inputs to neutral state | `test_mil882e_nan_inf_poisoning_mitigation` |
| **MIL-STD-882E** | Unintended Post-E-Stop Motion | Atomic queue flush purges latent command buffers during emergency stops | `test_mil882e_estop_under_saturation_and_phantom_immunity` |
| **DO-178C (DAL-A)** | Boundary Condition Robustness | Precision WGS84 geodesy at poles ($\pm 90^\circ$), antimeridian ($180^\circ$), and high altitude | `test_do178c_wgs84_poles_antimeridian_and_stratosphere` |
| **DO-178C (DAL-A)** | Singularity & Gimbal Lock | Pure unit quaternion rotation algebra prevents Euler gimbal lock anomalies | `test_do178c_quaternion_gimbal_lock_and_high_tumbling` |
| **STANAG 4586** | Telemetry Protocol Integrity | 1000-iteration pseudorandom bit-flip mutation fuzzing with zero memory leaks | `test_stanag_1000_bit_flip_mutations_zero_leakage` |
| **DO-178C / FDR** | Catastrophic Power Disconnect | Binary flight recorder isolates truncated tails and recovers 100% of intact frames | `test_blackbox_power_loss_truncation_recovery` |

---

## ⚡ Quick Start

### Prerequisites
- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Python 3.10+ (with `numpy`)
- Modern C++ compiler (`g++` 11+ or `clang++` 14+ for C++20)

### 1. Build and Run Full Test Suite (88/88 Tests)
```bash
# Clone the repository
git clone https://github.com/YavuzSB/autocore.git
cd autocore

# Run all 88 unit, integration, and defense tests
cargo test --workspace

# Static verification across all targets
cargo check --workspace --all-targets
```

### 2. Run Autonomous Python SITL Mission
Simulates a 5-waypoint autonomous mission over Linux virtual PTY without physical hardware:
```bash
python3 templates/starter-python/mission.py --sitl
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

client = AutocoreSitlClient()
client.connect()

# Command vehicle: 2.5 m/s forward, 0.1 rad/s yaw rate
cmd = KinematicCommand(linear=[2.5, 0.0, 0.0], angular=[0.0, 0.0, 0.1])
client.send_command(cmd)

# Read 50 Hz state vector directly via zero-copy memoryview
telemetry = client.read_telemetry()
print(f"Position: {telemetry.position}, Mode: {telemetry.safety_mode}")
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

## 📄 License & Attribution

Designed and developed for mission-critical autonomous defense and robotics systems.  
Licensed under the **MIT License** ([LICENSE](LICENSE)).
