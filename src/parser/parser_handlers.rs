use crate::builtin_types::*;
use crate::markov_sequence_generator::{Rule, MarkovSequenceGenerator};
use crate::event::*;
use crate::parameter::*;
use crate::session::SyncContext;
use crate::generator::{Generator, haste, relax};
use crate::generator_processor::*;
use crate::parser::parser_helpers::*;

use std::collections::{HashMap, HashSet, BTreeSet};
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
    
    let mut event_mapping = HashMap::<char, Vec<Event>>::new();
    let mut duration_mapping = HashMap::<(char,char), Event>::new();
    let mut rules = Vec::new();
    
    let mut collect_events = false;
    let mut collect_rules = false;
    let mut dur:f32 = 200.0;
    
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {
	
	if collect_events {
	    if let Atom::Symbol(ref s) = c {
		let mut ev_vec = Vec::new();
		if let Expr::Constant(Atom::Event(e)) = tail_drain.next().unwrap() {
		    ev_vec.push(e);
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

pub fn handle_builtin_sound_event(event_type: &BuiltInSoundEvent, tail: &mut Vec<Expr>) -> Atom {
    
    let mut tail_drain = tail.drain(..);
    
    let mut ev = match event_type {
	BuiltInSoundEvent::Sine(o) => Event::with_name_and_operation("sine".to_string(), *o),
	BuiltInSoundEvent::Saw(o) => Event::with_name_and_operation("saw".to_string(), *o),
	BuiltInSoundEvent::Square(o) => Event::with_name_and_operation("sqr".to_string(), *o),
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
	static_val:0.0,
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

    let mut count = 0;
    while let Some(Expr::Constant(c)) = tail_drain.next() {		
	match c {
	    Atom::Generator(mut k) => {
		k.id_tags.insert(format!("{}-{}", name.clone(), count));
		count += 1;
		gens.push(k);
	    }
	    Atom::GeneratorList(mut kl) => {
		for k in kl.iter_mut() {
		    k.id_tags.insert(format!("{}-{}", name.clone(), count));
		    count += 1;
		}
		gens.append(&mut kl);
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

pub fn handle_builtin_mod_event(event_type: &BuiltInParameterEvent, tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    let mut ev = match event_type {
	BuiltInParameterEvent::PitchFrequency(o) => Event::with_name_and_operation("freq".to_string(), *o),
	BuiltInParameterEvent::Attack(o) => Event::with_name_and_operation("atk".to_string(), *o),
	BuiltInParameterEvent::Release(o) => Event::with_name_and_operation("rel".to_string(), *o),
	BuiltInParameterEvent::Sustain(o) => Event::with_name_and_operation("sus".to_string(), *o),
	BuiltInParameterEvent::ChannelPosition(o) => Event::with_name_and_operation("pos".to_string(), *o),    
	BuiltInParameterEvent::Level(o) => Event::with_name_and_operation("lvl".to_string(), *o),
	BuiltInParameterEvent::Duration(o) => Event::with_name_and_operation("dur".to_string(), *o),    
	BuiltInParameterEvent::Reverb(o) => Event::with_name_and_operation("rev".to_string(), *o),
	BuiltInParameterEvent::Delay(o) => Event::with_name_and_operation("del".to_string(), *o),
	BuiltInParameterEvent::LpFreq(o) => Event::with_name_and_operation("lpf".to_string(), *o),
	BuiltInParameterEvent::LpQ(o) => Event::with_name_and_operation("lpq".to_string(), *o),
	BuiltInParameterEvent::LpDist(o) => Event::with_name_and_operation("lpd".to_string(), *o),
	BuiltInParameterEvent::PeakFreq(o) => Event::with_name_and_operation("pff".to_string(), *o),
	BuiltInParameterEvent::PeakQ(o) => Event::with_name_and_operation("pfq".to_string(), *o),
	BuiltInParameterEvent::PeakGain(o) => Event::with_name_and_operation("pfg".to_string(), *o),
	BuiltInParameterEvent::Pulsewidth(o) => Event::with_name_and_operation("pw".to_string(), *o),
	BuiltInParameterEvent::PlaybackStart(o) => Event::with_name_and_operation("start".to_string(), *o),
	BuiltInParameterEvent::PlaybackRate(o) => Event::with_name_and_operation("rate".to_string(), *o),	
    };

    let param_key = match event_type {
	BuiltInParameterEvent::PitchFrequency(_) => SynthParameter::PitchFrequency,
	BuiltInParameterEvent::Attack(_) => SynthParameter::Attack,
	BuiltInParameterEvent::Release(_) => SynthParameter::Release,
	BuiltInParameterEvent::Sustain(_) => SynthParameter::Sustain,
	BuiltInParameterEvent::ChannelPosition(_) => SynthParameter::ChannelPosition,
	BuiltInParameterEvent::Level(_) => SynthParameter::Level,
	BuiltInParameterEvent::Duration(_) => SynthParameter::Duration,
	BuiltInParameterEvent::Reverb(_) => SynthParameter::ReverbMix,
	BuiltInParameterEvent::Delay(_) => SynthParameter::DelayMix,
	BuiltInParameterEvent::LpFreq(_) => SynthParameter::LowpassCutoffFrequency,
	BuiltInParameterEvent::LpQ(_) => SynthParameter::LowpassQFactor,
	BuiltInParameterEvent::LpDist(_) => SynthParameter::LowpassFilterDistortion,
	BuiltInParameterEvent::PeakFreq(_) => SynthParameter::PeakFrequency,
	BuiltInParameterEvent::PeakQ(_) => SynthParameter::PeakQFactor,
	BuiltInParameterEvent::PeakGain(_) => SynthParameter::PeakGain,
	BuiltInParameterEvent::Pulsewidth(_) => SynthParameter::Pulsewidth,
	BuiltInParameterEvent::PlaybackStart(_) => SynthParameter::PlaybackStart,
	BuiltInParameterEvent::PlaybackRate(_) => SynthParameter::PlaybackRate,
    };

    ev.params.insert(param_key, Box::new(get_next_param(&mut tail_drain, 0.0)));
    
    Atom::Event (ev)
}

fn collect_every(tail: &mut Vec<Expr>) -> Box<EveryProcessor> {
    let mut tail_drain = tail.drain(..); 
    let mut proc = EveryProcessor::new();

    let mut last_filters = Vec::new();
    last_filters.push("".to_string());
    
    let mut cur_step = Parameter::with_value(1.0); // if nothing is specified, it's always applied
    let mut gen_mod_funs = Vec::new();
    let mut events = Vec::new();
    let mut collect_filters = false;
        
    while let Some(Expr::Constant(c)) = tail_drain.next() {				
	match c {
	    Atom::GeneratorModifierFunction(g) => {
		gen_mod_funs.push(g);
		collect_filters = false;
	    }
	    Atom::Event(e) => {
		events.push(e);
		collect_filters = false;
	    },
	    Atom::Symbol(s) => {
		if collect_filters {
		    last_filters.push(s)
		}
	    },
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "for" => {
			if !events.is_empty() || !gen_mod_funs.is_empty() {
			    let mut n_mods = Vec::new();
			    n_mods.append(&mut gen_mod_funs);
			    
			    let mut filtered_events = HashMap::new();
			    let mut n_evs = Vec::new();
			    let mut n_filters = Vec::new();
			    n_evs.append(&mut events);
			    n_filters.append(&mut last_filters);
			    filtered_events.insert(n_filters, n_evs);
			    
			    proc.things_to_be_applied.push((cur_step.clone(), filtered_events, n_mods));
			}
			// collect new filters
			collect_filters = true;
		    },
		    "n" => {
			if !events.is_empty() || !gen_mod_funs.is_empty() {
			    let mut n_mods = Vec::new();
			    n_mods.append(&mut gen_mod_funs);
			    
			    let mut filtered_events = HashMap::new();
			    let mut n_evs = Vec::new();
			    let mut n_filters = Vec::new();
			    n_evs.append(&mut events);
			    n_filters.append(&mut last_filters);
			    filtered_events.insert(n_filters, n_evs);
			    
			    proc.things_to_be_applied.push((cur_step.clone(), filtered_events, n_mods));
			}
			// grab new probability
			cur_step = get_next_param(&mut tail_drain, 1.0);
			collect_filters = false;
		    },		    
		    _ => {}
		}
	    },	    
	    _ => {}
	}
    }

    // save last context
    if !events.is_empty() || !gen_mod_funs.is_empty() {			
	let mut filtered_events = HashMap::new();	
	filtered_events.insert(last_filters, events);	
	proc.things_to_be_applied.push((cur_step, filtered_events, gen_mod_funs));
    }
    
    Box::new(proc)
}

fn collect_pear (tail: &mut Vec<Expr>) -> Box<PearProcessor> {
    let mut tail_drain = tail.drain(..);
    let mut proc = PearProcessor::new();

    let mut last_filters = Vec::new();
    last_filters.push("".to_string());
    
    let mut evs = Vec::new();
    let mut collect_filters = false;
    let mut cur_prob = Parameter::with_value(100.0); // if nothing is specified, it's always or prob 100
    
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
		    "p" => {
			// save current context, if something has been found
			if !evs.is_empty() {
			    let mut filtered_events = HashMap::new();
			    let mut n_evs = Vec::new();
			    let mut n_filters = Vec::new();
			    n_evs.append(&mut evs);
			    n_filters.extend_from_slice(&last_filters);
			    filtered_events.insert(n_filters, n_evs);
			    proc.events_to_be_applied.push((cur_prob.clone(), filtered_events));
			}				
			// grab new probability
			cur_prob = get_next_param(&mut tail_drain, 100.0);
			collect_filters = false;
		    },
		    "for" => {
			if !evs.is_empty() {
			    let mut filtered_events = HashMap::new();
			    let mut n_evs = Vec::new();
			    let mut n_filters = Vec::new();
			    n_evs.append(&mut evs);
			    n_filters.append(&mut last_filters);
			    filtered_events.insert(n_filters, n_evs);
			    proc.events_to_be_applied.push((cur_prob.clone(), filtered_events));
			}
			// collect new filters
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

    // save last context
    if !evs.is_empty() {
	let mut filtered_events = HashMap::new();
	filtered_events.insert(last_filters, evs);
	proc.events_to_be_applied.push((cur_prob, filtered_events));
    }	    	    
    Box::new(proc)
}


fn collect_apple (tail: &mut Vec<Expr>) -> Box<AppleProcessor> {
    let mut tail_drain = tail.drain(..); 
    let mut proc = AppleProcessor::new();
            
    let mut cur_prob = Parameter::with_value(100.0); // if nothing is specified, it's always or prob 100
    let mut gen_mod_funs = Vec::new();
        
    while let Some(Expr::Constant(c)) = tail_drain.next() {				
	match c {
	    Atom::GeneratorModifierFunction(g) => {
		gen_mod_funs.push(g);
	    }
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "p" => {
			if !gen_mod_funs.is_empty() {
			    let mut new_mods = Vec::new();
			    new_mods.append(&mut gen_mod_funs);			    
			    proc.modifiers_to_be_applied.push((cur_prob.clone(), new_mods));
			}
			// grab new probability
			cur_prob = get_next_param(&mut tail_drain, 100.0);
		    },		    
		    _ => {}
		}
	    },	    
	    _ => {}
	}
    }

    // save last context
    if !gen_mod_funs.is_empty() {	
	proc.modifiers_to_be_applied.push((cur_prob, gen_mod_funs));
    }
    
    Box::new(proc)
}

pub fn collect_gen_proc(proc_type: &BuiltInGenProc, tail: &mut Vec<Expr>) -> Box<dyn GeneratorProcessor + Send> {
    match proc_type {
	BuiltInGenProc::Pear => collect_pear(tail),
	BuiltInGenProc::Apple => collect_apple(tail),
	BuiltInGenProc::Every => collect_every(tail),
    }        
}

// store list of genProcs in a vec if there's no root gen ???
pub fn handle_builtin_gen_proc(proc_type: &BuiltInGenProc, tail: &mut Vec<Expr>) -> Atom {    
    let last = tail.pop();
    match last {
	Some(Expr::Constant(Atom::Generator(mut g))) => {
	    g.processors.push(collect_gen_proc(proc_type, tail));
	    Atom::Generator(g)
	},
	Some(Expr::Constant(Atom::GeneratorList(mut gl))) => {
	    let gp = collect_gen_proc(proc_type, tail);
	    for gen in gl.iter_mut() {
		gen.processors.push(gp.clone());
	    }	    
	    Atom::GeneratorList(gl)
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

pub fn handle_builtin_gen_mod_fun(gen_mod: &BuiltInGenModFun, tail: &mut Vec<Expr>) -> Atom {

    let last = tail.pop();
    match last {
	Some(Expr::Constant(Atom::Generator(mut g))) => {
	    let mut tail_drain = tail.drain(..); 	    
	    let mut args = Vec::new();

	    while let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
		args.push(f);
	    }

	    match gen_mod {
		BuiltInGenModFun::Haste => haste(&mut g.root_generator, &mut g.time_mods, &args),
		BuiltInGenModFun::Relax => relax(&mut g.root_generator, &mut g.time_mods, &args),
	    }
	    Atom::Generator(g)
	},	
	
	Some(l) => {
	    tail.push(l);

	    let mut tail_drain = tail.drain(..); 	    
	    let mut args = Vec::new();

	    while let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
		args.push(f);
	    }
    
	    Atom::GeneratorModifierFunction (match gen_mod {
		BuiltInGenModFun::Haste => (haste, args),
		BuiltInGenModFun::Relax => (relax, args),
	    })
	},
	None => {
	    Atom::Nothing
	}
    } 
}



pub fn handle_builtin_multiplexer(_mul: &BuiltInMultiplexer, tail: &mut Vec<Expr>) -> Atom {
    let last = tail.pop(); // generator or generator list ...

    let mut gen_proc_list_list = Vec::new();
    
    let mut tail_drain = tail.drain(..);
    while let Some(Expr::Constant(c)) = tail_drain.next() {
	match c {
	    Atom::GeneratorProcessorList(gpl) => {
		gen_proc_list_list.push(gpl);
	    },
	    Atom::GeneratorProcessor(gp) => {
		let mut gpl = Vec::new();
		gpl.push(gp);
		gen_proc_list_list.push(gpl);
	    },
	    Atom::GeneratorModifierFunction(_gm) => {
		//???
	    },
	    _ => { println!("can't multiplex this ..."); },
	}
    }

    let mut gens = Vec::new();

    match last {
	Some(Expr::Constant(Atom::Generator(g))) => {	    
	     // multiplex into duplicates by cloning ...
	    for mut gpl in gen_proc_list_list.drain(..) {
		let mut pclone = g.clone();

		// this isn't super elegant but hey ... 
		for i in 0..100 {
		    let tag = format!("mpx-{}", i);
		    if !pclone.id_tags.contains(&tag) {
			pclone.id_tags.insert(tag);
			break;
		    } 		    
		}
		
		pclone.processors.append(&mut gpl);
		gens.push(pclone);
	     }
  	     gens.push(g);
	 },
	Some(Expr::Constant(Atom::GeneratorList(mut gl))) => {
	    for gen in gl.drain(..) {
		// multiplex into duplicates by cloning ...		
		for gpl in gen_proc_list_list.iter() {
		    let mut pclone = gen.clone();

		    // this isn't super elegant but hey ... 
		    for i in 0..100 {
			let tag = format!("mpx-{}", i);
			if !pclone.id_tags.contains(&tag) {
			    pclone.id_tags.insert(tag);
			    break;
			} 		    
		    }
		    
		    pclone.processors.append(&mut gpl.clone());
		    gens.push(pclone);
		}
		gens.push(gen);
	    }	    
	},
	_ => {}	
    }
            	
    // for xdup, this would be enough ... for xspread etc, we need to prepend another processor ... 
                
    Atom::GeneratorList(gens)
}
