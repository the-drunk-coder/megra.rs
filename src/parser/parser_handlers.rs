use ruffbox_synth::ruffbox::synth::SynthParameter;
use crate::builtin_types::*;
use crate::markov_sequence_generator::{Rule, MarkovSequenceGenerator};
use crate::event::*;
use crate::parameter::*;

use crate::session::SyncContext;
use crate::generator::Generator;

use crate::parser::parser_helpers::*;

use std::collections::{HashMap, HashSet, BTreeSet};
use vom_rs::pfa::Pfa;


pub fn handle_learn(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    
    let mut sample:String = "".to_string();
    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    
    let mut collect_events = false;			
    let mut dur = 200;

    while let Some(Expr::Constant(c)) = tail_drain.next() {
	
	if collect_events {
	    if let Atom::Symbol(ref s) = c {
		let mut ev_vec = Vec::new();
		if let Expr::Constant(Atom::SoundEvent(e)) = tail_drain.next().unwrap() {
		    ev_vec.push(SourceEvent::Sound(e));
		}
		event_mapping.insert(s.chars().next().unwrap(), ev_vec);
		continue;
	    } else {
		collect_events = false;
	    }				    
	}
	
	match c {
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "sample" => {
			if let Expr::Constant(Atom::Description(desc)) = tail_drain.next().unwrap() {
			    sample = desc.to_string();
			}	
		    },
		    "events" => {
			collect_events = true;
			continue;
		    },
		    "dur" => {
			if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
			    dur = n as i32;
			}
		    },
		    _ => println!("{}", k)
		}
	    }
	    _ => println!{"ignored"}
	}
    }
    
    let s_v: std::vec::Vec<char> = sample.chars().collect();
    let pfa = Pfa::<char>::learn(&s_v, 3, 0.01, 30);
    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());
    
    Atom::Generator(Generator {
	id_tags: id_tags,
	root_generator: MarkovSequenceGenerator {
	    name: name,
	    generator: pfa,
	    event_mapping: event_mapping,
	    duration_mapping: HashMap::new(),
	    modified: false,
	    symbol_ages: HashMap::new(),
	    default_duration: dur as u64,
	    last_transition: None,			
	},
	processors: Vec::new(),
	time_mods: Vec::new(),
    })  
}

pub fn handle_infer(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    
    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char,char), Event>::new();
    let mut rules = Vec::new();
    
    let mut collect_events = false;
    let mut collect_rules = false;
    let mut dur:f32 = 200.0;
        
    while let Some(Expr::Constant(c)) = tail_drain.next() {
	
	if collect_events {
	    if let Atom::Symbol(ref s) = c {
		let mut ev_vec = Vec::new();
		match tail_drain.next().unwrap() {
		    Expr::Constant(Atom::SoundEvent(e)) => ev_vec.push(SourceEvent::Sound(e)),
		    Expr::Constant(Atom::ControlEvent(c)) => ev_vec.push(SourceEvent::Control(c)),
		    _ => {}
		}		
		let sym = s.chars().next().unwrap();
		event_mapping.insert(sym, ev_vec);		
		continue;
	    } else {
		collect_events = false;
	    }				    
	}
	
	if collect_rules {
	    if let Atom::Rule(s) = c {
		let mut dur_ev =  Event::with_name("transition".to_string());
		dur_ev.params.insert(SynthParameter::Duration, Box::new(Parameter::with_value(s.duration as f32)));
		duration_mapping.insert((*s.source.last().unwrap(), s.symbol), dur_ev);
		rules.push(s.to_pfa_rule());
		continue;
	    } else {
		collect_rules = false;
	    }				    
	}
	
	match c {
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "rules" => {
			collect_rules = true;
			continue;	
		    },
		    "events" => {
			collect_events = true;
			continue;
		    },
		    "dur" => {
			if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
			    dur = n;
			}
		    },
		    _ => println!("{}", k)
		}
	    }
	    _ => println!{"ignored"}
	}
    }
    
    let pfa = Pfa::<char>::infer_from_rules(&mut rules);
    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());
    
    Atom::Generator(Generator {
	id_tags: id_tags.clone(),
	root_generator: MarkovSequenceGenerator {
	    name: name,
	    generator: pfa,
	    event_mapping: event_mapping,
	    duration_mapping: duration_mapping,
	    modified: false,
	    symbol_ages: HashMap::new(),
	    default_duration: dur as u64,
	    last_transition: None,			    
	},
	processors: Vec::new(),
	time_mods: Vec::new(),
    })        
}

