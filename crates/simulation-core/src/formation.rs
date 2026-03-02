/// A rectangular grid formation for troop placement.
/// Produces positions centered around (0, 0).
pub struct Formation {
    pub width: u32,
    pub height: u32,
    pub spacing: f32,
}

impl Formation {
    pub fn new(width: u32, height: u32, spacing: f32) -> Self {
        Self { width, height, spacing }
    }

    /// Returns (x, y) offsets centered on (0, 0).
    /// X spans columns left-to-right, Y spans rows top-to-bottom (negative Y = up/north).
    pub fn positions(&self) -> Vec<(f32, f32)> {
        let mut result = Vec::with_capacity((self.width * self.height) as usize);
        let half_w = (self.width as f32 - 1.0) / 2.0;
        let half_h = (self.height as f32 - 1.0) / 2.0;

        for row in 0..self.height {
            for col in 0..self.width {
                let x = (col as f32 - half_w) * self.spacing;
                let y = (row as f32 - half_h) * self.spacing;
                result.push((x, y));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skull_formation_count() {
        let formation = Formation::new(3, 10, 32.0);
        let positions = formation.positions();
        assert_eq!(positions.len(), 30);
    }

    #[test]
    fn formation_centered() {
        let formation = Formation::new(3, 10, 32.0);
        let positions = formation.positions();

        let avg_x: f32 = positions.iter().map(|(x, _)| x).sum::<f32>() / positions.len() as f32;
        let avg_y: f32 = positions.iter().map(|(_, y)| y).sum::<f32>() / positions.len() as f32;

        assert!((avg_x).abs() < 0.001, "avg_x should be ~0, got {avg_x}");
        assert!((avg_y).abs() < 0.001, "avg_y should be ~0, got {avg_y}");
    }

    #[test]
    fn formation_spacing() {
        let spacing = 32.0;
        let formation = Formation::new(3, 10, spacing);
        let positions = formation.positions();

        // First row: positions [0], [1], [2] should be spaced by `spacing` in x
        let (x0, y0) = positions[0];
        let (x1, y1) = positions[1];
        assert!((x1 - x0 - spacing).abs() < 0.001);
        assert!((y1 - y0).abs() < 0.001);

        // First column: positions [0] and [3] should be spaced by `spacing` in y
        let (x3, y3) = positions[3];
        assert!((x3 - x0).abs() < 0.001);
        assert!((y3 - y0 - spacing).abs() < 0.001);
    }

    #[test]
    fn single_unit_formation() {
        let formation = Formation::new(1, 1, 32.0);
        let positions = formation.positions();
        assert_eq!(positions.len(), 1);
        assert!((positions[0].0).abs() < 0.001);
        assert!((positions[0].1).abs() < 0.001);
    }
}
