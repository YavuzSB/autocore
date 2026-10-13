#!/usr/bin/env python3
"""
Autocore Python Başlangıç Görev Şablonu (Starter Python Mission Node)
AutoCore Otonom Sistemler (HSS, İKA, İDA, İHA) Takım Görev Şablonu

Kullanım:
  python3 mission.py --sitl
  python3 mission.py --port /dev/pts/3
"""

import sys
import os
import time
import math
import argparse
import subprocess
import numpy as np

# autocore paketini arama yoluna ekle
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../../python")))

try:
    import autocore
except ImportError:
    print("HATA: 'autocore' Python paketi bulunamadı!")
    sys.exit(1)

def run_sitl_subprocess():
    """Arka planda autocore-sitl simülasyon daemon'ını başlatır ve PTY yolunu döner"""
    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
    bin_path = os.path.join(repo_root, "target/debug/autocore-sitl")

    if not os.path.exists(bin_path):
        print("[MISSION] autocore-sitl derleniyor...")
        subprocess.run(["cargo", "build", "--bin", "autocore-sitl"], cwd=repo_root, check=True)

    print(f"[MISSION] SITL Daemon başlatılıyor: {bin_path}")
    proc = subprocess.Popen(
        [bin_path],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )

    pty_path = None
    for _ in range(30):
        line = proc.stdout.readline()
        if "READY:" in line:
            pty_path = line.split("READY:")[1].strip()
            break
        time.sleep(0.1)

    if not pty_path:
        proc.kill()
        raise RuntimeError("SITL PTY hazır mesajı alınamadı!")

    print(f"[MISSION] SITL Sanal Seri Port Hazır: {pty_path}")
    return proc, pty_path

def main():
    parser = argparse.ArgumentParser(description="Autocore Otonom Görev Düğümü")
    parser.add_argument("--sitl", action="store_true", help="Masaüstü SITL simülasyonunu başlat")
    parser.add_argument("--port", type=str, help="Bağlanılacak seri port (Örn: /dev/pts/3 veya /dev/ttyUSB0)")
    args = parser.parse_args()

    sitl_proc = None
    port_path = args.port

    if args.sitl:
        sitl_proc, port_path = run_sitl_subprocess()
    elif not port_path:
        print("HATA: Lütfen --sitl veya --port belirtin.")
        sys.exit(1)

    print("=" * 60)
    print("🚀 AUTOCORE OTONOM GÖREV KONTROLCÜSÜ BAŞLATILDI")
    print("=" * 60)

    try:
        client = autocore.AutocoreSitlClient(port_path)
        print("[MISSION] Port bağlantısı kuruldu.")

        # 1. Emniyet Durumu: Arm Et
        client.safety_mode = autocore.SafetyMode.ARMED
        print(f"[MISSION] Emniyet Modu: {client.safety_mode.name}")

        # 2. Görev Yol Noktaları (Metre cinsinden 2D Hedefler)
        waypoints = [
            (2.0, 0.0),
            (5.0, 2.0),
            (8.0, 0.0),
            (10.0, -2.0),
            (12.0, 0.0),
        ]

        pos_x = 0.0
        pos_y = 0.0
        yaw = 0.0
        dt = 0.02 # 50 Hz döngü
        current_wp_idx = 0

        print(f"[MISSION] {len(waypoints)} Yol Noktası yüklendi. Otonom seyir başlıyor...")

        start_time = time.time()
        step_count = 0

        while current_wp_idx < len(waypoints):
            target_x, target_y = waypoints[current_wp_idx]
            dx = target_x - pos_x
            dy = target_y - pos_y
            dist = math.hypot(dx, dy)

            # Hedefe ulaşıldı mı? (Eşik: 0.3 metre)
            if dist < 0.3:
                print(f"[MISSION] ✅ Yol Noktası {current_wp_idx + 1}/{len(waypoints)} ulaşıldı! (X={pos_x:.2f}, Y={pos_y:.2f})")
                current_wp_idx += 1
                if current_wp_idx >= len(waypoints):
                    break
                continue

            target_yaw = math.atan2(dy, dx)
            yaw_error = target_yaw - yaw

            # Açı normalizasyonu [-pi, pi]
            while yaw_error > math.pi:
                yaw_error -= 2 * math.pi
            while yaw_error < -math.pi:
                yaw_error += 2 * math.pi

            # Kontrol komutları (Doğrusal hız + Açısal yönelim)
            target_speed = min(3.0, max(0.5, dist * 0.8))
            yaw_rate = float(np.clip(yaw_error * 2.0, -1.0, 1.0))

            # Komutu aktüatöre gönder
            client.send_cmd_vel(vx=target_speed, vy=0.0, vz=0.0, roll=0.0, pitch=0.0, yaw=yaw_rate)

            # Kinematik durum güncelleme
            yaw += yaw_rate * dt
            pos_x += target_speed * math.cos(yaw) * dt
            pos_y += target_speed * math.sin(yaw) * dt

            # Aktüatör geri beslemesini oku (50 Hz telemetri)
            fb = client.poll_feedback(timeout_sec=0.005)
            step_count += 1

            if step_count % 25 == 0:
                curr_amps = fb.motor_currents[0] if fb else 0.0
                print(
                    f"  [TELEMETRİ] Hedef {current_wp_idx+1} | Kalan: {dist:.2f}m | "
                    f"Konum: ({pos_x:.2f}, {pos_y:.2f}) | Hız: {target_speed:.2f} m/s | "
                    f"Akım: {curr_amps:.1f}A"
                )

            time.sleep(dt)

        total_time = time.time() - start_time
        print("=" * 60)
        print(f"[MISSION] 🎉 TÜM HEDEFLER BAŞARIYLA TAMAMLANDI! Süre: {total_time:.2f}s | Adım: {step_count}")
        print("=" * 60)

        # Fren yap ve motorları kilitle
        client.send_cmd_vel(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        client.emergency_stop()
        print("[MISSION] Araç durduruldu ve SafeLock moduna alındı.")

    finally:
        if sitl_proc:
            print("[MISSION] SITL Daemon sonlandırılıyor...")
            sitl_proc.terminate()
            sitl_proc.wait(timeout=2)
            print("[MISSION] SITL Daemon kapandı.")

if __name__ == "__main__":
    main()
