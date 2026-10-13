---
title: KARARLAR.md
repo: autocore
guncelleme: 2026-10-13
---

# KARARLAR.md

- **ADR-001 [Cargo Workspace & Crate İzolasyonu]:** Büyük monolitik paketler yerine `crates/autocore-*` ayrımı seçildi. Görüntü işleme kullanmayan bir İKA veya roket takımı OpenCV/CUDA derlemek zorunda kalmayacak, bağımlılık şişkinliği önlendi.
- **ADR-002 [Dora-rs & Apache Arrow Zero-Copy IPC]:** ROS 2'nin DDS serileştirme vergisi ve CPU yükü yerine Unix proses izolasyonu sunan `dora-rs` ve `/dev/shm` üzerinden Apache Arrow sıfır kopyalama bellek modeli seçildi. Python düğümlerine `numpy.ndarray` görünümü (`view`) ile 0 ms gecikmeyle aktarım garanti edildi.
- **ADR-003 [Linux Native PTY & vcan SITL]:** Gazebo veya Unity gibi ağır 3D simülasyon motorları yerine mikro saniyelik Linux sanal terminali (`libc::openpty` raw mode) ve `vcan` seçildi. Donanım bağımsız masaüstü testi ve CI koşuları sağlandı.
- **ADR-004 [7-Durumlu Atomik CAS Safety FSM]:** Emniyet durum makinesi kilitlenmelere (`mutex deadlock`) karşı `Arc<AtomicU8>` Compare-And-Swap (CAS) döngüsüyle inşa edildi. Üst katman kilitlense bile bağımsız 500ms Watchdog aracı `SafeLock` moduna çeker.
- **ADR-005 [Drop-Old Bellek Kiralama Politikası]:** Yavaş çalışan karar/algılama düğümlerinin tamponları şişirip eski karelerle gecikmeli kararlar üretmesini engellemek için `Drop-Old` politikası kabul edildi; daima en taze veri işlenir.
- **ADR-006 [Domain Modül Ayrımı (guidance, ground, vision)]:** Hava, yüzey, kara ve suüstü ekiplerinin gereksiz bağımlılık (OpenCV, GDAL vb.) yükü taşımaması için domain kütüphaneleri üç ayrı bağımsız crate olarak yapılandırıldı (`crates/autocore-guidance`, `autocore-ground`, `autocore-vision`).
- **ADR-007 [Deterministik SORT & Kapalı Form Lead-Angle Solver]:** Hedef takibi ve mühimmat/araç önleme geometrisinde belirsiz veya yüksek gecikmeli derin modeller yerine $O(N)$ karmaşıklığında SORT IoU eşleme ve analitik 2. derece kinematik kök çözücü ($a t^2 + b t + c = 0$) standardı kabul edildi.
- **ADR-008 [Çok Dilli Sözleşme Tabanlı Mimari (Polyglot SDK)]:** Farklı takım dilleri (C++, Python, Rust) arasındaki uyumsuzluğu önlemek için C-ABI seviyesinde `#[repr(C)]`, C++20 `static_assert` ve Python `ctypes.Structure` eşlemesi zorunlu kılındı. Apache Arrow ve NumPy bellek adresleri sıfır kopyalama (0 ms gecikme) ile paylaşıldı.
- **ADR-009 [184-Bayt Dairesel Siyah Kutu (Blackbox) Formatı]:** Uçuş ve saha teşhisi için her karede State (64B) + Command (32B) + Feedback (72B) + Zaman Damgası (8B) + CRC-16 (2B) olmak üzere sabit 184 baytlık dairesel ikili bellek tamponu (`Ring Buffer`) standardı getirildi; disk şişmesi önlenip post-mortem analiz sağlandı.
- **ADR-010 [Linux systemd SCHED_FIFO 99 ve CPU Affinity İzolasyonu]:** Kritik emniyet ve aktüatör döngülerinin işletim sistemi arka plan görevleri tarafından geciktirilmemesi için Linux gerçek zamanlı `SCHED_FIFO` 99 önceliği ve `CPUAffinity=2,3` çekirdek izolasyonu operasyonel standart olarak belirlendi.


