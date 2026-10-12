//! autocore-safety :: Kinematik Sönümleme ve İvme Limitörü (Slew-Rate Limiter)

use autocore_base::KinematicCommand;

/// Birim zamandaki azami hız ve ivme değişimini sınırlayan süzgeç
#[derive(Debug, Clone)]
pub struct SlewRateLimiter {
    /// Azami doğrusal ivme (m/s²)
    pub max_linear_accel: f32,
    /// Azami açısal ivme (rad/s²)
    pub max_angular_accel: f32,
    /// Bir önceki filtrelenmiş komut
    prev_cmd: Option<KinematicCommand>,
    /// Bir önceki zaman damgası (us)
    prev_time_us: u64,
}

impl Default for SlewRateLimiter {
    fn default() -> Self {
        Self {
            max_linear_accel: 5.0,  // 5 m/s²
            max_angular_accel: 2.0, // 2 rad/s²
            prev_cmd: None,
            prev_time_us: 0,
        }
    }
}

impl SlewRateLimiter {
    pub fn new(max_linear_accel: f32, max_angular_accel: f32) -> Self {
        Self {
            max_linear_accel,
            max_angular_accel,
            prev_cmd: None,
            prev_time_us: 0,
        }
    }

    /// Filtreyi sıfırlar
    pub fn reset(&mut self) {
        self.prev_cmd = None;
        self.prev_time_us = 0;
    }

    /// Yeni gelen hedef komutu azami ivme sınırları dahilinde yumuşatır
    pub fn filter(&mut self, mut target: KinematicCommand, now_us: u64) -> KinematicCommand {
        let (prev, dt_s) = match (self.prev_cmd, self.prev_time_us) {
            (Some(prev), prev_time) if now_us > prev_time => {
                let dt = (now_us - prev_time) as f32 / 1_000_000.0;
                // dt'yi 1ms ile 1s arasında sınırla (ani saat sıçramalarına karşı)
                (prev, dt.clamp(0.001, 1.0))
            }
            _ => {
                // İlk komut veya zaman geri gitmişse doğrudan kabul et
                self.prev_cmd = Some(target);
                self.prev_time_us = now_us;
                return target;
            }
        };

        // Doğrusal bileşenleri sınırla: dv_max = a_max * dt
        let max_dv = self.max_linear_accel * dt_s;
        for i in 0..3 {
            let diff = target.linear[i] - prev.linear[i];
            let clamped_diff = diff.clamp(-max_dv, max_dv);
            target.linear[i] = prev.linear[i] + clamped_diff;
        }

        // Açısal bileşenleri sınırla: dw_max = alpha_max * dt
        let max_dw = self.max_angular_accel * dt_s;
        for i in 0..3 {
            let diff = target.angular[i] - prev.angular[i];
            let clamped_diff = diff.clamp(-max_dw, max_dw);
            target.angular[i] = prev.angular[i] + clamped_diff;
        }

        target.timestamp_us = now_us;
        self.prev_cmd = Some(target);
        self.prev_time_us = now_us;

        target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slew_rate_limiting() {
        let mut limiter = SlewRateLimiter::new(2.0, 1.0); // max 2 m/s² ve 1 rad/s²

        // 1. Başlangıç komutu: 0 m/s
        let cmd0 = KinematicCommand::new([0.0; 3], [0.0; 3], 1_000_000);
        let out0 = limiter.filter(cmd0, 1_000_000);
        assert_eq!(out0.linear[0], 0.0);

        // 2. 500 ms sonra ani 10 m/s hız isteği (dt = 0.5s, max_dv = 2.0 * 0.5 = 1.0 m/s)
        let cmd1 = KinematicCommand::new([10.0, 0.0, 0.0], [5.0, 0.0, 0.0], 1_500_000);
        let out1 = limiter.filter(cmd1, 1_500_000);

        // Hız 0.0 + 1.0 = 1.0 m/s olmalı (10.0'a sıçrayamaz)
        assert!((out1.linear[0] - 1.0).abs() < 1e-4);
        // Açısal hız 0.0 + (1.0 * 0.5) = 0.5 rad/s olmalı
        assert!((out1.angular[0] - 0.5).abs() < 1e-4);
    }
}
