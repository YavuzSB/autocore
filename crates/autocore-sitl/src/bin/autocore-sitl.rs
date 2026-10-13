//! autocore-sitl CLI Daemon
//!
//! Masaüstü SITL köprüsünü başlatır ve sanal PTY slave yolunu standart çıktıya yazar.
//! Takım görev düğümleri bu porta bağlanarak 50 Hz gerçek zamanlı simülasyon icra eder.

use autocore_sitl::sim_bridge::SitlSerialBridge;
use std::io::Write;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[AUTOCORE-SITL] Sanal Aktüatör Simülasyon Köprüsü başlatılıyor...");
    let bridge = SitlSerialBridge::start()?;
    let path = bridge.serial_port_path().display().to_string();

    println!("[AUTOCORE-SITL] READY: {}", path);
    std::io::stdout().flush()?;

    println!("[AUTOCORE-SITL] 50 Hz Telemetri yayını aktif. Çıkış için Ctrl+C tuşlayın.");
    signal::ctrl_c().await?;

    println!("\n[AUTOCORE-SITL] Kapatılıyor...");
    Ok(())
}
