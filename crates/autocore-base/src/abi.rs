//! autocore-base :: C-ABI Boyut ve Hizalama Doğrulama Katmanı

use crate::types::{ActuatorFeedback, KinematicCommand, StateVector6DoF};
use static_assertions::{assert_eq_align, assert_eq_size};

// Derleme zamanında struct boyutlarını (byte size) kilitliyoruz:
// 1. KinematicCommand: 32 Bayt
assert_eq_size!(KinematicCommand, [u8; 32]);

// 2. StateVector6DoF: 64 Bayt
assert_eq_size!(StateVector6DoF, [u8; 64]);

// 3. ActuatorFeedback: 72 Bayt
assert_eq_size!(ActuatorFeedback, [u8; 72]);

// Hizalama (Alignment) doğrulamaları:
// 64-bit mimarilerde u64 timestamp nedeniyle 8-bayt hizalama olmalı
assert_eq_align!(KinematicCommand, u64);
assert_eq_align!(StateVector6DoF, u64);
assert_eq_align!(ActuatorFeedback, u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_sizes_and_alignment() {
        assert_eq!(std::mem::size_of::<KinematicCommand>(), 32);
        assert_eq!(std::mem::size_of::<StateVector6DoF>(), 64);
        assert_eq!(std::mem::size_of::<ActuatorFeedback>(), 72);

        assert_eq!(std::mem::align_of::<KinematicCommand>(), 8);
        assert_eq!(std::mem::align_of::<StateVector6DoF>(), 8);
        assert_eq!(std::mem::align_of::<ActuatorFeedback>(), 8);
    }
}
