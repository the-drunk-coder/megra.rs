use crate::builtin_types::*;
use crate::markov_sequence_generator::{Rule, MarkovSequenceGenerator};
use crate::event::*;
use crate::parameter::*;
use crate::session::SyncContext;
use crate::generator::Generator;
use crate::generator_processor::*;
use crate::parser::parser_helpers::*;

use std::collections::{HashMap,HashSet};
use vom_rs::pfa::Pfa;
use ruffbox_synth::ruffbox::synth::SynthParameter;


pub fn handle_learn(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    
    let mut sample:String = "".to_string();
    let mut event_mapping = HashMap::<char, Vec<Event>>::new();
    
    let mut collect_events = false;			
    let mut dur = 200;

    while let Some(Expr::Constant(c)) = tail_drain.next() {
	
	if collect_events {
	    if let Atom::Symbol(ref s) = c {
		let mut ev_vec = Vec::new();
		if let Expr::Constant(Atom::Event(e)) = tail_drain.next().unwrap() {
		    ev_vec.push(e);
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
    
    Atom::Generator(Generator {
	name: name.clone(),
	root_generator: MarkovSequenceGenerator {
	    name: name,
	    generator: pfa,
	    event_mapping: event_mapping,
	    duration_mapping: HashMap::new(),
	    modified: false,
	    symbol_ages: HashMap::new(),
	    default_duration: dur as u64,
	    init_symbol: s_v[0],
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
    
    let mut event_mapping = HashMap::<char, Vec<Event>>::new();
    let mut duration_mapping = HashMap::<(char,char), Event>::new();
    let mut rules = Vec::new();
    
    let mut collect_events = false;
    let mut collect_rules = false;
    let mut dur:f32 = 200.0;
    let mut init_sym:Option<char> = None;
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {
	
	if collect_events {
	    if let Atom::Symbol(ref s) = c {
		let mut ev_vec = Vec::new();
		if let Expr::Constant(Atom::Event(e)) = tail_drain.next().unwrap() {
		    ev_vec.push(e);
		}
		let sym = s.chars().next().unwrap();
		event_mapping.insert(sym, ev_vec);
		if init_sym == None {
		    init_sym = Some(sym);
		}
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
    
    Atom::Generator(Generator {
	name: name.clone(),
	root_generator: MarkovSequenceGenerator {
	    name: name,
	    generator: pfa,
	    event_mapping: event_mapping,
	    duration_mapping: duration_mapping,
	    modified: false,
	    symbol_ages: HashMap::new(),
	    default_duration: dur as u64,
	    init_symbol: init_sym.unwrap(),
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

pub fn handle_builtin_sound_event(event_type: &BuiltInEvent, tail: &mut Vec<Expr>) -> Atom {
    
    let mut tail_drain = tail.drain(..);
    
    let mut ev = match event_type {
	BuiltInEvent::Sine(o) => Event::with_name_and_operation("sine".to_string(), *o),
	BuiltInEvent::Saw(o) => Event::with_name_and_operation("saw".to_string(), *o),
	BuiltInEvent::Square(o) => Event::with_name_and_operation("sqr".to_string(), *o),
	_ => Event::with_name("sine".to_string()),
    };

    // first arg is always freq ...
    ev.params.insert(SynthParameter::PitchFrequency, Box::new(Parameter::with_value(get_float_from_expr(&tail_drain.next().unwrap()).unwrap())));

    // set some defaults 2
    ev.params.insert(SynthParameter::Level, Box::new(Parameter::with_value(0.3)));
    ev.params.insert(SynthParameter::Attack, Box::new(Parameter::with_value(0.005)));
    ev.params.insert(SynthParameter::Sustain, Box::new(Parameter::with_value(0.1)));
    ev.params.insert(SynthParameter::Release, Box::new(Parameter::with_value(0.01)));
    ev.params.insert(SynthParameter::ChannelPosition, Box::new(Parameter::with_value(0.00)));
    
    get_keyword_params(&mut ev.params, &mut tail_drain);
    
    Atom::Event (ev)
}


pub fn handle_sample(tail: &mut Vec<Expr>, bufnum: usize, set: &String, keywords: &HashSet<String>) -> Atom {
    
    let mut tail_drain = tail.drain(..);
    
    let mut ev = Event::with_name("sampler".to_string());
    ev.tags.insert(set.to_string());
    for k in keywords.iter() {
	ev.tags.insert(k.to_string());
    }
    
    ev.params.insert(SynthParameter::SampleBufferNumber, Box::new(Parameter::with_value(bufnum as f32)));
    
    // set some defaults
    ev.params.insert(SynthParameter::Level, Box::new(Parameter::with_value(0.3)));
    ev.params.insert(SynthParameter::Attack, Box::new(Parameter::with_value(0.005)));
    ev.params.insert(SynthParameter::Sustain, Box::new(Parameter::with_value(0.1)));
    ev.params.insert(SynthParameter::Release, Box::new(Parameter::with_value(0.01)));
    ev.params.insert(SynthParameter::ChannelPosition, Box::new(Parameter::with_value(0.00)));
    ev.params.insert(SynthParameter::PlaybackRate, Box::new(Parameter::with_value(1.0)));
    ev.params.insert(SynthParameter::LowpassFilterDistortion, Box::new(Parameter::with_value(0.0)));
    ev.params.insert(SynthParameter::PlaybackStart, Box::new(Parameter::with_value(0.0)));    
    
    get_keyword_params(&mut ev.params, &mut tail_drain);
    
    Atom::Event (ev)
}

pub fn handle_builtin_dynamic_parameter(par: &BuiltInDynamicParameter, tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
            
    Atom::Parameter(Parameter {
	val:0.0,
	modifier: Some(Box::new(
	    match par {
		BuiltInDynamicParameter::Bounce => {
		    let min = get_next_param(&mut tail_drain, 0.0);    
		    let max = get_next_param(&mut tail_drain, 0.0);    
		    let steps = get_next_param(&mut tail_drain, 0.0);
		    BounceModifier {                        
			min: min,
			max: max,            
			steps: steps,
			step_count: (0.0).into(),
		    }
		}
	    }	    
	))
    })
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

pub fn handle_sync_context(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    let _active = get_bool_from_expr(&tail_drain.next().unwrap()).unwrap();
    let mut gens: Vec<Generator> = Vec::new();
    let mut _syncs: Vec<String> = Vec::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {		
	match c {
	    Atom::Generator(k) => {
		gens.push(k);
	    }
	    _ => println!{"ignored"}
	}
    }
    
    Atom::SyncContext(SyncContext {
	name: name,
	generators: gens,
	synced: _syncs
    })
}

pub fn handle_builtin_mod_event(event_type: &BuiltInEvent, tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    let mut ev = match event_type {
	BuiltInEvent::Level(o) => Event::with_name_and_operation("lvl".to_string(), *o),	
	BuiltInEvent::Reverb(o) => Event::with_name_and_operation("rev".to_string(), *o),
	BuiltInEvent::Duration(o) => Event::with_name_and_operation("dur".to_string(), *o),	
	_ => Event::with_name("lvl".to_string()),
    };

    let param_key = match event_type {
	BuiltInEvent::Level(_) => SynthParameter::Level,
	BuiltInEvent::Reverb(_) => SynthParameter::ReverbMix,
	BuiltInEvent::Duration(_) => SynthParameter::Duration,
	_ => SynthParameter::Level,
    };

    ev.params.insert(param_key, Box::new(get_next_param(&mut tail_drain, 0.0)));
    
    Atom::Event (ev)
}

pub fn collect_gen_proc(proc_type: &BuiltInGenProc, tail: &mut Vec<Expr>) -> Box<dyn GeneratorProcessor + Send> {
    let mut tail_drain = tail.drain(..);
    Box::new(match proc_type {
	BuiltInGenProc::Pear => {
	    let mut proc = PearProcessor::new();

	    let mut last_filters = Vec::new();
	    last_filters.push("".to_string());
	    
	    let mut evs = Vec::new();
	    let mut collect_filters = false;
	    
	    while let Some(Expr::Constant(c)) = tail_drain.next() {				
		match c {
		    Atom::Event(e) => {
			evs.push(e);
			if collect_filters {
			    collect_filters = false;
			}
		    },
		    Atom::Keyword(k) => {
			match k.as_str() {
			    "for" => {
				let mut n_evs = Vec::new();
				let mut n_filters = Vec::new();
				n_evs.append(&mut evs);
				n_filters.append(&mut last_filters);
				proc.events_to_be_applied.insert(n_filters, n_evs);
				collect_filters = true;
			    },
			    _ => {}
			}
		    },
		    Atom::Symbol(s) => {
			if collect_filters {
			    last_filters.push(s)
			}
		    },
		    _ => {}
		}
	    }

	    proc.events_to_be_applied.insert(last_filters, evs);	    	    
	    proc
	}
    })        
}
// store list of genProcs in a vec if there's no root gen ???
pub fn handle_builtin_gen_proc(proc_type: &BuiltInGenProc, tail: &mut Vec<Expr>) -> Atom {
        
    let last = tail.pop();
    match last {
	Some(Expr::Constant(Atom::Generator(mut g))) => {
	    g.processors.push(collect_gen_proc(proc_type, tail));
	    Atom::Generator(g)
	},
	Some(Expr::Constant(Atom::GeneratorProcessor(gp)))=> {
	    let mut v = Vec::new();
	    v.push(gp);
	    v.push(collect_gen_proc(proc_type, tail));
	    Atom::GeneratorProcessorList(v)
	},
	Some(Expr::Constant(Atom::GeneratorProcessorList(mut l)))=> {
	    l.push(collect_gen_proc(proc_type, tail));
	    Atom::GeneratorProcessorList(l)
	},
	Some(l) => {
	    tail.push(l);
	    Atom::GeneratorProcessor(collect_gen_proc(proc_type, tail))
	},
	None => {
	    Atom::Nothing
	}
    }    
}