use std::collections::{HashMap, BTreeSet};
use vom_rs::pfa::Pfa;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use crate::builtin_types::*;
use crate::markov_sequence_generator::{Rule, MarkovSequenceGenerator};
use crate::event::*;
use crate::parameter::*;
use crate::generator::Generator;
use crate::parser::parser_helpers::*;
use crate::cyc_parser;
use crate::session::OutputMode;
use crate::sample_set::SampleSet;

pub fn construct_learn(tail: &mut Vec<Expr>) -> Atom {
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

pub fn construct_infer(tail: &mut Vec<Expr>) -> Atom {
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

pub fn construct_nucleus(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char,char), Event>::new();
    let mut rules = Vec::new();
    
    let mut dur:f32 = 200.0;
    let mut ev_vec = Vec::new();
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {
			
	match c {
	    Atom::SoundEvent(e) => ev_vec.push(SourceEvent::Sound(e)),
	    Atom::ControlEvent(c) => ev_vec.push(SourceEvent::Control(c)),
	    Atom::Keyword(k) => {
		match k.as_str() {		    
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

    event_mapping.insert('a', ev_vec);

    let mut dur_ev =  Event::with_name("transition".to_string());
    dur_ev.params.insert(SynthParameter::Duration, Box::new(Parameter::with_value(dur)));
    duration_mapping.insert(('a','a'), dur_ev);
    // one rule to rule them all
    rules.push(Rule {
	source: vec!['a'],
	symbol: 'a',
	probability: 1.0,
	duration: dur as u64,
    }.to_pfa_rule());
    
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

pub fn construct_rule(tail: &mut Vec<Expr>) -> Atom {
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

pub fn construct_cycle(tail: &mut Vec<Expr>, sample_set: &SampleSet, parts_store: &PartsStore, out_mode: OutputMode) -> Atom {

    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char,char), Event>::new();    
    
    let mut dur:f32 = 200.0;
    let mut ev_vec = Vec::new();
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {
			
	match c {
	    Atom::Keyword(k) => {
		match k.as_str() {		    
		    "dur" => {
			if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
			    dur = n;
			}
		    },		    
		    _ => println!("{}", k)
		}
		
	    },
	    Atom::Description(d) => {
		let parsed_cycle = cyc_parser::eval_cyc_from_str(&d, sample_set, parts_store, out_mode);
		match parsed_cycle {
		    Ok(mut c) => {
			let mut cycle_drain = c.drain(..);
			while let Some(Some(Expr::Constant(cc))) = cycle_drain.next() {
			    
			    match cc {
				Atom::SoundEvent(e) => {				  
				    ev_vec.push(e)
				},				
				_ => {/* ignore */}
			    }
			}
		    },
		    _ => {
			println!("couldn't parse cycle: {}", d);
		    }		    
		} 		
	    }
	    _ => println!{"ignored"}
	}
    }
    
    // collect cycle rules 
    let mut rules = Vec::new();
    let mut last_char:char = '!';
    let first_char = last_char;

    let mut count = 0;
    let num_events = ev_vec.len();
    for ev in ev_vec.drain(..) {
	let next_char:char = std::char::from_u32(last_char as u32 + 1).unwrap();
	
	event_mapping.insert(last_char, vec![SourceEvent::Sound(ev)]);
	
	let mut dur_ev =  Event::with_name("transition".to_string());
	dur_ev.params.insert(SynthParameter::Duration, Box::new(Parameter::with_value(dur)));
	duration_mapping.insert((last_char, next_char), dur_ev);

	if count < num_events - 1 {	    
	    rules.push(Rule {
		source: vec![last_char],
		symbol: next_char,
		probability: 1.0,
		duration: dur as u64,
	    }.to_pfa_rule());
	    
	    last_char = next_char;
	}

	count += 1;
    }
    
    // close the cycle 
    let mut dur_ev =  Event::with_name("transition".to_string());
    dur_ev.params.insert(SynthParameter::Duration, Box::new(Parameter::with_value(dur)));
    duration_mapping.insert((last_char, first_char), dur_ev);
    
    rules.push(Rule {
	source: vec![last_char],
	symbol: first_char,
	probability: 1.0,
	duration: dur as u64,
    }.to_pfa_rule());
    
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

pub fn handle(constructor_type: &BuiltInConstructor, tail: &mut Vec<Expr>, sample_set: &SampleSet, parts_store: &PartsStore, out_mode: OutputMode) -> Atom {
    match constructor_type {
	BuiltInConstructor::Infer => construct_infer(tail),
	BuiltInConstructor::Learn => construct_learn(tail),
	BuiltInConstructor::Rule => construct_rule(tail),
	BuiltInConstructor::Nucleus => construct_nucleus(tail),
	BuiltInConstructor::Cycle => construct_cycle(tail, sample_set, parts_store, out_mode),
    }        
}
