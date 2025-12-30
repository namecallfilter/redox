use crate::{
	config::PhysicsParams,
	state::{Action, GameMode, State},
};

pub fn simulate_step(state: &State, action: Action, params: &PhysicsParams) -> State {
	let mut new_state = *state;

	match action {
		Action::Press => new_state.pressing = true,
		Action::Release => new_state.pressing = false,
		Action::None => {} // Keep current state
	}

	let gravity_mult = if state.gravity_flipped { -1.0 } else { 1.0 };
	let effective_gravity = params.gravities[state.speed] * gravity_mult;

	match state.mode {
		GameMode::Cube => {
			if action == Action::Press && new_state.on_ground {
				new_state.vy = params.jump_velocities[state.speed] * gravity_mult;
				new_state.on_ground = false;
			}

			new_state.vy += effective_gravity * params.dt;
			new_state.vy =
				(new_state.vy * params.vy_quantize_step).round() / params.vy_quantize_step;

			new_state.position.y += new_state.vy * params.dt * params.vertical_dt_scale;

			if !new_state.on_ground {
				new_state.rotation -= 360.0 * params.dt * gravity_mult;
			} else {
				new_state.rotation = (new_state.rotation / 90.0).round() * 90.0;
			}
		}
		GameMode::Ship => {
			let threshold = params.ship_velocities[state.speed] * gravity_mult;

			let effective_accel = if new_state.pressing {
				if (gravity_mult > 0.0 && new_state.vy <= threshold)
					|| (gravity_mult < 0.0 && new_state.vy >= threshold)
				{
					1397.0491 * gravity_mult
				} else {
					1_117.643_3 * gravity_mult
				}
			} else if (gravity_mult > 0.0 && new_state.vy >= threshold)
				|| (gravity_mult < 0.0 && new_state.vy <= threshold)
			{
				-1341.1719 * gravity_mult
			} else {
				-894.114_6 * gravity_mult
			};

			new_state.vy += effective_accel * params.dt;
			new_state.vy = new_state.vy.clamp(-800.0, 800.0);
			new_state.vy =
				(new_state.vy * params.vy_quantize_step).round() / params.vy_quantize_step;

			new_state.position.y += new_state.vy * params.dt * params.vertical_dt_scale;

			if new_state.ceiling < f32::MAX / 2.0 {
				let half_height = params.player_height * 0.5;
				let player_top = new_state.position.y + half_height;
				let player_bottom = new_state.position.y - half_height;

				if state.gravity_flipped {
					if player_bottom < new_state.floor {
						if new_state.vy < 0.0 {
							new_state.vy = 0.0;
						}
						new_state.position.y = new_state.floor + half_height;
					}
					if player_top > new_state.ceiling {
						if new_state.vy > 0.0 {
							new_state.vy = 0.0;
						}
						new_state.position.y = new_state.ceiling - half_height;
					}
				} else {
					if player_top > new_state.ceiling {
						if new_state.vy > 0.0 {
							new_state.vy = 0.0;
						}
						new_state.position.y = new_state.ceiling - half_height;
					}
					if player_bottom < new_state.floor {
						if new_state.vy < 0.0 {
							new_state.vy = 0.0;
						}
						new_state.position.y = new_state.floor + half_height;
					}
				}
			}

			let target_rotation = (new_state.vy / 8.0).clamp(-45.0, 45.0) * gravity_mult;
			new_state.rotation = target_rotation;

			// Ship is never "on_ground" in the cube sense
			new_state.on_ground = false;
		}
	}

	new_state.position.x += params.player_speeds[state.speed] * params.dt;

	new_state
}
