//! autocore-pointing :: Eylem Disiplini ve Güvenlik Mandalı (ActionDiscipline)
//!
//! Açısal tolerans penceresi, servo salınım sönümlenmesi (settling time),
//! atış soğuma süresi ve taban emniyeti sağlanmadan fire_gate kilidini açmaz.

use autocore_base::SafetyMode;

/// Tetik ve Angajman Güvenlik Mandalı Denetleyicisi
#[derive(Debug, Clone)]
pub struct ActionDiscipline {
    /// Maksimum izin verilen açısal yönelim hatası (Radyan) - örn. 0.5 derece (~0.0087 rad)
    pub angular_tolerance_rad: f32,
    /// Salınım sönümlenmiş kabul edilen maksimum açısal hız (rad/s)
    pub max_settling_rate_rad_s: f32,
    /// Salınımın sönük kalması gereken minimum süre (mikrosaniye) - örn. 50,000 us (50 ms)
    pub settling_time_us: u64,
    /// İki atış arası zorunlu soğuma süresi (mikrosaniye) - örn. 100,000 us (100 ms)
    pub cooldown_us: u64,

    /// Salınımın tolerans içine girdiği ilk an
    settling_start_us: Option<u64>,
    /// Son ateş edilme zamanı
    last_fire_us: u64,
}

impl Default for ActionDiscipline {
    fn default() -> Self {
        Self {
            angular_tolerance_rad: 0.5_f32.to_radians(), // 0.5 derece
            max_settling_rate_rad_s: 0.05,               // 0.05 rad/s
            settling_time_us: 50_000,                    // 50 ms
            cooldown_us: 100_000,                        // 100 ms
            settling_start_us: None,
            last_fire_us: 0,
        }
    }
}

impl ActionDiscipline {
    pub fn new(
        angular_tolerance_deg: f32,
        max_settling_rate_rad_s: f32,
        settling_time_ms: u64,
        cooldown_ms: u64,
    ) -> Self {
        Self {
            angular_tolerance_rad: angular_tolerance_deg.to_radians(),
            max_settling_rate_rad_s,
            settling_time_us: settling_time_ms * 1_000,
            cooldown_us: cooldown_ms * 1_000,
            settling_start_us: None,
            last_fire_us: 0,
        }
    }

    /// Ateş iznini (fire_gate) 4 katı kurala göre değerlendirir
    pub fn evaluate(
        &mut self,
        current_pan: f32,
        current_tilt: f32,
        target_pan: f32,
        target_tilt: f32,
        current_pan_rate: f32,
        current_tilt_rate: f32,
        safety_mode: SafetyMode,
        now_us: u64,
    ) -> bool {
        // Kural 4: Taban Emniyeti (Yalnızca Armed veya Mission)
        let is_armed = safety_mode == SafetyMode::Armed || safety_mode == SafetyMode::Mission;
        if !is_armed {
            self.settling_start_us = None;
            return false;
        }

        // Kural 3: Soğuma Süresi (Cooldown)
        if self.last_fire_us > 0 && now_us.saturating_sub(self.last_fire_us) < self.cooldown_us {
            return false;
        }

        // Kural 1: Açısal Hata Eşiği
        let pan_err = (current_pan - target_pan).abs();
        let tilt_err = (current_tilt - target_tilt).abs();
        let within_tolerance =
            pan_err <= self.angular_tolerance_rad && tilt_err <= self.angular_tolerance_rad;

        // Kural 2: Mekanik Salınım Sönümü (Hız eşiği)
        let is_rate_calm = current_pan_rate.abs() <= self.max_settling_rate_rad_s
            && current_tilt_rate.abs() <= self.max_settling_rate_rad_s;

        if within_tolerance && is_rate_calm {
            match self.settling_start_us {
                None => {
                    // Toleransa yeni girdi, süreyi başlat
                    self.settling_start_us = Some(now_us);
                    false
                }
                Some(start_time) => {
                    let elapsed = now_us.saturating_sub(start_time);
                    if elapsed >= self.settling_time_us {
                        // Tüm koşullar sağlandı! Ateş serbest
                        true
                    } else {
                        // Henüz salınım sönüm süresi dolmadı
                        false
                    }
                }
            }
        } else {
            // Tolerans dışına taştı veya hızlandı, sayacı sıfırla
            self.settling_start_us = None;
            false
        }
    }

    /// Ateş edildiğinde kaydedilir ve soğuma sayacı başlar
    pub fn record_fire(&mut self, now_us: u64) {
        self.last_fire_us = now_us;
        self.settling_start_us = None;
    }

    /// Durumu tamamen sıfırlar
    pub fn reset(&mut self) {
        self.settling_start_us = None;
        self.last_fire_us = 0;
    }
}
