---
title: DURUM.md
repo: autocore
guncelleme: 2026-10-13
---

# DURUM.md

- 2026-10-13: Workspace `/home/yavuz/Belgeler/autocore` kuruldu; `autocore-base` crate'i tamamlandı. `KinematicCommand` (32B), `StateVector6DoF` (64B), `ActuatorFeedback` (72B) C-ABI struct'ları `#[repr(C)]`, `bytemuck` Pod/Zeroable ve `static_assertions` ile kilitlendi (3/3 test OK, commit `64154eb`).
- 2026-10-13: Faz 1 (`autocore-safety`) tamamlandı. 7 Durumlu Atomic CAS Safety FSM (`Booting`..`SafeLock`), 500ms Sentinel Watchdog, 2D Geofence Clamping, Slew-Rate ivme sınırlandırması ve Lock-Free Queue Flush (Hayalet Hareket Düzeltmesi) mühürlendi (11/11 test OK, commit `64154eb`).
- 2026-10-13: Faz 2 (`autocore-hal` ve `autocore-sitl`) tamamlandı. Linux `openpty` tabanlı sanal seri port (`VirtualPtyPair`), `cfmakeraw` ikili veri garantisi, $\tau=80\text{ms}$ atalet ve ısıl modelli `MockActuator`, 50 Hz telemetrili uçtan uca köprü (`SitlSerialBridge`) ve `VirtualCanBus` devreye alındı. 24/24 test %100 yeşil (commit `b60b3b4`).
- 2026-10-13: Proje dokümantasyon ağacı `HukukAI` standardında `/home/yavuz/Belgeler/autocore/docs` altına kuruldu (`KURALLAR.md`, `KALDIGIMIZ_YER.md`, `BRAIN-KOPRU.md`, `plan/GENEL_PLAN.md`, `ajan-hafizasi/`, `reports/`).
- 2026-10-13: Sıradaki odak: Faz 3 (Dora-rs Veri Akış Çizgesi & Apache Arrow Sıfır Kopyalama IPC — `autocore-ipc`).
