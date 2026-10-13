# 📋 FAZ 4 GELİŞTİRME VE TEST RAPORU (DOMAIN KÜTÜPHANELERİ)

**Tarih:** 13.10.2026  
**Durum:** %100 TAMAMLANDI VE YEŞİL  
**İlgili Paketler:** `crates/autocore-guidance`, `crates/autocore-ground`, `crates/autocore-vision`  
**Test Başarısı:** 21/21 passed (Toplam sistem: 54/54 passed)  

---

## 1. Faz 4: Alana Özgü Domain Kütüphaneleri

### 1.1 `autocore-guidance` (Hava, Deniz ve Yüzey Güdüm Modülü)
- **Koordinat Dönüşümleri (`coordinates.rs`):**
  - WGS84 Geodetik (Enlem/Boylam/Yükseklik) $\longleftrightarrow$ ECEF (Earth-Centered, Earth-Fixed) dönüşümü ($a = 6378137.0\text{ m}$, $f = 1/298.257223563$).
  - ECEF $\longleftrightarrow$ Yerel Teğet Düzlem NED (North-East-Down) dönüşüm matrisleri.
  - Küresel yeryüzü eğriliği ve geoid toleransı gözetilerek Haversine büyük daire mesafesi ve başlangıç pusula açısı (Initial Bearing) hesabı.
- **L1 Doğrusal Olmayan Yol Takip Güdümü (`l1_guidance.rs`):**
  - Park et al. L1 adaptif kılavuz algoritması.
  - İki yol noktası ($A \rightarrow B$) arasındaki doğruya göre aracın anlık konumu ve hız vektöründen referans L1 noktası ($L_1 = \max(L_{1,\min}, V \cdot T_{L1})$) seçimi.
  - Çapraz hat hatasına (crosstrack error) bağlı yanal ivme komutu: $a_{sCmd} = 2 \frac{V^2}{L_1} \sin\eta$.
- **MAVLink v2 İletişim Protokolü (`mavlink.rs`):**
  - MAVLink v2 çerçeve ayrıştırma ve kodlama (`STX: 0xFD`, bayraklar, sayaç, sysid, compid, 24-bit msgid, yük, CRC-16/MCRF4XX X.25).
  - `Heartbeat` (MsgId 0) ve `Attitude` (MsgId 30) mesajlarının tip güvenli kodlanması ve ayrıştırılması.
- **6-DoF Strapdown Ataletsel Durum Kestiricisi (`state.rs`):**
  - Hamilton kuaterniyon matematiği ile yönelim (orientation) ve Euler açı dönüşümleri (Roll, Pitch, Yaw).
  - İvmeölçer ve jiroskop verilerinden mikrosaniyelik zaman adımı ($\Delta t$) ile strapdown hız ve konum entegrasyonu.
- **Test Kanıtı:** 8/8 passed.

---

### 1.2 `autocore-ground` (Kara Araçları ve Yüzey Platformları)
- **Kinematik Modeller (`kinematics.rs`):**
  - Diferansiyel sürüş (Differential Drive) ileri ve ters kinematiği; tekerlek kayma/patinaj faktörü (slip scaling) hesabı.
  - Ackermann direksiyon geometrisi; iç/dış tekerlek dönüş açıları ve dönüş yarıçapı matrisi.
  - İDA (İnsansız Deniz Aracı) USV çift itici tahsis matrisi; itki farkından sapma (yaw) momenti türetimi.
- **2D Yerel Engel Maliyet Haritası (`costmap.rs`):**
  - Metrik dünya koordinatları $\longleftrightarrow$ ayrık ızgara (grid) hücre dönüşümü.
  - BFS kuyruğu ve üstel sönüm ($\exp(-d / \sigma)$) tabanlı engel şişirme (inflation layer) katmanı.
