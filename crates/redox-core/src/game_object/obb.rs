use glam::Vec2;

#[derive(Debug, Clone)]
pub struct OBB2D {
	pub center: Vec2,
	pub corners: [Vec2; 4],
	pub axes: [Vec2; 2],
}

impl OBB2D {
	pub fn new(center: Vec2, width: f32, height: f32, rotation_degrees: f32) -> Self {
		let rotation_rad = rotation_degrees.to_radians();
		let cos_a = rotation_rad.cos();
		let sin_a = rotation_rad.sin();
		let hw = width * 0.5;
		let hh = height * 0.5;
		let axis_x = Vec2::new(cos_a, sin_a);
		let axis_y = Vec2::new(-sin_a, cos_a);
		let x = axis_x * hw;
		let y = axis_y * hh;
		let corners = [
			center - x - y,
			center + x - y,
			center + x + y,
			center - x + y,
		];
		let axes = [axis_x, axis_y];
		OBB2D {
			center,
			corners,
			axes,
		}
	}

	pub fn project_onto(&self, axis: Vec2) -> (f32, f32) {
		let mut min = self.corners[0].dot(axis);
		let mut max = min;
		for corner in &self.corners[1..] {
			let p = corner.dot(axis);
			if p < min {
				min = p;
			} else if p > max {
				max = p;
			}
		}
		(min, max)
	}

	pub fn overlaps(&self, other: &OBB2D) -> bool {
		for axis in self.axes.iter().chain(other.axes.iter()) {
			let (min_a, max_a) = self.project_onto(*axis);
			let (min_b, max_b) = other.project_onto(*axis);
			if max_a < min_b || max_b < min_a {
				return false;
			}
		}
		true
	}
}
