# Security Policy

## 🛡️ Security Overview & Mission

AutoCore is an industrial-grade, mission-critical autonomous robotics runtime and safety kernel designed for uncrewed defense and robotic platforms (HSS, İKA, İDA, İHA). Because failures in autonomous actuators and control loops can result in loss of platform or physical hazards, security and safety are treated as inseparable design requirements under **MIL-STD-882E** (System Safety) and **DO-178C** (Airborne Software).

---

## 📦 Supported Versions

Security updates, bug fixes, and safety hardening patches are applied to the following versions:

| Version | Supported | Safety Status |
| :--- | :---: | :--- |
| **`0.1.x`** (Main) | ✅ Yes | Active development, MIL-STD-882E & DO-178C tested |
| `< 0.1.0` | ❌ No | Deprecated development snapshots |

---

## 🚨 Reporting a Vulnerability

We take the security and integrity of AutoCore extremely seriously. If you discover a security vulnerability or safety hazard, please do **NOT** open a public issue.

### Reporting Procedure

1. **GitHub Security Advisory (Recommended):**  
   Navigate to the repository's **Security** tab and click **"Report a vulnerability"** to submit a private advisory.
2. **Email Disclosure:**  
   Send an encrypted report to the maintainer at `yavuzzselim07@gmail.com` with the subject tag `[SECURITY: AutoCore]`.

### What to Include in Your Report

To help us triage and resolve the issue quickly, please include:
- A clear description of the vulnerability or safety violation.
- Platform architecture (x86_64, aarch64, PREEMPT_RT kernel, Jetson Orin, Raspberry Pi CM4).
- Crate or component involved (`autocore-safety`, `autocore-ipc`, `autocore-hal`, etc.).
- Minimal, deterministic proof-of-concept (PoC) code or test case reproducing the flaw.
- Assessment of the safety impact according to the **MIL-STD-882E Hazard Severity** scale below.

### Response Timelines

- **Initial Acknowledgment:** Within **24 hours**.
- **Triage & Hazard Assessment:** Within **72 hours**.
- **Remediation & Patch Release:** Coordinated with the reporter before public disclosure.

---

## 🎯 Threat Model & Safety Severity Matrix

Vulnerabilities in AutoCore are triaged against both software exploitability and physical safety impact:

| Hazard Severity (MIL-STD-882E) | Failure Modes & Vulnerability Categories |
| :--- | :--- |
| **Catastrophic (Severity I)** | • **SafeLock Bypass:** Bypassing the hardware latch without authorized `reset_safelock()`.<br>• **E-Stop Phantom Movement:** Failure of `flush_all()` allowing stale actuator commands to execute after safety release.<br>• **Watchdog Deadline Evasion:** Autonomy lockups failing to trip into SafeLock within 500 ms.<br>• **IEEE 754 NaN Poisoning:** Unsanitized non-finite floats escaping clamping into actuators. |
| **Critical (Severity II)** | • **Zero-Copy IPC Corruption:** Race conditions or memory leaks in `/dev/shm` Arrow buffer pools.<br>• **CAN Frame Spoofing / Replay:** Undetected packet injections bypassing CRC-16 protocol framing.<br>• **Geofence Boundary Invalidation:** Polygon vertex overflows causing vehicle to exit mission corridor. |
| **Moderate (Severity III)** | • **Sensor Desynchronization:** Telemetry fragment reassembly timeouts under heavy link noise.<br>• **IFF Optical Chattering:** Hysteresis filter failure under deliberate optical jamming/glare.<br>• **Blackbox Truncation Loss:** Loss of intact flight recorder frames during sudden power cutoffs. |

---

## 🔒 Architectural Safety & Security Guardrails

AutoCore enforces architectural barriers to prevent entire classes of security vulnerabilities:

1. **Memory Safety by Construction:**  
   Written in 100% Rust without unsafe abstractions in control logic. Fixed-size C-ABI struct layouts (`#[repr(C)]`) are statically locked with compile-time `static_assert` assertions.
2. **Fail-Stop Principle (Fail-Safe Over Fallback):**  
   AutoCore enforces immediate hard-latch fail-stop behavior rather than silent degradation. An unverified state or safety violation never silently defaults to a weaker model or actuator command.
3. **Queue Saturation Flush:**  
   Command queues are lock-free and drop-old enabled. Actuators cannot buffer stale velocity commands that could cause sudden delayed surges.
4. **Deterministic Fuzz Testing:**  
   Continuous verification includes 1000-iteration bit-flip mutations (`stanag_integrity_fuzzing.rs`) verifying zero CRC-16 leakages.

---

## 🛠️ Security Best Practices for Integrators

When deploying AutoCore on physical vehicles and embedded hardware:

- **Filesystem Permissions:** Ensure `/dev/shm` shared memory segments are restricted to the designated `autocore` system group.
- **Process Isolation:** Run `autocore-daemon` under Linux `systemd` with `LimitMEMLOCK=infinity` and `CPUAffinity` isolating real-time threads from general user-space processes.
- **CAN Bus Isolation:** Physically isolate mission control CAN interfaces (`can0`) from telemetry or payload radios using dedicated hardware transceivers with galvanic optical isolation.
- **Do Not Disable Watchdog:** Never set `SentinelWatchdog` timeout beyond 500 ms in production vehicles without formal safety review.
