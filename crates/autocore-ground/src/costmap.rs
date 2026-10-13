pub const COST_FREE: u8 = 0;
pub const COST_INFLATED_MAX: u8 = 253;
pub const COST_LETHAL: u8 = 254;
pub const COST_UNKNOWN: u8 = 255;

/// 2B Yerel Maliyet Haritası (Local Costmap 2D).
/// Robot merkezli, engelleri ve güvenlik şişirme katmanını (Inflation Layer) tutar.
#[derive(Debug, Clone)]
pub struct LocalCostmap2D {
    pub width: usize,
    pub height: usize,
    pub resolution_m: f64,
    pub origin_x_m: f64,
    pub origin_y_m: f64,
    data: Vec<u8>,
}

impl LocalCostmap2D {
    pub fn new(width: usize, height: usize, resolution_m: f64) -> Self {
        assert!(width > 0 && height > 0 && resolution_m > 0.0);
        let origin_x_m = -(width as f64 * resolution_m) * 0.5;
        let origin_y_m = -(height as f64 * resolution_m) * 0.5;
        Self {
            width,
            height,
            resolution_m,
            origin_x_m,
            origin_y_m,
            data: vec![COST_FREE; width * height],
        }
    }

    pub fn metric_to_grid(&self, x_m: f64, y_m: f64) -> Option<(usize, usize)> {
        if !x_m.is_finite() || !y_m.is_finite() {
            return None;
        }
        let gx = ((x_m - self.origin_x_m) / self.resolution_m).floor() as isize;
        let gy = ((y_m - self.origin_y_m) / self.resolution_m).floor() as isize;

        if gx >= 0 && gx < self.width as isize && gy >= 0 && gy < self.height as isize {
            Some((gx as usize, gy as usize))
        } else {
            None
        }
    }

    pub fn grid_to_metric(&self, gx: usize, gy: usize) -> (f64, f64) {
        let x = self.origin_x_m + (gx as f64 + 0.5) * self.resolution_m;
        let y = self.origin_y_m + (gy as f64 + 0.5) * self.resolution_m;
        (x, y)
    }

    pub fn get_cost(&self, gx: usize, gy: usize) -> u8 {
        if gx < self.width && gy < self.height {
            self.data[gy * self.width + gx]
        } else {
            COST_UNKNOWN
        }
    }

    pub fn set_cost(&mut self, gx: usize, gy: usize, cost: u8) {
        if gx < self.width && gy < self.height {
            self.data[gy * self.width + gx] = cost;
        }
    }

    /// Metrik koordinatta ölümcül engel (Lethal Obstacle) işaretler.
    pub fn set_obstacle(&mut self, x_m: f64, y_m: f64) -> bool {
        if let Some((gx, gy)) = self.metric_to_grid(x_m, y_m) {
            self.set_cost(gx, gy, COST_LETHAL);
            true
        } else {
            false
        }
    }

    /// Güvenlik payı şişirme katmanı (Inflation Layer) uygular.
    /// Robot yarıçapı içindeki hücreleri ölümcül (254), şişirme yarıçapı içindekileri üstel sönümlü (1..253) yapar.
    pub fn apply_inflation(&mut self, robot_radius_m: f64, inflation_radius_m: f64, decay_alpha: f64) {
        let mut lethal_cells = Vec::new();

        for gy in 0..self.height {
            for gx in 0..self.width {
                if self.get_cost(gx, gy) == COST_LETHAL {
                    lethal_cells.push((gx, gy));
                }
            }
        }

        let max_cell_dist = (inflation_radius_m / self.resolution_m).ceil() as isize;

        for (lx, ly) in lethal_cells {
            let min_x = (lx as isize - max_cell_dist).max(0) as usize;
            let max_x = (lx as isize + max_cell_dist).min(self.width as isize - 1) as usize;
            let min_y = (ly as isize - max_cell_dist).max(0) as usize;
            let max_y = (ly as isize + max_cell_dist).min(self.height as isize - 1) as usize;

            for gy in min_y..=max_y {
                for gx in min_x..=max_x {
                    let dx = (gx as f64 - lx as f64) * self.resolution_m;
                    let dy = (gy as f64 - ly as f64) * self.resolution_m;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist <= robot_radius_m {
                        self.set_cost(gx, gy, COST_LETHAL);
                    } else if dist <= inflation_radius_m {
                        let factor = (-decay_alpha * (dist - robot_radius_m)).exp();
                        let cost = (COST_INFLATED_MAX as f64 * factor).round() as u8;
                        let curr_cost = self.get_cost(gx, gy);
                        if curr_cost < cost && curr_cost != COST_LETHAL {
                            self.set_cost(gx, gy, cost);
                        }
                    }
                }
            }
        }
    }

    /// Verilen metrik koordinat ve güvenlik yarıçapında çarpışma riski olup olmadığını denetler.
    pub fn is_collision_free(&self, x_m: f64, y_m: f64, radius_m: f64) -> bool {
        let cell_radius = (radius_m / self.resolution_m).ceil() as isize;
        if let Some((cx, cy)) = self.metric_to_grid(x_m, y_m) {
            let min_x = (cx as isize - cell_radius).max(0) as usize;
            let max_x = (cx as isize + cell_radius).min(self.width as isize - 1) as usize;
            let min_y = (cy as isize - cell_radius).max(0) as usize;
            let max_y = (cy as isize + cell_radius).min(self.height as isize - 1) as usize;

            for gy in min_y..=max_y {
                for gx in min_x..=max_x {
                    let (mx, my) = self.grid_to_metric(gx, gy);
                    let d = ((mx - x_m).powi(2) + (my - y_m).powi(2)).sqrt();
                    if d <= radius_m && self.get_cost(gx, gy) >= COST_LETHAL {
                        return false;
                    }
                }
            }
            true
        } else {
            false // Harita dışı güvensiz kabul edilir
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(COST_FREE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_costmap_metric_grid_transform() {
        let map = LocalCostmap2D::new(100, 100, 0.1); // 10m x 10m harita, robot (0,0)'da

        let grid_center = map.metric_to_grid(0.0, 0.0).unwrap();
        assert_eq!(grid_center, (50, 50));

        let (mx, my) = map.grid_to_metric(50, 50);
        assert!(mx.abs() <= 0.06);
        assert!(my.abs() <= 0.06);
    }

    #[test]
    fn test_costmap_inflation() {
        let mut map = LocalCostmap2D::new(50, 50, 0.1); // 5m x 5m

        // (1.0, 1.0) noktasına engel koy
        map.set_obstacle(1.0, 1.0);

        // Şişir: robot yarıçapı 0.2m, şişirme 0.5m
        map.apply_inflation(0.2, 0.5, 3.0);

        // Engelin tam üstü ölümcül olmalı
        assert!(!map.is_collision_free(1.0, 1.0, 0.1));

        // Engelin 0.15m yakını ölümcül bölge içinde olmalı
        assert!(!map.is_collision_free(1.15, 1.0, 0.05));

        // Engelin 2m uzağı tamamen serbest olmalı
        assert!(map.is_collision_free(-1.0, -1.0, 0.2));
    }
}
