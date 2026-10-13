# 👁️ autocore-vision :: Mimari Şartname
### Algılama, Çıkarım, Çoklu Hedef Takip & Uzaysal İzdüşüm Çekirdeği

**Sürüm:** 0.1.0  
**Durum:** Şartname Mühürlendi  
**Modül:** `crates/autocore-vision`  

---

## 1. Problem - Bağlam - Gerekçe
- **Problem:** Kenar bilgisayarlarda (NVIDIA Jetson, Rockchip NPU) görüntü işleme boru hatları ağır bellek kopyalamaları ve verimsiz takipçiler nedeniyle kare düşmesine (frame drop) ve 100 ms üzeri gecikmelere yol açar. Ayrıca 2D piksel kutularının araca göre 3B metre koordinatına çevrilememesi, otonom seyrüsefer ve taret angajmanını imkansız kılar. Anlık ışık parlamaları ve optik karıştırma ise sahte düşman/hedef alarmları üretir.
- **Çözüm:** `/dev/shm` paylaşılan bellek üzerinden sıfır kopyalama tensör çıkarımı yapan, ByteTrack/SORT ile hedefleri zamansal olarak sabitleyen (ID persistence), kamera iç kalibrasyonu ($K$) ile 2D pikseli 3B uzaya izdüşüren ve çoğunluk oylamalı dinamik IFF filtresiyle parlamaları süzen bağımsız bir algılama çekirdeği kurulur.

---

## 2. Modüler Mimari Bileşenleri

```
                         ┌─────────────────────────────┐
                         │   Shared Memory Camera Feed │
                         │     (Apache Arrow Record)   │
                         └──────────────┬──────────────┘
                                        │
                         ┌──────────────▼──────────────┐
                         │       InferenceEngine       │
                         │    ONNX Runtime / TensorRT  │
                         │    (Zero-Heap Yolo/NMS-Free)│
                         └──────────────┬──────────────┘
                                        │ 2D Bounding Boxes
                         ┌──────────────▼──────────────┐
                         │        SpatialTracker       │
                         │    ByteTrack / SORT Kalman  │
                         │   (ID Retention & Coasting) │
                         └──────────────┬──────────────┘
                                        │
             ┌──────────────────────────┴──────────────────────────┐
             ▼                                                     ▼
┌─────────────────────────────┐                       ┌─────────────────────────────┐
│        DepthProjector       │                       │       HypothesisFilter      │
│  Pinhole Matrisi K + Z Mes. │                       │  Dinamik IFF Çoğunluk Oyu   │
│  2D Piksel -> 3D Metre [X,Y]│                       │  Optik Karıştırma Reddi     │
└─────────────────────────────┘                       └─────────────────────────────┘
```

### 2.1. `InferenceEngine` (Çıkarım Arka Ucu)
- TensorRT C-API veya `ort` (ONNX Runtime) arka ucu ile NPU/GPU hızlandırması.
- Giriş tensörleri paylaşılan bellek işaretçisi (`/dev/shm`) üzerinden sıfır kopyalama ile modele beslenir.
- NMS-free (End-to-End) YOLO mimarileri desteklenerek CPU tarafında NMS maliyeti sıfırlanır.

### 2.2. `SpatialTracker` (Zamansal Çoklu Hedef Takipçisi)
- 8 Durumlu Kalman Durum Modeli: $[u, v, s, r, \dot{u}, \dot{v}, \dot{s}, 0]^T$
- Yüksek ve düşük güvenilirlikli tespitleri iki aşamalı açgözlü IoU (Greedy IoU) eşleme ile bağlayan ByteTrack algoritması.
- **Coasting (Kör Uçuş):** Hedef kamera açısından anlık olarak bir direğin veya engelin arkasına geçtiğinde takip 5-10 kare boyunca Kalman hızıyla sürdürülür; hedef tekrar belirdiğinde aynı ID ile devam eder.

### 2.3. `DepthProjector` (3B Uzaysal İzdüşüm)
- İğne deliği kamera modeli (Pinhole Camera Model) ve iç kalibrasyon matrisi $K$:
  $$K = \begin{bmatrix} f_x & 0 & c_x \\ 0 & f_y & c_y \\ 0 & 0 & 1 \end{bmatrix}$$
- Piksel merkezinden Kartezyen mesafe çözümü:
  $$X_c = \frac{(u - c_x) Z_c}{f_x}, \quad Y_c = \frac{(v - c_y) Z_c}{f_y}$$

### 2.4. `HypothesisFilter` (Dinamik IFF ve Histerezis)
- Güneş parlamaları, far yansımaları ve optik karıştırma sahte hedef alarmı üretemez.
- En az `min_samples` (örn. 5 kare) boyunca teyit edilmeyen hedefler `Unknown` statüsünde bekletilir.
- Çoğunluk oylaması eşiği ($\ge \%70$) aşıldığında hedef `Confirmed` (Dost/Düşman) olarak yayınlanır.

---

## 3. Giriş ve Çıkış Veri Sözleşmesi (Data Contracts)

- **Girdiler:**
  - `camera_frame`: Arrow Buffer (`shm_handle`, `width`, `height`, `stride`, `format`, `timestamp_us`)
- **Çıktılar:**
  - `detected_tracks`: Apache Arrow RecordBatch tablosu:
    - `track_id: u32`
    - `class_id: u16`
    - `confidence: f32`
    - `x: f32, y: f32, z: f32` (Kamera optik ekseninde metre)
    - `vx: f32, vy: f32, vz: f32` (Metre/saniye)
    - `timestamp_us: u64`

---

## 4. SITL ve Test İkizi (`synthetic_camera_feed`)
- Yerel bir video dosyasını (`test_flight.mp4`) veya matematiksel olarak üretilen sentetik BBox'ları 50 Hz'de belleğe basarak fiziksel kamerasız uçtan uca test imkanı sunar.

---

## 5. Doğrulama Kontrol Listesi
- [ ] `test_spatial_tracker_coasting`: 3 kare kaybolan hedefin aynı ID ve doğru tahmin koordinatıyla devam etmesi.
- [ ] `test_depth_projection`: Bilinen odak uzaklığında $u=c_x, v=c_y, Z=10\text{m}$ için $X=0, Y=0$ çıkması.
- [ ] `test_iff_optical_glare_rejection`: Tek karelik parlamaların onaylı hedefe dönüşmemesi.
