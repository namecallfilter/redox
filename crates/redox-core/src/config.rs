pub struct PhysicsParams {
	pub gravities: [f32; 5],
	pub jump_velocities: [f32; 5],
	pub player_speeds: [f32; 5],
	pub player_width: f32,
	pub player_height: f32,
	pub ship_velocities: [f32; 5],
	pub ship_bounds: f32,
	pub dt: f32,
	pub vertical_dt_scale: f32,
	pub vy_quantize_step: f32,
}

impl Default for PhysicsParams {
	fn default() -> Self {
		Self {
			gravities: [-2747.52, -2794.1082, -2786.4, -2799.36, -2799.36],
			jump_velocities: [573.481_75, 603.721_74, 616.681_7, 606.421_75, 606.421_75],
			player_speeds: [251.16008, 311.58009, 387.42014, 468.00014, 576.0002],
			player_width: 30.0,
			player_height: 30.0,
			ship_velocities: [101.541_49, 103.485_5, 103.377_49, 103.809_49, 103.809_49],
			ship_bounds: 300.0,
			dt: 1.0 / 240.0,
			vertical_dt_scale: 1.0,
			vy_quantize_step: 1000.0,
		}
	}
}

pub struct SearchConfig {
	pub heuristic_weight: f32,
	pub x_quant: f32,
	pub y_quant: f32,
	pub vy_quant: f32,
	pub stagnation_check_interval: usize,
	pub min_progress_per_interval: f32,
}

impl Default for SearchConfig {
	fn default() -> Self {
		Self {
			heuristic_weight: 1.8,
			x_quant: 1.0,
			y_quant: 1.0,
			vy_quant: 10.0,
			stagnation_check_interval: 50_000_000,
			min_progress_per_interval: 15.0,
		}
	}
}

#[derive(Default)]
pub struct Config {
	pub physics: PhysicsParams,
	pub search: SearchConfig,
}
