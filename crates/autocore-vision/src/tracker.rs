use crate::types::{Detection, Track, TrackState};

/// SORT (Simple Online and Realtime Tracking) Çoklu Nesne Takipçisi.
#[derive(Debug, Clone)]
pub struct SortTracker {
    pub max_age: u32,
    pub min_hits: u32,
    pub iou_threshold: f32,
    next_id: u32,
    tracks: Vec<Track>,
}

impl Default for SortTracker {
    fn default() -> Self {
        Self::new(5, 3, 0.3)
    }
}

impl SortTracker {
    pub fn new(max_age: u32, min_hits: u32, iou_threshold: f32) -> Self {
        Self {
            max_age,
            min_hits,
            iou_threshold,
            next_id: 1,
            tracks: Vec::new(),
        }
    }

    /// Yeni kamera karesindeki tespitler ile mevcut hedefleri eşler ve güncel takip listesini döner.
    pub fn update(&mut self, detections: &[Detection]) -> Vec<Track> {
        // 1. Mevcut hedeflerin konumunu sabit hız modeliyle tahmin et
        for track in &mut self.tracks {
            track.bbox.x += track.velocity[0];
            track.bbox.y += track.velocity[1];
            track.age += 1;
            track.time_since_update += 1;
        }

        // 2. Eşleme matrisi (IoU Matching)
        let num_tracks = self.tracks.len();
        let num_dets = detections.len();

        let mut matched_tracks = vec![false; num_tracks];
        let mut matched_dets = vec![false; num_dets];

        // Greedy IoU eşlemesi
        if num_tracks > 0 && num_dets > 0 {
            let mut iou_pairs = Vec::new();
            for (t_idx, track) in self.tracks.iter().enumerate() {
                for (d_idx, det) in detections.iter().enumerate() {
                    let iou = track.bbox.iou(&det.bbox);
                    if iou >= self.iou_threshold {
                        iou_pairs.push((iou, t_idx, d_idx));
                    }
                }
            }

            // En yüksek IoU'ya göre sırala
            iou_pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

            for (_iou, t_idx, d_idx) in iou_pairs {
                if !matched_tracks[t_idx] && !matched_dets[d_idx] {
                    matched_tracks[t_idx] = true;
                    matched_dets[d_idx] = true;

                    // Takip güncellemesi
                    let track = &mut self.tracks[t_idx];
                    let det = &detections[d_idx];

                    let (prev_cx, prev_cy) = track.bbox.center();
                    let (new_cx, new_cy) = det.bbox.center();

                    let vx = new_cx - prev_cx;
                    let vy = new_cy - prev_cy;

                    // Hız filtresi (alpha yumuşatma)
                    track.velocity[0] = track.velocity[0] * 0.4 + vx * 0.6;
                    track.velocity[1] = track.velocity[1] * 0.4 + vy * 0.6;

                    track.bbox = det.bbox;
                    track.class_id = det.class_id;
                    track.hits += 1;
                    track.time_since_update = 0;

                    if track.hits >= self.min_hits {
                        track.state = TrackState::Confirmed;
                    }
                }
            }
        }

        // 3. Eşleşmeyen tespitler için yeni takip oluştur
        for (d_idx, det) in detections.iter().enumerate() {
            if !matched_dets[d_idx] {
                let track = Track {
                    id: self.next_id,
                    bbox: det.bbox,
                    velocity: [0.0, 0.0],
                    state: TrackState::Tentative,
                    age: 1,
                    hits: 1,
                    time_since_update: 0,
                    class_id: det.class_id,
                };
                self.next_id += 1;
                self.tracks.push(track);
            }
        }

        // 4. Güncellenmeyen eski takipleri düşür veya Coasting (Lost) durumuna al
        for (t_idx, track) in self.tracks.iter_mut().enumerate() {
            if !matched_tracks.get(t_idx).copied().unwrap_or(false) {
                if track.time_since_update > self.max_age {
                    track.state = TrackState::Deleted;
                } else if track.state == TrackState::Confirmed {
                    track.state = TrackState::Lost; // Coasting (Kör Uçuş)
                }
            }
        }

        self.tracks.retain(|t| t.state != TrackState::Deleted);

        self.tracks.clone()
    }

    pub fn active_tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn confirmed_tracks(&self) -> Vec<Track> {
        self.tracks
            .iter()
            .filter(|t| t.state == TrackState::Confirmed)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoundingBox;

    #[test]
    fn test_tracker_lifecycle_and_confirmation() {
        let mut tracker = SortTracker::new(3, 2, 0.3); // 2 hit'te Confirmed olsun

        // Kare 1: İlk tespit
        let det1 = Detection {
            bbox: BoundingBox::new(10.0, 10.0, 20.0, 20.0),
            confidence: 0.9,
            class_id: 1,
            class_name: "car".into(),
        };
        let tracks1 = tracker.update(&[det1]);
        assert_eq!(tracks1.len(), 1);
        assert_eq!(tracks1[0].state, TrackState::Tentative);

        // Kare 2: Hafif hareket etmiş tespit
        let det2 = Detection {
            bbox: BoundingBox::new(12.0, 10.0, 20.0, 20.0),
            confidence: 0.92,
            class_id: 1,
            class_name: "car".into(),
        };
        let tracks2 = tracker.update(&[det2]);
        assert_eq!(tracks2.len(), 1);
        assert_eq!(tracks2[0].id, tracks1[0].id); // Aynı hedef
        assert_eq!(tracks2[0].state, TrackState::Confirmed); // 2 hit ile onaylandı
        assert!(tracks2[0].velocity[0] > 0.0); // Sağa doğru x hızı ölçüldü
    }

    #[test]
    fn test_tracker_drops_lost_targets() {
        let mut tracker = SortTracker::new(2, 1, 0.3);

        let det = Detection {
            bbox: BoundingBox::new(50.0, 50.0, 10.0, 10.0),
            confidence: 0.85,
            class_id: 2,
            class_name: "drone".into(),
        };
        tracker.update(&[det]);
        assert_eq!(tracker.active_tracks().len(), 1);

        // 3 boş kare geçsin
        tracker.update(&[]);
        tracker.update(&[]);
        tracker.update(&[]);

        // max_age = 2 aşıldığı için hedef silinmiş olmalı
        assert_eq!(tracker.active_tracks().len(), 0);
    }
}
