use crate::{
    event::{Event, EventOperation, SourceEvent},
    generator::TimeMod,
    markov_sequence_generator::MarkovSequenceGenerator,
    parameter::Parameter,
};
use rand::seq::SliceRandom;
use ruffbox_synth::ruffbox::synth::SynthParameter;
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

pub fn grow_raw(
    gen: &mut MarkovSequenceGenerator,
    m: &str, // method
    variance: f32,
    keep: &HashSet<SynthParameter>,
    durations: &[Parameter],
) {
    if let Some(result) = match m {
        "flower" => gen.generator.grow_flower(),
        "old" => gen.generator.grow_old(),
        "loop" => gen.generator.grow_loop(),
        "triloop" => gen.generator.grow_triloop(),
        "quadloop" => gen.generator.grow_quadloop(),
        _ => gen.generator.grow_old(),
    } {
        //println!("grow!");
        let template_sym = result.template_symbol.unwrap();
        let added_sym = result.added_symbol.unwrap();
        if let Some(old_evs) = gen.event_mapping.get(&template_sym) {
            let mut new_evs = old_evs.clone();
            for ev in new_evs.iter_mut() {
                match ev {
                    SourceEvent::Sound(s) => s.shake(variance, keep),
                    SourceEvent::Control(_) => {}
                }
            }

            gen.event_mapping.insert(added_sym, new_evs);
            gen.symbol_ages.insert(added_sym, 0);
            // is this ok or should it rather follow the actually added transitions ??
            let mut dur_mapping_to_add = HashMap::new();
            for sym in gen.generator.alphabet.iter() {
                if let Some(dur) = gen.duration_mapping.get(&(*sym, template_sym)) {
                    if !durations.is_empty() {
                        let mut dur_ev = Event::with_name("transition".to_string());
                        let dur_val = durations.choose(&mut rand::thread_rng()).unwrap().clone();
                        //println!("add from stash {} {} {}", sym, added_sym, dur_val.static_val);
                        dur_ev
                            .params
                            .insert(SynthParameter::Duration, Box::new(dur_val));
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
                        dur_ev
                            .params
                            .insert(SynthParameter::Duration, Box::new(dur_val));
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
                            //println!("add from stash {} {} {}", sym, added_sym, dur_val.static_val);
                            dur_ev
                                .params
                                .insert(SynthParameter::Duration, Box::new(dur_val));
                            dur_mapping_to_add.insert((*src, *dest), dur_ev);
                        }
                    }
                }
            }

            for (k, v) in dur_mapping_to_add.drain() {
		//println!("add dur map {:?} {}", k, v.params[&SynthParameter::Duration].static_val);
                gen.duration_mapping.insert(k, v);
            }
        }
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
    }
    // remove eventual duration mappings ?
}

pub fn blur_raw(gen: &mut MarkovSequenceGenerator, factor: f32) {
    gen.generator.blur(factor);
}

pub fn sharpen_raw(gen: &mut MarkovSequenceGenerator, factor: f32) {
    gen.generator.sharpen(factor);
}

pub fn shake_raw(gen: &mut MarkovSequenceGenerator, keep: &HashSet<SynthParameter>, factor: f32) {
    gen.generator.blur(factor);
    for (_, evs) in gen.event_mapping.iter_mut() {
        for ev in evs {
            if let SourceEvent::Sound(e) = ev {
                e.shake(factor, keep)
            }
        }
    }
}

pub fn skip_raw(gen: &mut MarkovSequenceGenerator, times: usize) {
    for _ in 0..times {
        gen.current_events();
        gen.current_transition();
    }
}

pub fn rewind_raw(gen: &mut MarkovSequenceGenerator, states: usize) {
    gen.generator.rewind(states);
}

pub fn rnd_raw(gen: &mut MarkovSequenceGenerator, randomize_chance: f32) {
    if randomize_chance > 0.0 {
        gen.generator.randomize_edges(randomize_chance, randomize_chance);
        gen.generator.rebalance();
    }    
}

pub fn rep_raw(gen: &mut MarkovSequenceGenerator, repetition_chance: f32, max_repetitions: usize) {
    if repetition_chance > 0.0 {
        gen.generator.repeat(repetition_chance, max_repetitions);
    }    
}
