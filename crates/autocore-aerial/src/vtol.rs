//! autocore-aerial :: VTOL Geçiş Kontrolörü (VtolTransitionController)
//!
//! Döner kanat (Hover) ile sabit kanat (Cruise) arasındaki aerodinamik
//! itki transferini ve durum makinesini (FSM) yönetir.

use serde::{Deserialize, Serialize};

/// VTOL Uçuş Durumları
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VtolState {
    /// Tamamen döner kanat / çok rotorlu dikey uçuş
    Hover,
    /// İleri geçiş: Dikey motorlardan öne itici motora kademeli devir
    TransitionForward,
    /// Tamamen sabit kanat aerodinamik seyir uçuşu (dikey motorlar kapalı)
    CruiseFixedWing,
    /// Geri geçiş: Sabit kanattan hover moduna yavaşlama
    TransitionBack,
}

/// VTOL Aktüatör ve İtki Karışım Çıktısı
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VtolActuatorMixing {
    pub state: VtolState,
    /// Çok rotorlu dikey motor gücü [0.0, 1.0]
    pub mc_throttle_pct: f32,
    /// Sabit kanat öne itici motor gücü [0.0, 1.0]
    pub fw_throttle_pct: f32,
    /// Geçiş ilerleme oranı [0.0 = Tam Hover, 1.0 = Tam Sabit Kanat]
    pub transition_progress: f32,
}

pub struct VtolTransitionController {
    state: VtolState,
    /// Geçişin tamamlanması için gereken minimum hava hızı (m/s) - tipik 16.0 m/s
    pub transition_airspeed_m_s: f32,
    /// Geçiş süresi (saniye) - tipik 5.0 s
    pub transition_duration_s: f32,
    /// Mevcut fazda geçen süre (saniye)
    phase_time_s: f32,
}

impl Default for VtolTransitionController {
    fn default() -> Self {
        Self {
            state: VtolState::Hover,
            transition_airspeed_m_s: 16.0,
            transition_duration_s: 5.0,
            phase_time_s: 0.0,
        }
    }
}

impl VtolTransitionController {
    pub fn new(transition_airspeed_m_s: f32, transition_duration_s: f32) -> Self {
        Self {
            state: VtolState::Hover,
            transition_airspeed_m_s,
            transition_duration_s: transition_duration_s.max(0.5),
            phase_time_s: 0.0,
        }
    }

    pub fn current_state(&self) -> VtolState {
        self.state
    }

    /// İleri geçiş (Hover -> Cruise) komutu verir
    pub fn request_transition_forward(&mut self) {
        if self.state == VtolState::Hover {
            self.state = VtolState::TransitionForward;
            self.phase_time_s = 0.0;
        }
    }

    /// Geri geçiş (Cruise -> Hover) komutu verir
    pub fn request_transition_back(&mut self) {
        if self.state == VtolState::CruiseFixedWing {
            self.state = VtolState::TransitionBack;
            self.phase_time_s = 0.0;
        }
    }

    /// Her döngüde çağrılan durum makinesi ve itki mikseri
    pub fn update(&mut self, airspeed_m_s: f32, dt_s: f32) -> VtolActuatorMixing {
        self.phase_time_s += dt_s;

        match self.state {
            VtolState::Hover => VtolActuatorMixing {
                state: VtolState::Hover,
                mc_throttle_pct: 1.0,
                fw_throttle_pct: 0.0,
                transition_progress: 0.0,
            },

            VtolState::TransitionForward => {
                let time_prog = (self.phase_time_s / self.transition_duration_s).clamp(0.0, 1.0);
                let speed_prog =
                    (airspeed_m_s / self.transition_airspeed_m_s).clamp(0.0, 1.0);
                let progress = time_prog.min(speed_prog);

                // Dikey motorlar 1.0'dan 0.0'a iner, itici motor 0.0'dan 1.0'a çıkar
                let mc_throttle = 1.0 - progress;
                let fw_throttle = 0.5 + 0.5 * progress;

                // Geçiş tamamlandı mı? (Hem süre hem hız sağlandıysa)
                if time_prog >= 1.0 && airspeed_m_s >= self.transition_airspeed_m_s {
                    self.state = VtolState::CruiseFixedWing;
                    self.phase_time_s = 0.0;
                    return VtolActuatorMixing {
                        state: VtolState::CruiseFixedWing,
                        mc_throttle_pct: 0.0,
                        fw_throttle_pct: 1.0,
                        transition_progress: 1.0,
                    };
                }

                VtolActuatorMixing {
                    state: VtolState::TransitionForward,
                    mc_throttle_pct: mc_throttle,
                    fw_throttle_pct: fw_throttle,
                    transition_progress: progress,
                }
            }

            VtolState::CruiseFixedWing => VtolActuatorMixing {
                state: VtolState::CruiseFixedWing,
                mc_throttle_pct: 0.0,
                fw_throttle_pct: 1.0,
                transition_progress: 1.0,
            },

            VtolState::TransitionBack => {
                let progress =
                    1.0 - (self.phase_time_s / self.transition_duration_s).clamp(0.0, 1.0);
                let mc_throttle = 1.0 - progress;
                let fw_throttle = progress;

                if self.phase_time_s >= self.transition_duration_s {
                    self.state = VtolState::Hover;
                    self.phase_time_s = 0.0;
                    return VtolActuatorMixing {
                        state: VtolState::Hover,
                        mc_throttle_pct: 1.0,
                        fw_throttle_pct: 0.0,
                        transition_progress: 0.0,
                    };
                }

                VtolActuatorMixing {
                    state: VtolState::TransitionBack,
                    mc_throttle_pct: mc_throttle,
                    fw_throttle_pct: fw_throttle,
                    transition_progress: progress,
                }
            }
        }
    }
}
