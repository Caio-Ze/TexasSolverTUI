use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;

// We will resolve these relative to the executable path at runtime
const JOB_FILE_REL_PATH: &str = "resources/text/job_config_debug.txt";
const OUTPUT_JSON_REL_PATH: &str = "strategy_debug.json";
const RESOURCE_DIR_REL: &str = "resources";
const CONSOLE_SOLVER_REL_PATH: &str = "TexasSolver-v0.2.0-MacOs/console_solver";

// Public preflop ranges used in the job config. These come from the solver's
// own presets (qb_ranges, 100bb 2.5x 500rake) so that our TUI matches a
// coherent BTN-open vs BB-call scenario.
// IP (in position) = BTN open range at 100bb 2.5x.
pub const RANGE_IP: &str = "AA:1.0,A2s:1.0,A2o:0.0,A3s:1.0,A3o:0.016,A4s:1.0,A4o:1.0,A5s:1.0,A5o:1.0,A6s:1.0,A6o:1.0,A7s:1.0,A7o:1.0,A8s:1.0,A8o:1.0,A9s:1.0,A9o:1.0,ATs:1.0,ATo:1.0,AJs:1.0,AJo:1.0,AQs:1.0,AQo:1.0,AKs:1.0,AKo:1.0,22:1.0,32s:0.0,32o:0.0,42s:0.0,42o:0.0,52s:0.0,52o:0.0,62s:0.0,62o:0.0,72s:0.0,72o:0.0,82s:0.0,82o:0.0,92s:0.0,92o:0.0,T2s:0.0,T2o:0.0,J2s:0.0,J2o:0.0,Q2s:0.066,Q2o:0.0,K2s:1.0,K2o:0.0,33:1.0,43s:0.0,43o:0.0,53s:0.0,53o:0.0,63s:0.0,63o:0.0,73s:0.0,73o:0.0,83s:0.0,83o:0.0,93s:0.0,93o:0.0,T3s:0.0,T3o:0.0,J3s:0.0,J3o:0.0,Q3s:1.0,Q3o:0.0,K3s:1.0,K3o:0.0,44:1.0,54s:1.0,54o:0.0,64s:0.0,64o:0.0,74s:0.0,74o:0.0,84s:0.0,84o:0.0,94s:0.0,94o:0.0,T4s:0.0,T4o:0.0,J4s:0.256,J4o:0.0,Q4s:1.0,Q4o:0.0,K4s:1.0,K4o:0.0,55:1.0,65s:1.0,65o:0.0,75s:1.0,75o:0.0,85s:0.09,85o:0.0,95s:0.0,95o:0.0,T5s:0.0,T5o:0.0,J5s:1.0,J5o:0.0,Q5s:1.0,Q5o:0.0,K5s:1.0,K5o:0.0,66:1.0,76s:1.0,76o:0.0,86s:1.0,86o:0.0,96s:1.0,96o:0.0,T6s:1.0,T6o:0.0,J6s:1.0,J6o:0.0,Q6s:1.0,Q6o:0.0,K6s:1.0,K6o:0.0,77:1.0,87s:1.0,87o:0.0,97s:1.0,97o:0.0,T7s:1.0,T7o:0.0,J7s:1.0,J7o:0.0,Q7s:1.0,Q7o:0.0,K7s:1.0,K7o:0.0,88:1.0,98s:1.0,98o:0.486,T8s:1.0,T8o:0.558,J8s:1.0,J8o:0.43,Q8s:1.0,Q8o:0.082,K8s:1.0,K8o:0.7,99:1.0,T9s:1.0,T9o:1.0,J9s:1.0,J9o:1.0,Q9s:1.0,Q9o:1.0,K9s:1.0,K9o:1.0,TT:1.0,JTs:1.0,JTo:1.0,QTs:1.0,QTo:1.0,KTs:1.0,KTo:1.0,JJ:1.0,QJs:1.0,QJo:1.0,KJs:1.0,KJo:1.0,QQ:1.0,KQs:1.0,KQo:1.0,KK:1.0";
// OOP (out of position) = BB call vs BTN 2.5x open at 100bb.
pub const RANGE_OOP: &str = "AA:0.0,A2s:1.0,A2o:0.0,A3s:0.822,A3o:0.0,A4s:0.282,A4o:0.48,A5s:0.0,A5o:0.93,A6s:0.766,A6o:0.432,A7s:0.412,A7o:0.976,A8s:0.616,A8o:0.928,A9s:0.818,A9o:0.876,ATs:0.13,ATo:0.918,AJs:0.0,AJo:0.526,AQs:0.0,AQo:0.03,AKs:0.0,AKo:0.0,22:1.0,32s:0.278,32o:0.0,42s:0.796,42o:0.0,52s:1.0,52o:0.0,62s:0.0,62o:0.0,72s:0.0,72o:0.0,82s:0.0,82o:0.0,92s:0.0,92o:0.0,T2s:0.0,T2o:0.0,J2s:0.782,J2o:0.0,Q2s:1.0,Q2o:0.0,K2s:1.0,K2o:0.0,33:1.0,43s:1.0,43o:0.0,53s:0.904,53o:0.0,63s:1.0,63o:0.0,73s:0.032,73o:0.0,83s:0.0,83o:0.0,93s:0.0,93o:0.0,T3s:0.23,T3o:0.0,J3s:1.0,J3o:0.0,Q3s:1.0,Q3o:0.0,K3s:1.0,K3o:0.0,44:1.0,54s:0.396,54o:0.0,64s:0.904,64o:0.0,74s:1.0,74o:0.0,84s:0.136,84o:0.0,94s:0.0,94o:0.0,T4s:0.252,T4o:0.0,J4s:0.996,J4o:0.0,Q4s:1.0,Q4o:0.0,K4s:1.0,K4o:0.0,55:0.972,65s:0.456,65o:0.0,75s:0.82,75o:0.0,85s:1.0,85o:0.0,95s:0.22,95o:0.0,T5s:0.622,T5o:0.0,J5s:0.802,J5o:0.0,Q5s:0.98,Q5o:0.0,K5s:0.898,K5o:0.0,66:0.832,76s:0.346,76o:0.224,86s:0.824,86o:0.0,96s:0.924,96o:0.0,T6s:0.758,T6o:0.0,J6s:0.84,J6o:0.0,Q6s:0.932,Q6o:0.0,K6s:0.736,K6o:0.0,77:0.704,87s:0.212,87o:0.382,97s:0.818,97o:0.0,T7s:0.726,T7o:0.0,J7s:0.55,J7o:0.0,Q7s:0.992,Q7o:0.0,K7s:0.856,K7o:0.0,88:0.486,98s:0.338,98o:0.372,T8s:0.248,T8o:0.42,J8s:0.606,J8o:0.038,Q8s:0.766,Q8o:0.0,K8s:0.64,K8o:0.442,99:0.084,T9s:0.0,T9o:0.876,J9s:0.0,J9o:0.89,Q9s:0.068,Q9o:1.0,K9s:0.306,K9o:0.91,TT:0.0,JTs:0.0,JTo:0.776,QTs:0.122,QTo:0.796,KTs:0.026,KTo:0.802,JJ:0.0,QJs:0.06,QJo:0.904,KJs:0.0,KJo:0.696,QQ:0.0,KQs:0.0,KQo:0.474,KK:0.0";

