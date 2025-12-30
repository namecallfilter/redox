use std::{
	fs,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
		mpsc,
	},
	thread,
};

use macroquad::prelude::*;
use redox_core::{
	game_object::{GameObject, GameObjectType},
	parser,
	pathfinder::Pathfinder,
	state::{Action, GameMode, State},
};

// Use glam from the redox crate to avoid conflict with macroquad's re-export
type Vec2 = ::glam::Vec2;

pub fn window_conf() -> Conf {
	Conf {
		window_title: "Pathfinder Visualizer".to_string(),
		window_width: 1280,
		window_height: 720,
		..Default::default()
	}
}

#[derive(PartialEq)]
enum VisualizerState {
	Computing,
	Playback,
	NoSolution,
}

/// Messages sent from the search thread to the main thread
enum SearchMessage {
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
	let decompressed = parser::parse_level_data(&content).expect("Failed to parse level data");
	let raw_objects = parser::parse_objects(&decompressed);
	let game_objects: Vec<GameObject> = raw_objects.iter().map(GameObject::from_raw).collect();

	// Compute level bounds
	let mut max_x = 0.0f32;
	for obj in &game_objects {
		max_x = max_x.max(obj.position.x + obj.width * 0.5);
	}

	let goal_x = max_x + 200.0;
	let start_pos = Vec2::new(0.0, 15.0);

	// Create pathfinder for the search thread
	let pf = Pathfinder::new(game_objects.clone(), goal_x);
	let dt = pf.dt();

	// Channel for communication between search thread and main thread
	let (tx, rx) = mpsc::channel::<SearchMessage>();

	// Stop flag for early termination
	let stop_flag = Arc::new(AtomicBool::new(false));
	let stop_flag_thread = Arc::clone(&stop_flag);

