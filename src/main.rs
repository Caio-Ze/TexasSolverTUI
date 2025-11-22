fn main() {
    if let Err(err) = texas_solver_tui::run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

