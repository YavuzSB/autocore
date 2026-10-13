//! Savunma Sanayii Uçuş Verisi ve Siyah Kutu (FDR) Kaza Dayanıklılık Testleri
//!
//! Kapsam:
//! - Ani Elektrik/Güç Kesintisi ve Yarıda Kalan Dosya Kurtarma (Crash Survivability)
//! - Kırpılmış (Truncated) Dosyada Sağlam Çerçevelerin %100 Kurtarılması
//! - 2000+ Karelik Yüksek Hızlı Dairesel Doygunluk ve Kronolojik Monotonluk Doğrulaması

use autocore_base::{ActuatorFeedback, KinematicCommand, StateVector6DoF};
use autocore_sdk::blackbox::{BlackboxReader, BlackboxRecorder};
use std::fs::File;
use std::io::{Read, Write};

#[test]
fn test_blackbox_power_loss_truncation_recovery() {
    let mut recorder = BlackboxRecorder::new(50);

    // 25 adet uçuş telemetri karesi kaydet
    for i in 1..=25 {
        let mut state = StateVector6DoF::default();
        state.position[0] = i as f32 * 10.0;
        let cmd = KinematicCommand::new([2.0, 0.0, 0.0], [0.0, 0.0, 0.1], i * 20_000);
        let fb = ActuatorFeedback::default();
        recorder.record(i * 20_000, state, cmd, fb);
    }

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("flight_crash.bbox");
    recorder.save_to_file(&file_path).unwrap();

    // 2. Ani Güç Kesintisi (Power-Loss Simulation):
    // Dosyanın son 70 baytını kes (Son 25. kare yarım yazılmış olsun)
    let mut raw_bytes = Vec::new();
    {
        let mut file = File::open(&file_path).unwrap();
        file.read_to_end(&mut raw_bytes).unwrap();
    }

    let truncated_bytes = &raw_bytes[..raw_bytes.len() - 70];
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(truncated_bytes).unwrap();
    }

    // 3. Standart katı okuyucu boyut uyumsuzluğunu tespit etmeli
    let strict_result = BlackboxReader::load_from_file(&file_path);
    assert!(strict_result.is_err(), "Eksik bayt içeren dosya katı modda reddedilmeli");

    // 4. Kaza kurtarma modu (load_with_recovery) önceki 24 sağlam kareyi eksiksiz kurtarmalı
    let (header, recovered_frames, discarded) = BlackboxReader::load_with_recovery(&file_path).unwrap();

    assert_eq!(recovered_frames.len(), 24, "Önceki 24 kare eksiksiz kurtarılmalı!");
    assert_eq!(discarded, 184 - 70, "Yarım kalan 114 bayt güvenle tasfiye edilmeli");
    assert_eq!(header.capacity, 50);

    // Kurtarılan her bir karenin CRC-16 doğruluğunu teyit et
    for (idx, frame) in recovered_frames.iter().enumerate() {
        assert!(frame.verify_crc(), "Kurtarılan {} numaralı karenin CRC sağlaması geçerli olmalı", idx);
        assert_eq!(frame.state.position[0], (idx + 1) as f32 * 10.0);
    }
}

#[test]
fn test_blackbox_sustained_high_throughput_saturation() {
    let mut recorder = BlackboxRecorder::new(100); // 100 karelik halka tampon

    // 2000 karelik yoğun görev kaydı
    for i in 1..=2000 {
        let mut state = StateVector6DoF::default();
        state.velocity[0] = (i as f32 * 0.1).sin();
        let cmd = KinematicCommand::new([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], i * 10_000);
        let fb = ActuatorFeedback::default();
        recorder.record(i * 10_000, state, cmd, fb);
    }

    let frames = recorder.ordered_frames();
    assert_eq!(frames.len(), 100, "Tampon boyutu 100 ile sınırlı kalmalı");

    // Kronolojik monotonluk denetimi (En eskiden en yeniye doğru sıralı olmalı)
    for i in 0..frames.len() - 1 {
        assert!(
            frames[i].timestamp_us < frames[i + 1].timestamp_us,
            "Zaman damgası monoton artan sırada olmalı: {} vs {}",
            frames[i].timestamp_us,
            frames[i + 1].timestamp_us
        );
        assert!(frames[i].verify_crc());
    }

    // Son karenin 2000. kare olduğunu doğrula (2000 * 10_000 = 20_000_000 us)
    assert_eq!(frames.last().unwrap().timestamp_us, 20_000_000);
}
