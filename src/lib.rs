use std::env;

pub mod ui;
pub mod json_out;
pub mod solver;
pub mod evaluator;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
	let mut args = env::args().skip(1);

	let hero = args.next();
	let flop = args.next();
	let turn = args.next();
	let river = args.next();

	if hero.is_none() || flop.is_none() {
		return ui::flow::run();
	}

	ui::flow::run_batch(
		hero.as_deref().unwrap(),
		flop.as_deref().unwrap(),
		turn.as_deref(),
		river.as_deref(),
	)
}
