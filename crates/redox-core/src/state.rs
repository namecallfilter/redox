use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameMode {
	Cube,
	Ship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
	None,    // No change to press state
	Press,   // Toggle press ON (start holding)
	Release, // Toggle press OFF (stop holding)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
	pub position: Vec2,
	pub vy: f32,
	pub on_ground: bool,
	pub rotation: f32,
	pub mode: GameMode,
	pub gravity_flipped: bool,
	/// Floor position for vehicle bounds (set by portals)
	pub floor: f32,
	/// Ceiling position for vehicle bounds (set by portals)
	pub ceiling: f32,
	/// Whether the button is currently being held (press state)
	pub pressing: bool,
	/// Speed index (0=0.5x, 1=1x, 2=2x, 3=3x, 4=4x)
	pub speed: usize,
}

impl Eq for State {}

// We use a bit-packed u128 for the StateKey to speed up hashing and comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateKey(pub u128);

impl StateKey {
	pub fn from_state(state: &State, x_quant: f32, y_quant: f32, vy_quant: f32) -> Self {
		let xi = (state.position.x / x_quant).floor() as i32;
		let yi = (state.position.y / y_quant).floor() as i32;
		let vyi = (state.vy / vy_quant).floor() as i32;

		let ceiling_i = if state.ceiling >= f32::MAX / 2.0 {
			10000 // Large constant for cube mode
		} else {
			(state.ceiling / 30.0).floor() as i32
		};

		let mode_bit = match state.mode {
			GameMode::Cube => 0,
			GameMode::Ship => 1,
		};

		let mut packed = 0u128;
		// xi: 32 bits (offset 0)
		packed |= xi as u32 as u128;
		// yi: 32 bits (offset 32)
		packed |= (yi as u32 as u128) << 32;
		// vyi: 24 bits (offset 64)
		packed |= ((vyi & 0xFFFFFF) as u128) << 64;
		// ceiling_i: 16 bits (offset 88)
		packed |= ((ceiling_i & 0xFFFF) as u128) << 88;

		// Flags: 8 bits (offset 104)
		if state.on_ground {
			packed |= 1 << 104;
		}

		if state.gravity_flipped {
			packed |= 1 << 105;
		}

		if state.pressing {
			packed |= 1 << 106;
		}

		packed |= (mode_bit as u128) << 107;
		packed |= ((state.speed as u128) & 0x7) << 108;

		Self(packed)
	}
}

// A wrapper for the priority queue that orders by f-score (lowest first)
#[derive(Debug, Clone, Copy)]
pub struct Node {
	pub g: f32, // cost so far (time)
	pub f: f32, // estimated total cost (g + h)
	pub state: State,
	pub parent_index: Option<usize>,
	pub action: Option<Action>,
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.f == other.f
	}
}

impl Eq for Node {}

impl PartialOrd for Node {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Node {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Reverse ordering because BinaryHeap is a max-heap, and we want min-f
		other
			.f
			.partial_cmp(&self.f)
			.unwrap_or(std::cmp::Ordering::Equal)
	}
}
