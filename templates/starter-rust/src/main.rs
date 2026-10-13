//! Autocore Rust Başlangıç Görev Düğümü (Starter Rust Mission Node)
//!
//! AutoCore Otonom Sistemler (HSS, İKA, İDA, İHA) Rust Görev Şablonu.
//! `autocore-sdk` üzerinden SITL simülasyonuna bağlanır ve 50 Hz otonom kontrol döngüsü icra eder.

use autocore_sdk::{AutocoreClient, SafetyMode};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("🚀 AUTOCORE RUST OTONOM GÖREV DÜĞÜMÜ");
    println!("============================================================");

    // 1. Masaüstü SITL ortamına bağlan
    let mut client = AutocoreClient::connect_sitl().await?;
    println!("[RUST-MISSION] SITL bağlantısı kuruldu. Port aktif.");

    // 2. Siyah kutu kaydedicisini aç
    client.enable_blackbox(500);

    // 3. Emniyet durumunu Arm ve Mission yap
    client.set_safety_mode(SafetyMode::Mission);
    println!("[RUST-MISSION] Emniyet Modu: {:?}", client.safety_mode());

    // 4. Otonom devriye döngüsü (50 Hz, 100 adım)
    println!("[RUST-MISSION] Otonom devriye görevi başlatılıyor...");
    for step in 1..=100 {
        let vx = 2.0;
        let yaw_rate = 0.1 * (step as f32 * 0.1).sin();
        let timestamp_us = step as u64 * 20_000;

        client
            .send_cmd_vel([vx, 0.0, 0.0], [0.0, 0.0, yaw_rate], timestamp_us)
            .await?;

        // Geri beslemeyi oku
        let fb = client.read_feedback().await?;
        if step % 25 == 0 {
            println!(
                "  [RUST-TELEMETRİ] Adım: {:3} | Motor Akım 0: {:.2} A | Sıcaklık: {:.1} °C",
                step, fb.motor_currents[0], fb.temperatures[0]
            );
        }

        sleep(Duration::from_millis(20)).await;
    }

    // 5. Görevi tamamla ve SafeLock'a geç
    client.emergency_stop().await?;
    println!("[RUST-MISSION] Araç SafeLock moduna alındı.");

    // 6. Siyah kutu loglarını dışa aktar
    if let Some(bbox) = client.blackbox() {
        let frames = bbox.ordered_frames();
        println!(
            "[RUST-MISSION] ✅ Siyah Kutu Kaydı: {} adet telemetri karesi başarıyla mühürlendi.",
            frames.len()
        );
    }

    println!("============================================================");
    println!("[RUST-MISSION] 🎉 GÖREV BAŞARIYLA TAMAMLANDI!");
    println!("============================================================");

    Ok(())
}
