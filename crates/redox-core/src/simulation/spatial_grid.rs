use std::collections::{HashMap, HashSet};

use glam::Vec2;

use crate::game_object::GameObject;

type Cells = HashMap<(i32, i32), Vec<usize>>;

pub struct SpatialGrid {
	cell_size: f32,
	cells: Cells,
}

impl SpatialGrid {
	pub fn new(objects: &[GameObject], cell_size: f32) -> Self {
		let mut cells: Cells = HashMap::new();

		for (idx, obj) in objects.iter().enumerate() {
			// Get the bounding box of the object
			let (min_x, min_y, max_x, max_y) = if let Some(obb) = &obj.obb {
				let mut min_x = f32::MAX;
				let mut min_y = f32::MAX;
				let mut max_x = f32::MIN;
				let mut max_y = f32::MIN;
				for corner in &obb.corners {
					min_x = min_x.min(corner.x);
					min_y = min_y.min(corner.y);
					max_x = max_x.max(corner.x);
					max_y = max_y.max(corner.y);
				}
				(min_x, min_y, max_x, max_y)
			} else {
				// Circular object or fallback
				let radius = obj.width * 0.5;
				(
					obj.position.x - radius,
					obj.position.y - radius,
					obj.position.x + radius,
					obj.position.y + radius,
				)
			};

			let start_x = (min_x / cell_size).floor() as i32;
			let end_x = (max_x / cell_size).floor() as i32;
			let start_y = (min_y / cell_size).floor() as i32;
			let end_y = (max_y / cell_size).floor() as i32;

			for cx in start_x..=end_x {
				for cy in start_y..=end_y {
					cells.entry((cx, cy)).or_default().push(idx);
				}
			}
		}

		Self { cell_size, cells }
	}

	pub fn query(
		&self, position: Vec2, width: f32, height: f32,
	) -> impl Iterator<Item = usize> + '_ {
		let half_w = width * 0.5;
		let half_h = height * 0.5;

		let min_x = position.x - half_w;
		let max_x = position.x + half_w;

		let min_y = position.y - half_h;
		let max_y = position.y + half_h;

		let start_x = (min_x / self.cell_size).floor() as i32;
		let end_x = (max_x / self.cell_size).floor() as i32;

		let start_y = (min_y / self.cell_size).floor() as i32;
		let end_y = (max_y / self.cell_size).floor() as i32;

		let mut unique_indices = HashSet::new();
		for cx in start_x..=end_x {
			for cy in start_y..=end_y {
				if let Some(indices) = self.cells.get(&(cx, cy)) {
					for &idx in indices {
						unique_indices.insert(idx);
					}
				}
			}
		}

		unique_indices.into_iter()
	}
}
