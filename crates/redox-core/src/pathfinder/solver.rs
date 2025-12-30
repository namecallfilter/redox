use glam::Vec2;
use tracing::info;

use super::{
	Pathfinder,
	search::{self, NodeIndexWrapper, SearchSession},
};
use crate::{
	simulation::collision,
	state::{Action, GameMode, Node, State, StateKey},
};

impl Pathfinder {
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
