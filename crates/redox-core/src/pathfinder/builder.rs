use super::Pathfinder;
use crate::{config::Config, game_object::GameObject, simulation::spatial_grid::SpatialGrid};

impl Pathfinder {
	pub fn new(mut objects: Vec<GameObject>, _goal_x: f32) -> Self {
		let config = Config::default();

		objects.sort_by(|a, b| {
			let min_a = a.position.x - a.width * 0.5;
			let min_b = b.position.x - b.width * 0.5;
			min_a.partial_cmp(&min_b).unwrap()
		});

		let mut max_obj_width = 0.0f32;
		for obj in &objects {
			if obj.width > max_obj_width {
				max_obj_width = obj.width;
			}
		}

		max_obj_width += 10.0;

		let grid = SpatialGrid::new(&objects, 128.0);

		Self {
			objects,
			config,
			max_obj_width,
			grid,
		}
	}

	pub fn with_config(mut objects: Vec<GameObject>, config: Config) -> Self {
		objects.sort_by(|a, b| {
			let min_a = a.position.x - a.width * 0.5;
			let min_b = b.position.x - b.width * 0.5;
			min_a.partial_cmp(&min_b).unwrap()
		});

		let mut max_obj_width = 0.0f32;
		for obj in &objects {
			if obj.width > max_obj_width {
				max_obj_width = obj.width;
			}
		}

		max_obj_width += 10.0;

		let grid = SpatialGrid::new(&objects, 128.0);

		Self {
			objects,
			config,
			max_obj_width,
			grid,
		}
	}
}
