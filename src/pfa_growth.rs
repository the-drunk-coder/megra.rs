use rand::{seq::SliceRandom, Rng};

use vom_rs::pfa::*;
use vom_rs::pst;

/// growth methods that allow us to expand a pfa using certain heuristic principles

#[allow(dead_code)]
pub fn grow_old(pfa: &mut Pfa<char>) -> Option<PfaOperationResult<char>> {
    //pfa.pad_history();
    if pfa.history.is_empty() {
        return None;
    }

    let source_id = vec![*pfa.history.first().unwrap()];
    let dest_id = vec![*pfa.history.last().unwrap()];
    let node_id = *pfa.history.choose(&mut rand::thread_rng()).unwrap();

    // make sure states exists
    if !(pfa.has_state(&source_id) && pfa.has_state(&dest_id)) {
        return None;
    }

    let mut rand_state = Vec::new();

    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        let c: char = rng.gen();
        if !pfa.alphabet.contains(&c) {
            rand_state.push(c);
            break;
        }
    }

    if rand_state.len() == 0 {
        // can't find random state id
        return None;
    }

    pfa.add_state(&rand_state);

    // update pst
    if let Some(mut root) = pfa.pst_root.as_mut() {
        pst::add_leaf(&mut root, &rand_state);
    }

    let mut additions = Vec::new();
    additions.push(pfa.add_state_transition(
        &rand_state,
        &source_id,
        0.05 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));
    additions.push(pfa.add_state_transition(
        &rand_state,
        &dest_id,
        0.05 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));
    additions.append(&mut pfa.add_symbol_transition(
        *source_id.first().unwrap(),
        &rand_state,
        0.05 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));
    additions.append(&mut pfa.add_symbol_transition(
        *dest_id.first().unwrap(),
        &rand_state,
        0.05 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));

    pfa.rebalance();

    // let the outside world know what's happening ...
    Some(PfaOperationResult {
        added_states: vec![rand_state.clone()],
        removed_states: Vec::new(),
        added_transitions: additions,
        removed_transitions: Vec::new(),
        template_symbol: Some(node_id),
        added_symbol: Some(*rand_state.first().unwrap()),
    })
}

#[allow(dead_code)]
pub fn grow_flower(pfa: &mut Pfa<char>) -> Option<PfaOperationResult<char>> {
    //pfa.pad_history();

    if pfa.history.is_empty() {
        return None;
    }

    let mut source_id = Label::new();
    while let Some(s) = pfa.history.iter().rev().next() {
        source_id.insert(0, *s);
        let source_hash = calculate_hash(&source_id);
        if pfa.has_state_hash(source_hash) {
            break;
        }
        if source_id.len() > 4 {
            // only look for a certain lenght
            return None;
        }
    }

    let mut rand_state = Vec::new();

    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        let c: char = rng.gen(); // this is a bit critical because it causes unprintable chars ...
        if !pfa.alphabet.contains(&c) {
            rand_state.push(c);
            break;
        }
    }

    if rand_state.len() == 0 {
        // can't find random state id
        return None;
    }

    pfa.add_state(&rand_state);

    // update pst
    if let Some(mut root) = pfa.pst_root.as_mut() {
        pst::add_leaf(&mut root, &rand_state);
    }

    let mut additions = Vec::new();
    additions.push(pfa.add_state_transition(
        &rand_state,
        &source_id,
        0.2 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));
    additions.append(&mut pfa.add_symbol_transition(
        *source_id.first().unwrap(),
        &rand_state,
        0.2 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));

    pfa.rebalance();

    // let the outside world know what's happening ...
    Some(PfaOperationResult {
        added_states: vec![rand_state.clone()],
        removed_states: Vec::new(),
        added_transitions: additions,
        removed_transitions: Vec::new(),
        template_symbol: Some(*source_id.first().unwrap()),
        added_symbol: Some(*rand_state.first().unwrap()),
    })
}

