# 📋 FAZ 3 GELİŞTİRME VE TEST RAPORU (DORA-RS & ARROW IPC)

**Tarih:** 13.10.2026  
**Durum:** %100 TAMAMLANDI VE YEŞİL  
**İlgili Paketler:** `crates/autocore-ipc`  
**Test Başarısı:** 9/9 passed (Toplam sistem: 33/33 passed)  

---

## 1. Faz 3: Dora-rs ve Apache Arrow Sıfır Kopyalama IPC (`autocore-ipc`)

### 1.1 Gerçekleştirilen Mühendislik
- **Dora-rs Düğüm Sarmalayıcısı (`DoraNodeWrapper`):**
  - `dora-node-api v1.1.0` entegrasyonu sağlandı.
  - İki çalışma modu:
    1. `DoraCoordinator`: Gerçek Dora koordinatörü altında çalışan, `EventStream::recv_async()` ile asenkron olay dinleyen ve `send_output_bytes` ile veri basan mod.
    2. `Standalone`: Harici Dora kurulumu gerektirmeden, masaüstü SITL simülasyonu ve birim testler için in-process Tokio kanallı döngü modu.
  - Tip güvenli olay eşleme: `AutocoreEvent::Kinematic`, `AutocoreEvent::Feedback`, `AutocoreEvent::Raw`, `AutocoreEvent::Stop`.
- **Apache Arrow v59 Sıfır Kopyalama Bellek Havuzu (`SharedRingBufferPool`):**
  - Sabit boyutlu önceden tahsis edilmiş bellek yuvaları (Pre-allocated Shared Ring Buffer).
  - RAII tabanlı `BufferLease` bekçisi; `drop` edildiğinde yuva atomik olarak tekrar havuza döner.
  - `ArrowKinematicEncoder`: C-ABI `KinematicCommand`, `StateVector6DoF` ve `ActuatorFeedback` struct'larını sıfır kopyalama ile Apache Arrow `UInt8Array` dizisine sarar ve çözer.
- **Drop-Old Bellek Kiralama Politikası (`DropOldMailbox` & `DropOldLeaseQueue`):**
  - Gecikme birikimini ve kuyruk şişmesini önleyen mekanizma.
  - Tüketici yavaş kaldığında en eski paket sessizce atılarak (`dropped_count` sayacı artırılarak) daima en taze durumun işlenmesi garanti edilir.
- **1200 Bayt MTU Telemetri Parçalama ve Birleştirme (`PacketFragmenter` & `PacketReassembler`):**
  - Telsiz / GCS / UDP ağ sınırları için 12 baytlık başlık (`magic: 0x5343`, `msg_id`, `seq_idx`, `total_frags`, `crc16`) ve azami 1188 bayt yük taşıyan parçalar.
  - CRC-16-CCITT doğrulaması ile bozuk parçaların anında reddedilmesi.
  - Eksik kalan veya yolda düşen parçaların bellek sızdırmaması için zaman aşımlı (`cleanup_expired`) temizlik.
- **Bildirimsel Veri Akış Çizgesi (`flows/dataflow-sitl.yml`):**
  - `mission_controller` $\rightarrow$ `safety_sentinel` $\rightarrow$ `actuator_bridge` boru hattı tanımlandı.

### 1.2 Test Kanıtı
```text
running 9 tests
test arrow_pool::tests::test_arrow_kinematic_roundtrip ... ok
test arrow_pool::tests::test_shared_ring_buffer_pool_lease_and_release ... ok
test drop_old::tests::test_mailbox_drop_old ... ok
test drop_old::tests::test_queue_drop_old_capacity ... ok
test node::tests::test_standalone_node_wrapper_roundtrip ... ok
test packet::tests::test_fragment_binary_roundtrip ... ok
test packet::tests::test_packet_fragmentation_and_reassembly ... ok
test packet::tests::test_reassembler_cleanup_expired ... ok
test tests::test_ipc_end_to_end_pipeline ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---
*Kaynak: `crates/autocore-ipc`, `flows/dataflow-sitl.yml`*
