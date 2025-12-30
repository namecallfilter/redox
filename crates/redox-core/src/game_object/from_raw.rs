use glam::Vec2;

use super::{
	mapping::{get_hitbox_for_id, get_object_type_for_id},
	obb::OBB2D,
	types::{GameObject, HitboxShape},
};
use crate::parser::RawObject;

impl GameObject {
	pub fn from_raw(raw: &RawObject) -> Self {
		let mut id = 0;

		let mut x = 0.0;
		let mut y = 0.0;

		let mut rotation: f32 = 0.0;

		let mut scale = 1.0_f32;
		let mut scale_x = 1.0_f32;
		let mut scale_y = 1.0_f32;

		let mut flip_x = false;
		let mut flip_y = false;

		for (key, val) in &raw.properties {
			match key.as_str() {
				"1" => id = val.parse().unwrap_or(0),
				"2" => x = val.parse().unwrap_or(0.0),
				"3" => y = val.parse().unwrap_or(0.0),
				"4" => flip_x = val == "1",
				"5" => flip_y = val == "1",
				"6" => rotation = val.parse().unwrap_or(0.0),
				"32" => scale = val.parse().unwrap_or(1.0),
				"128" => scale_x = val.parse().unwrap_or(1.0),
				"129" => scale_y = val.parse().unwrap_or(1.0),
				_ => {}
			}
		}

		scale_x *= scale;
		scale_y *= scale;

		let (hitbox_shape, base_width, base_height) = get_hitbox_for_id(id);
		let width = base_width * scale_x;
		let height = base_height * scale_y;

		let position = Vec2::new(x, y);
		let object_type = get_object_type_for_id(id);

		let obb = if hitbox_shape == HitboxShape::Rectangle {
			Some(OBB2D::new(position, width, height, rotation))
		} else {
			None
		};

		GameObject {
			id,
			object_type,
			position,
			rotation,
			scale: Vec2::new(scale_x, scale_y),
			flip_x,
			flip_y,
			hitbox_shape,
			width,
			height,
			obb,
		}
	}
}
