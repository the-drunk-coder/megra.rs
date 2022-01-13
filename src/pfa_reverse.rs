use vom_rs::pfa::*;

/// This one is tricky because it leaves the PFA in a state that,
/// from a mathematical standpoint, doesn't make much sense.
/// The PST will be invalid, so subsequent growth operations might lead
/// to a nonsensical state.
/// Use with caution !
pub fn reverse_pfa(pfa: &Pfa<char>) -> Pfa<char> {
    let mut reversed_pfa = Pfa::<char> {
        pst_root: None,
        current_state: pfa.current_state,
        current_symbol: pfa.current_symbol,
        ..Default::default()
    };

    // copy states
    for (_, l) in pfa.labels.iter() {
        reversed_pfa.add_state(l);
    }

    // reverse transitions
    for (lh, chn) in pfa.children.iter() {
        let label = pfa.labels[lh].clone(); // temp label
        for ch in chn.iter() {
            reversed_pfa.add_state_transition(&ch.child, &label, ch.prob, false);
        }
    }

    reversed_pfa.rebalance();
    println!("reverse");

    reversed_pfa
}
