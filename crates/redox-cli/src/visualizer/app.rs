use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
	mpsc,
};

use macroquad::prelude::*;
use redox_core::{
	game_object::GameObject,
	pathfinder::Pathfinder,
	state::{Action, GameMode, State},
};

use super::{
	renderer::Renderer,
	types::{SearchMessage, Vec2, VisualizerState},
};

pub struct VisualizerApp {
	pub game_objects: Vec<GameObject>,
	pub goal_x: f32,
	pub start_pos: Vec2,
	pub dt: f32,

	pub viz_state: VisualizerState,
	pub renderer: Renderer,
	pub camera_lerp_alpha: f32,

	pub current_best_path: Vec<Vec2>,
	pub current_best_x: f32,
	pub nodes_expanded: usize,
	pub open_set_size: usize,

	pub states: Vec<State>,
	pub path_points: Vec<Vec2>,
	pub elapsed: f32,
	pub total_time: f32,
	pub paused: bool,
	pub speed: f32,
	pub final_actions: Option<Vec<(Action, f32)>>,

	pub rx: mpsc::Receiver<SearchMessage>,
	pub stop_flag: Arc<AtomicBool>,
}

impl VisualizerApp {
	pub fn new(
		game_objects: Vec<GameObject>, goal_x: f32, start_pos: Vec2, dt: f32,
		rx: mpsc::Receiver<SearchMessage>, stop_flag: Arc<AtomicBool>,
	) -> Self {
		Self {
			game_objects,
			goal_x,
			start_pos,
			dt,
			viz_state: VisualizerState::Computing,
			renderer: Renderer::new(start_pos),
			camera_lerp_alpha: 0.08,
			current_best_path: Vec::new(),
			current_best_x: 0.0,
			nodes_expanded: 0,
			open_set_size: 0,
			states: Vec::new(),
			path_points: Vec::new(),
			elapsed: 0.0,
			total_time: 0.0,
			paused: true,
			speed: 1.0,
			final_actions: None,
			rx,
			stop_flag,
		}
	}

