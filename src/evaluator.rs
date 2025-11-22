pub fn evaluate_hand(hero_hand: &str, board: &str) -> String {
    let cards = parse_cards(hero_hand, board);
    if cards.is_empty() {
        return "Unknown".to_string();
    }

    let (desc, score) = calculate_strength(&cards);
    format!("{} (Strength: {}/100)", desc, score)
}

fn parse_cards(hero_hand: &str, board: &str) -> Vec<(usize, usize)> {
    let mut cards = Vec::new();
    let full_str = format!("{}{}", hero_hand, board).replace(",", "");
    let mut chars = full_str.chars();
    while let (Some(r), Some(s)) = (chars.next(), chars.next()) {
        let s_str = format!("{}{}", r, s);
        if let Some(card) = parse_single_card(&s_str) {
            cards.push(card);
        }
    }
    cards
}

fn parse_single_card(c: &str) -> Option<(usize, usize)> {
    if c.len() != 2 { return None; }
    let r_char = c.chars().nth(0)?;
    let s_char = c.chars().nth(1)?;
    
    let rank = match r_char {
        '2' => 0, '3' => 1, '4' => 2, '5' => 3, '6' => 4, '7' => 5, '8' => 6, '9' => 7,
        'T' => 8, 'J' => 9, 'Q' => 10, 'K' => 11, 'A' => 12,
        _ => return None,
    };
    
    let suit = match s_char {
        'h' => 0, 'd' => 1, 'c' => 2, 's' => 3,
        _ => return None,
    };
    
    Some((rank, suit))
}

fn calculate_strength(cards: &[(usize, usize)]) -> (String, u8) {
    // 1. Check Flush
    let mut suits = [0; 4];
    for &(_, s) in cards {
        suits[s] += 1;
    }
    let flush_suit = suits.iter().position(|&count| count >= 5);
    if let Some(s) = flush_suit {
        // Get highest rank in that suit
        let mut flush_ranks: Vec<usize> = cards.iter()
            .filter(|(_, suit)| *suit == s)
            .map(|(r, _)| *r)
            .collect();
        flush_ranks.sort_by(|a, b| b.cmp(a));
        let high = flush_ranks[0];
        
        // Check Straight Flush
        if is_straight(&flush_ranks) {
             return ("Straight Flush".to_string(), 95 + (high as u8 * 5 / 13));
        }
        
        return (format!("Flush ({})", suit_name(s)), 75 + (high as u8 * 5 / 13));
    }

    // 2. Check Straight
    let mut ranks: Vec<usize> = cards.iter().map(|(r, _)| *r).collect();
    ranks.sort_by(|a, b| b.cmp(a));
    ranks.dedup();
    
    if let Some(high) = check_straight(&ranks) {
        return ("Straight".to_string(), 70 + (high as u8 * 5 / 13));
    }

    // 3. Check Pairs/Trips/Quads
    let mut rank_counts = [0; 13];
    for &(r, _) in cards {
        rank_counts[r] += 1;
    }
    
    let mut pairs = Vec::new();
    let mut trips = Vec::new();
    let mut quads = Vec::new();
    
    for (r, &count) in rank_counts.iter().enumerate() {
        if count == 2 { pairs.push(r); }
        if count == 3 { trips.push(r); }
        if count == 4 { quads.push(r); }
    }
    
    pairs.sort_by(|a, b| b.cmp(a));
    trips.sort_by(|a, b| b.cmp(a));
    quads.sort_by(|a, b| b.cmp(a));

    if let Some(&q) = quads.first() {
        return ("Four of a Kind".to_string(), 90 + (q as u8 * 5 / 13));
    }
    
    if !trips.is_empty() && (!pairs.is_empty() || trips.len() > 1) {
        let t = trips[0];
        return ("Full House".to_string(), 80 + (t as u8 * 10 / 13));
    }
    
    if let Some(&t) = trips.first() {
        return ("Three of a Kind".to_string(), 60 + (t as u8 * 10 / 13));
    }
    
    if pairs.len() >= 2 {
        let p1 = pairs[0];
        // let p2 = pairs[1];
        return ("Two Pair".to_string(), 40 + (p1 as u8 * 20 / 13));
    }
    
    if let Some(&p) = pairs.first() {
        return (format!("Pair of {}s", rank_name(p)), 20 + (p as u8 * 20 / 13));
    }

    // 4. High Card
    let max_rank = ranks.first().unwrap_or(&0);
    (format!("High Card ({})", rank_name(*max_rank)), (*max_rank as u8 * 20 / 13))
}

fn is_straight(ranks: &[usize]) -> bool {
    check_straight(ranks).is_some()
}

fn check_straight(ranks: &[usize]) -> Option<usize> {
    // Ranks are sorted descending and unique
    if ranks.len() < 5 { return None; }
    
    let mut consecutive = 1;
    for i in 0..ranks.len()-1 {
        if ranks[i] == ranks[i+1] + 1 {
            consecutive += 1;
            if consecutive >= 5 {
                return Some(ranks[i - (consecutive - 2)]); // Top of the sequence
            }
        } else {
            consecutive = 1;
        }
    }
    
    // Wheel
    if ranks.contains(&12) && ranks.contains(&0) && ranks.contains(&1) && ranks.contains(&2) && ranks.contains(&3) {
        return Some(3); // 5 high straight
    }
    
    None
}

fn suit_name(s: usize) -> &'static str {
    match s {
        0 => "Hearts", 1 => "Diamonds", 2 => "Clubs", 3 => "Spades",
        _ => "?",
    }
}

fn rank_name(r: usize) -> &'static str {
    match r {
        0 => "Two", 1 => "Three", 2 => "Four", 3 => "Five", 4 => "Six", 5 => "Seven", 6 => "Eight", 7 => "Nine",
        8 => "Ten", 9 => "Jack", 10 => "Queen", 11 => "King", 12 => "Ace",
        _ => "?",
    }
}
