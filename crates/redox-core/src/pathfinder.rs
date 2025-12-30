use glam::Vec2;
use tracing::info;

use crate::{
	collision,
	config::Config,
	game_object::{GameObject, GameObjectType, OBB2D},
	physics,
	search::{self, NodeIndexWrapper, SearchSession},
	spatial_grid::SpatialGrid,
	state::{Action, GameMode, Node, State, StateKey},
};

pub struct Pathfinder {
	objects: Vec<GameObject>,
	config: Config,
	max_obj_width: f32,
	grid: SpatialGrid,
}

impl Pathfinder {
	pub fn new(mut objects: Vec<GameObject>, _goal_x: f32) -> Self {
		let config = Config::default();

		// objects.retain(|obj| obj.object_type != GameObjectType::Unknown);

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
		// objects.retain(|obj| obj.object_type != GameObjectType::Unknown);

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

	pub fn start_search(&self, start_pos: Vec2, goal_x: f32) -> SearchSession {
		let start_state = State {
			position: start_pos,
			vy: 0.0,
			on_ground: true,
			rotation: 0.0,
			mode: GameMode::Cube,
			gravity_flipped: false,
			floor: 0.0,
			ceiling: f32::MAX,
			pressing: false,
			speed: 1,
		};

		let start_node = Node {
			g: 0.0,
			f: search::heuristic(
				&start_state,
				goal_x,
				&self.config.physics.player_speeds,
				self.config.search.heuristic_weight,
			),
			state: start_state,
			parent_index: None,
			action: None,
		};

		SearchSession::new(start_node, start_pos.x)
	}

	pub fn step_single(&self, session: &mut SearchSession, goal_x: f32) -> bool {
		if session.goal_reached_index.is_some() || session.open_set.is_empty() {
			return true;
		}

		if session.nodes_expanded
			>= session.checkpoint_nodes + self.config.search.stagnation_check_interval
		{
			let progress = session.best_x - session.checkpoint_best_x;
			if progress < self.config.search.min_progress_per_interval {
				info!(
					"Stagnation detected: only {:.2} units progress in {} nodes. Stopping at x={:.2}",
					progress, self.config.search.stagnation_check_interval, session.best_x
				);

				session.goal_reached_index = Some(session.best_x_index);
				return true;
			}

			session.checkpoint_best_x = session.best_x;
			session.checkpoint_nodes = session.nodes_expanded;
		}

		if let Some(wrapper) = session.open_set.pop() {
			let current_idx = wrapper.index;
			let current_node = session.all_nodes[current_idx];

			session.nodes_expanded += 1;

			if current_node.state.position.x > session.best_x {
				session.best_x = current_node.state.position.x;
				session.best_x_index = current_idx;
			}

			if current_node.state.position.x >= goal_x {
				info!("Goal reached after {} nodes!", session.nodes_expanded);

				session.goal_reached_index = Some(current_idx);
				return true;
			}

			let key = StateKey::from_state(
				&current_node.state,
				self.config.search.x_quant,
				self.config.search.y_quant,
				self.config.search.vy_quant,
			);

			if let Some(&best_g) = session.closed_set.get(&key)
				&& current_node.g > best_g + (self.config.physics.dt * 0.5)
			{
				return false;
			}

			session.closed_set.insert(key, current_node.g);

			let mut actions_to_try: [Action; 2] = [Action::None, Action::None];
			let mut action_count = 1;
			// actions_to_try[0] is already Action::None

			if current_node.state.pressing {
				actions_to_try[1] = Action::Release;
				action_count = 2;
			} else {
				match current_node.state.mode {
					GameMode::Cube => {
						if current_node.state.on_ground {
							actions_to_try[1] = Action::Press;
							action_count = 2;
						}
					}
					GameMode::Ship => {
						actions_to_try[1] = Action::Press;
						action_count = 2;
					}
				}
			}

			for action in actions_to_try.iter().take(action_count) {
				let action = *action;
				let next_state = self.simulate_step(&current_node.state, action);

				if next_state.position.y < -100.0 {
					continue;
				}

				if collision::collides_info(
					&next_state,
					&self.objects,
					&self.grid,
					&self.config.physics,
				)
				.is_some()
				{
					continue;
				}

				let mut new_g = current_node.g + self.config.physics.dt;
				if action == Action::Press {
					if current_node.state.mode == GameMode::Cube {
						new_g += 15.0 * self.config.physics.dt;
					} else {
						new_g += 0.5 * self.config.physics.dt;
					}
				}

				let new_f = new_g
					+ search::heuristic(
						&next_state,
						goal_x,
						&self.config.physics.player_speeds,
						self.config.search.heuristic_weight,
					);

				let next_node = Node {
					g: new_g,
					f: new_f,
					state: next_state,
					parent_index: Some(current_idx),
					action: Some(action),
				};

				let next_idx = session.all_nodes.len();

				session.all_nodes.push(next_node);
				session.open_set.push(NodeIndexWrapper {
					f: new_f,
					index: next_idx,
					x: next_state.position.x,
				});
			}
		}

		false
	}

	pub fn step(&self, session: &mut SearchSession, goal_x: f32) -> bool {
		while !session.open_set.is_empty() {
			if self.step_single(session, goal_x) {
				return true;
			}
		}

		false
	}

	fn apply_landing_logic(&self, prev_state: &State, mut next_state: State) -> State {
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

	fn check_portal_collisions(&self, mut state: State) -> State {
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

		// Portal collisions
		next_state = self.check_portal_collisions(next_state);

		next_state
	}

	pub fn dt(&self) -> f32 {
		self.config.physics.dt
	}

	pub fn reconstruct_path(&self, nodes: &[Node], end_node: &Node) -> Vec<(Action, f32)> {
		let mut path = Vec::new();
		let mut current = end_node;
		while let Some(parent_idx) = current.parent_index {
			if let Some(action) = current.action {
				path.push((action, self.config.physics.dt));
			}
			current = &nodes[parent_idx];
		}
		path.reverse();

		let mut merged: Vec<(Action, f32)> = Vec::new();
		for (action, dur) in path {
			match (&action, merged.last_mut()) {
				(Action::None, Some((Action::None, existing_dur))) => {
					*existing_dur += dur;
				}
				_ => {
					merged.push((action, dur));
				}
			}
		}

		merged
	}
}
