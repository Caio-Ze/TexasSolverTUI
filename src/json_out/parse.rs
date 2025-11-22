use std::error::Error;
use std::fs;

use serde_json::Value;

pub struct HeroStrategy {
    pub actions: Vec<String>,
    pub probs: Vec<f64>,
}

pub fn load_tree(json_path: &str) -> Result<Value, Box<dyn Error>> {
    let data = fs::read_to_string(json_path)?;
    let root: Value = serde_json::from_str(&data)?;
    Ok(root)
}

pub fn find_hero_strategy_vector(
    json_path: &str,
    hero_hand: &str,
) -> Result<Option<Vec<f64>>, Box<dyn Error>> {
    Ok(find_hero_strategy(json_path, hero_hand)?.map(|h| h.probs))
}

pub fn find_hero_strategy(
    json_path: &str,
    hero_hand: &str,
) -> Result<Option<HeroStrategy>, Box<dyn Error>> {
    let root = load_tree(json_path)?;
    if let Some(node) = find_node_with_hero_strategy(&root, hero_hand) {
        Ok(hero_strategy_from_node(node, hero_hand))
    } else {
        Ok(None)
    }
}

pub fn hero_strategy_from_node(node: &Value, hero_hand: &str) -> Option<HeroStrategy> {
    if let Some(mut probs) = extract_strategy_vector(node, hero_hand) {
        let mut actions = extract_actions(node);
        if !actions.is_empty() {
            let n = actions.len().min(probs.len());
            actions.truncate(n);
            probs.truncate(n);
        } else {
            let n = probs.len();
            actions = (0..n).map(|i| format!("action #{}", i)).collect();
        }
        Some(HeroStrategy { actions, probs })
    } else {
        None
    }
}

pub fn extract_street_strategies(
    start_node: &Value,
    hero_hand: &str
) -> (Option<HeroStrategy>, Option<HeroStrategy>, Option<HeroStrategy>) {
    // 1. OOP Open Strategy (Root of the street)
    let oop_open = hero_strategy_from_node(start_node, hero_hand);
    
    // 2. IP Strategy (After OOP Checks)
    let check_node = start_node.get("childrens").and_then(|c| c.get("CHECK"));
    let ip_vs_check = check_node.and_then(|n| hero_strategy_from_node(n, hero_hand));
    
    // 3. OOP Response to Bet (After OOP Checks -> IP Bets)
    // We look for any child of the CHECK node that contains "BET"
    let oop_vs_bet = check_node.and_then(|n| {
         n.get("childrens")?.as_object()?.iter()
            .find(|(k, _)| k.contains("BET"))
            .map(|(_, v)| hero_strategy_from_node(v, hero_hand))
    }).flatten();
    
    (oop_open, ip_vs_check, oop_vs_bet)
}

pub fn hero_strategy_flop_both(root: &Value, hero_hand: &str) -> (Option<HeroStrategy>, Option<HeroStrategy>, Option<HeroStrategy>) {
    extract_street_strategies(root, hero_hand)
}

pub fn hero_strategy_flop(root: &Value, hero_hand: &str) -> Option<HeroStrategy> {
    let (oop, ip, _) = hero_strategy_flop_both(root, hero_hand);
    ip.or(oop)
}

pub fn hero_strategy_turn_both(
    root: &Value,
    hero_hand: &str,
    turn_card: &str,
) -> Option<(Option<HeroStrategy>, Option<HeroStrategy>, Option<HeroStrategy>)> {
    // Path to Turn: Root(OOP) -> Check -> IP -> Check -> Deal Turn Card
    let childrens = root.get("childrens").and_then(|c| c.as_object())?;
    let c1 = childrens.get("CHECK")?; // OOP Checked
    let ch1 = c1.get("childrens").and_then(|c| c.as_object())?;
    let chance = ch1.get("CHECK")?; // IP Checked
    let dealcards = chance.get("dealcards").and_then(|c| c.as_object())?;
    let turn_node = dealcards.get(turn_card)?; // Turn Card Dealt

    Some(extract_street_strategies(turn_node, hero_hand))
}

pub fn hero_strategy_turn_check(
    root: &Value,
    hero_hand: &str,
    turn_card: &str,
) -> Option<HeroStrategy> {
    let (oop, ip, _) = hero_strategy_turn_both(root, hero_hand, turn_card)?;
    ip.or(oop)
}

