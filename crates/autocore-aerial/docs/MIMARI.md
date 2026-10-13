# ✈️ autocore-aerial :: Mimari Şartname
### Uçuş Dinamiği, Otopilot Köprüsü & 3B Hava Güdümü Çekirdeği

**Sürüm:** 0.1.0  
**Durum:** Şartname Mühürlendi  
**Modül:** `crates/autocore-aerial`  

---

## 1. Problem - Bağlam - Gerekçe
- **Problem:** Döner kanat, sabit kanat ve VTOL İHA'lar aerodinamik sınırların dışına çıktığında perdövitese (stall) girerek kırıma uğrayabilir. Ayrıca otopilot sistemleri (PX4, ArduPilot) ile iletişimde MAVLink paketlerinin bayt seviyesinde parse edilmesi ve uçuş modlarının (Offboard, Guided, Loiter, RTL) yönetimi takımlar için büyük bir karmaşa yaratır. Uçuş koridorunun dışına çıkılması (Geofence ihlali) ise yasal ve fiziksel felaketlere yol açar.
- **Çözüm:** Asenkron MAVLink v2 köprüsü, rüzgar sapmasını kompanse eden Park et al. L1 doğrusal olmayan güdüm kanunu, uçağın hücum açısı ve yatışını koruyan 3B uçuş zarfı bekçisi (`FlightEnvelopeGuard`), 3B hava sahası kafesi (`AirGeofence`) ve VTOL geçiş kontrol durum makinesi kurulur.

---

## 2. Modüler Mimari Bileşenleri

```
                         ┌─────────────────────────────┐
                         │    Ham Otopilot Akışı       │
                         │    (Seri Port / UDP MAVLink)│
                         └──────────────┬──────────────┘
                                        │
                         ┌──────────────▼──────────────┐
                         │        MavlinkBridge        │
                         │   Asenkron MAVLink v2       │
                         │  (Offboard Mod Kontrolü)    │
                         └──────────────┬──────────────┘
                                        │
             ┌──────────────────────────┴──────────────────────────┐
             ▼                                                     ▼
┌─────────────────────────────┐                       ┌─────────────────────────────┐
│          L1Guidance         │                       │     FlightEnvelopeGuard     │
│  Park et al. Non-linear L1  │                       │  AoA, Bank Açısı (<=45 deg) │
│  Yanal İvme & Yol Güdümü    │                       │  Stall Önleme & İrtifa Limiti│
└──────────────┬──────────────┘                       └──────────────┬──────────────┘
               │                                                     │
               └──────────────────────────┬──────────────────────────┘
                                          │
                         ┌────────────────▼─────────────┐
                         │         AirGeofence          │
                         │  3B İrtifa ve Poligon Sınırı │
                         │  (İhlalde Otomatik RTL/Fail) │
                         └──────────────┬───────────────┘
                                        │
                         ┌──────────────▼───────────────┐
                         │    VtolTransitionController  │
                         │  Hover -> Blend -> Cruise    │
                         │  Kademeli İtki Transferi     │
                         └──────────────────────────────┘
```

### 2.1. `MavlinkBridge` (Evrensel Otopilot Köprüsü)
- Asenkron Tokio soketleri üzerinde MAVLink v2 `HEARTBEAT`, `ATTITUDE`, `GLOBAL_POSITION_INT` paketlerini dinler.
- Otopilota `SET_POSITION_TARGET_LOCAL_NED` paketi ile hedef hız, ivme veya açısal oran basar.
- Offboard / Guided modunun canlılığını denetler; telemetri kesilirse taban emniyetini uyarır.

### 2.2. `L1Guidance` (L1 Doğrusal Olmayan Yol Güdümü)
- İleri bakış mesafesi $L_1$ ve yol çizgisi ile yer hızı vektörü arasındaki açı $\eta$:
  $$a_s = 2 \frac{V^2}{L_1} \sin \eta$$
- Rüzgar sürüklenmesini otomatik olarak kompanse eder; sabit kanat uçağın virajlarda çizgiden taşmasını engeller.

### 2.3. `FlightEnvelopeGuard` (3B Uçuş Zarfı Koruyucusu)
- **Stall Koruması:** Hava hızı $V_{\text{air}} < V_{\text{stall}}$ olduğunda burnu aşağı ezer.
- **Maksimum Yatış (Bank Angle):** $|\phi| \le 45^\circ$ ile sınırlandırılır.
- **Tırmanma/Dalış Sınırı:** $V_z \in [-10\text{ m/s}, +5\text{ m/s}]$.

### 2.4. `AirGeofence` (3B Hava Sahası Kafesi)
- $Z_{\text{min}} \le z \le Z_{\text{max}}$ irtifa penceresi.
- 2D Yatay Poligon sınırları her 20 ms'de ray-casting algoritması ile denetlenir.
- Sınır ihlalinde üst katman komutları veto edilir; otopilota anında `MAV_CMD_NAV_RETURN_TO_LAUNCH` (RTL) emri verilir.

### 2.5. `VtolTransitionController` (VTOL Geçiş Mantığı)
- Dikey kalkıştan (Multicopter) yatay uçuşa (Fixed-Wing):
  1. *Hover Fazı:* Dikey motorlar irtifa alır.
  2. *Transition/Blend Fazı:* İtici motor öne güç verirken dikey motorlar lift oluşana kadar kademeli kısılır.
  3. *Cruise Fazı:* Dikey motorlar tamamen kapatılır, aerodinamik kontrol yüzeyleri (elevon, rudder) devralır.

---

## 3. Giriş ve Çıkış Veri Sözleşmesi (Data Contracts)

- **Girdiler:**
  - `mavlink_stream`: Seri/UDP ikili bayt akışı
  - `flight_goal`: `[ned_x: f32, ned_y: f32, altitude: f32, speed_m_s: f32]`
- **Çıktılar:**
  - `vehicle_state_6dof`: `StateVector6DoF` (64B C-ABI)
  - `autopilot_target`: `[vx: f32, vy: f32, vz: f32, yaw_rate: f32, mode_flags: u32]`

---

## 4. SITL ve Test İkizi (`mock_mavlink_vehicle`)
- TCP `127.0.0.1:5760` portu üzerinden sanal PX4/ArduPilot otopilotu gibi davranır; 50 Hz telemetri üretir ve komutları doğrular.

---

## 5. Doğrulama Kontrol Listesi
- [ ] `test_l1_crosstrack_correction`: Rüzgar altında çizgiden 5 m sapan uçağın $a_s$ ile merkeze dönmesi.
- [ ] `test_envelope_bank_limit`: $60^\circ$ yatış komutunun $45^\circ$ değerine kırpılması.
- [ ] `test_air_geofence_rtl`: 120m irtifa tavanı aşıldığında RTL komutunun tetiklenmesi.