- **Reaktif Engel Kaçınma (`avoidance.rs`):**
  - Yapay Potansiyel Alanlar (APF - Artificial Potential Fields): Hedefe çekim kuvveti ($F_{att}$), engellerden itme kuvveti ($F_{rep}$) ve bileşke kaçış vektörü hesabı.
  - Dinamik Pencere Yaklaşımı (DWA - Dynamic Window Approach): Aracın hız/açısal hız dinamik sınırlarında örnekleme ve engel mesafesi/ilerleme optimizasyonu.
- **Test Kanıtı:** 7/7 passed.

---

### 1.3 `autocore-vision` (Algılama, Takip ve Angajman)
- **Hedef ve Tespit Tipleri (`types.rs`):**
  - Eksen hizalı `BoundingBox`, alan ve kesişim/birleşim oranı (IoU - Intersection over Union) hesabı.
  - `Detection2D` ve hedef sınıflandırması (`DetectionClass`).
- **SORT Çoklu Hedef Takipçisi (`tracker.rs`):**
  - Sabit hızlı Kalman/kinematik tahmin modeli.
  - Açgözlü (greedy) IoU maliyet matrisi hedef eşlemesi.
  - Hedef onaylama eşiği (`min_hits`) ve yaşam döngüsü tasfiyesi (`time_since_update > max_age`).
- **Dinamik IFF (Dost/Düşman Tanıma) Histerezis Filtresi (`iff.rs`):**
  - Kamera gürültüsü ve anlık ışık parlamalarına karşı `min_samples` asgari gözlem kapısı.
  - Çoklu kare gözlem geçmişinden dost/düşman oranına göre güvenilir `Friendly`, `Hostile` veya `Unknown` kararı.
- **Angajman Önleme Geometrisi (`lead_angle.rs`):**
  - Hedefin anlık konumu, hız vektörü ve önleme mühimmatı/aracı hızından ikinci dereceden kinematik kök denklemi ($a t^2 + b t + c = 0$) çözümü.
  - Gelecekteki çarpışma noktası ve önleme açısı (lead azimuth & elevation) türetimi.
- **Test Kanıtı:** 6/6 passed.

---

## 2. Test Kanıtı (21/21 passed)

```text
running 8 tests
test coordinates::tests::test_geodetic_to_ecef_roundtrip ... ok
test coordinates::tests::test_haversine_and_bearing ... ok
test mavlink::tests::test_mavlink_v2_frame_encode_decode ... ok
test l1_guidance::tests::test_l1_crosstrack_correction ... ok
test state::tests::test_quaternion_euler_roundtrip ... ok
test state::tests::test_state_estimator_integration ... ok
test coordinates::tests::test_geodetic_to_ned_and_inverse ... ok
test l1_guidance::tests::test_l1_on_track_zero_accel ... ok
test result: ok. 8 passed; 0 failed; finished in 0.00s

running 7 tests
test avoidance::tests::test_apf_obstacle_repulsion ... ok
test avoidance::tests::test_apf_clear_path ... ok
test costmap::tests::test_costmap_metric_grid_transform ... ok
test kinematics::tests::test_ida_thruster_allocation ... ok
test kinematics::tests::test_diff_drive_roundtrip ... ok
test costmap::tests::test_costmap_inflation ... ok
test kinematics::tests::test_ackermann_kinematics ... ok
test result: ok. 7 passed; 0 failed; finished in 0.00s

running 6 tests
test lead_angle::tests::test_lead_angle_crossing_target ... ok
test lead_angle::tests::test_lead_angle_stationary_target ... ok
test tracker::tests::test_tracker_drops_lost_targets ... ok
test iff::tests::test_iff_hysteresis_and_majority_voting ... ok
test tracker::tests::test_tracker_lifecycle_and_confirmation ... ok
test types::tests::test_bbox_iou ... ok
test result: ok. 6 passed; 0 failed; finished in 0.00s
```

---
*Kaynak: `crates/autocore-guidance`, `crates/autocore-ground`, `crates/autocore-vision`*
