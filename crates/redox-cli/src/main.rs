mod visualizer;

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use glam::Vec2;
use redox_core::{formats::level, game_object::GameObject, gdr, pathfinder::Pathfinder, state};
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, prelude::*};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Path to the level file
	#[arg(short, long)]
	level: PathBuf,

	/// Output path for the replay file
	#[arg(short, long, default_value = "replay.gdr")]
	output: PathBuf,

	/// Open the graphical visualizer
	#[arg(short, long)]
	visualize: bool,
}

fn main() -> Result<()> {
	let args = Args::parse();

	tracing_subscriber::registry()
		.with(tracing_subscriber::fmt::layer())
		.with(
			EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| EnvFilter::new(format!("{}=debug", env!("CARGO_CRATE_NAME")))),
		)
		.init();

	if args.visualize {
		info!("Launching visualizer for level: {:?}", args.level);
		macroquad::Window::from_config(
			visualizer::window_conf(),
			visualizer::run_visualizer(args.level),
		);
		return Ok(());
	}

	if !args.level.exists() {
		error!("Level file not found: {:?}", args.level);
		return Ok(());
	}

	let content = fs::read_to_string(&args.level)
		.with_context(|| format!("Failed to read level file: {:?}", args.level))?;

	info!("Parsing level data...");
	let decompressed = level::parse_level_data(&content)?;

	let raw_objects = level::parse_objects(&decompressed);
	let game_objects: Vec<GameObject> = raw_objects.iter().map(GameObject::from_raw).collect();

	info!("Converted {} game objects", game_objects.len());

	let mut max_x = 0.0;
	for obj in &game_objects {
		if obj.position.x > max_x {
			max_x = obj.position.x;
		}
	}

	info!("Level Max X: {}", max_x);
	let goal_x = max_x + 200.0; // Aim a bit past the last object

	info!("Initializing Pathfinder...");

	let start_pos = Vec2::new(0.0, 15.0); // Start 15 units up (center of 30 unit cube)

	let pathfinder = Pathfinder::new(game_objects, goal_x);

	info!("Starting search...");

	let mut session = pathfinder.start_search(start_pos, goal_x);
	loop {
		let done = pathfinder.step(&mut session, goal_x);
		if done {
			break;
		}
	}

	let chosen_idx = if let Some(idx) = session.goal_reached_index {
		info!("Path found! Reached goal at node index {}", idx);
		idx
	} else {
		let mut best_i = 0usize;
		for (i, node) in session.all_nodes.iter().enumerate() {
			let best_node = &session.all_nodes[best_i];
			let better = if node.state.position.x > best_node.state.position.x + 0.1 {
				true
			} else if (node.state.position.x - best_node.state.position.x).abs() < 0.1 {
				node.g < best_node.g
			} else {
				false
			};

			if better {
				best_i = i;
			}
		}

		let best_node = &session.all_nodes[best_i];
		warn!("No path to goal found.");
		warn!(
			"Generating partial replay to furthest point: x={:.1} (node {})",
			best_node.state.position.x, best_i
		);

		best_i
	};

	let path = pathfinder.reconstruct_path(&session.all_nodes, &session.all_nodes[chosen_idx]);

	info!("Path Length: {} steps", path.len());
	let presses = path
		.iter()
		.filter(|(a, _)| matches!(a, state::Action::Press))
		.count();
	info!("Total Presses: {}", presses);

	match gdr::save_gdr(&path, args.output.to_str().unwrap(), 240.0) {
		Ok(()) => info!("Saved replay to {}", args.output.display()),
		Err(e) => error!("Failed to save replay: {:?}", e),
	}

	Ok(())
}