pub fn hero_strategy_river_both(
    root: &Value,
    hero_hand: &str,
    turn_card: &str,
    river_card: &str,
) -> Option<(Option<HeroStrategy>, Option<HeroStrategy>, Option<HeroStrategy>)> {
    // Path to River: ... Turn Node -> OOP Check -> IP Check -> Deal River Card
    
    // 1. Navigate to Turn Node
    let childrens = root.get("childrens").and_then(|c| c.as_object())?;
    let c1 = childrens.get("CHECK")?;
    let ch1 = c1.get("childrens").and_then(|c| c.as_object())?;
    let chance_turn = ch1.get("CHECK")?;
    let dealcards_turn = chance_turn.get("dealcards").and_then(|c| c.as_object())?;
    let turn_node = dealcards_turn.get(turn_card)?;

    // 2. Navigate through Turn Check-Check to River
    let ch_t1 = turn_node.get("childrens").and_then(|c| c.as_object())?;
    let c_t1 = ch_t1.get("CHECK")?; // OOP Checked Turn
    let ch_t2 = c_t1.get("childrens").and_then(|c| c.as_object())?;
    let c_t2 = ch_t2.get("CHECK")?; // IP Checked Turn
    let dealcards_river = c_t2.get("dealcards").and_then(|c| c.as_object())?;
    let river_node = dealcards_river.get(river_card)?; // River Card Dealt

    Some(extract_street_strategies(river_node, hero_hand))
}

pub fn hero_strategy_river_check(
    root: &Value,
    hero_hand: &str,
    turn_card: &str,
    river_card: &str,
) -> Option<HeroStrategy> {
    let (oop, ip, _) = hero_strategy_river_both(root, hero_hand, turn_card, river_card)?;
    ip.or(oop)
}

fn find_node_with_hero_strategy<'a>(value: &'a Value, hero_hand: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            // Check if this node has a strategy for the hero hand
            if let Some(strategy_obj) = map.get("strategy") {
                if let Some(inner) = strategy_obj.get("strategy") {
                    if inner.get(hero_hand).is_some() {
                        return Some(value);
                    }
                }
            }

            // Otherwise recurse into childrens
            if let Some(childrens) = map.get("childrens") {
                if let Value::Object(children_map) = childrens {
                    for child in children_map.values() {
                        if let Some(found) = find_node_with_hero_strategy(child, hero_hand) {
                            return Some(found);
                        }
                    }
                }
            }

            None
        }
        _ => None,
    }
}

fn extract_actions(node: &Value) -> Vec<String> {
    if let Some(arr) = node.get("actions").and_then(|v| v.as_array()) {
        let mut result = Vec::with_capacity(arr.len());
        for v in arr {
            if let Some(s) = v.as_str() {
                result.push(s.to_string());
            }
        }
        result
    } else {
        Vec::new()
    }
}

fn extract_strategy_vector(node: &Value, hero_hand: &str) -> Option<Vec<f64>> {
    let obj = node.as_object()?;
    let strategy_obj = obj.get("strategy")?;
    let inner = strategy_obj.get("strategy")?;
    let inner_obj = inner.as_object()?;

    // 1) Try exact key match first (works for most non-pair hands like AdTh, Tc9c, AsKd).
    let mut key_to_use: Option<&str> = None;
    if inner_obj.get(hero_hand).is_some() {
        key_to_use = Some(hero_hand);
    } else if hero_hand.len() == 4 {
        // 2) Fallback: some combos (especially pocket pairs) may be stored with
        // the two hole cards in the opposite order to how the user typed them.
        // If so, we treat AB and BA with the same two cards as equivalent.
        let h1 = &hero_hand[0..2];
        let h2 = &hero_hand[2..4];

        for k in inner_obj.keys() {
            if k.len() != 4 {
                continue;
            }
            let c1 = &k[0..2];
            let c2 = &k[2..4];
            if (c1 == h1 && c2 == h2) || (c1 == h2 && c2 == h1) {
                key_to_use = Some(k.as_str());
                break;
            }
        }
    }

    let hand_entry = key_to_use.and_then(|k| inner_obj.get(k))?;
    let arr = hand_entry.as_array()?;

    let mut result = Vec::with_capacity(arr.len());
    for v in arr {
        if let Some(f) = v.as_f64() {
            result.push(f);
        } else if let Some(i) = v.as_i64() {
            result.push(i as f64);
        } else {
            // unsupported type, skip
        }
    }

    Some(result)
}
