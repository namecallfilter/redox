mod app;
mod renderer;
mod types;

use std::{
	fs,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
		mpsc,
	},
	thread,
};

use app::VisualizerApp;
use macroquad::prelude::*;
use redox_core::{formats::level, game_object::GameObject, pathfinder::Pathfinder};
pub use types::SearchMessage;

type Vec2 = ::glam::Vec2;

pub fn window_conf() -> Conf {
	Conf {
		window_title: "Pathfinder Visualizer".to_string(),
		window_width: 1280,
		window_height: 720,
		..Default::default()
	}
}

pub async fn run_visualizer(level_path: std::path::PathBuf) {
	if !level_path.exists() {
		loop {
			clear_background(BLACK);

			draw_text(
				&format!("Error: {:?} not found", level_path),
				20.0,
				40.0,
				30.0,
				RED,
			);

			draw_text("Press ESC to quit", 20.0, 80.0, 20.0, WHITE);

			if is_key_pressed(KeyCode::Escape) {
				return;
			}

			next_frame().await;
		}
	}

	let content = fs::read_to_string(level_path).expect("Failed to read level file");
	let decompressed = level::parse_level_data(&content).expect("Failed to parse level data");
	let raw_objects = level::parse_objects(&decompressed);
	let game_objects: Vec<GameObject> = raw_objects.iter().map(GameObject::from_raw).collect();

	let mut max_x = 0.0f32;
	for obj in &game_objects {
		max_x = max_x.max(obj.position.x + obj.width * 0.5);
	}

	let goal_x = max_x + 200.0;
	let start_pos = Vec2::new(0.0, 15.0);

	let pf = Pathfinder::new(game_objects.clone(), goal_x);
	let dt = pf.dt();

	let (tx, rx) = mpsc::channel::<SearchMessage>();
	let stop_flag = Arc::new(AtomicBool::new(false));
	let stop_flag_thread = Arc::clone(&stop_flag);

	thread::spawn(move || {
		let mut session = pf.start_search(start_pos, goal_x);
		let mut steps_since_update = 0;
		let update_interval = 1000;

		loop {
			if stop_flag_thread.load(Ordering::Relaxed) {
				let end_node = &session.all_nodes[session.best_x_index];
				let actions = pf.reconstruct_path(&session.all_nodes, end_node);

				let _ = tx.send(SearchMessage::Stopped {
					actions,
					best_x: session.best_x,
					nodes_expanded: session.nodes_expanded,
				});

				break;
			}

			let mut done = false;
			for _ in 0..100 {
				done = pf.step_single(&mut session, goal_x);
				steps_since_update += 1;

				if done {
					break;
				}
			}

			if steps_since_update >= update_interval || done {
				steps_since_update = 0;
				if done {
					if let Some(idx) = session.goal_reached_index {
						let end_node = &session.all_nodes[idx];
						let actions = pf.reconstruct_path(&session.all_nodes, end_node);
						let _ = tx.send(SearchMessage::Done {
							actions,
							best_x: session.best_x,
							nodes_expanded: session.nodes_expanded,
						});
					} else {
						let _ = tx.send(SearchMessage::NoSolution {
							best_x: session.best_x,
							nodes_expanded: session.nodes_expanded,
						});
					}

					break;
				} else {
					let mut best_path: Vec<Vec2> = Vec::new();
					let mut idx = session.best_x_index;
					while idx < session.all_nodes.len() {
						best_path.push(session.all_nodes[idx].state.position);
						if let Some(parent_idx) = session.all_nodes[idx].parent_index {
							idx = parent_idx;
						} else {
							break;
						}
					}
					best_path.reverse();

					let _ = tx.send(SearchMessage::Progress {
						best_path,
						best_x: session.best_x,
						nodes_expanded: session.nodes_expanded,
						open_set_size: session.open_set.len(),
					});
				}
			}
		}
	});

	let mut app = VisualizerApp::new(game_objects, goal_x, start_pos, dt, rx, stop_flag);

	loop {
		if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
			break;
		}

		app.update(get_frame_time());
		app.draw();

		next_frame().await;
	}
}
