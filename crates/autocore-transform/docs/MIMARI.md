# 📐 autocore-transform :: Mimari Şartname
### Koordinat Ağacı, $SE(3)$ Lie Grubu & Küresel Seyrüsefer Çekirdeği

**Sürüm:** 0.1.0  
**Durum:** Şartname Mühürlendi  
**Modül:** `crates/autocore-transform`  

---

## 1. Problem - Bağlam - Gerekçe
- **Problem:** Otonom araçlarda kamera, LiDAR, taret, tekerlekler ve antenler farklı fiziksel konumlarda ve farklı açısal yönelimlerde monte edilir. Kameranın optik çerçevesinde ($Z$ ileri, $X$ sağ, $Y$ aşağı) tespit edilen bir hedefin metre koordinatı doğrudan gövdeye veya tarete verilirse, eksen uyumsuzluğu ve montaj mesafesi (paralaks) nedeniyle araç veya taret hedefin metrelerce uzağına bakar. Ayrıca küresel GNSS (WGS84) verisi ile yerel Kartezyen (NED) koordinatları arasındaki dönüşümün hantal kütüphanelerle yapılması mikro saniyelik kontrol döngülerinde faz gecikmesi yaratır.
- **Çözüm:** $SE(3)$ Lie grubu matematiğini kilitsiz (`ArcSwap`) $4\times 4$ homojen dönüşüm matrisleriyle tutan, kamera optik eksenini mekanik eksene sıfır kopyalama ile çeviren ve WGS84 $\longleftrightarrow$ ECEF $\longleftrightarrow$ NED dönüşümlerini $<1\text{ mm}$ hata payıyla çözen ultra hafif bir koordinat omurgası kurulur.

---

## 2. Modüler Mimari Bileşenleri

```
                         ┌─────────────────────────────┐
                         │      transforms.toml        │
                         │ (Statik Eklem/Sensör Ofseti)│
                         └──────────────┬──────────────┘
                                        │
                         ┌──────────────▼──────────────┐
                         │        TransformTree        │
                         │    Lock-Free ArcSwap SE(3)  │
                         │  (base_link, camera, turret)│
                         └──────────────┬──────────────┘
                                        │
             ┌──────────────────────────┴──────────────────────────┐
             ▼                                                     ▼
┌─────────────────────────────┐                       ┌─────────────────────────────┐
│        GeoConverter         │                       │       FrameProjector        │
│  WGS84 <-> ECEF <-> NED/ENU │                       │  Kamera Optik (Z-ileri) ->  │
│    Elipsoit Geodezi Motoru  │                       │  Mekanik Gövde (X-ileri)    │
└─────────────────────────────┘                       └─────────────────────────────┘
```

### 2.1. `TransformTree` (Homojen Koordinat Ağacı)
- Rijit cisim dönüşümleri $T \in SE(3)$ ile ifade edilir:
  $$T = \begin{bmatrix} R & \mathbf{p} \\ \mathbf{0}^T & 1 \end{bmatrix}, \quad R \in SO(3), \ \mathbf{p} \in \mathbb{R}^3$$
- Yönelimler Gimbal Lock riskini önlemek için birim kuaterniyon $\mathbf{q} = (q_w, q_x, q_y, q_z)$ ile tutulur.
- Çerçeveler arası zincirleme dönüşüm:
  $$T_{A \to C} = T_{A \to B} \cdot T_{B \to C}$$

### 2.2. `GeoConverter` (Küresel Elipsoit Motoru)
- WGS84 parametreleri: Yarı büyük eksen $a = 6378137.0\text{ m}$, basıklık $f = 1 / 298.257223563$.
- `geodetic_to_ecef(lat, lon, alt)` ve `ecef_to_ned(x, y, z, ref_lat, ref_lon, ref_alt)` mikro saniyelik formüllerle çalışır.
- Yerküre eğriliği $z$ ekseninde hesaba katılır, Kartezyen mesafe ve yön açısı (bearing) kesin hesaplanır.

### 2.3. `FrameProjector` (Optik/Mekanik Eksen Dönüştürücü)
- Bilgisayarlı görü standartları görüntüyü $Z$ ileri, $X$ sağ, $Y$ aşağı olarak tanımlar.
- Robotik/havacılık mekanik standartları ise $X$ ileri, $Y$ sol, $Z$ yukarı (veya NED: $X$ kuzey, $Y$ doğu, $Z$ aşağı) kullanır.
- Sabit ortogonal dönüşüm tensörü ile sıfır bellek kopyalaması ile dönüştürülür:
  $$\mathbf{p}_{\text{mech}} = \begin{bmatrix} 0 & 0 & 1 \\ -1 & 0 & 0 \\ 0 & -1 & 0 \end{bmatrix} \mathbf{p}_{\text{optic}}$$

---

## 3. Giriş ve Çıkış Veri Sözleşmesi (Data Contracts)

- **Girdiler:**
  - `gnss_raw`: `[lat: f64, lon: f64, alt: f64, hdop: f32, timestamp_us: u64]` (32B)
  - `joint_states`: `Vec<JointState>` (`joint_name`, `angle_rad`, `velocity_rad_s`)
- **Çıktılar:**
  - `local_pose`: `[x: f32, y: f32, z: f32, qw: f32, qx: f32, qy: f32, qz: f32, timestamp_us: u64]` (36B C-ABI)
  - `tf_static_tree`: Bellekte paylaşılan $SE(3)$ dönüşüm matrisleri tablosu.

---

## 4. SITL ve Test İkizi (`mock_transform_broadcaster`)
- Donanımsal sensör veya eklem olmadan `transforms.toml` dosyasındaki kinematik zinciri okur.
- 50 Hz frekansında sanal şasi salınımı ve kamera yönelimi basarak tüm boru hattını test eder.

---

## 5. Doğrulama Kontrol Listesi
- [ ] `test_geo_conversions`: WGS84 $\longleftrightarrow$ NED dönüşümünde $<1\text{ mm}$ hata payı.
- [ ] `test_optical_to_mechanical_projection`: Kamera eksenindeki $(0, 0, 10\text{m})$ vektörünün gövdede $(10\text{m}, 0, 0)$ çıkması.
- [ ] `test_transform_tree_chaining`: `map` $\to$ `base_link` $\to$ `camera_link` $\to$ `target` zincir çarpımı doğruluğu.