fn get_exe_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn get_absolute_path(rel_path: &str) -> PathBuf {
    get_exe_dir().join(rel_path)
}

fn build_job_content(board: &str, hero_hand: &str) -> String {
    let generic = get_generic_hand(hero_hand);
    
    // Determine how many rounds to dump based on the board length.
    // Board format is comma separated cards.
    // 3 cards = Flop (dump 1 round)
    // 4 cards = Turn (dump 2 rounds)
    // 5 cards = River (dump 3 rounds)
    let card_count = board.split(',').filter(|s| !s.is_empty()).count();
    let dump_rounds = match card_count {
        3 => 1,
        4 => 2,
        _ => 3,
    };

    let output_path = get_absolute_path(OUTPUT_JSON_REL_PATH);
    // We need to escape the path for the config file if it contains spaces, 
    // but the solver might just take the string.
    // For safety, we just pass the path string.
    let output_path_str = output_path.to_string_lossy();

    let content = format!(
        r#"set_pot 50
set_effective_stack 200
set_board {board}
set_range_ip {range_ip}
set_range_oop {range_oop}
set_bet_sizes ip,flop,bet,50
set_bet_sizes ip,turn,bet,50
set_allin_threshold 0.8
set_thread_num 8
set_accuracy 5.0
set_max_iteration 10
set_print_interval 10
set_use_isomorphism 1
build_tree
start_solve
set_dump_rounds {dump_rounds}
dump_result {output_path}
"#,
        board = board,
        range_ip = activate_hand_in_range(RANGE_IP, &generic),
        range_oop = activate_hand_in_range(RANGE_OOP, &generic),
        output_path = output_path_str,
        dump_rounds = dump_rounds
    );
    content.trim().to_string() + "\n"
}

