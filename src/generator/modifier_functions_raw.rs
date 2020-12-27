use std::collections::HashMap;
use crate::{event::{EventOperation, SourceEvent},
	    generator::TimeMod,
	    markov_sequence_generator::MarkovSequenceGenerator};

pub fn haste_raw(time_mods: &mut Vec<TimeMod>,
		 v: f32,
		 n: usize) {    
    for _ in 0..n {
	time_mods.push(TimeMod{
	    val: v,
	    op: EventOperation::Multiply		
	});
    }    
}

pub fn relax_raw(time_mods: &mut Vec<TimeMod>,
		 v: f32,
		 n: usize) {    
    for _ in 0..n {
	time_mods.push(TimeMod{
	    val: v,
	    op: EventOperation::Divide		
	});
    }    
}

pub fn grow_raw(gen: &mut MarkovSequenceGenerator,
		m: &String, // method
		variance: f32) {
       
    if let Some(result) = match m.as_str() {
	"flower" => gen.generator.grow_flower(),
	"old" => gen.generator.grow_old(),
	_ => gen.generator.grow_old(),
    } {
	//println!("grow!");
	let template_sym = result.template_symbol.unwrap();
	let added_sym = result.added_symbol.unwrap();
	if let Some(old_evs) = gen.event_mapping.get(&template_sym) {
	    let mut new_evs = old_evs.clone();
	    for ev in new_evs.iter_mut() {
		match ev {
		    SourceEvent::Sound(s) => s.shake(variance),
		    SourceEvent::Control(_) => {}
		}
	    }
	    
	    gen.event_mapping.insert(added_sym, new_evs);
	    gen.symbol_ages.insert(added_sym, 0);
	    // is this ok or should it rather follow the actually added transitions ??
	    let mut dur_mapping_to_add = HashMap::new();
	    for sym in gen.generator.alphabet.iter() {
		if let Some(dur) = gen.duration_mapping.get(&(*sym, template_sym)) {
		    dur_mapping_to_add.insert((*sym, added_sym), dur.clone());
		}
		if let Some(dur) = gen.duration_mapping.get(&(template_sym, *sym)) {
		    dur_mapping_to_add.insert((added_sym, *sym), dur.clone());
		}	   	    
	    }
	    for (k, v) in dur_mapping_to_add.drain() {
		gen.duration_mapping.insert(k,v);
	    }
	}    	    	    
    } else {
	println!("can't grow!");
    }
}    

pub fn shrink_raw(gen: &mut MarkovSequenceGenerator,
		  sym: char,
		  rebalance: bool) {
    // check if it is even present (not removed by previous operation)
    if gen.generator.alphabet.contains(&sym) {
	gen.generator.remove_symbol(sym, rebalance);
	gen.event_mapping.remove(&sym);
	gen.symbol_ages.remove(&sym);
    }    
}
