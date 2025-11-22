use std::fs::OpenOptions;
use std::io::{self, Write};

use crate::json_out::parse::{
    hero_strategy_flop_both,
    hero_strategy_river_both,
    hero_strategy_turn_both,
    load_tree,
    HeroStrategy,
};
use crate::solver::run_sample_job;
use colored::*;

fn colorize_card(card: &str) -> String {
    if card.len() < 2 { return card.to_string(); }
    let suit = &card[1..2];
    match suit {
        "h" | "d" => card.red().bold().to_string(),
        "s" | "c" => card.cyan().bold().to_string(),
        _ => card.normal().to_string(),
    }
}

fn colorize_board(board: &str) -> String {
    board.split(',')
        .map(|c| colorize_card(c))
        .collect::<Vec<String>>()
        .join(" ")
}

fn print_strategy_section(
    title: &str,
    oop: Option<&HeroStrategy>,
    ip: Option<&HeroStrategy>,
    oop_vs_bet: Option<&HeroStrategy>,
    hand: &str
) {
    // Parse the title to extract the board cards if possible, or just print the title.
    // The title format is usually "FLOP (Ah,Kd,Qs)" or "TURN (..., ...)"
    // We will just print the title as is but maybe colorize the cards inside it if we parsed them?
    // For simplicity, let's just print the title cleanly and then the specific cards below.
    
    println!("\n{}", format!("=== {} ===", title).bold().white());
    
    // Colorize hero hand
    let h1 = &hand[0..2];
    let h2 = &hand[2..4];
    
    // Evaluate hand strength
    let strength = crate::evaluator::evaluate_hand(hand, title); // Title contains the board e.g. "FLOP (Ah,Kd,Qs)"
    // Actually title has "FLOP (...)", we need just the board.
    // But wait, evaluate_hand just scans for cards, so passing the title string works fine if it contains the cards!
    
    println!("Hero Hand: {} {}  ({})", colorize_card(h1), colorize_card(h2), strength.italic().yellow());
    
    // OOP Box
    print_educational_box(
        "OUT OF POSITION (Big Blind)",
        "The Defender",
        "They raised, you called. Check to the raiser?",
        oop,
        true, // is_oop (red dot)
        oop_vs_bet
    );

    // IP Box
    print_educational_box(
        "IN POSITION (Button)",
        "The Aggressor",
        "You raised, they called. They checked to you.",
        ip,
        false, // is_ip (green dot)
        None // IP doesn't face a bet immediately in this tree (since we removed donk bets)
    );
}

