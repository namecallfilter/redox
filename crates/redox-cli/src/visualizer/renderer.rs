use macroquad::prelude::*;
use redox_core::{
	game_object::{GameObject, GameObjectType},
	state::State,
};

use super::types::{Vec2, VisualizerState};

pub struct Renderer {
	pub pixels_per_unit: f32,
	pub camera_pos: Vec2,
}

impl Renderer {
	pub fn new(start_pos: Vec2) -> Self {
		Self {
			pixels_per_unit: 3.0,
			camera_pos: start_pos,
		}
	}

	pub fn world_to_screen(&self, world: Vec2) -> (f32, f32) {
		let screen_center_x = screen_width() / 2.0;
		let screen_center_y = screen_height() / 2.0;
		let dx = (world.x - self.camera_pos.x) * self.pixels_per_unit + screen_center_x;
		let dy = screen_center_y - (world.y - self.camera_pos.y) * self.pixels_per_unit;
		(dx, dy)
	}

	pub fn draw_game_objects(&self, objects: &[GameObject], viz_state: VisualizerState) {
		let screen_w = screen_width();
		let cull_distance = screen_w / self.pixels_per_unit * 0.7;

		for obj in objects {
			if (obj.position.x - self.camera_pos.x).abs() > cull_distance {
				continue;
			}

			let color = match viz_state {
				VisualizerState::Computing => match obj.object_type {
					GameObjectType::Solid => Color::from_rgba(40, 70, 120, 255),
					GameObjectType::Hazard => Color::from_rgba(150, 40, 40, 255),
					GameObjectType::Unknown => continue,
					_ => Color::from_rgba(100, 100, 100, 255),
				},
				VisualizerState::Playback => match obj.object_type {
					GameObjectType::Solid => Color::from_rgba(60, 100, 180, 255),
					GameObjectType::Hazard => Color::from_rgba(220, 60, 60, 255),
					GameObjectType::Unknown => continue,
					_ => Color::from_rgba(150, 150, 150, 255),
				},
				_ => continue,
			};

			let world_top_left = Vec2::new(
				obj.position.x - obj.width * 0.5,
				obj.position.y + obj.height * 0.5,
			);
			let (sx, sy) = self.world_to_screen(world_top_left);
			let rect_w = obj.width * self.pixels_per_unit;
			let rect_h = obj.height * self.pixels_per_unit;

			draw_rectangle(sx, sy, rect_w, rect_h, color);
		}
	}

	pub fn draw_goal_line(&self, goal_x: f32) {
		let (goal_screen_x, _) = self.world_to_screen(Vec2::new(goal_x, 0.0));
		draw_line(
			goal_screen_x,
			0.0,
			goal_screen_x,
			screen_height(),
			3.0,
			Color::from_rgba(180, 50, 200, 255),
		);
	}

	pub fn draw_path(&self, path: &[Vec2], color: Color) {
		if path.len() > 1 {
			for i in 0..path.len() - 1 {
				let (p1x, p1y) = self.world_to_screen(path[i]);
				let (p2x, p2y) = self.world_to_screen(path[i + 1]);
				draw_line(p1x, p1y, p2x, p2y, 2.0, color);
			}
		}
	}

	pub fn draw_player(&self, pos: Vec2, state: &State) {
		let (player_sx, player_sy) = self.world_to_screen(pos);
		let player_size = 30.0 * self.pixels_per_unit;

		draw_rectangle(
			player_sx - player_size * 0.5,
			player_sy - player_size * 0.5,
			player_size,
			player_size,
			Color::from_rgba(255, 140, 0, 255),
		);

		draw_rectangle_lines(
			player_sx - player_size * 0.5,
			player_sy - player_size * 0.5,
			player_size,
			player_size,
			2.0,
			Color::from_rgba(200, 100, 0, 255),
		);

		if state.pressing {
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
	}
}
