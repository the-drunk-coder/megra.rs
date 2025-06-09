use crate::{
    duration_tree::{add_leaf, DurationTreeNode},
    event::{Event, EventOperation, SourceEvent},
    generator::TimeMod,
    markov_sequence_generator::MarkovSequenceGenerator,
    parameter::{DynVal, ParameterAddress},
    pfa_growth::*,
    pfa_reverse::*,
    GlobalVariables,
};
use rand::seq::SliceRandom;
use std::collections::HashSet;

pub fn haste_raw(time_mods: &mut Vec<TimeMod>, v: f32, n: usize) {
    for _ in 0..n {
        //println!("push time mod {}", v);
        time_mods.push(TimeMod {
            val: v,
            op: EventOperation::Multiply,
        });
    }
}

pub fn relax_raw(time_mods: &mut Vec<TimeMod>, v: f32, n: usize) {
    for _ in 0..n {
        time_mods.push(TimeMod {
            val: v,
            op: EventOperation::Divide,
        });
    }
}

pub fn reverse_raw(gen: &mut MarkovSequenceGenerator) {
    gen.generator = reverse_pfa(&gen.generator);
    gen.set_modified();
}

pub fn grown_raw(
    gen: &mut MarkovSequenceGenerator,
    m: &str, // method
    variance: f32,
    keep: &HashSet<ParameterAddress>,
    durations: &[DynVal],
    iterations: usize,
) {
    for _ in 0..iterations {
        grow_raw(gen, m, variance, keep, durations);
    }
}

pub fn grow_raw(
    gen: &mut MarkovSequenceGenerator,
    m: &str, // method
    variance: f32,
    keep: &HashSet<ParameterAddress>,
    durations: &[DynVal],
) {
    if let Some(result) = match m {
        "flower" => grow_flower(&mut gen.generator),
        "old" => grow_old(&mut gen.generator),
        "loop" => grow_loop(&mut gen.generator),
        "triloop" => grow_triloop(&mut gen.generator),
        "quadloop" => grow_quadloop(&mut gen.generator),
        _ => grow_old(&mut gen.generator),
    } {
        //println!("grow!");
        /*
           println!(
           "state history: {}",
           gen.generator.get_state_history_string());
           println!(
           "symbol history: {}",
           gen.generator.get_symbol_history_string());
        */
        let template_sym = result.template_symbol.unwrap();
        let added_sym = result.added_symbol.unwrap();
        //println!("GROW {}\n{}", m, result);

        if let Some((old_evs, old_dur)) = gen.event_mapping.get(&template_sym) {
            let mut new_evs = old_evs.clone();
            for ev in new_evs.iter_mut() {
                match ev {
                    SourceEvent::Sound(s) => s.shake(variance, keep),
                    SourceEvent::Control(_) => {}
                }
            }

            if !durations.is_empty() {
                // this will shake things up a little, even though it's purely
                // intuitive ... there's no real deep thought behind it ...
                if !result.added_transitions.is_empty() {
                    let dur_tree = gen
                        .override_durations
                        .get_or_insert(DurationTreeNode::new(&vec![], None));
                    for t in result.added_transitions.iter() {
                        let mut label = t.source.clone();
                        label.push(t.symbol);
                        let dur_val = durations.choose(&mut rand::thread_rng()).unwrap().clone();
                        add_leaf(dur_tree, &label, Some(dur_val.static_val as u64));
                    }
                }

                let dur_val = durations.choose(&mut rand::thread_rng()).unwrap().clone();
                gen.event_mapping
                    .insert(added_sym, (new_evs, Event::transition(dur_val)));
            } else {
                gen.event_mapping
                    .insert(added_sym, (new_evs, old_dur.clone()));
            }

            gen.symbol_ages.insert(added_sym, 0);
        }
        gen.set_modified();
    } else {
        println!("can't grow!");
    }
}

pub fn shrink_raw(gen: &mut MarkovSequenceGenerator, sym: char, rebalance: bool) {
    // check if it is even present (not removed by previous operation)
    if gen.generator.alphabet.contains(&sym) {
        gen.generator.remove_symbol(sym, rebalance);
        gen.event_mapping.remove(&sym);
        gen.symbol_ages.remove(&sym);
        gen.set_modified();
    }
    // remove eventual duration mappings ?
}

pub fn blur_raw(gen: &mut MarkovSequenceGenerator, factor: f32) {
    gen.generator.blur(factor);
    gen.set_modified();
}

pub fn sharpen_raw(gen: &mut MarkovSequenceGenerator, factor: f32) {
    gen.generator.sharpen(factor);
    gen.set_modified();
}

pub fn shake_raw(gen: &mut MarkovSequenceGenerator, keep: &HashSet<ParameterAddress>, factor: f32) {
    gen.generator.blur(factor);
    for (_, (evs, _)) in gen.event_mapping.iter_mut() {
        for ev in evs {
            if let SourceEvent::Sound(e) = ev {
                e.shake(factor, keep)
            }
        }
    }
    gen.set_modified();
}

pub fn skip_raw(
    gen: &mut MarkovSequenceGenerator,
    times: usize,
    globals: &std::sync::Arc<GlobalVariables>,
) {
    for _ in 0..times {
        gen.current_events(globals);
        gen.current_transition(globals);
    }
}

pub fn rewind_raw(gen: &mut MarkovSequenceGenerator, states: usize) {
    gen.generator.rewind(states);
}

pub fn solidify_raw(gen: &mut MarkovSequenceGenerator, ctx_len: usize) {
    gen.generator.solidify(ctx_len);
    gen.set_modified();
}

pub fn rnd_raw(gen: &mut MarkovSequenceGenerator, randomize_chance: f32) {
    if randomize_chance > 0.0 {
        gen.generator
            .randomize_edges(randomize_chance, randomize_chance);
        gen.generator.rebalance();
        gen.set_modified();
    }
}

pub fn rep_raw(gen: &mut MarkovSequenceGenerator, repetition_chance: f32, max_repetitions: usize) {
    if repetition_chance > 0.0 {
        gen.generator.repeat(repetition_chance, max_repetitions);
        gen.set_modified();
    }
}
