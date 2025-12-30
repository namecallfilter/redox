use redox_core::state::Action;

pub type Vec2 = ::glam::Vec2;

#[derive(PartialEq, Clone, Copy)]
pub enum VisualizerState {
	Computing,
	Playback,
	NoSolution,
}

/// Messages sent from the search thread to the main thread
pub enum SearchMessage {
	/// Progress update with current best path and stats
	Progress {
		best_path: Vec<Vec2>,
		best_x: f32,
		nodes_expanded: usize,
		open_set_size: usize,
	},
	/// Search completed successfully
	Done {
		actions: Vec<(Action, f32)>,
		best_x: f32,
		nodes_expanded: usize,
	},
	/// Search stopped by user - returns partial path
	Stopped {
		actions: Vec<(Action, f32)>,
		best_x: f32,
		nodes_expanded: usize,
	},
	/// No solution found
	NoSolution { best_x: f32, nodes_expanded: usize },
}
