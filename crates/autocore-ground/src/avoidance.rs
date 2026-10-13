use crate::costmap::LocalCostmap2D;

/// Yapay Potansiyel Alanlar (Artificial Potential Fields - APF) ile Reaktif Engelden Kaçınma.
#[derive(Debug, Clone)]
pub struct ArtificialPotentialField {
    pub k_att: f64,
    pub k_rep: f64,
    pub influence_radius_m: f64,
    pub max_velocity_mps: f64,
}

impl Default for ArtificialPotentialField {
    fn default() -> Self {
        Self {
            k_att: 1.0,
            k_rep: 2.5,
            influence_radius_m: 2.0,
            max_velocity_mps: 2.0,
        }
    }
}

impl ArtificialPotentialField {
    pub fn new(k_att: f64, k_rep: f64, influence_radius_m: f64, max_velocity_mps: f64) -> Self {
        Self {
            k_att,
            k_rep,
            influence_radius_m,
            max_velocity_mps,
        }
    }

    /// Hedef konumu ve etraftaki engel listesini alarak net kaçınma hız vektörünü ($v_x, v_y$) hesaplar.
    pub fn compute_velocity_vector(
        &self,
        current_pos: (f64, f64),
        goal_pos: (f64, f64),
        obstacles: &[(f64, f64)],
    ) -> (f64, f64) {
        // Çekici kuvvet (Attractive Force)
        let to_goal_x = goal_pos.0 - current_pos.0;
        let to_goal_y = goal_pos.1 - current_pos.1;
        let dist_to_goal = (to_goal_x * to_goal_x + to_goal_y * to_goal_y).sqrt();

        let (f_att_x, f_att_y) = if dist_to_goal > 1e-4 {
            (
                self.k_att * (to_goal_x / dist_to_goal),
                self.k_att * (to_goal_y / dist_to_goal),
            )
        } else {
            (0.0, 0.0)
        };

        // İtici kuvvet (Repulsive Force)
        let mut f_rep_x = 0.0;
        let mut f_rep_y = 0.0;

        for &(ox, oy) in obstacles {
            let dx = current_pos.0 - ox;
            let dy = current_pos.1 - oy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > 1e-4 && dist <= self.influence_radius_m {
                let rep_mag = self.k_rep
                    * (1.0 / dist - 1.0 / self.influence_radius_m)
                    * (1.0 / (dist * dist));
                f_rep_x += rep_mag * (dx / dist);
                f_rep_y += rep_mag * (dy / dist);
            }
        }

        // Toplam kuvvet
        let total_x = f_att_x + f_rep_x;
        let total_y = f_att_y + f_rep_y;
        let total_mag = (total_x * total_x + total_y * total_y).sqrt();

        if total_mag > 1e-4 {
            let scale = (total_mag.min(self.max_velocity_mps)) / total_mag;
            (total_x * scale, total_y * scale)
        } else {
            (0.0, 0.0)
        }
    }
}

/// Dinamik Pencere Yaklaşımı (Dynamic Window Approach - DWA)
#[derive(Debug, Clone)]
pub struct DynamicWindowApproach {
    pub max_v: f64,
    pub max_omega: f64,
    pub max_accel_v: f64,
    pub max_accel_omega: f64,
    pub sim_time_s: f64,
    pub sim_step_s: f64,
}

impl Default for DynamicWindowApproach {
    fn default() -> Self {
        Self {
            max_v: 2.0,
            max_omega: 1.5,
            max_accel_v: 1.5,
            max_accel_omega: 2.5,
            sim_time_s: 1.5,
            sim_step_s: 0.1,
        }
    }
}

impl DynamicWindowApproach {
    /// Dinamik pencere içinde en iyi hız çiftini $(v^*, \omega^*)$ seçer.
    pub fn select_velocity(
        &self,
        current_v: f64,
        current_omega: f64,
        dt_s: f64,
        goal: (f64, f64),
        costmap: &LocalCostmap2D,
    ) -> (f64, f64) {
        // Dinamik hız sınırları (Dynamic Window)
        let min_v = (current_v - self.max_accel_v * dt_s).max(0.0);
        let max_v = (current_v + self.max_accel_v * dt_s).min(self.max_v);

        let min_w = (current_omega - self.max_accel_omega * dt_s).max(-self.max_omega);
        let max_w = (current_omega + self.max_accel_omega * dt_s).min(self.max_omega);

        let mut best_score = -1e9;
        let mut best_cmd = (0.0, 0.0);

        // Hız örnekleme
        let v_steps = 10;
        let w_steps = 15;

        for i in 0..=v_steps {
            let v_cand = min_v + (max_v - min_v) * (i as f64 / v_steps as f64);
            for j in 0..=w_steps {
                let w_cand = min_w + (max_w - min_w) * (j as f64 / w_steps as f64);

                if let Some(score) = self.evaluate_trajectory(v_cand, w_cand, goal, costmap) {
                    if score > best_score {
                        best_score = score;
                        best_cmd = (v_cand, w_cand);
                    }
                }
            }
        }

        best_cmd
    }

    fn evaluate_trajectory(
        &self,
        v: f64,
        w: f64,
        goal: (f64, f64),
        costmap: &LocalCostmap2D,
    ) -> Option<f64> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut theta = 0.0;

        let steps = (self.sim_time_s / self.sim_step_s).ceil() as usize;
        let mut min_clearance_cost = 0.0;

        for _ in 0..steps {
            theta += w * self.sim_step_s;
            x += v * theta.cos() * self.sim_step_s;
            y += v * theta.sin() * self.sim_step_s;

            // Çarpışma kontrolü
            if !costmap.is_collision_free(x, y, 0.25) {
                return None; // Çarpışmalı rota elendi
            }

            if let Some((gx, gy)) = costmap.metric_to_grid(x, y) {
                min_clearance_cost += costmap.get_cost(gx, gy) as f64;
            }
        }

        // Skor = Hedefe yaklaşım + Hız ödülü - Engel maliyeti
        let dist_to_goal = ((goal.0 - x).powi(2) + (goal.1 - y).powi(2)).sqrt();
        let heading_score = -dist_to_goal;
        let vel_score = v;
        let obstacle_penalty = min_clearance_cost * 0.05;

        Some(heading_score * 2.0 + vel_score * 1.0 - obstacle_penalty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apf_clear_path() {
        let apf = ArtificialPotentialField::default();
        let curr = (0.0, 0.0);
        let goal = (10.0, 0.0); // Tam x ekseni üzerinde

        let (vx, vy) = apf.compute_velocity_vector(curr, goal, &[]);
        assert!(vx > 0.0);
        assert!(vy.abs() < 1e-4);
    }

    #[test]
    fn test_apf_obstacle_repulsion() {
        let apf = ArtificialPotentialField::default();
        let curr = (0.0, 0.0);
        let goal = (5.0, 0.0);

        // Tam yolun ortasında engel: (2.0, 0.0)
        let obstacles = vec![(2.0, 0.0)];

        let (vx, _vy) = apf.compute_velocity_vector(curr, goal, &obstacles);

        // Engel tam önümüzde olduğu için net x itişi azalmalı veya y yönünde sapmalı
        assert!(vx < 1.5);
    }
}