fn print_educational_box(
    position_title: &str,
    role: &str,
    context: &str,
    strategy: Option<&HeroStrategy>,
    is_oop: bool,
    response_strategy: Option<&HeroStrategy>
) {
    let width = 70;
    let horizontal_line = "─".repeat(width);
    
    // Header
    println!("{}", horizontal_line.dimmed());
    let dot = if is_oop { "🔴" } else { "🟢" };
    
    // Header Line: Dot + Title ...... Role
    // We calculate spacing based on visible length (stripping colors effectively)
    // But since we aren't using a right border anymore, we can just print them with a nice gap.
    println!("{} {}   {}", dot, position_title.bold(), format!("(Role: {})", role).italic().dimmed());
    
    println!("{}", horizontal_line.dimmed());

    // Context
    println!("📝 {}", context.yellow());
    println!("{}", horizontal_line.dimmed());

    // Strategy
    if let Some(hero) = strategy {
        let max_action_len = hero.actions.iter().map(|s| s.len()).max().unwrap_or(0);
        
        for (action, prob) in hero.actions.iter().zip(hero.probs.iter()) {
            let percentage = prob * 100.0;
            if percentage < 0.1 { continue; }

            let action_colored = if action.contains("CHECK") {
                action.green()
            } else if action.contains("BET") {
                action.red()
            } else if action.contains("FOLD") {
                action.blue()
            } else {
                action.normal()
            };

            let bar_len = (percentage / 2.5) as usize; // Scale down a bit to fit
            let bar = "█".repeat(bar_len);
            
            // Format: CHECK : 52.4% |||||
            println!(
                "  {:<w$} : {:>5.1}% {}", 
                action_colored, 
                percentage, 
                bar.truecolor(200, 200, 200), 
                w = max_action_len
            );
        }
    } else {
        println!("  (No strategy found for this range)");
    }

    // Response Strategy (e.g. vs Bet)
    if let Some(resp) = response_strategy {
        println!("{}", horizontal_line.dimmed());
        println!("📝 {}", "If they BET, your response:".yellow());
        println!("{}", horizontal_line.dimmed());
        
        let max_action_len = resp.actions.iter().map(|s| s.len()).max().unwrap_or(0);
        for (action, prob) in resp.actions.iter().zip(resp.probs.iter()) {
            let percentage = prob * 100.0;
            if percentage < 0.1 { continue; }

            let action_colored = if action.contains("CHECK") {
                action.green()
            } else if action.contains("BET") {
                action.red()
            } else if action.contains("FOLD") {
                action.blue()
            } else if action.contains("CALL") {
                action.yellow()
            } else {
                action.normal()
            };

            let bar_len = (percentage / 2.5) as usize;
            let bar = "█".repeat(bar_len);
            
            println!(
                "  {:<w$} : {:>5.1}% {}", 
                action_colored, 
                percentage, 
                bar.truecolor(200, 200, 200), 
                w = max_action_len
            );
        }
    }

    println!("{}", horizontal_line.dimmed());
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "TexasSolver TUI (Rust) - prototype".bold().cyan());

    // Ask for hero hand
    print!("Enter hero hand (e.g. AhKd): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let hero_hand_raw = input.trim();
    // Normalize: remove spaces and commas so user can type "Ah Kd" or "Ah,Kd",
    // then normalize cards (rank uppercase, suit lowercase) so it matches JSON keys.
    let cleaned: String = hero_hand_raw
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',')
        .collect();
    let hero_hand: String = if cleaned.len() == 4 {
        let c1 = normalize_card(&cleaned[0..2]);
        let c2 = normalize_card(&cleaned[2..4]);
        format!("{}{}", c1, c2)
    } else {
        cleaned
    };

    if hero_hand.len() != 4 {
        println!(
            "{}",
            format!("Warning: hero hand '{}' does not look like a 4-char hand string (e.g. AhKd)", hero_hand_raw).red()
        );
    }


    // === FLOP ===
    print!("Enter flop cards (e.g. QsJh2h). You can also enter Turn/River (e.g. QsJh2hAcTh): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let all_cards_norm = normalize_board_fragment(&input);
    let all_cards: Vec<&str> = all_cards_norm
        .split(',')
        .filter(|s| !s.is_empty())
        .collect();

    let flop_board;
    let mut prefilled_turn = None;
    let mut prefilled_river = None;

    if all_cards.len() == 3 {
        flop_board = all_cards_norm;
    } else if all_cards.len() == 4 {
        flop_board = all_cards[0..3].join(",");
        prefilled_turn = Some(all_cards[3].to_string());
    } else if all_cards.len() == 5 {
        flop_board = all_cards[0..3].join(",");
        prefilled_turn = Some(all_cards[3].to_string());
        prefilled_river = Some(all_cards[4].to_string());
    } else {
        println!(
            "{}",
            format!("Input '{}' is invalid (need 3, 4, or 5 cards). Aborting.", all_cards_norm).red()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("Running C++ solver (single job) for flop {}... This may take some time.", flop_board).dimmed()
    );
    if let Err(e) = run_sample_job(&flop_board, &hero_hand) {
        println!("{}", format!("Solver error: {}", e).red());
        return Ok(());
    }

    let json_path = "./strategy_debug.json";
    let tree = load_tree(json_path)?;

    let (flop_oop, flop_ip, flop_oop_vs_bet) = hero_strategy_flop_both(&tree, &hero_hand);
    print_strategy_section(
        &format!("FLOP ({})", colorize_board(&flop_board)), 
        flop_oop.as_ref(), 
        flop_ip.as_ref(), 
        flop_oop_vs_bet.as_ref(),
        &hero_hand
    );

    // === TURN ===
    let turn_card = if let Some(t) = prefilled_turn {
        println!("\nTurn card pre-filled: {}", colorize_card(&t));
        t
    } else {
        println!("\nEnter turn card (e.g. 9d). Leave empty to skip: ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let turn_raw = normalize_board_fragment(&input);
        turn_raw
            .split(',')
            .filter(|s| !s.is_empty())
            .next()
            .unwrap_or("")
            .to_string()
    };

    let (_turn_oop, turn_ip) = if !turn_card.is_empty() {
        let (t_oop, t_ip, t_oop_vs_bet) = hero_strategy_turn_both(&tree, &hero_hand, &turn_card).unwrap_or((None, None, None));
        print_strategy_section(
            &format!("TURN ({}, {})", colorize_board(&flop_board), colorize_card(&turn_card)), 
            t_oop.as_ref(), 
            t_ip.as_ref(), 
            t_oop_vs_bet.as_ref(),
            &hero_hand
        );
        (t_oop, t_ip)
    } else {
        (None, None)
    };

    // === RIVER ===
    let river_card = if let Some(r) = prefilled_river {
        println!("\nRiver card pre-filled: {}", colorize_card(&r));
        r
    } else {
        println!("\nEnter river card (e.g. 3c). Leave empty to skip: ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let river_raw = normalize_board_fragment(&input);
        river_raw
            .split(',')
            .filter(|s| !s.is_empty())
            .next()
            .unwrap_or("")
            .to_string()
    };

    let (_river_oop, river_ip) = if !river_card.is_empty() {
        if turn_card.is_empty() {
            println!(
                "{}",
                format!("River '{}' given without a turn card. Please provide a turn to see river strategy.", river_card).red()
            );
            (None, None)
        } else {
            let (r_oop, r_ip, r_oop_vs_bet) = hero_strategy_river_both(&tree, &hero_hand, &turn_card, &river_card).unwrap_or((None, None, None));
            print_strategy_section(
                &format!("RIVER ({}, {}, {})", colorize_board(&flop_board), colorize_card(&turn_card), colorize_card(&river_card)), 
                r_oop.as_ref(), 
                r_ip.as_ref(), 
                r_oop_vs_bet.as_ref(),
                &hero_hand
            );
            (r_oop, r_ip)
        }
    } else {
        (None, None)
    };

    if let Err(e) = append_summary(
        &hero_hand,
        &flop_board,
        if turn_card.is_empty() {
            None
        } else {
            Some(turn_card.as_str())
        },
        if river_card.is_empty() {
            None
        } else {
            Some(river_card.as_str())
        },
        flop_ip.as_ref(), // Default to IP for summary, or we could expand this later
        turn_ip.as_ref(),
        river_ip.as_ref(),
    ) {
        println!("Warning: failed to write summary file: {}", e);
    }

    Ok(())
}

pub fn run_batch(
    hero_input: &str,
    flop_input: &str,
    turn_input: Option<&str>,
    river_input: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "TexasSolver TUI (Rust) - prototype (batch mode)".bold().cyan());

    // Normalize hero hand in the same way as interactive mode
    let hero_hand_raw = hero_input.trim();
    let cleaned: String = hero_hand_raw
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',')
        .collect();
    let hero_hand: String = if cleaned.len() == 4 {
        let c1 = normalize_card(&cleaned[0..2]);
        let c2 = normalize_card(&cleaned[2..4]);
        format!("{}{}", c1, c2)
    } else {
        cleaned
    };

    if hero_hand.len() != 4 {
        println!(
            "{}",
            format!("Warning: hero hand '{}' does not look like a 4-char hand string (e.g. AhKd)", hero_hand_raw).red()
        );
    }

    // === FLOP ===
    let flop_board = normalize_board_fragment(flop_input);
    let flop_cards: Vec<&str> = flop_board
        .split(',')
        .filter(|s| !s.is_empty())
        .collect();
    if flop_cards.len() != 3 {
        println!(
            "{}",
            format!("Flop '{}' is invalid (need exactly 3 cards like 'Qs,Jh,2h'). Aborting.", flop_board).red()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("Running C++ solver (single job) for flop {}... This may take some time.", flop_board).dimmed()
    );
    if let Err(e) = run_sample_job(&flop_board, &hero_hand) {
        println!("{}", format!("Solver error: {}", e).red());
        return Ok(());
    }

    let json_path = "./strategy_debug.json";
    let tree = load_tree(json_path)?;

    let (flop_oop, flop_ip, flop_oop_vs_bet) = hero_strategy_flop_both(&tree, &hero_hand);
    print_strategy_section(
        &format!("FLOP ({})", colorize_board(&flop_board)), 
        flop_oop.as_ref(), 
        flop_ip.as_ref(), 
        flop_oop_vs_bet.as_ref(),
        &hero_hand
    );

    // === TURN ===
    let mut turn_card = String::new();
    if let Some(raw_turn) = turn_input {
        let turn_raw = normalize_board_fragment(raw_turn);
        turn_card = turn_raw
            .split(',')
            .filter(|s| !s.is_empty())
            .next()
            .unwrap_or("")
            .to_string();
    }

    let (_turn_oop, turn_ip) = if !turn_card.is_empty() {
        let (t_oop, t_ip, t_oop_vs_bet) = hero_strategy_turn_both(&tree, &hero_hand, &turn_card).unwrap_or((None, None, None));
        print_strategy_section(
            &format!("TURN ({}, {})", colorize_board(&flop_board), colorize_card(&turn_card)), 
            t_oop.as_ref(), 
            t_ip.as_ref(), 
            t_oop_vs_bet.as_ref(),
            &hero_hand
        );
        (t_oop, t_ip)
    } else {
        (None, None)
    };

    // === RIVER ===
    let mut river_card = String::new();
    if let Some(raw_river) = river_input {
        let river_raw = normalize_board_fragment(raw_river);
        river_card = river_raw
            .split(',')
            .filter(|s| !s.is_empty())
            .next()
            .unwrap_or("")
            .to_string();
    }

    let (_river_oop, river_ip) = if !river_card.is_empty() {
        if turn_card.is_empty() {
            println!(
                "{}",
                format!("River '{}' given without a turn card. Please provide a turn to see river strategy.", river_card).red()
            );
            (None, None)
        } else {
            let (r_oop, r_ip, r_oop_vs_bet) = hero_strategy_river_both(&tree, &hero_hand, &turn_card, &river_card).unwrap_or((None, None, None));
            print_strategy_section(
                &format!("RIVER ({}, {}, {})", colorize_board(&flop_board), colorize_card(&turn_card), colorize_card(&river_card)), 
                r_oop.as_ref(), 
                r_ip.as_ref(), 
                r_oop_vs_bet.as_ref(),
                &hero_hand
            );
            (r_oop, r_ip)
        }
    } else {
        (None, None)
    };

    if let Err(e) = append_summary(
        &hero_hand,
        &flop_board,
        if turn_card.is_empty() {
            None
        } else {
            Some(turn_card.as_str())
        },
        if river_card.is_empty() {
            None
        } else {
            Some(river_card.as_str())
        },
        flop_ip.as_ref(),
        turn_ip.as_ref(),
        river_ip.as_ref(),
    ) {
        println!("Warning: failed to write summary file: {}", e);
    }

    Ok(())
}

fn append_summary(
    hero_hand: &str,
    flop_board: &str,
    turn_card: Option<&str>,
    river_card: Option<&str>,
    flop: Option<&HeroStrategy>,
    turn: Option<&HeroStrategy>,
    river: Option<&HeroStrategy>,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary_path = "resources/outputs/tui_summary.txt";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(summary_path)?;

    writeln!(file, "=== TUI RUN ===")?;
    writeln!(file, "Hero: {}", hero_hand)?;
    writeln!(file, "Flop: {}", flop_board)?;
    if let Some(t) = turn_card {
        writeln!(file, "Turn: {}", t)?;
    }
    if let Some(r) = river_card {
        writeln!(file, "River: {}", r)?;
    }

    if let Some(h) = flop {
        writeln!(file, "Flop strategy:")?;
        for (action, p) in h.actions.iter().zip(h.probs.iter()) {
            writeln!(file, "  {}: {:.4}", action, p)?;
        }
    }
    if let Some(h) = turn {
        writeln!(file, "Turn strategy:")?;
        for (action, p) in h.actions.iter().zip(h.probs.iter()) {
            writeln!(file, "  {}: {:.4}", action, p)?;
        }
    }
    if let Some(h) = river {
        writeln!(file, "River strategy:")?;
        for (action, p) in h.actions.iter().zip(h.probs.iter()) {
            writeln!(file, "  {}: {:.4}", action, p)?;
        }
    }

    writeln!(file)?;

    Ok(())
}



fn normalize_board_fragment(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return String::new();
    }
    // If user already used commas, normalize each card around the commas.
    if t.contains(',') {
        let mut cards = Vec::new();
        for raw_card in t.split(',') {
            let c = raw_card.trim();
            if c.is_empty() {
                continue;
            }
            cards.push(normalize_card(c));
        }
        return cards.join(",");
    }

    // Otherwise, treat a continuous string like "QsJh2h" as 3 consecutive 2-char cards.
    let cleaned: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = cleaned.as_bytes();
    if bytes.len() % 2 != 0 {
        return cleaned;
    }
    let mut cards = Vec::new();
    let mut i = 0;
    while i + 2 <= bytes.len() {
        let card = &cleaned[i..i + 2];
        cards.push(normalize_card(card));
        i += 2;
    }
    cards.join(",")
}

fn normalize_card(card: &str) -> String {
    let mut chars = card.chars();
    let rank = chars.next().unwrap_or('X');
    let suit = chars.next().unwrap_or('x');
    format!("{}{}", rank.to_ascii_uppercase(), suit.to_ascii_lowercase())
}
