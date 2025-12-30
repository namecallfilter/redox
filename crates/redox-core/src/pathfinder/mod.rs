pub mod builder;
pub mod search;
pub mod sim;
pub mod solver;

use crate::{config::Config, game_object::GameObject, simulation::spatial_grid::SpatialGrid};

pub struct Pathfinder {
	pub(crate) objects: Vec<GameObject>,
	pub(crate) config: Config,
	pub(crate) max_obj_width: f32,
	pub(crate) grid: SpatialGrid,
}

impl Pathfinder {
	pub fn dt(&self) -> f32 {
		self.config.physics.dt
	}
}