	// Spawn search thread
	thread::spawn(move || {
		let mut session = pf.start_search(start_pos, goal_x);
		let mut steps_since_update = 0;
		let update_interval = 1000; // Send update every N steps

		loop {
			// Check if we should stop
			if stop_flag_thread.load(Ordering::Relaxed) {
				// Return the current best path
				let end_node = &session.all_nodes[session.best_x_index];
				let actions = pf.reconstruct_path(&session.all_nodes, end_node);
				let _ = tx.send(SearchMessage::Stopped {
					actions,
					best_x: session.best_x,
					nodes_expanded: session.nodes_expanded,
				});
				break;
			}

			// Run a batch of single steps
			let mut done = false;
			for _ in 0..100 {
				done = pf.step_single(&mut session, goal_x);
				steps_since_update += 1;
				if done {
					break;
				}
			}

			// Send progress updates periodically or when done
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
					// Reconstruct current best path for visualization
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

	// Visualization state
	let mut viz_state = VisualizerState::Computing;

	// Camera state
	let mut camera_pos = Vec2::new(start_pos.x, start_pos.y);
	let pixels_per_unit = 3.0;
	let camera_lerp_alpha = 0.08;

	// Search progress state (updated from messages)
	let mut current_best_path: Vec<Vec2> = Vec::new();
	let mut current_best_x = 0.0f32;
	let mut nodes_expanded = 0usize;
	let mut open_set_size = 0usize;

	// Playback state (used after computing is done)
	let mut states: Vec<State> = Vec::new();
	let mut path_points: Vec<Vec2> = Vec::new();
	let mut elapsed = 0.0f32;
	let mut total_time = 0.0f32;
	let mut paused = true;
	let mut speed = 1.0f32;
	let mut final_actions: Option<Vec<(Action, f32)>> = None;

	// Main render loop
	loop {
		let frame_dt = get_frame_time();

		// Screen dimensions
		let screen_w = screen_width();
		let screen_h = screen_height();
		let screen_center_x = screen_w / 2.0;
		let screen_center_y = screen_h / 2.0;

		// Input handling
		if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
			break;
		}

		// Process messages from search thread (non-blocking)
		while let Ok(msg) = rx.try_recv() {
			match msg {
				SearchMessage::Progress {
					best_path,
					best_x,
					nodes_expanded: ne,
					open_set_size: oss,
				} => {
					current_best_path = best_path;
					current_best_x = best_x;
					nodes_expanded = ne;
					open_set_size = oss;
				}
				SearchMessage::Done {
					actions,
					best_x,
					nodes_expanded: ne,
				} => {
					current_best_x = best_x;
					nodes_expanded = ne;
					final_actions = Some(actions);
					viz_state = VisualizerState::Playback;
				}
				SearchMessage::Stopped {
					actions,
					best_x,
					nodes_expanded: ne,
				} => {
					current_best_x = best_x;
					nodes_expanded = ne;
					final_actions = Some(actions);
					viz_state = VisualizerState::Playback;
				}
				SearchMessage::NoSolution {
					best_x,
					nodes_expanded: ne,
				} => {
					current_best_x = best_x;
					nodes_expanded = ne;
					viz_state = VisualizerState::NoSolution;
				}
			}
		}

		// Setup playback if we just got the final actions
		if viz_state == VisualizerState::Playback
			&& states.is_empty()
			&& let Some(ref actions) = final_actions
		{
			// Create a new pathfinder for simulation (we need it on main thread)
			let pf_playback = Pathfinder::new(game_objects.clone(), goal_x);

			states = Vec::with_capacity(actions.len() + 1);
			let mut curr_state = State {
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
			states.push(curr_state);

			// Expand merged actions back into per-frame states
			// Each action has a duration - we need to simulate that many frames
			let dt = pf_playback.dt();
			for (action, duration) in actions {
				// Apply the action (Press/Release changes state, None keeps it)
				curr_state = pf_playback.simulate_step(&curr_state, *action);
				states.push(curr_state);

				// For the remaining duration, simulate with Action::None
				// (the pressing state is already set, so physics continues correctly)
				let remaining_frames = ((duration / dt).round() as usize).saturating_sub(1);
				for _ in 0..remaining_frames {
					curr_state = pf_playback.simulate_step(&curr_state, Action::None);
					states.push(curr_state);
				}
			}

			total_time = (states.len() as f32 - 1.0) * dt;

			// Precompute path line points (downsampled for performance)
			let sample_step = 4.max(states.len() / 2000);
			path_points = states
				.iter()
				.step_by(sample_step)
				.map(|s| s.position)
				.collect();

			// Reset camera for playback
			camera_pos = start_pos;
			elapsed = 0.0;
			paused = true;
		}

		match viz_state {
			VisualizerState::Computing => {
				// Follow the search frontier
				if let Some(last_pos) = current_best_path.last() {
					let target_camera = *last_pos;
					camera_pos = camera_pos + (target_camera - camera_pos) * camera_lerp_alpha;
				}

				// World to screen conversion
				let world_to_screen = |world: Vec2| -> (f32, f32) {
					let dx = (world.x - camera_pos.x) * pixels_per_unit + screen_center_x;
					let dy = screen_center_y - (world.y - camera_pos.y) * pixels_per_unit;
					(dx, dy)
				};

				// Clear background
				clear_background(Color::from_rgba(30, 30, 40, 255));

				// Cull distance for objects
				let cull_distance = screen_w / pixels_per_unit * 0.7;

				// Draw game objects
				for obj in &game_objects {
					if (obj.position.x - camera_pos.x).abs() > cull_distance {
						continue;
					}

					let color = match obj.object_type {
						GameObjectType::Solid => Color::from_rgba(40, 70, 120, 255),
						GameObjectType::Hazard => Color::from_rgba(150, 40, 40, 255),
						GameObjectType::Unknown => continue,
						_ => Color::from_rgba(100, 100, 100, 255),
					};

					let world_top_left = Vec2::new(
						obj.position.x - obj.width * 0.5,
						obj.position.y + obj.height * 0.5,
					);
					let (sx, sy) = world_to_screen(world_top_left);
					let rect_w = obj.width * pixels_per_unit;
					let rect_h = obj.height * pixels_per_unit;

					draw_rectangle(sx, sy, rect_w, rect_h, color);
				}

				// Draw goal line
				let (goal_screen_x, _) = world_to_screen(Vec2::new(goal_x, 0.0));
				draw_line(
					goal_screen_x,
					0.0,
					goal_screen_x,
					screen_h,
					3.0,
					Color::from_rgba(180, 50, 200, 255),
				);

				// Draw current best path as a bright line
				if current_best_path.len() > 1 {
					for i in 0..current_best_path.len() - 1 {
						let (p1x, p1y) = world_to_screen(current_best_path[i]);
						let (p2x, p2y) = world_to_screen(current_best_path[i + 1]);
						draw_line(p1x, p1y, p2x, p2y, 2.0, Color::from_rgba(50, 255, 50, 220));
					}
				}

				// Draw marker at current best position
				if let Some(best_pos) = current_best_path.last() {
					let (bx, by) = world_to_screen(*best_pos);
					draw_circle(bx, by, 6.0, Color::from_rgba(255, 200, 50, 255));
				}

				// Draw HUD
				let hud_bg = Color::from_rgba(0, 0, 0, 200);
				draw_rectangle(0.0, 0.0, screen_w, 80.0, hud_bg);

				draw_text(
					&format!(
						"COMPUTING PATH... | Nodes: {} | Best X: {:.1} / {:.1}",
						nodes_expanded, current_best_x, goal_x
					),
					15.0,
					25.0,
					22.0,
					Color::from_rgba(50, 255, 50, 255),
				);

				let progress = (current_best_x / goal_x).clamp(0.0, 1.0);
				draw_text(
					&format!(
						"Progress: {:.1}% | Open set: {}",
						progress * 100.0,
						open_set_size
					),
					15.0,
					48.0,
					18.0,
					Color::from_rgba(180, 180, 180, 255),
				);

				draw_text(
					"Q: Quit | S: Stop & Play",
					15.0,
					68.0,
					16.0,
					Color::from_rgba(150, 150, 150, 255),
				);

				// Draw Stop button
				let button_w = 120.0;
				let button_h = 35.0;
				let button_x = screen_w - button_w - 15.0;
				let button_y = 22.0;

				let mouse_pos = mouse_position();
				let mouse_over_button = mouse_pos.0 >= button_x
					&& mouse_pos.0 <= button_x + button_w
					&& mouse_pos.1 >= button_y
					&& mouse_pos.1 <= button_y + button_h;

				let button_color = if mouse_over_button {
					Color::from_rgba(220, 80, 80, 255)
				} else {
					Color::from_rgba(180, 50, 50, 255)
				};

				draw_rectangle(button_x, button_y, button_w, button_h, button_color);
				draw_rectangle_lines(
					button_x,
					button_y,
					button_w,
					button_h,
					2.0,
					Color::from_rgba(255, 100, 100, 255),
				);
				draw_text("STOP", button_x + 35.0, button_y + 24.0, 22.0, WHITE);

				// Handle Stop button click or S key
				if (mouse_over_button && is_mouse_button_pressed(MouseButton::Left))
					|| is_key_pressed(KeyCode::S)
				{
					stop_flag.store(true, Ordering::Relaxed);
				}

				// Progress bar
				let bar_y = 75.0;
				let bar_height = 4.0;
				draw_rectangle(
					0.0,
					bar_y,
					screen_w,
					bar_height,
					Color::from_rgba(60, 60, 60, 255),
				);
				draw_rectangle(
					0.0,
					bar_y,
					screen_w * progress,
					bar_height,
					Color::from_rgba(50, 200, 50, 255),
				);
			}

			VisualizerState::Playback => {
				// Playback input handling
				if is_key_pressed(KeyCode::Space) {
					paused = !paused;
				}
				if is_key_pressed(KeyCode::Key1) {
					speed = 0.5;
				}
				if is_key_pressed(KeyCode::Key2) {
					speed = 1.0;
				}
				if is_key_pressed(KeyCode::Key3) {
					speed = 2.0;
				}
				if is_key_pressed(KeyCode::Key4) {
					speed = 4.0;
				}
				if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Equal) {
					speed = (speed * 2.0).min(8.0);
				}
				if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Minus) {
					speed = (speed / 2.0).max(0.25);
				}
				if is_key_pressed(KeyCode::R) {
					elapsed = 0.0;
					paused = true;
					camera_pos = start_pos;
				}

				// Update playback time
				if !paused {
					elapsed += frame_dt * speed;
					if elapsed >= total_time {
						elapsed = total_time;
						paused = true;
					}
				}

				// Compute interpolated player state
				let idx_f = (elapsed / dt).clamp(0.0, (states.len() - 1) as f32);
				let idx = idx_f.floor() as usize;
				let t = idx_f - (idx as f32);
				let s0 = &states[idx];
				let s1 = if idx + 1 < states.len() {
					&states[idx + 1]
				} else {
					&states[idx]
				};
				let player_pos = s0.position * (1.0 - t) + s1.position * t;

				// Smooth camera follow
				let target_camera = player_pos;
				camera_pos = camera_pos + (target_camera - camera_pos) * camera_lerp_alpha;

				// World to screen conversion
				let world_to_screen = |world: Vec2| -> (f32, f32) {
					let dx = (world.x - camera_pos.x) * pixels_per_unit + screen_center_x;
					let dy = screen_center_y - (world.y - camera_pos.y) * pixels_per_unit;
					(dx, dy)
				};

				// Clear background
				clear_background(Color::from_rgba(240, 240, 240, 255));

				// Cull distance for objects
				let cull_distance = screen_w / pixels_per_unit * 0.7;

				// Draw game objects
				for obj in &game_objects {
					if (obj.position.x - camera_pos.x).abs() > cull_distance {
						continue;
					}

					let color = match obj.object_type {
						GameObjectType::Solid => Color::from_rgba(60, 100, 180, 255),
						GameObjectType::Hazard => Color::from_rgba(220, 60, 60, 255),
						GameObjectType::Unknown => continue,
						_ => Color::from_rgba(150, 150, 150, 255),
					};

					let world_top_left = Vec2::new(
						obj.position.x - obj.width * 0.5,
						obj.position.y + obj.height * 0.5,
					);
					let (sx, sy) = world_to_screen(world_top_left);
					let rect_w = obj.width * pixels_per_unit;
					let rect_h = obj.height * pixels_per_unit;

					draw_rectangle(sx, sy, rect_w, rect_h, color);
				}

				// Draw goal line
				let (goal_screen_x, _) = world_to_screen(Vec2::new(goal_x, 0.0));
				draw_line(
					goal_screen_x,
					0.0,
					goal_screen_x,
					screen_h,
					3.0,
					Color::from_rgba(180, 50, 200, 255),
				);

				// Draw path trace (thin green line)
				if path_points.len() > 1 {
					for i in 0..path_points.len() - 1 {
						let (p1x, p1y) = world_to_screen(path_points[i]);
						let (p2x, p2y) = world_to_screen(path_points[i + 1]);
						draw_line(p1x, p1y, p2x, p2y, 1.5, Color::from_rgba(50, 200, 50, 180));
					}
				}

				// Draw player (orange square)
				let (player_sx, player_sy) = world_to_screen(player_pos);
				let player_size = 30.0 * pixels_per_unit;
				draw_rectangle(
					player_sx - player_size * 0.5,
					player_sy - player_size * 0.5,
					player_size,
					player_size,
					Color::from_rgba(255, 140, 0, 255),
				);
				// Player outline
				draw_rectangle_lines(
					player_sx - player_size * 0.5,
					player_sy - player_size * 0.5,
					player_size,
					player_size,
					2.0,
					Color::from_rgba(200, 100, 0, 255),
				);

				// Get current action for display
				let current_action = if let Some(ref actions) = final_actions {
					if idx < actions.len() {
						Some(actions[idx].0)
					} else {
						None
					}
				} else {
					None
				};

				// Draw input indicator next to player (show when pressing)
				let is_input_active = s0.pressing;
				if is_input_active {
					// Draw a bright indicator when input is active
					let indicator_x = player_sx + player_size * 0.5 + 10.0;
					let indicator_y = player_sy;
					draw_circle(
						indicator_x,
						indicator_y,
						8.0,
						Color::from_rgba(50, 255, 50, 255),
					);
					draw_circle_lines(
						indicator_x,
						indicator_y,
						8.0,
						2.0,
						Color::from_rgba(30, 200, 30, 255),
					);
				}

				// Draw HUD
				let hud_bg = Color::from_rgba(0, 0, 0, 180);
				draw_rectangle(0.0, 0.0, screen_w, 80.0, hud_bg);

				let status = if paused { "PAUSED" } else { "PLAYING" };
				let action_str = match current_action {
					Some(Action::Press) => "PRESS".to_string(),
					Some(Action::Release) => "RELEASE".to_string(),
					Some(Action::None) => "---".to_string(),
					None => "---".to_string(),
				};
				let mode_str = match s0.mode {
					GameMode::Cube => "CUBE",
					GameMode::Ship => "SHIP",
				};
				draw_text(
					&format!(
						"{} | Speed: {:.2}x | Time: {:.2}s / {:.2}s",
						status, speed, elapsed, total_time
					),
					15.0,
					25.0,
					22.0,
					WHITE,
				);
				draw_text(
					&format!(
						"Mode: {} | Input: {} | Frame: {} / {}",
						mode_str,
						action_str,
						idx,
						states.len() - 1
					),
					15.0,
					48.0,
					18.0,
					Color::from_rgba(180, 180, 180, 255),
				);
				draw_text(
					"Space: Play/Pause | 1-4: Speed | R: Reset | Q: Quit",
					15.0,
					68.0,
					16.0,
					Color::from_rgba(150, 150, 150, 255),
				);

				// Progress bar
				let bar_y = 76.0;
				let bar_height = 4.0;
				let progress = elapsed / total_time;
				draw_rectangle(
					0.0,
					bar_y,
					screen_w,
					bar_height,
					Color::from_rgba(60, 60, 60, 255),
				);
				draw_rectangle(
					0.0,
					bar_y,
					screen_w * progress,
					bar_height,
					Color::from_rgba(50, 200, 50, 255),
				);
			}

			VisualizerState::NoSolution => {
				clear_background(BLACK);
				draw_text("No solution found!", 20.0, 40.0, 30.0, RED);
				draw_text(
					&format!(
						"Explored {} nodes, best X: {:.1}",
						nodes_expanded, current_best_x
					),
					20.0,
					80.0,
					20.0,
					WHITE,
				);
				draw_text("Press ESC to quit", 20.0, 120.0, 20.0, WHITE);
			}
		}

		next_frame().await;
	}
}
