//! autocore-sitl :: Fiziksel Atalet Modelli Mock Aktüatör Simülatörü

use async_trait::async_trait;
use autocore_base::{ActuatorFeedback, HalError, KinematicCommand};
use autocore_hal::ActuatorPort;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::info;

/// Gerçekçi motor ataleti, akım çekimi ve sıcaklık simülasyonuna sahip Mock Aktüatör
#[derive(Debug, Clone)]
pub struct MockActuator {
    target_cmd: Arc<Mutex<KinematicCommand>>,
    actual_linear: Arc<Mutex<[f32; 3]>>,
    actual_angular: Arc<Mutex<[f32; 3]>>,
    temperatures: Arc<Mutex<[f32; 4]>>,
    e_stop_active: Arc<AtomicBool>,
    time_constant_tau: f32, // Mekanik tepki süresi sabiti (sn)
    last_update_us: Arc<Mutex<u64>>,
}

impl Default for MockActuator {
    fn default() -> Self {
        Self::new(0.08) // 80 ms mekanik tepki süresi
    }
}

impl MockActuator {
    pub fn new(time_constant_tau: f32) -> Self {
        Self {
            target_cmd: Arc::new(Mutex::new(KinematicCommand::default())),
            actual_linear: Arc::new(Mutex::new([0.0; 3])),
            actual_angular: Arc::new(Mutex::new([0.0; 3])),
            temperatures: Arc::new(Mutex::new([25.0; 4])), // 25°C oda sıcaklığı
            e_stop_active: Arc::new(AtomicBool::new(false)),
            time_constant_tau: time_constant_tau.max(0.001),
            last_update_us: Arc::new(Mutex::new(0)),
        }
    }

    /// Fiziksel simülasyonu bir adım ilerletir (Adım süresi: dt)
    pub fn step(&self, now_us: u64) {
        let mut last_us = self.last_update_us.lock().unwrap();
        let dt = if *last_us == 0 || now_us <= *last_us {
            0.02 // Varsayılan 20 ms
        } else {
            ((now_us - *last_us) as f32 / 1_000_000.0).clamp(0.001, 0.5)
        };
        *last_us = now_us;

        let target = *self.target_cmd.lock().unwrap();
        let mut actual_lin = self.actual_linear.lock().unwrap();
        let mut actual_ang = self.actual_angular.lock().unwrap();
        let mut temps = self.temperatures.lock().unwrap();

        let is_estop = self.e_stop_active.load(Ordering::SeqCst);

        // Atalet integrasyonu: v(t+dt) = v(t) + (v_target - v(t)) * (dt / tau)
        let alpha = (dt / self.time_constant_tau).clamp(0.0, 1.0);

        for i in 0..3 {
            let target_lin = if is_estop { 0.0 } else { target.linear[i] };
            let target_ang = if is_estop { 0.0 } else { target.angular[i] };

            actual_lin[i] += (target_lin - actual_lin[i]) * alpha;
            actual_ang[i] += (target_ang - actual_ang[i]) * alpha;
        }

        // Sıcaklık ve motor yükü simülasyonu
        let speed_mag = (actual_lin[0].powi(2) + actual_lin[1].powi(2) + actual_lin[2].powi(2)).sqrt();
        for temp in temps.iter_mut() {
            if speed_mag > 0.1 {
                *temp = (*temp + speed_mag * 0.05 * dt).min(85.0); // Yük altında ısınma
            } else {
                *temp = (*temp - 0.02 * dt).max(25.0); // Boşta soğuma
            }
        }
    }

    /// E-Stop durumunu sıfırlar
    pub fn clear_estop(&self) {
        self.e_stop_active.store(false, Ordering::SeqCst);
        info!("Mock Aktüatör E-Stop sıfırlandı.");
    }
}

#[async_trait]
impl ActuatorPort for MockActuator {
    async fn send_command(&mut self, cmd: &KinematicCommand) -> Result<(), HalError> {
        if self.e_stop_active.load(Ordering::SeqCst) {
            return Err(HalError::EmergencyStopActive);
        }

        *self.target_cmd.lock().unwrap() = *cmd;
        self.step(cmd.timestamp_us);
        Ok(())
    }

    async fn read_feedback(&mut self) -> Result<ActuatorFeedback, HalError> {
        let now_us = *self.last_update_us.lock().unwrap();
        self.step(now_us + 20_000); // 20 ms ilerlet

        let actual_lin = *self.actual_linear.lock().unwrap();
        let actual_ang = *self.actual_angular.lock().unwrap();
        let temps = *self.temperatures.lock().unwrap();
        let is_estop = self.e_stop_active.load(Ordering::SeqCst);

        // Akım hesabı: Hız ile orantılı yaklaşık akım
        let speed = (actual_lin[0].abs() + actual_ang[2].abs()) * 2.0;
        let currents = [speed, speed, speed, speed];

        let mut status = 0u32;
        if is_estop {
            status |= 0x01; // Bit 0: E-Stop
        }

        Ok(ActuatorFeedback {
            actual_linear: actual_lin,
            actual_angular: actual_ang,
            motor_currents: currents,
            temperatures: temps,
            status_flags: status,
            _pad: 0,
            timestamp_us: now_us,
        })
    }

    async fn emergency_stop(&mut self) -> Result<(), HalError> {
        self.e_stop_active.store(true, Ordering::SeqCst);
        *self.target_cmd.lock().unwrap() = KinematicCommand::default();
        let now = *self.last_update_us.lock().unwrap();
        self.step(now);
        info!("MockAktüatör ACİL DURDURULDU!");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_actuator_inertia_and_feedback() {
        let mut actuator = MockActuator::new(0.05);

        let target_speed = 10.0;
        let cmd = KinematicCommand::new([target_speed, 0.0, 0.0], [0.0; 3], 1_000_000);
        actuator.send_command(&cmd).await.unwrap();

        // Birkaç adım ilerlet
        for i in 1..=10 {
            actuator.step(1_000_000 + i * 20_000);
        }

        let feedback = actuator.read_feedback().await.unwrap();
        // Hız 0'dan 10'a doğru yaklaşmış olmalı
        assert!(feedback.actual_linear[0] > 5.0);
        assert!(feedback.actual_linear[0] <= target_speed);

        // E-Stop tetikle
        actuator.emergency_stop().await.unwrap();
        assert!(actuator.send_command(&cmd).await.is_err());
    }
}
