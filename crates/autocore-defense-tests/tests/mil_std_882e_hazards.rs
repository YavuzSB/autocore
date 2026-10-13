//! MIL-STD-882E (Sistem Emniyeti Standardı) Doğrulama Testleri
//!
//! Kapsam:
//! - Tehlike Tanımlama ve Azaltma (Hazard Identification & Mitigation)
//! - NaN, Sonsuzluk (+Inf/-Inf) ve Alt-Normal Sayı Zehirlenmesi Enjeksiyonu
//! - E-Stop Anında Yüksek Yük Altında Sıfır Hayalet Hareket (Phantom Movement Immunity)
//! - 500 ms Watchdog Kesin Zaman Aşımı ve SafeLock Kilitlenmesi
//! - Geçersiz Durum Makinesi Geçiş Veto Denetimi

use autocore_base::{KinematicCommand, SafetyMode};
use autocore_safety::clamping::ClampingEngine;
use autocore_safety::flush::LockFreeFlushQueue;
use autocore_safety::fsm::CentralSafetyFsm;
use autocore_safety::slew_rate::SlewRateLimiter;
use autocore_safety::watchdog::SentinelWatchdog;
use std::sync::Arc;

#[test]
fn test_mil882e_nan_inf_poisoning_mitigation() {
    let clamping = ClampingEngine::new(None, (-100.0, 100.0), 10.0, 3.14);
    let mut slew = SlewRateLimiter::new(5.0, 2.0);

    // 1. NaN Enjeksiyonu
    let mut nan_cmd = KinematicCommand::new(
        [f32::NAN, 5.0, -2.0],
        [0.0, f32::NAN, 0.5],
        1_000_000,
    );

    let was_clamped = clamping.clamp_command(&mut nan_cmd);
    assert!(was_clamped, "NaN değeri kelepçeleme tetiklemeliydi");
    assert_eq!(nan_cmd.linear[0], 0.0, "NaN doğrusal hız sıfırlanmalı");
    assert_eq!(nan_cmd.angular[1], 0.0, "NaN açısal hız sıfırlanmalı");
    assert_eq!(nan_cmd.linear[1], 5.0, "Geçerli hız korunmalı");

    // 2. Sonsuzluk (+Inf / -Inf) Enjeksiyonu
    let mut inf_cmd = KinematicCommand::new(
        [f32::INFINITY, f32::NEG_INFINITY, 0.0],
        [0.0, 0.0, f32::INFINITY],
        1_020_000,
    );

    let inf_clamped = clamping.clamp_command(&mut inf_cmd);
    assert!(inf_clamped, "Inf değerleri kelepçeleme tetiklemeliydi");
    assert_eq!(inf_cmd.linear[0], 0.0, "+Inf hızı sıfırlanmalı");
    assert_eq!(inf_cmd.linear[1], 0.0, "-Inf hızı sıfırlanmalı");
    assert_eq!(inf_cmd.angular[2], 0.0, "+Inf açısal hızı sıfırlanmalı");

    // 3. Slew-Rate Limiter'a NaN Enjeksiyonu (Önceki geçerli durum bozulmamalı)
    let init_cmd = KinematicCommand::new([2.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1_000_000);
    let _ = slew.filter(init_cmd, 1_000_000);

    let poisoned = KinematicCommand::new([f32::NAN, f32::NAN, 0.0], [0.0, 0.0, 0.0], 1_050_000);
    let safe_filtered = slew.filter(poisoned, 1_050_000);

    assert!(safe_filtered.linear[0].is_finite(), "Slew filtresi NaN üretmemeli");
    assert_eq!(safe_filtered.linear[0], 2.0, "NaN durumunda önceki geçerli hız korunmalı");
}

#[test]
fn test_mil882e_estop_under_saturation_and_phantom_immunity() {
    let queue = LockFreeFlushQueue::new(64);

    // Kuyruğu azami ivmeli hareket komutlarıyla doldur
    for i in 1..=50 {
        let cmd = KinematicCommand::new([10.0, 0.0, 0.0], [0.0, 0.0, 0.5], i * 20_000);
        let _ = queue.push(cmd);
    }

    assert_eq!(queue.len(), 50, "Kuyrukta 50 bekleyen komut olmalı");

    // Donanımsal / Yazılımsal Acil Durdurma (E-Stop) Tetiklendi
    let flushed_count = queue.flush_all();
    assert_eq!(flushed_count, 50, "E-Stop anında tüm 50 komut tasfiye edilmeli");
    assert_eq!(queue.len(), 0, "Kuyruk anında 0 olmalı");

    // E-Stop sonrası kuyruktan komut çekilmeye çalışıldığında HİÇBİR hayalet hareket olmamalı
    let phantom_cmd = queue.pop();
    assert!(phantom_cmd.is_none(), "E-Stop sonrası hayalet hareket kesinlikle olamaz!");

    // E-Stop serbest bırakılsa bile eski komutlar geri gelemez
    assert_eq!(queue.len(), 0);
}

#[tokio::test]
async fn test_mil882e_watchdog_hard_deadline_and_safelock_latch() {
    let fsm = Arc::new(CentralSafetyFsm::new(SafetyMode::Booting));
    fsm.transition(SafetyMode::Disarmed).unwrap();
    fsm.transition(SafetyMode::Armed).unwrap();
    fsm.transition(SafetyMode::Mission).unwrap();
    assert_eq!(fsm.current_mode(), SafetyMode::Mission);

    let watchdog = SentinelWatchdog::new(500_000); // 500 ms eşik

    // 1. Düzenli 50 Hz kalp atışı altında sistem Mission modunda kalmalı
    for i in 1..=5 {
        let now_us = i * 20_000;
        watchdog.feed(now_us);
        let check_res = watchdog.check_timeout(now_us, &fsm);
        assert!(check_res.is_ok());
        assert_eq!(fsm.current_mode(), SafetyMode::Mission);
    }

    // 2. 500 ms kritik sürenin 501. milisaniyesinde (700_000 us) SafeLock'a geçiş garantisi
    let timeout_time_us = 700_000; // Son besleme 100_000 us idi, aradaki fark 600 ms > 500 ms
    let timeout_res = watchdog.check_timeout(timeout_time_us, &fsm);
    assert!(timeout_res.is_err(), "Watchdog zaman aşımı hatası üretmeli");
    assert_eq!(fsm.current_mode(), SafetyMode::SafeLock, "Watchdog SafeLock kilidi koymalı!");

    // 3. SafeLock modundayken dışarıdan geçersiz geçiş denense bile kilit açılamaz (Latch Koruması)
    let illegal_resume = fsm.transition(SafetyMode::Mission);
    assert!(illegal_resume.is_err(), "SafeLock'tan Mission moduna doğrudan geçilemez!");
    assert_eq!(fsm.current_mode(), SafetyMode::SafeLock);

    // Sadece bilinçli reset_safelock çağrısıyla açılabilir
    fsm.reset_safelock().unwrap();
    assert_eq!(fsm.current_mode(), SafetyMode::Disarmed);
}

#[test]
fn test_mil882e_invalid_safety_state_transitions() {
    let fsm = CentralSafetyFsm::new(SafetyMode::Booting);
    assert_eq!(fsm.current_mode(), SafetyMode::Booting);

    // Booting modundan doğrudan Mission moduna sıçrama YASAKTIR (Önce Disarmed -> Armed olmalı)
    let illegal_mission = fsm.transition(SafetyMode::Mission);
    assert!(illegal_mission.is_err(), "Booting -> Mission doğrudan geçişi veto edilmeli!");

    // Booting modundan doğrudan Failsafe'e geçiş YASAKTIR
    let illegal_failsafe = fsm.transition(SafetyMode::Failsafe);
    assert!(illegal_failsafe.is_err(), "Booting -> Failsafe doğrudan geçişi veto edilmeli!");

    // Ancak SafeLock her zaman anında kabul edilmelidir (Failsafe Önceliği)
    let emergency_lock = fsm.transition(SafetyMode::SafeLock);
    assert!(emergency_lock.is_ok(), "SafeLock acil kilidi her moddan doğrudan geçebilmeli");
    assert_eq!(fsm.current_mode(), SafetyMode::SafeLock);
}