fn get_generic_hand(hand: &str) -> String {
    // hand is like "AhKh" or "AsKs"
    let chars: Vec<char> = hand.chars().collect();
    if chars.len() < 4 { return "AA".to_string(); } // fallback
    
    let r1 = chars[0];
    let s1 = chars[1];
    let r2 = chars[2];
    let s2 = chars[3];

    // Sort ranks to match standard notation (AK, not KA)
    // Ranks: A, K, Q, J, T, 9, 8, 7, 6, 5, 4, 3, 2
    let rank_order = "AKQJT98765432";
    let idx1 = rank_order.find(r1).unwrap_or(0);
    let idx2 = rank_order.find(r2).unwrap_or(0);

    let (first, second) = if idx1 <= idx2 { (r1, r2) } else { (r2, r1) };

    if r1 == r2 {
        format!("{}{}", first, second) // Pair: "AA"
    } else if s1 == s2 {
        format!("{}{}{}", first, second, 's') // Suited: "AKs"
    } else {
        format!("{}{}{}", first, second, 'o') // Offsuit: "AKo"
    }
}

fn activate_hand_in_range(range: &str, target_generic: &str) -> String {
    // Optimization:
    // 1. If the hand is the user's hand (target_generic), we force it to 1.0 (even if it was 0.0).
    // 2. If the hand has 0.0 weight AND is NOT the user's hand, we remove it entirely.
    // 3. Otherwise (standard hands with weight > 0), we keep them as is.
    
    range.split(',')
        .filter_map(|token| {
            let parts: Vec<&str> = token.split(':').collect();
            if parts.len() == 2 {
                let hand = parts[0];
                let weight: f64 = parts[1].parse().unwrap_or(0.0);

                if hand == target_generic {
                    // Always keep Hero's hand, force to 1.0 if it was negligible
                    if weight < 0.01 {
                        return Some(format!("{}:1.0", hand));
                    } else {
                        return Some(token.to_string());
                    }
                }

                // If it's not Hero's hand, only keep it if it has real weight
                if weight > 0.0 {
                    return Some(token.to_string());
                } else {
                    // Prune it (return None)
                    return None;
                }
            }
            // Malformed token? Just keep it to be safe, though shouldn't happen.
            Some(token.to_string())
        })
        .collect::<Vec<String>>()
        .join(",")
}

fn write_job_file(board: &str, hero_hand: &str) -> Result<PathBuf, Box<dyn Error>> {
    println!("DEBUG: Writing job file...");
    let job_path = get_absolute_path(JOB_FILE_REL_PATH);
    if let Some(parent) = job_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(&job_path)?;
    let content = build_job_content(board, hero_hand);
    f.write_all(content.as_bytes())?;
    println!("DEBUG: Job file written to {}", job_path.display());
    Ok(job_path)
}

pub fn run_sample_job(board: &str, hero_hand: &str) -> Result<(), Box<dyn Error>> {
    let output_path = get_absolute_path(OUTPUT_JSON_REL_PATH);
    if output_path.exists() {
        fs::remove_file(&output_path)?;
    }
    
    let job_path = write_job_file(board, hero_hand)?;
    let solver_path = get_absolute_path(CONSOLE_SOLVER_REL_PATH);
    let resource_dir = get_absolute_path(RESOURCE_DIR_REL);
    
    // Run the external console solver binary on macOS.
    println!("DEBUG: About to run command: {}", solver_path.display());
    let status = Command::new(solver_path)
        .arg("--input_file")
        .arg(job_path)
        .arg("-r")
        .arg(resource_dir)
        .arg("-m")
        .arg("holdem")
        .status()?;

    if !status.success() {
        return Err(format!(
            "console_solver exited with non-zero status: {}",
            status
        )
        .into());
    }

    if !output_path.exists() {
        return Err(format!("expected output JSON not found at {}", output_path.display()).into());
    }

    Ok(())
}
