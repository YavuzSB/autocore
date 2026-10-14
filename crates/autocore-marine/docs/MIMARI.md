# 🌊 autocore-marine :: Mimari Şartname
### Hidrodinamik Seyrüsefer, İtki Dağıtımı & Akıntı Dengeleme Çekirdeği

**Sürüm:** 0.1.0  
**Durum:** Şartname Mühürlendi  
**Modül:** `crates/autocore-marine`  

---

## 1. Problem - Bağlam - Gerekçe
- **Problem:** İnsansız Deniz Araçları (İDA) ve su altı robotları (ROV/AUV), zemin veya hava araçlarından çok farklı olarak yüksek viskoziteli ve sürekli hareket halindeki bir akışkan içinde çalışır. Deniz akıntısı ve dalgalar aracı sürekli rotasından sürükler; pusula hedefi gösterse bile araç yanal kayarak kayalara veya şamandıraya çarpabilir. Ayrıca çift pervaneli botlardan 6-8 motorlu su altı araçlarına kadar genel kuvvet taleplerinin ($F_x, F_y, M_z$) motor bazlı itki yüzdelerine dağıtılması gerekir.
- **Çözüm:** Hidrodinamik itki dağıtım matrisi (Thruster Allocation Matrix), akıntı ve rüzgar kaynaklı yanal sapmayı integral hatasıyla pruva yengeç açısına (Crab Angle) çeviren sürüklenme kompanzatörü (`DriftCompensator`) ve kavitasyon/devrilme önleyici su üstü rota çözücüsü (`MarineNavigator`) kurulur.

---

## 2. Modüler Mimari Bileşenleri

```
                         ┌─────────────────────────────┐
                         │   Kuvvet & Seyrüsefer Talebi │
                         │      [Fx, Fy, Torque_Z]     │
                         └──────────────┬──────────────┘
                                        │
             ┌──────────────────────────┴──────────────────────────┐
             ▼                                                     ▼
┌─────────────────────────────┐                       ┌─────────────────────────────┐
│       DriftCompensator      │                       │       MarineNavigator       │
│  Akıntı / Dalga Kompanzasyonu│                       │  Şamandıra / Koridor Takibi │
│  Yengeç Açısı (Crab Angle)  │                       │  Kavitasyon Önleyici Profil │
└──────────────┬──────────────┘                       └──────────────┬──────────────┘
               │                                                     │
               └──────────────────────────┬──────────────────────────┘
                                          │ Düzeltilmiş Kuvvet Talebi
                         ┌────────────────▼─────────────┐
                         │   ThrusterAllocationMatrix   │
                         │    Genel [Fx, Fy, Mz] ->     │
                         │  Motor 1..N PWM / CAN Sinyali│
                         └──────────────┬───────────────┘
                                        │
                                        ▼
                         ┌──────────────────────────────┐
                         │       İtici Sinyalleri       │
                         │  (Çift Pervane, Dümen, Jet)  │
                         └──────────────────────────────┘
```

### 2.1. `ThrusterAllocationMatrix` (İtki Dağıtım Matrisi)
- $m$ DoF kuvvet/tork vektörü $\boldsymbol{\tau} = [F_x, F_y, M_z]^T$.
- $n$ adet itici motorun konfigürasyon matrisi $B \in \mathbb{R}^{m \times n}$.
- İtici kuvvetleri $\mathbf{u} = [u_1, u_2, \dots, u_n]^T$:
  $$\boldsymbol{\tau} = B \mathbf{u} \implies \mathbf{u} = B^\dagger \boldsymbol{\tau} = B^T (B B^T)^{-1} \boldsymbol{\tau}$$
- Çift motorlu diferansiyel İDA için:
  $$u_{\text{sol}} = \frac{F_x}{2} - \frac{M_z}{2 d_{\text{beam}}}, \quad u_{\text{sağ}} = \frac{F_x}{2} + \frac{M_z}{2 d_{\text{beam}}}$$

### 2.2. `DriftCompensator` (Akıntı ve Dalga Sürüklenme Kompanzatörü)
- Akıntı aracı yana sürüklediğinde araç rotadan sapar.
- Yanal mesafe hatası $e_y$ integral süzgeciyle toplanır:
  $$\beta_{\text{crab}} = \text{atan2}\left(v_{\text{akıntı\_yanal}} + k_i \int e_y dt, v_{\text{ileri}}\right)$$
- Araç burnunu akıntıya doğru $\beta$ kadar kırar; araç pruvası eğik dursa da yer hızı vektörü hedef rota çizgisi üzerinde dümdüz ilerler.

### 2.3. `MarineNavigator` (Su Üstü Rota Çözücü)
- Keskin dönüşlerde yüksek hız kavitasyona (pervane boşa dönmesi) ve su üstü aracının alabora olmasına (roll over) yol açar.
- Şamandıra etrafında yumuşak kavisli eğrilikler ($\kappa \le \kappa_{\text{max}}$) ve hız sönümleme profilleri üretir.

---

## 3. Giriş ve Çıkış Veri Sözleşmesi (Data Contracts)

- **Girdiler:**
  - `marine_force_req`: `[fx_newton: f32, fy_newton: f32, torque_z_nm: f32]`
  - `current_pose`: `[x: f32, y: f32, heading_rad: f32, speed_knots: f32]`
- **Çıktılar:**
  - `thruster_signals`: C-ABI Struct (16 bayt):
    ```rust
    #[repr(C)]
    pub struct ThrusterSignals {
        pub port_pct: f32,      // İskele / Sol motor (-1.0 .. +1.0)
        pub starboard_pct: f32, // Sağ motor (-1.0 .. +1.0)
        pub rudder_angle_rad: f32, // Varsa dümen açısı
        pub reserved: u32,
    }
    ```

---

## 4. SITL ve Test İkizi (`mock_marine_simulator`)
- Sanal su akıntısı ($1.5\text{ m/s}$ yanal akıntı) ve dalga salınımı enjekte ederek İDA'nın akıntı kompanzasyonunu test eden yerel döngü.

---

## 5. Doğrulama Kontrol Listesi
- [ ] `test_thruster_allocation_matrix`: $F_x = 100\text{ N}, M_z = 0$ için iskele ve sağ motorlara eşit itki dağıtılması.
- [ ] `test_drift_compensation_crabbing`: Yanal akıntıda pruvanın akıntıya doğru açı alarak yanal hatanın sıfırlanması.
- [ ] `test_cavitation_rate_limit`: Ani tork değişimlerinde motor ivmesinin sınırlandırılması.
