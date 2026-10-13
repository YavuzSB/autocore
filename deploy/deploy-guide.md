# 🚀 Autocore Operasyonel Dağıtım ve Saha Kurulum Rehberi (Deployment Guide)

Bu rehber, **Autocore Autonomous Robotics Runtime & Safety Kernel**'in Raspberry Pi 5, NVIDIA Jetson Orin/Nano veya endüstriyel x86/ARM görev bilgisayarlarında deterministik ve gerçek zamanlı (`Real-Time SCHED_FIFO 99`) olarak devreye alınmasını belgeler.

---

## 1. Sistem Gereksinimleri ve Çekirdek Yapılandırması

### 1.1 Gerçek Zamanlı Linux Öncelikleri (`/etc/security/limits.conf`)
Gerçek zamanlı FIFO önceliği (`SCHED_FIFO 99`) ve bellek kitleme (`mlockall`) yetkisi için aşağıdaki satırları `/etc/security/limits.d/99-autocore-rt.conf` dosyasına ekleyin:

```ini
*               soft    rtprio          99
*               hard    rtprio          99
*               soft    memlock         unlimited
*               hard    memlock         unlimited
```

### 1.2 Sıfır Kopyalama Paylaşılan Bellek (`/dev/shm`)
Apache Arrow tamponlarının yüksek çözünürlüklü görüntü ve telemetri için yeterli alana sahip olduğundan emin olun:

```bash
# /dev/shm boyutunu 2GB veya 4GB olarak bağlayın (gerekirse /etc/fstab içine ekleyin)
sudo mount -o remount,size=2G /dev/shm
```

---

## 2. systemd Servis Kurulumu

```bash
# 1. Servis dosyasını kopyalayın
sudo cp deploy/autocore.service /etc/systemd/system/autocore.service

# 2. İkili dosyayı ve yapılandırmayı yerleştirin
sudo mkdir -p /opt/autocore /etc/autocore
sudo cp target/release/autocore-daemon /usr/local/bin/
sudo cp config/autocore.toml /etc/autocore/

# 3. systemd daemon'ı yenileyin ve servisi etkinleştirin
sudo systemctl daemon-reload
sudo systemctl enable autocore.service
sudo systemctl start autocore.service

# 4. Durum ve gerçek zamanlı önceliği doğrulayın
sudo systemctl status autocore.service
chrt -p $(pgrep autocore-daemon)
# Beklenen: pid XXX's current scheduling policy: SCHED_FIFO, current scheduling priority: 99
```

---

## 3. CAN ve Seri Port `udev` Kuralları

Donanım adaptörlerinin (`/dev/ttyUSB*` ve `can0`) yeniden başlatmalarda sabit isimler alması ve kullanıcı yetkisi için `/etc/udev/rules.d/99-autocore-hardware.rules`:

```udev
# USB-UART FTDI veya CP2102 Aktüatör Hattı
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6001", SYMLINK+="autocore/actuator0", MODE="0666"

# Sanal veya Fiziksel SocketCAN
SUBSYSTEM=="net", ACTION=="add", ATTRS{name}=="can*", RUN+="/sbin/ip link set %k type can bitrate 500000 && /sbin/ip link set %k up"
```

---

## 4. Teşhis ve Siyah Kutu (Blackbox) Analizi

Uçuş veya saha testi sonrası kaydedilen `.bbox` ikili log dosyasını CSV/JSON olarak dışa aktarmak için:

```bash
# Python ile dışa aktar ve görselleştir
python3 -c "
from autocore import BlackboxReader
header, frames = BlackboxReader.load_from_file('mission_flight.bbox')
print(f'Okunan kare sayısı: {len(frames)}, Toplam: {header.total_records}')
"
```
