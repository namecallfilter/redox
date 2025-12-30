use glam::Vec2;

use crate::{
	config::PhysicsParams,
	game_object::{GameObject, GameObjectType, HitboxShape, OBB2D},
	spatial_grid::SpatialGrid,
	state::{GameMode, State},
};

/// Check if a circle intersects with an axis-aligned rectangle.
pub fn circle_rect_intersects(
	circle_center: Vec2, radius: f32, rect_center: Vec2, rect_w: f32, rect_h: f32,
) -> bool {
	let half_w = rect_w / 2.0;
	let half_h = rect_h / 2.0;

	let closest_x = circle_center
		.x
		.clamp(rect_center.x - half_w, rect_center.x + half_w);
	let closest_y = circle_center
		.y
		.clamp(rect_center.y - half_h, rect_center.y + half_h);

	let dist_sq = (circle_center.x - closest_x).powi(2) + (circle_center.y - closest_y).powi(2);
	dist_sq <= radius * radius
}

pub fn collides_info(
	state: &State, objects: &[GameObject], grid: &SpatialGrid, params: &PhysicsParams,
) -> Option<i32> {
	let player_obb = OBB2D::new(
		state.position,
		params.player_width,
		params.player_height,
		0.0,
	);

	for obj_idx in grid.query(
		state.position,
		params.player_width + 400.0,
		params.player_height + 400.0,
	) {
		let obj = &objects[obj_idx];

		if (obj.position.x - state.position.x).abs() > 200.0 {
			continue;
		}

		let is_colliding = match obj.hitbox_shape {
			HitboxShape::Circle => {
				let radius = obj.width / 2.0;
				circle_rect_intersects(
					obj.position,
					radius,
					state.position,
					params.player_width,
					params.player_height,
				)
			}
			HitboxShape::Rectangle => {
				if let Some(obj_obb) = &obj.obb {
					player_obb.overlaps(obj_obb)
				} else {
					false
				}
			}
		};

		if is_colliding {
			if matches!(obj.object_type, GameObjectType::Sawblade) {
				return Some(obj.id);
			}

			if matches!(obj.object_type, GameObjectType::Hazard) {
				return Some(obj.id);
			}

			if matches!(obj.object_type, GameObjectType::Solid) {
				let obj_top = obj.position.y + obj.height * 0.5;
				let obj_bottom = obj.position.y - obj.height * 0.5;

				let obj_left = obj.position.x - obj.width * 0.5;
				let obj_right = obj.position.x + obj.width * 0.5;

				let player_top = state.position.y + params.player_height * 0.5;
				let player_bottom = state.position.y - params.player_height * 0.5;

				let player_left = state.position.x - params.player_width * 0.5;
				let player_right = state.position.x + params.player_width * 0.5;

				// Ship mode: crash on side collisions, but allow grazing top/bottom surfaces
				if state.mode == GameMode::Ship {
					let h_overlap = player_right.min(obj_right) - player_left.max(obj_left);
					let v_overlap = player_top.min(obj_top) - player_bottom.max(obj_bottom);

					let player_center_y = state.position.y;
					let is_above_obj = player_center_y >= obj_top - 5.0;
					let is_below_obj = player_center_y <= obj_bottom + 5.0;

					if (is_above_obj || is_below_obj) && v_overlap < 15.0 && h_overlap > 5.0 {
						continue;
					}

					return Some(obj.id);
				}

				// Cube mode: check surface zone logic
				let (player_feet, surface_level) = if state.gravity_flipped {
					(player_top, obj_bottom)
				} else {
					(player_bottom, obj_top)
				};

				let is_in_surface_zone = if state.gravity_flipped {
					player_feet <= surface_level + 5.0 && player_feet >= surface_level - 5.0
				} else {
					player_feet >= surface_level - 5.0 && player_feet <= surface_level + 5.0
				};

				if is_in_surface_zone {
					continue;
				}

				return Some(obj.id);
			}

			if matches!(obj.object_type, GameObjectType::Unknown) {
				continue;
			}
		}
	}
	None
}
