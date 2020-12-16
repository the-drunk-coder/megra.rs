use crate::builtin_types::*;
use crate::markov_sequence_generator::{Rule, MarkovSequenceGenerator};
use crate::event::*;
use crate::parameter::*;
use crate::parameter::modifier::{
    bounce_modifier::BounceModifier
};
use crate::session::{OutputMode, SyncContext};
use crate::generator::{Generator, haste, relax, grow};
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
    
    Atom::SoundEvent (ev)
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
    
    Atom::SoundEvent (ev)
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

pub fn handle_builtin_mod_event(event_type: &BuiltInParameterEvent, tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    let mut ev = match event_type {
	BuiltInParameterEvent::PitchFrequency(o) => Event::with_name_and_operation("freq".to_string(), *o),
	BuiltInParameterEvent::Attack(o) => Event::with_name_and_operation("atk".to_string(), *o), // milliseconds, not seconds
	BuiltInParameterEvent::Release(o) => Event::with_name_and_operation("rel".to_string(), *o), // milliseconds, not seconds
	BuiltInParameterEvent::Sustain(o) => Event::with_name_and_operation("sus".to_string(), *o), // milliseconds, not seconds
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

    match param_key {
	SynthParameter::Attack => ev.params.insert(param_key, Box::new(get_next_param_with_factor(&mut tail_drain, 100.0, 0.001))),
	SynthParameter::Sustain => ev.params.insert(param_key, Box::new(get_next_param_with_factor(&mut tail_drain, 100.0, 0.001))),
	SynthParameter::Release => ev.params.insert(param_key, Box::new(get_next_param_with_factor(&mut tail_drain, 100.0, 0.001))),
	_ => ev.params.insert(param_key, Box::new(get_next_param(&mut tail_drain, 0.0))),
    };
        
    Atom::SoundEvent (ev)
}


// needs to be made on generator lists first ...
pub fn handle_builtin_gen_mod_fun(gen_mod: &BuiltInGenModFun, tail: &mut Vec<Expr>, _parts_store: &PartsStore) -> Atom {

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
		BuiltInGenModFun::Grow => grow(&mut g.root_generator, &mut g.time_mods, &args),
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
		BuiltInGenModFun::Grow => (grow, args),
	    })
	},
	None => {
	    Atom::Nothing
	}
    } 
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
