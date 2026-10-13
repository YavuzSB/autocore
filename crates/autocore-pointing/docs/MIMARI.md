# 🎯 autocore-pointing :: Mimari Şartname
### Yönlendirme, Taret, Gimbal, Balistik Önleme & Angajman Çekirdeği

**Sürüm:** 0.1.0  
**Durum:** Şartname Mühürlendi  
**Modül:** `crates/autocore-pointing`  

---

## 1. Problem - Bağlam - Gerekçe
- **Problem:** Hava savunma taretleri, İHA altı elektro-optik gimballer ve yönlendirilebilir antenler hareketli hedeflere kilitlenirken iki ölümcül hata ile karşılaşır:
  1. *Boru Hattı Gecikmesi (Pipeline Lag):* Görüntünün kameradan çıkıp servo motorlara ulaşması arasında geçen 40–80 milisaniye içinde hedef hareket eder; doğrudan hedefin görüldüğü noktaya nişan alınırsa taret sürekli hedefin arkasını vurur.
  2. *Balistik ve Yerçekimi Düşüşü:* Mühimmatın veya lazerin hedefe varış süresi ($t_{\text{flight}}$) ve yerçekimi ($g$) hesaba katılmadan namlu açıları bulunamaz.
  3. *Mekanik Rezonans ve Titreşim:* Taret hedefe doğru hızla dönerken tam durma anında servo salınımı yapar. Bu salınım sönümlenmeden ateş açılırsa vuruş olasılığı sıfıra düşer.
- **Çözüm:** Sensör gecikmesini hız vektörüyle telafi eden ölü hesaplama (Dead Reckoning), 2. derece kuadratik önleme kurşun açısı çözücüsü (Lead-Angle Solver), 2-DoF ters kinematik ve salınım sönümlenmeden tetik izni vermeyen mekanik eylem disiplini kilidi (`ActionDiscipline`) inşa edilir.

---

## 2. Modüler Mimari Bileşenleri

```
                         ┌─────────────────────────────┐
                         │    Hedef Vektörü & Hızı     │
                         │  (autocore-transform çıkışı)│
                         └──────────────┬──────────────┘
                                        │
                         ┌──────────────▼──────────────┐
                         │      LatencyCompensator     │
                         │   Ölü Hesaplama Telafisi    │
                         │   P_real = P + V * delta_t  │
                         └──────────────┬──────────────┘
                                        │
                         ┌──────────────▼──────────────┐
                         │       LeadAngleSolver       │
                         │   2. Derece Kuadratik Kök   │
                         │   + Yerçekimi Düşüş Açısı   │
                         └──────────────┬──────────────┘
                                        │
             ┌──────────────────────────┴──────────────────────────┐
             ▼                                                     ▼
┌─────────────────────────────┐                       ┌─────────────────────────────┐
│    InverseKinematics2DoF    │                       │       ActionDiscipline      │
│  Kartezyen -> Pan / Tilt    │                       │  Tolerans Penceresi (eps)   │
│  Açı ve Açısal Hız (rad/s)  │                       │  Salınım Sönümü & Cooldown  │
└─────────────────────────────┘                       └─────────────────────────────┘
```

### 2.1. `LatencyCompensator` (Boru Hattı Gecikme Telafisi)
- Kameranın kareyi yakaladığı donanımsal an ($t_{\text{kamera}}$) ile taretin hedefe yöneleceği an ($t_{\text{eylem}}$) arasındaki gecikme süresi $\Delta t = t_{\text{eylem}} - t_{\text{kamera}}$ hesaplanır.
- Gelecekteki gerçek hedef konumu ekstrapole edilir:
  $$\vec{P}_{\text{gerçek}} = \vec{P}_{\text{tespit}} + \vec{V}_{\text{hedef}} \cdot \Delta t$$

