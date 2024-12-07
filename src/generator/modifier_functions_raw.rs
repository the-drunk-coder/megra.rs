use crate::{
    event::{Event, EventOperation, SourceEvent},
    generator::TimeMod,
    markov_sequence_generator::MarkovSequenceGenerator,
    parameter::{DynVal, ParameterValue},
    pfa_growth::*,
    pfa_reverse::*,
    GlobalVariables,
};
use rand::seq::SliceRandom;
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{HashMap, HashSet};

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
    keep: &HashSet<SynthParameterLabel>,
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
    keep: &HashSet<SynthParameterLabel>,
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

            gen.event_mapping
                .insert(added_sym, (new_evs, old_dur.clone()));
            gen.symbol_ages.insert(added_sym, 0);
            /*
                // is this ok or should it rather follow the actually added transitions ??
                let mut dur_mapping_to_add = HashMap::new();
                for sym in gen.generator.alphabet.iter() {
                    if let Some(dur) = gen.duration_mapping.get(&(*sym, template_sym)) {
                        if !durations.is_empty() {
                            let mut dur_ev = Event::with_name("transition".to_string());
                            let dur_val = durations.choose(&mut rand::thread_rng()).unwrap().clone();
                            //println!("add from stash {} {} {}", sym, added_sym, dur_val.static_val);
                            dur_ev.params.insert(
                                SynthParameterLabel::Duration.into(),
                                ParameterValue::Scalar(dur_val),
                            );
                            dur_mapping_to_add.insert((*sym, added_sym), dur_ev);
                        } else {
                            //println!("add from prev {} {}", sym, added_sym);
                            dur_mapping_to_add.insert((*sym, added_sym), dur.clone());
                        }
                    }

                    if let Some(dur) = gen.duration_mapping.get(&(template_sym, *sym)) {
                        if !durations.is_empty() {
                            let mut dur_ev = Event::with_name("transition".to_string());
                            let dur_val = durations.choose(&mut rand::thread_rng()).unwrap().clone();
                            //println!("add from stash {} {} {}", added_sym, sym, dur_val.static_val);
                            dur_ev.params.insert(
                                SynthParameterLabel::Duration.into(),
                                ParameterValue::Scalar(dur_val),
                            );
                            dur_mapping_to_add.insert((added_sym, *sym), dur_ev);
                        } else {
                            //println!("add from prev {} {}", added_sym, sym);
                            dur_mapping_to_add.insert((added_sym, *sym), dur.clone());
                        }
                    }
                }

                // add from the durations stash for newly added information ...
                if !durations.is_empty() {
                    for t in result.added_transitions.iter() {
                        if let Some(src) = t.source.last() {
                            if let Some(dest) = t.destination.last() {
                                let mut dur_ev = Event::with_name("transition".to_string());
                                let dur_val =
                                    durations.choose(&mut rand::thread_rng()).unwrap().clone();
                                //println!("add from stash {} {} {}", src, dest, dur_val.static_val);
                                dur_ev.params.insert(
                                    SynthParameterLabel::Duration.into(),
                                    ParameterValue::Scalar(dur_val),
                                );
                                dur_mapping_to_add.insert((*src, *dest), dur_ev);
                            }
                        }
            }

                }
            */

            //for (k, v) in dur_mapping_to_add.drain() {
            //println!("add dur map {:?} {}", k, v.params[&SynthParameterLabel::Duration].static_val);
            //    gen.duration_mapping.insert(k, v);
            //}
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

pub fn shake_raw(
    gen: &mut MarkovSequenceGenerator,
    keep: &HashSet<SynthParameterLabel>,
    factor: f32,
) {
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
