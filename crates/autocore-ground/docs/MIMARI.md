# 🚜 autocore-ground :: Mimari Şartname
### Zemin Seyrüsefer, Çoklu Şasi Kinematiği & Reaktif Engel Kaçınma Çekirdeği

**Sürüm:** 0.1.0  
**Durum:** Şartname Mühürlendi  
**Modül:** `crates/autocore-ground`  

---

## 1. Problem - Bağlam - Gerekçe
- **Problem:** Kara robotları (İKA) paletli, diferansiyel, 4 tekerlek yönlendirmeli (Ackermann) veya omni/mecanum gibi çok farklı şasilere sahiptir. Şasi kinematiğinin ve tekerlek kayma faktörünün (wheel slip) seyrüsefer algoritmasıyla iç içe geçmesi, kodun başka bir araca taşınmasını engeller. Ayrıca yol üstünde beliren dinamik engeller karşısında robotun duraksamadan yumuşak manevra yapabilmesi için yerel ızgara haritası ve reaktif kaçınma algoritmalarının mikro saniyeler içinde çalışması şarttır.
- **Çözüm:** Standart Twist komutunu ($v_x, \omega_z$) tek satır yapılandırmayla diferansiyel, Ackermann veya Mecanum tekerlek hızlarına bölen kinematik çözücü; BFS üstel sönümlü 2D yerel maliyet haritası (Costmap2D); Pure Pursuit / Stanley rota takipçileri ve DWA / APF reaktif engel kaçınma motoru kurulur.

---

## 2. Modüler Mimari Bileşenleri

```
                         ┌─────────────────────────────┐
                         │   Sensörler (LiDAR, Sonar)  │
                         │    ve Hedef Rota (Waypoints)│
                         └──────────────┬──────────────┘
                                        │
             ┌──────────────────────────┴──────────────────────────┐
             ▼                                                     ▼
┌─────────────────────────────┐                       ┌─────────────────────────────┐
│          Costmap2D          │                       │       GroundEstimator       │
│  BFS Üstel Şişirme (254..0) │                       │  Tekerlek Enkoderi + IMU    │
│  Lethal / Inflated Katmanı  │                       │  EKF Kayma Dengelemeli Odom │
└──────────────┬──────────────┘                       └──────────────┬──────────────┘
               │                                                     │
               └──────────────────────────┬──────────────────────────┘
                                          │
                         ┌────────────────▼─────────────┐
                         │    DwaPlanner & ApfSolver    │
                         │   Reaktif Dinamik Pencere    │
                         │    ve Sanal İtici Alanlar    │
                         └──────────────┬───────────────┘
                                        │ İstenen [v_x, omega_z]
                         ┌──────────────▼───────────────┐
                         │      ChassisKinematics       │
                         │  Diferansiyel / Ackermann /  │
                         │  Mecanum / Skid-Steer        │
                         └──────────────┬───────────────┘
                                        │
                                        ▼
                         ┌──────────────────────────────┐
                         │      Tekerlek Komutları      │
                         │  (HAL Motor Sürücü PWM/CAN)  │
                         └──────────────────────────────┘
```

### 2.1. `ChassisKinematics` (Çoklu Şasi Kinematik Çözücüsü)
- **Diferansiyel Sürüş:**
  $$v_L = v_x - \frac{\omega_z \cdot L}{2}, \quad v_R = v_x + \frac{\omega_z \cdot L}{2}$$
- **Ackermann Geometrisi:**
  $$\delta_{\text{steer}} = \text{atan2}(\omega_z \cdot L_{\text{wheelbase}}, v_x), \quad v_{\text{drive}} = v_x$$
- **Mecanum (Omni-Yönlü 4 Tekerlek):**
  $$v_i = v_x \pm v_y \pm \omega_z \cdot (d_x + d_y)$$

### 2.2. `Costmap2D` (2B Yerel Izgara Haritası & Şişirme)
- 2D dizi ($W \times H$) çözünürlük $r$ (örn. $0.05\text{ m/hücre}$).
- Ölümcül Engel (Lethal = 254), Serbest Alan (Free = 0).
- BFS algoritması ile engelin etrafına robot yarıçapı $R_{\text{robot}}$ kadar ölümcül halka, $R_{\text{inflation}}$ mesafesine kadar üstel sönümlü güvenlik maliyeti yayılır:
  $$\text{Cost}(d) = 253 \cdot \exp(-\alpha \cdot (d - R_{\text{robot}}))$$

### 2.3. `DwaPlanner & ApfSolver` (Engel Kaçınma)
- **DWA (Dynamic Window Approach):** Aracın mevcut hız ve ivme limitleri içindeki $(v, \omega)$ uzayını örnekler; çarpışmasız, hedefe en hızlı yaklaştıran adımı seçer.
- **APF (Artificial Potential Fields):** Hedefe çekici kuvvet ($F_{\text{att}} = k_a (p_{\text{goal}} - p)$), engellere itici kuvvet ($F_{\text{rep}} = k_r (\frac{1}{d} - \frac{1}{d_0}) \frac{1}{d^2}$) uygulayarak vektörel kaçış açısı üretir.

### 2.4. `PathTracker` (Yol Takip Kontrolcüsü)
- **Pure Pursuit:** İleri bakış mesafesi $L_d$ üzerindeki hedef noktaya doğru eğrilik:
  $$\kappa = \frac{2 \sin \alpha}{L_d} \implies \omega_z = v_x \cdot \kappa$$
- **Stanley Kontrolcüsü:** Yanal mesafe hatasını ($e_{fa}$) ve pruva hatasını ($\theta_e$) ön tekerlek açısına çevirir.

---

## 3. Giriş ve Çıkış Veri Sözleşmesi (Data Contracts)

- **Girdiler:**
  - `lidar_ranges`: `Vec<f32>` (360 derece mesafe dizisi)
  - `nav_goal`: `[target_x: f32, target_y: f32, target_heading: f32]`
  - `encoder_ticks`: `[left_ticks: i64, right_ticks: i64, dt_us: u32]`
- **Çıktılar:**
  - `drive_twist`: `KinematicCommand` (32B C-ABI: `linear_x`, `angular_z`)
  - `wheel_actuators`: `[w1_rad_s, w2_rad_s, w3_rad_s, w4_rad_s]`

---

## 4. SITL ve Test İkizi (`mock_ground_chassis`)
- Linux sanal terminali veya SocketCAN `vcan0` üzerinde motor komutlarını okur.
- Tekerlek kayma katsayısı ($\mu = 0.85$) ve eyleyici ataleti uygulayarak 50 Hz gerçekçi odometri geri beslemesi üretir.

---

## 5. Doğrulama Kontrol Listesi
- [ ] `test_diff_drive_roundtrip`: $v=2.0\text{ m/s}, \omega=0.5\text{ rad/s}$ için sağ/sol tekerlek hızlarının simetrik ayrışması.
- [ ] `test_costmap_inflation`: Ölümcül hücrenin etrafında üstel maliyet katmanının yarıçap boyunca düşmesi.
- [ ] `test_apf_obstacle_repulsion`: Yol üstündeki engele yaklaşırken çekici kuvvetin itici kuvvet tarafından kırılması.