### 2.2. `LeadAngleSolver` (Önleme Açısı & Balistik Çözücü)
- Önleyici mühimmat veya füzenin hızı $v_p$, hedefin göreli konumu $\mathbf{r} = \mathbf{p}_{\text{target}} - \mathbf{p}_{\text{turret}}$ ve hedef hızı $\mathbf{v}$ olsun.
- Kesişim süresi $t$ için 2. derece kuadratik denklem:
  $$\|\mathbf{r} + \mathbf{v} t\|^2 = (v_p t)^2 \implies (v^2 - v_p^2) t^2 + 2 (\mathbf{r} \cdot \mathbf{v}) t + r^2 = 0$$
- Diskriminant $\Delta \ge 0$ ise zamanda ileri yöndeki en küçük pozitif gerçek kök $t > 0$ seçilir.
- Mühimmat uçuş süresindeki yerçekimi düşüşü ($\Delta z = \frac{1}{2} g t^2$) hedefe eklenerek namlu yukarı kaldırılır.

### 2.3. `InverseKinematics2DoF` (Ters Kinematik)
- Hedef konumu $\mathbf{p}^* = [X^*, Y^*, Z^*]^T$:
  $$\text{Azimut (Pan)} = \text{atan2}(Y^*, X^*)$$
  $$\text{Elevasyon (Tilt)} = \text{atan2}\left(Z^*, \sqrt{{X^*}^2 + {Y^*}^2}\right)$$
- Sınır kelepçelemesi: Azimut $[-180^\circ, +180^\circ]$, Elevasyon $[-15^\circ, +85^\circ]$.

### 2.4. `ActionDiscipline` (Tetik ve Eylem Güvenlik Kilidi)
- Taret hedefe dönse bile `fire_gate` sinyali şu 4 kural sağlanmadan **ASLA TRUE OLAMAZ:**
  1. *Açısal Hata Eşiği:* $|\Delta \theta_{\text{az}}| \le \epsilon$ ve $|\Delta \theta_{\text{el}}| \le \epsilon$ (örn. $\le 0.5^\circ$).
  2. *Mekanik Salınım Sönümü:* Açısal hız $|\omega| \le \omega_{\text{damp}}$ eşiğinin altına inmiş olmalı ve en az $T_{\text{settling}}$ (örn. 50 ms) boyunca orada kalmalıdır.
  3. *Soğuma Süresi (Cooldown):* Son atışın üzerinden $T_{\text{cool}}$ süresi geçmiş olmalıdır.
  4. *Taban Güvenliği:* `autocore` emniyet durumu `Armed` veya `Mission` olmalıdır.

---

## 3. Giriş ve Çıkış Veri Sözleşmesi (Data Contracts)

- **Girdiler:**
  - `target_track`: `[x, y, z, vx, vy, vz, timestamp_us]`
  - `turret_feedback`: `[current_pan_rad, current_tilt_rad, pan_velocity, tilt_velocity]`
- **Çıktılar:**
  - `actuator_setpoints`: C-ABI Struct (20 bayt):
    ```rust
    #[repr(C)]
    pub struct PointingCommand {
        pub target_pan_rad: f32,
        pub target_tilt_rad: f32,
        pub target_pan_rate_rad_s: f32,
        pub target_tilt_rate_rad_s: f32,
        pub fire_gate: u8, // 1: Ateş serbest, 0: Kilitli
    }
    ```

---

## 4. SITL ve Test İkizi (`mock_pan_tilt_rig`)
- Linux PTY üzerinden taret servolarının fiziksel ataletini ($\tau = 40\text{ ms}$) simüle eder.
- `fire_gate` açıkken sanal mermi geometrisini test ederek hedef vuruş olasılığını (Pk) loglar.

---

## 5. Doğrulama Kontrol Listesi
- [ ] `test_latency_compensation`: 50 ms gecikmede $20\text{ m/s}$ hedefin tam $1.0\text{ m}$ ileri ekstrapole edilmesi.
- [ ] `test_lead_angle_quadratic`: Mach 3.5 çapraz hedefe karşı $<1\text{ m}$ hata payı ile pozitif $t$ kökü.
- [ ] `test_action_discipline_veto`: Açısal salınım sürerken `fire_gate == 0` olduğunu kanıtlama.