#[allow(dead_code)]
pub fn grow_triloop(pfa: &mut Pfa<char>) -> Option<PfaOperationResult<char>> {
    //pfa.pad_history();

    if pfa.history.len() < 3 {
        return None;
    }

    let source_id = vec![*pfa.history.last().unwrap()];
    let dest_id = vec![*pfa.history.get(pfa.history.len() - 2).unwrap()];

    // make sure states exists
    if pfa.has_state(&source_id) && pfa.has_state(&dest_id) {
        return None;
    }

    let mut rand_state = Vec::new();

    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        let c: char = rng.gen();
        if !pfa.alphabet.contains(&c) {
            rand_state.push(c);
            break;
        }
    }

    if rand_state.len() == 0 {
        // can't find random state id
        return None;
    }

    pfa.add_state(&rand_state);

    // update pst
    if let Some(mut root) = pfa.pst_root.as_mut() {
        pst::add_leaf(&mut root, &rand_state);
    }

    let mut additions = Vec::new();
    additions.push(pfa.add_state_transition(
        &rand_state,
        &dest_id,
        0.2 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));
    additions.append(&mut pfa.add_symbol_transition(
        *source_id.first().unwrap(),
        &rand_state,
        0.2 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));

    pfa.rebuild_pst();
    pfa.rebalance();

    // let the outside world know what's happening ...
    Some(PfaOperationResult {
        added_states: vec![rand_state.clone()],
        removed_states: Vec::new(),
        added_transitions: additions,
        removed_transitions: Vec::new(), //removals,
        template_symbol: Some(*source_id.first().unwrap()),
        added_symbol: Some(*rand_state.first().unwrap()),
    })
}

#[allow(dead_code)]
// if this is called when the history is still padded, it shows
// an interesting splitting behaviour, as source and dest id are the
// same and no transition is removed ...
// it's not quite "correct" but somehow neat ...
pub fn grow_loop(pfa: &mut Pfa<char>) -> Option<PfaOperationResult<char>> {
    //pfa.pad_history();

    // unwraps should be fine because the history is padded ...
    if pfa.history.len() < 3 {
        return None;
    }

    let dest_id = vec![*pfa.history.last().unwrap()];
    let source_id = vec![*pfa.history.get(pfa.history.len() - 2).unwrap()];

    // make sure states exists
    if pfa.has_state(&source_id) && pfa.has_state(&dest_id) {
        return None;
    }

    let mut rand_state = Vec::new();

    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        let c: char = rng.gen();
        if !pfa.alphabet.contains(&c) {
            rand_state.push(c);
            break;
        }
    }

    if rand_state.len() == 0 {
        // can't find random state id
        return None;
    }

    pfa.add_state(&rand_state);

    // update pst
    if let Some(mut root) = pfa.pst_root.as_mut() {
        pst::add_leaf(&mut root, &rand_state);
    }

    let mut removals = Vec::new();
    removals.append(&mut pfa.remove_symbol_transition(
        *source_id.first().unwrap(),
        *dest_id.first().unwrap(),
        false,
    ));

    let mut additions = Vec::new();
    additions.push(pfa.add_state_transition(
        &rand_state,
        &dest_id,
        0.2 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));
    //println!("{:?}", pfa.alphabet);

    additions.append(&mut pfa.add_symbol_transition(
        *source_id.first().unwrap(),
        &rand_state,
        0.2 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));

    //pfa.remove_orphaned_states();
    pfa.rebuild_pst();

    pfa.rebalance();

    // let the outside world know what's happening ...
    Some(PfaOperationResult {
        added_states: vec![rand_state.clone()],
        removed_states: Vec::new(),
        added_transitions: additions,
        removed_transitions: removals,
        template_symbol: Some(*source_id.first().unwrap()),
        added_symbol: Some(*rand_state.first().unwrap()),
    })
}

