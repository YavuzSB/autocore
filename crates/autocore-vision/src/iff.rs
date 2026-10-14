use std::collections::HashMap;
use crate::types::TargetClassification;

/// Dinamik IFF (Dost / Düşman Teşhis) Histerezis ve Çoğunluk Oylaması Filtresi.
/// Kamera görüntüsündeki tekil kare gürültülerinin yanlış angajmana yol açmasını engeller.
#[derive(Debug, Clone)]
pub struct DynamicIffFilter {
    pub window_size: usize,
    pub min_samples: usize,
    pub confidence_threshold: f32,
    histories: HashMap<u32, Vec<(TargetClassification, f32)>>,
}

impl Default for DynamicIffFilter {
    fn default() -> Self {
        Self::new(7, 3, 0.65)
    }
}

impl DynamicIffFilter {
    pub fn new(window_size: usize, min_samples: usize, confidence_threshold: f32) -> Self {
        assert!(window_size > 0 && min_samples > 0 && confidence_threshold > 0.0);
        Self {
            window_size,
            min_samples,
            confidence_threshold,
            histories: HashMap::new(),
        }
    }

    /// Hedefin yeni karedeki sınıflandırma ve güven skorunu ekler; filtrelenmiş kararı döner.
    pub fn update(
        &mut self,
        track_id: u32,
        raw_class: TargetClassification,
        confidence: f32,
    ) -> TargetClassification {
        let history = self
            .histories
            .entry(track_id)
            .or_insert_with(|| Vec::with_capacity(self.window_size));

        if history.len() >= self.window_size {
            history.remove(0);
        }
        history.push((raw_class, confidence));

        // Asgari gözlem sayısı dolmadan karar üretme (Güvenlik Kapısı)
        if history.len() < self.min_samples {
            return TargetClassification::Unknown;
        }

        // Ağırlıklı çoğunluk oylaması (Weighted Majority Voting)
        let mut score_foe = 0.0f32;
        let mut score_friend = 0.0f32;
        let mut score_neutral = 0.0f32;
        let mut total_weight = 0.0f32;

        for &(class, conf) in history.iter() {
            total_weight += conf;
            match class {
                TargetClassification::Foe => score_foe += conf,
                TargetClassification::Friend => score_friend += conf,
                TargetClassification::Neutral => score_neutral += conf,
                TargetClassification::Unknown => {}
            }
        }

        if total_weight < 1e-4 {
            return TargetClassification::Unknown;
        }

        let ratio_foe = score_foe / total_weight;
        let ratio_friend = score_friend / total_weight;
        let ratio_neutral = score_neutral / total_weight;

        if ratio_foe >= self.confidence_threshold {
            TargetClassification::Foe
        } else if ratio_friend >= self.confidence_threshold {
            TargetClassification::Friend
        } else if ratio_neutral >= self.confidence_threshold {
            TargetClassification::Neutral
        } else {
            TargetClassification::Unknown
        }
    }

    /// Mevcut hedefin güncel sınıflandırma durumunu okur (yeni gözlem eklemeden)
    pub fn classify(&self, track_id: u32) -> TargetClassification {
        let history = match self.histories.get(&track_id) {
            Some(h) => h,
            None => return TargetClassification::Unknown,
        };

        if history.len() < self.min_samples {
            return TargetClassification::Unknown;
        }

        let mut score_foe = 0.0f32;
        let mut score_friend = 0.0f32;
        let mut score_neutral = 0.0f32;
        let mut total_weight = 0.0f32;

        for &(class, conf) in history.iter() {
            total_weight += conf;
            match class {
                TargetClassification::Foe => score_foe += conf,
                TargetClassification::Friend => score_friend += conf,
                TargetClassification::Neutral => score_neutral += conf,
                TargetClassification::Unknown => {}
            }
        }

        if total_weight < 1e-4 {
            return TargetClassification::Unknown;
        }

        let ratio_foe = score_foe / total_weight;
        let ratio_friend = score_friend / total_weight;
        let ratio_neutral = score_neutral / total_weight;

        if ratio_foe >= self.confidence_threshold {
            TargetClassification::Foe
        } else if ratio_friend >= self.confidence_threshold {
            TargetClassification::Friend
        } else if ratio_neutral >= self.confidence_threshold {
            TargetClassification::Neutral
        } else {
            TargetClassification::Unknown
        }
    }

    pub fn remove_track(&mut self, track_id: u32) {
        self.histories.remove(&track_id);
    }

    pub fn clear(&mut self) {
        self.histories.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iff_hysteresis_and_majority_voting() {
        let mut iff = DynamicIffFilter::new(5, 3, 0.6);

        // 1. İlk kare Foe gelse bile henüz min_samples (3) dolmadığı için Unknown kalmalı
        assert_eq!(
            iff.update(1, TargetClassification::Foe, 0.5),
            TargetClassification::Unknown
        );

        // 2. Ardışık 3 gözlem tamamlanınca Foe olarak onaylanır
        iff.update(1, TargetClassification::Foe, 0.9);
        let status = iff.update(1, TargetClassification::Foe, 0.95);
        assert_eq!(status, TargetClassification::Foe);

        // 3. Tek bir gürültülü kare (Friend) durumu anında bozamaz (histerezis)
        let noisy = iff.update(1, TargetClassification::Friend, 0.7);
        assert_eq!(noisy, TargetClassification::Foe);
    }
}
