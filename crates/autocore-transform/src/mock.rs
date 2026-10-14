//! autocore-transform :: SITL Test İkizi ve Mock Transform Yayıncısı

use crate::error::TransformError;
use crate::se3::Transform3D;
use crate::tree::TransformTree;
use crate::types::{LocalPose, TfStamped};
use std::sync::Arc;

/// SITL Test İkizi: Sanal robot kinematiği ve hareketli eklemleri simüle eder
pub struct MockTransformBroadcaster {
    tree: Arc<TransformTree>,
}

impl MockTransformBroadcaster {
    pub fn new(tree: Arc<TransformTree>) -> Self {
        Self { tree }
    }

    /// Standart robot kinematik ağacını kurar (map -> odom -> base_link -> camera_link / turret_link)
    pub fn setup_default_robot_tree(&self, timestamp_us: u64) -> Result<(), TransformError> {
        // map -> odom (Kimlik)
        self.tree
            .set_transform("map", "odom", Transform3D::identity(), timestamp_us)?;

        // odom -> base_link (Başlangıçta orijinde)
        self.tree.set_transform(
            "odom",
            "base_link",
            Transform3D::identity(),
            timestamp_us,
        )?;

        // base_link -> camera_link (Ön: +0.5m, Yukarı: +0.2m)
        self.tree.set_transform(
            "base_link",
            "camera_link",
            Transform3D::from_xyz_rpy(0.5, 0.0, 0.2, 0.0, 0.0, 0.0),
            timestamp_us,
        )?;

        // base_link -> turret_link (Merkezde, Yukarı: +0.4m)
        self.tree.set_transform(
            "base_link",
            "turret_link",
            Transform3D::from_xyz_rpy(0.0, 0.0, 0.4, 0.0, 0.0, 0.0),
            timestamp_us,
        )?;

        Ok(())
    }

    /// Zaman t (saniye) simülasyon adımı: aracı ileri sürer ve tareti sinüzoidal döndürür
    pub fn step_simulation(
        &self,
        t_sec: f32,
        timestamp_us: u64,
    ) -> Result<LocalPose, TransformError> {
        // Araç X ekseninde 2.0 m/s hızla ilerliyor
        let x = 2.0 * t_sec;
        let y = 0.5 * (t_sec * 0.5).sin();
        let yaw = 0.1 * (t_sec * 0.5).cos();

        let base_tf = Transform3D::from_xyz_rpy(x, y, 0.0, 0.0, 0.0, yaw);
        self.tree
            .set_transform("odom", "base_link", base_tf, timestamp_us)?;

        // Taret sinüzoidal salınım (azimut +- 45 derece)
        let turret_yaw = (t_sec * 2.0).sin() * 0.785398;
        let turret_tf = Transform3D::from_xyz_rpy(0.0, 0.0, 0.4, 0.0, 0.0, turret_yaw);
        self.tree
            .set_transform("base_link", "turret_link", turret_tf, timestamp_us)?;

        let quat_arr = base_tf.rotation.to_array();
        let pose = LocalPose::new(
            [x, y, 0.0],
            quat_arr,
            [2.0, 0.25 * (t_sec * 0.5).cos(), 0.0],
            timestamp_us,
        );

        Ok(pose)
    }

    /// Bir dönüşümü TfStamped C-ABI paketine dönüştürür
    pub fn create_stamped_tf(
        parent: &str,
        child: &str,
        tf: &Transform3D,
        timestamp_us: u64,
    ) -> TfStamped {
        TfStamped::new(
            parent,
            child,
            tf.translation,
            tf.rotation.to_array(),
            timestamp_us,
        )
    }
}
