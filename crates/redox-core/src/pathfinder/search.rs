use std::collections::{BinaryHeap, HashMap};

use crate::state::{Node, State, StateKey};

pub struct SearchSession {
	pub open_set: BinaryHeap<NodeIndexWrapper>,
	pub closed_set: HashMap<StateKey, f32>,
	pub all_nodes: Vec<Node>,
	pub nodes_expanded: usize,
	pub goal_reached_index: Option<usize>,
	pub best_x_index: usize,
	pub best_x: f32,
	pub checkpoint_best_x: f32,
	pub checkpoint_nodes: usize,
}

impl SearchSession {
	pub fn new(start_node: Node, start_pos_x: f32) -> Self {
		let all_nodes = vec![start_node];

		let mut open_set = BinaryHeap::new();
		open_set.push(NodeIndexWrapper {
			f: start_node.f,
			index: 0,
			x: start_node.state.position.x,
		});

		Self {
			open_set,
			closed_set: HashMap::new(),
			all_nodes,
			nodes_expanded: 0,
			goal_reached_index: None,
			best_x_index: 0,
			best_x: start_pos_x,
			checkpoint_best_x: start_pos_x,
			checkpoint_nodes: 0,
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct NodeIndexWrapper {
	pub f: f32,
	pub index: usize,
	pub x: f32,
}

impl Eq for NodeIndexWrapper {}

impl PartialOrd for NodeIndexWrapper {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for NodeIndexWrapper {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match other
			.f
			.partial_cmp(&self.f)
			.unwrap_or(std::cmp::Ordering::Equal)
		{
			std::cmp::Ordering::Equal => {
				match self
					.x
					.partial_cmp(&other.x)
					.unwrap_or(std::cmp::Ordering::Equal)
				{
					std::cmp::Ordering::Equal => self.index.cmp(&other.index),
					ord => ord,
				}
			}
			ord => ord,
		}
	}
}

pub fn heuristic(
	state: &State, goal_x: f32, player_speeds: &[f32; 5], heuristic_weight: f32,
) -> f32 {
	let dist = (goal_x - state.position.x).max(0.0);
	let time_to_goal = dist / player_speeds[state.speed];

	let mut penalty = 0.0;
	if state.mode == crate::state::GameMode::Ship && state.ceiling < f32::MAX / 2.0 {
		let center_y = (state.floor + state.ceiling) / 2.0;
		let vertical_offset = (state.position.y - center_y).abs();
		penalty = (vertical_offset / 150.0) * 0.5;
	}

	(time_to_goal + penalty) * heuristic_weight
}
