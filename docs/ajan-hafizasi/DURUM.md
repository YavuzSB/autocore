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
- 2026-10-13: Faz 3 (`autocore-ipc`) tamamlandı. `dora-node-api v1.1.0`, Apache Arrow v59 sabit boyutlu halka havuzu (`SharedRingBufferPool`), `Drop-Old` kiralama mekanizması (`DropOldMailbox`, `DropOldLeaseQueue`), 1200 bayt MTU telemetri parçalayıcı (`PacketFragmenter`, `PacketReassembler`) ve `flows/dataflow-sitl.yml` çizgesi devreye alındı. Toplam 33/33 test %100 yeşil.
- 2026-10-13: Faz 4 (`autocore-guidance`, `autocore-ground`, `autocore-vision`) tamamlandı. WGS84 $\leftrightarrow$ NED dönüşümleri, Park et al. L1 adaptif kılavuzu, MAVLink v2 enkoder/dekoder, 6-DoF strapdown kuaterniyon durum kestiricisi; diferansiyel/Ackermann/İDA kinematiği (slip faktörlü), 2D Local Costmap BFS üstel şişirme, APF/DWA engel kaçınma; BBox IoU, SORT çoklu hedef takipçisi, dinamik IFF histerezisi ve Lead-Angle Solver devreye alındı. Toplam 54/54 test %100 yeşil.
- 2026-10-13: Sıradaki odak: Faz 5 (SDK, Takım Başlangıç Şablonları, Siyah Kutu Teşhis Kaydedici ve Linux Dağıtımı).
