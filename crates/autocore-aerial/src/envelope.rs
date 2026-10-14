//! autocore-aerial :: 3B Uçuş Zarfı ve Aerodinamik Güvenlik Bekçisi (FlightEnvelopeGuard)
//!
//! Stall (perdövites) koruması, maksimum yatış açısı (bank angle <= 45 deg),
//! ve tırmanma/çöküş limitleri ile uçağın aerodinamik sınırların dışına çıkmasını engeller.

/// Uçuş Güvenlik Zarfı Parametreleri
#[derive(Debug, Clone, Copy)]
pub struct FlightEnvelopeGuard {
    /// Perdövites hava hızı eşiği (m/s) - tipik 12.0 m/s
    pub stall_speed_m_s: f32,
    /// Maksimum izin verilen yatış açısı (radyan) - tipik 45 derece (~0.7854 rad)
    pub max_bank_angle_rad: f32,
    /// Maksimum tırmanma hızı (m/s) - pozitif değer
    pub max_climb_rate_m_s: f32,
    /// Maksimum çöküş / dalış hızı (m/s) - pozitif değer
    pub max_descent_rate_m_s: f32,
}

impl Default for FlightEnvelopeGuard {
    fn default() -> Self {
        Self {
            stall_speed_m_s: 12.0,
            max_bank_angle_rad: 45.0_f32.to_radians(), // 45 derece
            max_climb_rate_m_s: 10.0,                  // 10 m/s tırmanma
            max_descent_rate_m_s: 5.0,                 // 5 m/s çöküş
        }
    }
}

/// Koruma Altındaki Uçuş Komutları Çıktısı
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardedFlightCommand {
    pub roll_cmd_rad: f32,
    pub pitch_cmd_rad: f32,
    pub vz_cmd_m_s: f32, // NED çerçevesinde (negatif = tırmanma)
    pub is_stall_warning: bool,
    pub is_bank_clamped: bool,
}

impl FlightEnvelopeGuard {
    pub fn new(
        stall_speed_m_s: f32,
        max_bank_angle_deg: f32,
        max_climb_rate_m_s: f32,
        max_descent_rate_m_s: f32,
    ) -> Self {
        Self {
            stall_speed_m_s,
            max_bank_angle_rad: max_bank_angle_deg.to_radians(),
            max_climb_rate_m_s: max_climb_rate_m_s.abs(),
            max_descent_rate_m_s: max_descent_rate_m_s.abs(),
        }
    }

    /// Verilen ham uçuş komutlarını ve hava hızını denetleyerek güvenli zarf içine sokar
    pub fn guard(
        &self,
        raw_roll_rad: f32,
        raw_pitch_rad: f32,
        raw_vz_ned: f32, // NED: negatif = tırmanma, pozitif = çöküş
        airspeed_m_s: f32,
    ) -> GuardedFlightCommand {
        // 1. Bank Açısı Kısıtlaması (Roll Clamp: [-45, +45] deg)
        let clamped_roll = raw_roll_rad.clamp(-self.max_bank_angle_rad, self.max_bank_angle_rad);
        let is_bank_clamped = (raw_roll_rad - clamped_roll).abs() > 1e-4;

        // 2. Stall Koruması (Airspeed < Stall Speed)
        let is_stall = airspeed_m_s < self.stall_speed_m_s;
        let pitch_cmd = if is_stall {
            // Perdövites riski: burnu zorunlu olarak en az 5 derece aşağı ez (-0.087 rad)
            raw_pitch_rad.min(-5.0_f32.to_radians())
        } else {
            raw_pitch_rad
        };

        // 3. Tırmanma ve Çöküş Limitleri (NED: -climb <= vz <= descent)
        let clamped_vz = raw_vz_ned.clamp(-self.max_climb_rate_m_s, self.max_descent_rate_m_s);

        GuardedFlightCommand {
            roll_cmd_rad: clamped_roll,
            pitch_cmd_rad: pitch_cmd,
            vz_cmd_m_s: clamped_vz,
            is_stall_warning: is_stall,
            is_bank_clamped,
        }
    }
}
