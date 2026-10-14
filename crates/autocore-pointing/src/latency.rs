//! autocore-pointing :: Boru Hattı Gecikme Telafisi (LatencyCompensator)
//!
//! Kamera algılama anı ile motor pozisyonlanma anı arasındaki gecikmeyi
//! hedefin hız vektörüyle kompanse eder (Dead Reckoning).

use crate::types::TargetTrack;

pub struct LatencyCompensator;

impl LatencyCompensator {
    /// Gecikmeyi kompanse ederek eylem anındaki gerçek hedef konumunu hesaplar:
    /// P_real = P_detected + V_target * delta_t
    pub fn extrapolate(track: &TargetTrack, action_time_us: u64) -> [f32; 3] {
        if action_time_us <= track.timestamp_us {
            return track.position;
        }

        let dt_s = (action_time_us - track.timestamp_us) as f32 * 1e-6;

        [
            track.position[0] + track.velocity[0] * dt_s,
            track.position[1] + track.velocity[1] * dt_s,
            track.position[2] + track.velocity[2] * dt_s,
        ]
    }
}
