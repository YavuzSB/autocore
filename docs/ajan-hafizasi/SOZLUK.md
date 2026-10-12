---
title: SOZLUK.md
repo: autocore
guncelleme: 2026-10-13
---

# SOZLUK.md

- **C-ABI (Application Binary Interface):** C programlama dili standardında bellek dizilimi (`#[repr(C)]`). Farklı derleyicilerin veya dillerin (Rust, C++, Python) aynı bellek bloğunu bayt şeffaflığıyla okumasını sağlar.
- **Zero-Copy (Sıfır Kopyalama):** Bir veri bloğunun (kamera karesi, tensör) süreçler arasında aktarılırken CPU tarafından kopyalanmadan doğrudan aynı fiziksel RAM adresine referans verilmesi (`/dev/shm` & Apache Arrow).
- **dora-rs:** Otonom sistemler ve robotik için geliştirilmiş yüksek performanslı, DAG (Directed Acyclic Graph) tabanlı veri akışı ara katman yazılımı (middleware).
- **SITL (Software-in-the-Loop):** Gerçek donanım yerine yazılımsal sahte (mock) aygıtlar kullanılarak masaüstünde çalıştırılan tam simülasyon modu.
- **PTY (Pseudo-Terminal):** Linux çekirdeğinin sunduğu sanal seri terminal çifti (`/dev/pts/*`). Donanımsal UART/seri portu masaüstünde yazılımsal olarak emüle eder.
- **CAS (Compare-And-Swap):** Kilit kullanmadan (lock-free) donanım seviyesinde atomik değer karşılaştırma ve yazma işlemi. FSM durum geçişlerinde deadlock riskini sıfırlar.
- **Watchdog (Sentinel Bekçi):** Belirli bir süre (örn. 500 ms) boyunca geçerli bir kalp atışı alınmadığında sistemi otonom olarak acil durum kilitlenmesine (`SafeLock`) sokan bağımsız denetleyici.
- **Clamping (Kelepçeleme):** İzin verilen sınırların (geofence koordinatı veya açı limitleri) dışına taşan komutların yazılımsal olarak tavan/taban eşiğine kırpılması.
- **Slew-Rate Limiter:** Birim zamandaki azami hız ve açısal değişim ivmesini ($\Delta v / \Delta t$) sınırlandırarak aktüatör ve dişli sıyırmalarını engelleyen yumuşatıcı.
- **Hayalet Hareket (Phantom Movement):** Acil durdurma (E-Stop) çözüldüğünde tamponlarda kalmış eski hareket komutlarının ardışık işletilmesiyle aracın kontrolsüz fırlaması durumu.
- **Drop-Old Politikası:** Tüketici düğüm yavaş kaldığında, kuyrukta bekleyen eski veri bloklarının çöpe atılarak yalnızca en güncel blokların işlenmesi stratejisi.