pub fn handle_load_sample(tail: &mut Vec<Expr>) -> Atom {

    let mut tail_drain = tail.drain(..);
    
    let mut collect_keywords = false;
    
    let mut keywords:Vec<String> = Vec::new();
    let mut path:String = "".to_string();
    let mut set:String = "".to_string();
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {

	if collect_keywords {
	    if let Atom::Symbol(ref s) = c {
		keywords.push(s.to_string());
		continue;
	    } else {
		collect_keywords = false;
	    }				    
	}
	
	match c {
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "keywords" => {
			collect_keywords = true;
			continue;	
		    },
		    "set" => {
			if let Expr::Constant(Atom::Symbol(n)) = tail_drain.next().unwrap() {
			    set = n.to_string();
			}
		    },
		    "path" => {
			if let Expr::Constant(Atom::Description(n)) = tail_drain.next().unwrap() {
			    path = n.to_string();
			}
		    },
		    _ => println!("{}", k)
		}
	    }
	    _ => println!{"ignored"}
	}
    }
    
    Atom::Command(Command::LoadSample((set, keywords, path)))
}

pub fn handle_rule(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let source_vec:Vec<char> = get_string_from_expr(&tail_drain.next().unwrap()).unwrap().chars().collect();
    let sym_vec:Vec<char> = get_string_from_expr(&tail_drain.next().unwrap()).unwrap().chars().collect();
    
    Atom::Rule(Rule {
	source: source_vec,
	symbol: sym_vec[0],
	probability: get_float_from_expr(&tail_drain.next().unwrap()).unwrap() as f32 / 100.0,
	duration: get_float_from_expr(&tail_drain.next().unwrap()).unwrap() as u64				
    })
}

pub fn handle_sync_context(tail: &mut Vec<Expr>, parts_store: &PartsStore) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    let active = get_bool_from_expr(&tail_drain.next().unwrap()).unwrap();

    if !active {
	return Atom::SyncContext(SyncContext {
	    name: name,
	    generators: Vec::new(),	    
	    sync_to: None,
	    active: false,
	})
    }

    let mut gens: Vec<Generator> = Vec::new();
    let mut sync_to = None;
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {		
	match c {
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "sync" => {
			if let Expr::Constant(Atom::Symbol(sync)) = tail_drain.next().unwrap() {
			    sync_to = Some(sync);
			}			
		    }
		    _ => {} // ignore
		}
	    },
	    Atom::Symbol(s) => {
		if let Some(kl) = parts_store.get(&s) {
		    let mut klc = kl.clone();
		    for k in klc.iter_mut() {
			k.id_tags.insert(name.clone());
		    }
		    gens.append(&mut klc);
		} else {
		    println!("warning: '{} not defined!", s);
		}
	    },
	    Atom::Generator(mut k) => {
		k.id_tags.insert(name.clone());
		gens.push(k);
	    },
	    Atom::GeneratorList(mut kl) => {
		for k in kl.iter_mut() {
		    k.id_tags.insert(name.clone());
		}
		gens.append(&mut kl);
	    }
	    _ => println!{"ignored"}
	}
    }
    
    Atom::SyncContext(SyncContext {
	name: name,
	generators: gens,
	sync_to: sync_to,
	active: true,
    })
}






pub fn handle_control_event(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut sync_contexts = Vec::new();

    while let Some(Expr::Constant(Atom::SyncContext(s))) = tail_drain.next() {
	sync_contexts.push(s);
    }

    Atom::ControlEvent(ControlEvent {
	tags: HashSet::new(),
	ctx: if sync_contexts.is_empty() { None } else { Some(sync_contexts) },
    })
}

pub fn handle_load_part(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut gens = Vec::new();

    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {
	match c {
	    Atom::Generator(g) => gens.push(g),
	    Atom::GeneratorList(mut gl) => gens.append(&mut gl),
	    _ => {}
	}
    }
    
    Atom::Command(Command::LoadPart((name, gens)))
}