	pub fn update(&mut self, frame_dt: f32) {
		while let Ok(msg) = self.rx.try_recv() {
			match msg {
				SearchMessage::Progress {
					best_path,
					best_x,
					nodes_expanded,
					open_set_size,
				} => {
					self.current_best_path = best_path;
					self.current_best_x = best_x;
					self.nodes_expanded = nodes_expanded;
					self.open_set_size = open_set_size;
				}
				SearchMessage::Done {
					actions,
					best_x,
					nodes_expanded,
				} => {
					self.current_best_x = best_x;
					self.nodes_expanded = nodes_expanded;
					self.final_actions = Some(actions);
					self.viz_state = VisualizerState::Playback;
				}
				SearchMessage::Stopped {
					actions,
					best_x,
					nodes_expanded,
				} => {
					self.current_best_x = best_x;
					self.nodes_expanded = nodes_expanded;
					self.final_actions = Some(actions);
					self.viz_state = VisualizerState::Playback;
				}
				SearchMessage::NoSolution {
					best_x,
					nodes_expanded,
				} => {
					self.current_best_x = best_x;
					self.nodes_expanded = nodes_expanded;
					self.viz_state = VisualizerState::NoSolution;
				}
			}
		}

		if is_key_pressed(KeyCode::S) && self.viz_state == VisualizerState::Computing {
			self.stop_flag.store(true, Ordering::Relaxed);
		}

		match self.viz_state {
			VisualizerState::Computing => {
				if let Some(last_pos) = self.current_best_path.last() {
					let target_camera = *last_pos;
					self.renderer.camera_pos +=
						(target_camera - self.renderer.camera_pos) * self.camera_lerp_alpha;
				}
			}
			VisualizerState::Playback => {
				if self.states.is_empty()
					&& let Some(actions) = self.final_actions.clone()
				{
					self.setup_playback(&actions);
				}

				if is_key_pressed(KeyCode::Space) {
					self.paused = !self.paused;
				}

				if is_key_pressed(KeyCode::Key1) {
					self.speed = 0.5;
				}

				if is_key_pressed(KeyCode::Key2) {
					self.speed = 1.0;
				}

				if is_key_pressed(KeyCode::Key3) {
					self.speed = 2.0;
				}

				if is_key_pressed(KeyCode::Key4) {
					self.speed = 4.0;
				}

				if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Equal) {
					self.speed = (self.speed * 2.0).min(8.0);
				}

				if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Minus) {
					self.speed = (self.speed / 2.0).max(0.25);
				}

				if is_key_pressed(KeyCode::R) {
					self.elapsed = 0.0;
					self.paused = true;
					self.renderer.camera_pos = self.start_pos;
				}

				if !self.paused {
					self.elapsed += frame_dt * self.speed;

					if self.elapsed >= self.total_time {
						self.elapsed = self.total_time;
						self.paused = true;
					}
				}

				let player_pos = self.get_player_pos();
				self.renderer.camera_pos +=
					(player_pos - self.renderer.camera_pos) * self.camera_lerp_alpha;
			}
			VisualizerState::NoSolution => {}
		}
	}

	fn setup_playback(&mut self, actions: &[(Action, f32)]) {
		let pf_playback = Pathfinder::new(self.game_objects.clone(), self.goal_x);

		self.states = Vec::with_capacity(actions.len() + 1);

		let mut curr_state = State {
			position: self.start_pos,
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
		self.states.push(curr_state);

		let dt = self.dt;
		for (action, duration) in actions {
			curr_state = pf_playback.simulate_step(&curr_state, *action);
			self.states.push(curr_state);

			let remaining_frames = ((duration / dt).round() as usize).saturating_sub(1);
			for _ in 0..remaining_frames {
				curr_state = pf_playback.simulate_step(&curr_state, Action::None);
				self.states.push(curr_state);
			}
		}

		self.total_time = (self.states.len() as f32 - 1.0) * dt;

		let sample_step = 4.max(self.states.len() / 2000);
		self.path_points = self
			.states
			.iter()
			.step_by(sample_step)
			.map(|s| s.position)
			.collect();

		self.renderer.camera_pos = self.start_pos;
		self.elapsed = 0.0;
		self.paused = true;
	}

	pub fn get_player_pos(&self) -> Vec2 {
		if self.states.is_empty() {
			return self.start_pos;
		}

		let idx_f = (self.elapsed / self.dt).clamp(0.0, (self.states.len() - 1) as f32);
		let idx = idx_f.floor() as usize;
		let t = idx_f - (idx as f32);
		let s0 = &self.states[idx];

		let s1 = if idx + 1 < self.states.len() {
			&self.states[idx + 1]
		} else {
			s0
		};

		s0.position * (1.0 - t) + s1.position * t
	}

	pub fn get_current_state(&self) -> &State {
		let idx_f = (self.elapsed / self.dt).clamp(0.0, (self.states.len() - 1) as f32);
		&self.states[idx_f.floor() as usize]
	}

	pub fn draw(&self) {
		match self.viz_state {
			VisualizerState::Computing => {
				clear_background(Color::from_rgba(30, 30, 40, 255));

				self.renderer
					.draw_game_objects(&self.game_objects, self.viz_state);
				self.renderer.draw_goal_line(self.goal_x);
				self.renderer
					.draw_path(&self.current_best_path, Color::from_rgba(50, 255, 50, 220));

				if let Some(best_pos) = self.current_best_path.last() {
					let (bx, by) = self.renderer.world_to_screen(*best_pos);
					draw_circle(bx, by, 6.0, Color::from_rgba(255, 200, 50, 255));
				}

				self.draw_hud_computing();
			}
			VisualizerState::Playback => {
				clear_background(Color::from_rgba(240, 240, 240, 255));

				self.renderer
					.draw_game_objects(&self.game_objects, self.viz_state);
				self.renderer.draw_goal_line(self.goal_x);
				self.renderer
					.draw_path(&self.path_points, Color::from_rgba(50, 200, 50, 180));

				let player_pos = self.get_player_pos();
				let current_state = self.get_current_state();
				self.renderer.draw_player(player_pos, current_state);

				self.draw_hud_playback();
			}
			VisualizerState::NoSolution => {
				clear_background(BLACK);

				draw_text("No solution found!", 20.0, 40.0, 30.0, RED);

				draw_text(
					&format!(
						"Explored {} nodes, best X: {:.1}",
						self.nodes_expanded, self.current_best_x
					),
					20.0,
					80.0,
					20.0,
					WHITE,
				);

				draw_text("Press ESC to quit", 20.0, 120.0, 20.0, WHITE);
			}
		}
	}

	fn draw_hud_computing(&self) {
		let screen_w = screen_width();
		draw_rectangle(0.0, 0.0, screen_w, 80.0, Color::from_rgba(0, 0, 0, 200));

		draw_text(
			&format!(
				"COMPUTING PATH... | Nodes: {} | Best X: {:.1} / {:.1}",
				self.nodes_expanded, self.current_best_x, self.goal_x
			),
			15.0,
			25.0,
			22.0,
			Color::from_rgba(50, 255, 50, 255),
		);

		let progress = (self.current_best_x / self.goal_x).clamp(0.0, 1.0);
		draw_text(
			&format!(
				"Progress: {:.1}% | Open set: {}",
				progress * 100.0,
				self.open_set_size
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

		// Stop button
		let bw = 120.0;
		let bh = 35.0;

		let bx = screen_w - bw - 15.0;
		let by = 22.0;

		let mouse = mouse_position();
		let hover = mouse.0 >= bx && mouse.0 <= bx + bw && mouse.1 >= by && mouse.1 <= by + bh;
		let color = if hover {
			Color::from_rgba(220, 80, 80, 255)
		} else {
			Color::from_rgba(180, 50, 50, 255)
		};

		draw_rectangle(bx, by, bw, bh, color);
		draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::from_rgba(255, 100, 100, 255));
		draw_text("STOP", bx + 35.0, by + 24.0, 22.0, WHITE);

		// Progress bar
		draw_rectangle(0.0, 75.0, screen_w, 4.0, Color::from_rgba(60, 60, 60, 255));
		draw_rectangle(
			0.0,
			75.0,
			screen_w * progress,
			4.0,
			Color::from_rgba(50, 200, 50, 255),
		);
	}

	fn draw_hud_playback(&self) {
		let screen_w = screen_width();
		draw_rectangle(0.0, 0.0, screen_w, 80.0, Color::from_rgba(0, 0, 0, 180));

		let status = if self.paused { "PAUSED" } else { "PLAYING" };
		let current_state = self.get_current_state();
		let mode_str = match current_state.mode {
			GameMode::Cube => "CUBE",
			GameMode::Ship => "SHIP",
		};

		draw_text(
			&format!(
				"{} | Speed: {:.2}x | Time: {:.2}s / {:.2}s",
				status, self.speed, self.elapsed, self.total_time
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
				if current_state.pressing {
					"PRESS"
				} else {
					"---"
				},
				(self.elapsed / self.dt).floor() as usize,
				self.states.len() - 1
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
		let progress = (self.elapsed / self.total_time).clamp(0.0, 1.0);
		draw_rectangle(0.0, 76.0, screen_w, 4.0, Color::from_rgba(60, 60, 60, 255));
		draw_rectangle(
			0.0,
			76.0,
			screen_w * progress,
			4.0,
			Color::from_rgba(50, 200, 50, 255),
		);
	}
}