#[allow(dead_code)]
pub fn grow_quadloop(pfa: &mut Pfa<char>) -> Option<PfaOperationResult<char>> {
    //pfa.pad_history();
    // unwraps should be fine because the history is padded ...
    if pfa.history.len() < 4 {
        return None;
    }

    let source_id = vec![*pfa.history.last().unwrap()];
    let dest_id = vec![*pfa.history.get(pfa.history.len() - 3).unwrap()];

    // make sure states exists
    if pfa.has_state(&source_id) && pfa.has_state(&dest_id) {
        return None;
    }

    let mut rand_state = Vec::new();

    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        let c: char = rng.gen();
        if !pfa.alphabet.contains(&c) {
            rand_state.push(c);
            break;
        }
    }

    if rand_state.len() == 0 {
        // can't find random state id
        return None;
    }

    pfa.add_state(&rand_state);

    // update pst
    if let Some(mut root) = pfa.pst_root.as_mut() {
        pst::add_leaf(&mut root, &rand_state);
    }

    let mut additions = Vec::new();
    additions.push(pfa.add_state_transition(
        &rand_state,
        &dest_id,
        0.2 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));
    additions.append(&mut pfa.add_symbol_transition(
        *source_id.first().unwrap(),
        &rand_state,
        0.2 + (rng.gen_range(0.0..20.0) / 100.0),
        false,
    ));

    pfa.rebuild_pst();
    pfa.rebalance();

    // let the outside world know what's happening ...
    Some(PfaOperationResult {
        added_states: vec![rand_state.clone()],
        removed_states: Vec::new(),
        added_transitions: additions,
        removed_transitions: Vec::new(),
        template_symbol: Some(*source_id.first().unwrap()),
        added_symbol: Some(*rand_state.first().unwrap()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immediate_growth_old() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        assert!(!grow_old(pfa).is_none());
    }

    #[test]
    fn test_multi_growth_old() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        for _ in 0..1000 {
            assert!(!pfa.grow_old().is_none());
        }
    }

    #[test]
    fn test_immediate_growth_flower() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        assert!(!pfa.grow_flower().is_none());
    }

    #[test]
    fn test_multi_growth_flower() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        for _ in 0..1000 {
            assert!(!pfa.grow_flower().is_none());
        }
    }

    #[test]
    fn test_immediate_growth_loop() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        assert!(!pfa.grow_loop().is_none());
    }

    #[test]
    fn test_multi_growth_loop() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        for _ in 0..1000 {
            assert!(!pfa.grow_loop().is_none());
        }
    }

    #[test]
    fn test_immediate_growth_triloop() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        assert!(!pfa.grow_triloop().is_none());
    }

    #[test]
    fn test_multi_growth_triloop() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        for _ in 0..1000 {
            assert!(!pfa.grow_triloop().is_none());
        }
    }

    #[test]
    fn test_immediate_growth_quadloop() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        assert!(!pfa.grow_quadloop().is_none());
    }

    #[test]
    fn test_multi_growth_quadloop() {
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        for _ in 0..1000 {
            assert!(!pfa.grow_quadloop().is_none());
        }
    }

    #[test]
    fn test_current_state_consistency_triloop() {
        // only needed for growth methods that
        // remove transitions
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        pfa.grow_triloop();

        assert!(!pfa.current_state.is_none());
        assert!(pfa.labels.contains_key(&pfa.current_state.unwrap()));
    }

    #[test]
    fn test_current_state_consistency_loop() {
        // only needed for growth methods that
        // remove transitions
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        pfa.grow_loop();

        assert!(!pfa.current_state.is_none());
        assert!(pfa.labels.contains_key(&pfa.current_state.unwrap()));
    }

    #[test]
    fn test_current_state_consistency_quadloop() {
        // only needed for growth methods that
        // remove transitions
        let mut rules = Vec::new();

        rules.push(Rule {
            source: "a".chars().collect(),
            symbol: 'a',
            probability: 1.0,
        });

        let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

        pfa.grow_quadloop();

        assert!(!pfa.current_state.is_none());
        assert!(pfa.labels.contains_key(&pfa.current_state.unwrap()));
    }
}
