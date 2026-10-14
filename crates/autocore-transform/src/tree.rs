//! autocore-transform :: SE(3) Koordinat Çerçevesi Ağacı (TransformTree)
//!
//! Çerçeveler arası zincirleme dönüşümleri (ör. map -> base_link -> camera_link -> target)
//! döngüsel bağımlılık kontrolü ve ortak ata (common ancestor) çözümüyle kilit-hafif yürütür.

use crate::error::TransformError;
use crate::se3::Transform3D;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

#[derive(Debug, Clone)]
struct FrameEdge {
    parent: String,
    transform: Transform3D, // Parent -> Child dönüşümü
    pub timestamp_us: u64,
}

/// Eşzamanlı Okuma Korumalı Homojen Koordinat Ağacı
pub struct TransformTree {
    edges: RwLock<HashMap<String, FrameEdge>>,
}

impl Default for TransformTree {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformTree {
    pub fn new() -> Self {
        Self {
            edges: RwLock::new(HashMap::new()),
        }
    }

    /// Yeni bir çerçeve dönüşümü ekler veya günceller (parent -> child)
    pub fn set_transform(
        &self,
        parent: &str,
        child: &str,
        transform: Transform3D,
        timestamp_us: u64,
    ) -> Result<(), TransformError> {
        if parent == child {
            return Err(TransformError::CyclicDependency(format!(
                "Öz-döngü: {} -> {}",
                parent, child
            )));
        }

        let mut edges = self.edges.write().unwrap();

        // Döngüsel bağımlılık kontrolü: child, parent'ın bir atası mı?
        let mut curr = parent;
        let mut visited = HashSet::new();
        visited.insert(parent.to_string());

        while let Some(edge) = edges.get(curr) {
            if edge.parent == child {
                return Err(TransformError::CyclicDependency(format!(
                    "Döngü tespit edildi: {} eklenirken {} -> ... -> {} döngüsü oluşuyor",
                    child, parent, child
                )));
            }
            if !visited.insert(edge.parent.clone()) {
                return Err(TransformError::CyclicDependency(format!(
                    "Mevcut ağaçta döngü var: {}",
                    edge.parent
                )));
            }
            curr = &edge.parent;
        }

        edges.insert(
            child.to_string(),
            FrameEdge {
                parent: parent.to_string(),
                transform,
                timestamp_us,
            },
        );

        Ok(())
    }

    /// Bir çerçevenin son güncellenme zaman damgasını (mikrosaniye) döndürür
    pub fn get_transform_timestamp(&self, child: &str) -> Option<u64> {
        let edges = self.edges.read().unwrap();
        edges.get(child).map(|e| e.timestamp_us)
    }

    /// source_frame'den target_frame'e dönüşümü hesaplar:
    /// p_target = T * p_source
    pub fn lookup_transform(
        &self,
        source_frame: &str,
        target_frame: &str,
    ) -> Result<Transform3D, TransformError> {
        if source_frame == target_frame {
            return Ok(Transform3D::identity());
        }

        let edges = self.edges.read().unwrap();

        // Çerçevelerin varlığını denetle
        if !edges.contains_key(source_frame) && !edges.values().any(|e| e.parent == source_frame) {
            return Err(TransformError::FrameNotFound(source_frame.to_string()));
        }
        if !edges.contains_key(target_frame) && !edges.values().any(|e| e.parent == target_frame) {
            return Err(TransformError::FrameNotFound(target_frame.to_string()));
        }

        // 1. source_frame'den köke doğru ata zincirini ve kümülatif T_{node -> source} dönüşümlerini bul
        let mut source_chain: Vec<(String, Transform3D)> = Vec::new();
        source_chain.push((source_frame.to_string(), Transform3D::identity()));

        let mut curr = source_frame;
        let mut accum_to_source = Transform3D::identity();

        while let Some(edge) = edges.get(curr) {
            // edge.transform: parent -> curr
            // Dolayısıyla parent -> source = (parent -> curr) * (curr -> source)
            accum_to_source = edge.transform.compose(&accum_to_source);
            source_chain.push((edge.parent.clone(), accum_to_source));
            curr = &edge.parent;
        }

        // 2. target_frame'den köke doğru ata zincirini ve kümülatif T_{node -> target} dönüşümlerini bul
        let mut target_chain: Vec<(String, Transform3D)> = Vec::new();
        target_chain.push((target_frame.to_string(), Transform3D::identity()));

        let mut curr_t = target_frame;
        let mut accum_to_target = Transform3D::identity();

        while let Some(edge) = edges.get(curr_t) {
            accum_to_target = edge.transform.compose(&accum_to_target);
            target_chain.push((edge.parent.clone(), accum_to_target));
            curr_t = &edge.parent;
        }

        // 3. En yakın ortak atayı bul
        let mut common_ancestor = None;
        let mut t_ancestor_to_source = Transform3D::identity();
        let mut t_ancestor_to_target = Transform3D::identity();

        'outer: for (s_node, s_tf) in &source_chain {
            for (t_node, t_tf) in &target_chain {
                if s_node == t_node {
                    common_ancestor = Some(s_node.clone());
                    t_ancestor_to_source = *s_tf;
                    t_ancestor_to_target = *t_tf;
                    break 'outer;
                }
            }
        }

        let _ancestor = match common_ancestor {
            Some(a) => a,
            None => {
                return Err(TransformError::NoCommonParent {
                    from: source_frame.to_string(),
                    to: target_frame.to_string(),
                });
            }
        };

        // p_ancestor = T_{ancestor -> source} * p_source
        // p_ancestor = T_{ancestor -> target} * p_target
        // => p_target = (T_{ancestor -> target})^{-1} * T_{ancestor -> source} * p_source
        let t_target_from_ancestor = t_ancestor_to_target.inverse();
        let final_transform = t_target_from_ancestor.compose(&t_ancestor_to_source);

        Ok(final_transform)
    }

    /// source_frame koordinatındaki bir noktayı target_frame koordinatına dönüştürür
    pub fn transform_point(
        &self,
        source_frame: &str,
        target_frame: &str,
        point: [f32; 3],
    ) -> Result<[f32; 3], TransformError> {
        let tf = self.lookup_transform(source_frame, target_frame)?;
        Ok(tf.transform_point(point))
    }

    /// source_frame koordinatındaki bir vektörü target_frame koordinatına dönüştürür
    pub fn transform_vector(
        &self,
        source_frame: &str,
        target_frame: &str,
        vector: [f32; 3],
    ) -> Result<[f32; 3], TransformError> {
        let tf = self.lookup_transform(source_frame, target_frame)?;
        Ok(tf.transform_vector(vector))
    }

    /// Ağaçtaki tüm kayıtlı çerçeve isimlerini döndürür
    pub fn all_frames(&self) -> Vec<String> {
        let edges = self.edges.read().unwrap();
        let mut set = HashSet::new();
        for (child, edge) in edges.iter() {
            set.insert(child.clone());
            set.insert(edge.parent.clone());
        }
        set.into_iter().collect()
    }

    /// Ağacı temizler
    pub fn clear(&self) {
        let mut edges = self.edges.write().unwrap();
        edges.clear();
    }
}
