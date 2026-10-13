//! autocore-defense-tests :: Savunma Sanayii Test ve Doğrulama Çekirdeği
//!
//! MIL-STD-882E (Sistem Emniyeti & FMEA), DO-178C (Uçuş Yazılımı Güvenilirliği)
//! ve STANAG 4586 standartlarında hata enjeksiyonu, sınır değer stres testleri,
//! deterministik bit çevirme fuzzing ve dayanıklılık doğrulama araçları.

/// Deterministik, tekrarlanabilir XorShift64 sözde rastgele sayı üreteci (Seed tabanlı)
#[derive(Debug, Clone)]
pub struct DeterministicPrng {
    state: u64,
}

impl DeterministicPrng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x5EED_CAFE_BABE_FACE } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    pub fn next_usize(&mut self, bound: usize) -> usize {
        assert!(bound > 0);
        (self.next_u64() as usize) % bound
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }

    /// Verilen bayt diliminde rastgele 1 ila `max_flips` bitin değerini tersine çevirir (Bit-flip fuzzing)
    pub fn mutate_bits(&mut self, data: &mut [u8], max_flips: usize) -> Vec<(usize, u8)> {
        if data.is_empty() {
            return Vec::new();
        }
        let flips = (self.next_usize(max_flips)) + 1;
        let mut log = Vec::with_capacity(flips);

        for _ in 0..flips {
            let byte_idx = self.next_usize(data.len());
            let bit_idx = (self.next_u32() % 8) as u8;
            data[byte_idx] ^= 1 << bit_idx;
            log.push((byte_idx, bit_idx));
        }
        log
    }
}
