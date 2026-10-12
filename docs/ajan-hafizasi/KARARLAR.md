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
