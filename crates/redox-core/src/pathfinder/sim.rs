use super::Pathfinder;
use crate::{
	game_object::{GameObjectType, OBB2D},
	simulation::physics,
	state::{Action, GameMode, State},
};

impl Pathfinder {
	pub fn apply_landing_logic(&self, prev_state: &State, mut next_state: State) -> State {
		next_state.on_ground = false;

		let prev_player_bottom = if prev_state.gravity_flipped {
			prev_state.position.y + self.config.physics.player_height * 0.5
		} else {
			prev_state.position.y - self.config.physics.player_height * 0.5
		};

		let player_bottom = if next_state.gravity_flipped {
			next_state.position.y + self.config.physics.player_height * 0.5
		} else {
			next_state.position.y - self.config.physics.player_height * 0.5
		};

		let mut landed = false;

		let new_min_x = next_state.position.x - self.config.physics.player_width * 0.5;
		let new_max_x = next_state.position.x + self.config.physics.player_width * 0.5;
		let search_start_x = new_min_x - self.max_obj_width;

		let start_idx = self.objects.partition_point(|obj| {
			let obj_min_x = obj.position.x - obj.width * 0.5;
			obj_min_x < search_start_x
		});

		for obj in &self.objects[start_idx..] {
			let obj_min_x = obj.position.x - obj.width * 0.5;
			if obj_min_x > new_max_x {
				break;
			}

			if obj.object_type != GameObjectType::Solid {
				continue;
			}

			if (obj.position.x - next_state.position.x).abs() > 200.0 {
				continue;
			}

			if let Some(obj_obb) = &obj.obb {
				let player_obb = OBB2D::new(
					next_state.position,
					self.config.physics.player_width,
					self.config.physics.player_height,
					0.0,
				);

				if player_obb.overlaps(obj_obb) {
					let obj_top = obj.position.y + obj.height * 0.5;
					let obj_bottom = obj.position.y - obj.height * 0.5;

					let obj_left = obj.position.x - obj.width * 0.5;
					let obj_right = obj.position.x + obj.width * 0.5;

					let player_left =
						next_state.position.x - self.config.physics.player_width * 0.5;
					let player_right =
						next_state.position.x + self.config.physics.player_width * 0.5;

					let h_overlap =
						(player_right.min(obj_right) - player_left.max(obj_left)).max(0.0);
					let min_width = self.config.physics.player_width.min(obj.width);
					let sufficient_h_overlap = h_overlap >= min_width * 0.5;

					let (coming_from_correct_side, falling_towards_surface, landing_on_surface) =
						if next_state.gravity_flipped {
							(
								prev_player_bottom <= obj_bottom + 2.0,
								next_state.vy >= 0.0,
								player_bottom >= obj_bottom - 5.0 && player_bottom <= obj_top,
							)
						} else {
							(
								prev_player_bottom >= obj_top - 2.0,
								next_state.vy <= 0.0,
								player_bottom <= obj_top + 5.0 && player_bottom >= obj_bottom,
							)
						};

					if coming_from_correct_side
						&& falling_towards_surface
						&& landing_on_surface
						&& sufficient_h_overlap
					{
						if next_state.gravity_flipped {
							next_state.position.y =
								obj_bottom - self.config.physics.player_height * 0.5 - 0.001;
						} else {
							next_state.position.y =
								obj_top + self.config.physics.player_height * 0.5 + 0.001;
						}

						next_state.vy = 0.0;
						next_state.on_ground = true;
						next_state.rotation = 0.0;
						landed = true;
						break;
					}
				}
			}
		}

		if !landed
			&& !next_state.gravity_flipped
			&& next_state.position.y < self.config.physics.player_height * 0.5
		{
			next_state.position.y = self.config.physics.player_height * 0.5;
			next_state.vy = 0.0;
			next_state.on_ground = true;
			next_state.rotation = 0.0;
		}

		next_state
	}

	pub fn check_portal_collisions(&self, mut state: State) -> State {
		let player_obb = OBB2D::new(
			state.position,
			self.config.physics.player_width,
			self.config.physics.player_height,
			0.0,
		);

		let player_min_x = state.position.x - self.config.physics.player_width * 0.5;
		let player_max_x = state.position.x + self.config.physics.player_width * 0.5;
		let search_start_x = player_min_x - self.max_obj_width;

		let start_idx = self.objects.partition_point(|obj| {
			let obj_min_x = obj.position.x - obj.width * 0.5;
			obj_min_x < search_start_x
		});

		for obj in &self.objects[start_idx..] {
			let obj_min_x = obj.position.x - obj.width * 0.5;
			if obj_min_x > player_max_x {
				break;
			}
			if let Some(obj_obb) = &obj.obb
				&& player_obb.overlaps(obj_obb)
			{
				match obj.object_type {
					GameObjectType::ShipPortal => {
						state.mode = GameMode::Ship;
						state.on_ground = false;

						let portal_y = obj.position.y;

						let half_bounds = self.config.physics.ship_bounds / 2.0;
						state.floor =
							(30.0 * ((portal_y - (half_bounds + 30.0)) / 30.0).ceil()).max(0.0);
						state.ceiling = state.floor + self.config.physics.ship_bounds;
					}
					GameObjectType::CubePortal => {
						state.mode = GameMode::Cube;
						state.floor = 0.0;
						state.ceiling = f32::MAX;
					}
					GameObjectType::InverseGravityPortal => {
						if !state.gravity_flipped {
							state.gravity_flipped = true;
						}
					}
					GameObjectType::NormalGravityPortal => {
						if state.gravity_flipped {
							state.gravity_flipped = false;
						}
					}
					_ => {}
				}
			}
		}
		state
	}

	pub fn simulate_step(&self, state: &State, action: Action) -> State {
		let mut next_state = physics::simulate_step(state, action, &self.config.physics);

		// Landing logic (special case for Cube mode)
		if next_state.mode == GameMode::Cube {
			next_state = self.apply_landing_logic(state, next_state);
		}

		next_state = self.check_portal_collisions(next_state);

		next_state
	}
}
