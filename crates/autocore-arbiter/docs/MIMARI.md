# ⚖️ autocore-arbiter :: Mimari Şartname
### Komut Hakemi, Öncelik Hiyerarşisi & Çakışma Çözücü Çekirdeği

**Sürüm:** 0.1.0  
**Durum:** Şartname Mühürlendi  
**Modül:** `crates/autocore-arbiter`  

---

## 1. Problem - Bağlam - Gerekçe
- **Problem:** Otonom robotlarda birden fazla düğüm aynı anda hareket veya aktüatör komutu üretir:
  1. *Kullanıcı Görev Kodu (Mission Node):* "Hedefe doğru $2.5\text{ m/s}$ hızla sür."
  2. *Reaktif Engel Kaçınma (Costmap/DWA):* "Önünde 1 metrede kaya var, sağa dön veya dur!"
  3. *Taban Emniyet Çekirdeği (Sentinel Watchdog / Geofence / E-Stop):* "Sanal sınır aşıldı veya Watchdog gecikti, acil kilitlen!"
  Eğer bu komutlar arasında merkezi, katı ve deterministik bir hakem (arbiter) bulunmazsa; yarış durumu (race condition) oluşur. Üst seviye yapay zekâ kodu güvenlik emrini ezebilir ve araç duvara toslayabilir veya insana çarpabilir.
- **Çözüm:** 3 seviyeli katı öncelik hiyerarşisi (Strict Priority Tiers) ile çalışan, mikrosaniye seviyesinde deterministik komut filtreleme yapan ve her zaman en yüksek emniyet seviyesini aktüatöre geçiren `autocore-arbiter` kurulur.

---

## 2. Modüler Mimari Bileşenleri

```
┌────────────────────────────────────────────────────────┐
│     [ Seviye 1: Veto Yetkisi ]                         │
│     autocore Sentinel, E-Stop, SafeLock, Geofence Inhibit│
└───────────────────────────┬────────────────────────────┘
                            │ (Tetiklendiyse -> NÖTR FREN [0,0,0])
                            ▼
┌────────────────────────────────────────────────────────┐
│     [ Seviye 2: Reaktif Güvenlik ]                     │
│     autocore-ground DWA/APF, autocore-aerial Envelope  │
└───────────────────────────┬────────────────────────────┘
                            │ (Engel Varsa -> Kaçınma Manevrası)
                            ▼
┌────────────────────────────────────────────────────────┐
│     [ Seviye 3: Görev Katmanı ]                        │
│     Takımın Yazdığı Python / C++ Görev Algoritması     │
└───────────────────────────┬────────────────────────────┘
                            │ (Yalnızca Güvenliyse Geçirilir)
                            ▼
               ┌────────────────────────┐
               │  NİHAİ AKTÜATÖR KOMUTU │
               │   (Motor Sürücü HAL)   │
               └────────────────────────┘
```

### 2.1. Öncelik Seviyeleri (Priority Tiers)

1. **Tier 1 (Mutlak Emniyet & Veto - Öncelik 1):**
   - **Kaynak:** `autocore-safety` (FSM `SafeLock` modu, donanımsal E-Stop, 500 ms Watchdog aşımı, Geofence Inhibit).
   - **Davranış:** Alt seviyelerden gelen tüm hız ve taret komutlarını anında çöpe atar. Eyleyicilere koşulsuz nötr fren (`linear = 0.0, angular = 0.0`) gönderir ve hayalet hareket kuyruğunu boşaltır.

2. **Tier 2 (Reaktif Çarpışma Önleme - Öncelik 2):**
   - **Kaynak:** `autocore-ground` (DWA / APF reaktif engel kaçınma), `autocore-aerial` (Stall / AoA koruyucusu).
   - **Davranış:** Araç serbest seyrederken aniden önüne bir engel veya dinamik tehdit çıktığında; Tier 3 görev kodunun "İleri git" hedefini maskeler ve aracı çarpışmadan kurtaracak acil kaçınma açısını aktüatöre geçirir.

3. **Tier 3 (Görev ve Karar Katmanı - Öncelik 3):**
   - **Kaynak:** Takımın yazdığı otonom görev kararı (Dora-rs Python / Rust görev düğümü).
   - **Davranış:** Yalnızca ve yalnızca Tier 1 (güvenlik) ve Tier 2 (engel) temiz olduğunda aktüatörleri sürer.

### 2.2. Yumuşak Geçiş ve Histerezis (Smooth Blending)
- Tier 2 engel kaçınma manevrası bittiğinde araç aniden sarsılarak Tier 3 görev hedefine fırlamaz.
- `SlewRateLimiter` ile açısal ve çizgisel hızlar yumuşatılarak görev hedefine kademeli devredilir.

---

## 3. Giriş ve Çıkış Veri Sözleşmesi (Data Contracts)

- **Girdiler:**
  - `tier1_safety_status`: `SafetyMode` (Booting..SafeLock) + `estop_active: bool`
  - `tier2_reactive_cmd`: `Option<KinematicCommand>` (Varsa engel kaçınma komutu)
  - `tier3_mission_cmd`: `KinematicCommand` (Kullanıcı görev komutu)
- **Çıktılar:**
  - `arbitrated_cmd`: 36 Bayt C-ABI Struct:
    ```rust
    #[repr(C)]
    pub struct ArbitratedCommand {
        pub command: KinematicCommand, // 32 bayt
        pub active_tier: u8,          // 1, 2 veya 3
        pub override_reason: u8,       // 0: Normal, 1: E-Stop, 2: Obstacle, 3: Watchdog
        pub reserved: u16,
    }
    ```

---

## 4. SITL ve Test İkizi (`mock_arbiter_harness`)
- 50 Hz döngüde görev katmanından sürekli tam gaz ileri komutu verirken, 30. adımda sanal engel (Tier 2) ve 60. adımda E-Stop (Tier 1) tetikler; komutun $<50\ \mu\text{s}$ içinde doğru seviyeye pre-empt edildiğini doğrular.

---

## 5. Doğrulama Kontrol Listesi
- [ ] `test_tier1_preemption`: Tier 3 son sürat giderken E-Stop geldiğinde aktüatörün anında sıfırlanması (`active_tier == 1`).
- [ ] `test_tier2_obstacle_masking`: Önünde engel belirdiğinde görev komutunun engellenip kaçınma komutunun geçmesi (`active_tier == 2`).
- [ ] `test_clear_corridor_passthrough`: Engel ve emniyet sorunu yokken görev komutunun birebir iletilmesi (`active_tier == 3`).
