use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alphanumeric1, char, multispace0},
    number::complete::float,
    character::{is_alphanumeric, is_space},
    combinator::{cut, map, map_res, recognize},
    error::{context, VerboseError},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};

use std::collections::{HashMap,HashSet};
use vom_rs::pfa::Pfa;

use crate::markov_sequence_generator::{Rule, MarkovSequenceGenerator};
use crate::event::*;
use crate::parameter::*;
use crate::session::SyncContext;
use crate::generator::Generator;
use crate::generator_processor::*;
use crate::event_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;

/// maps an event type (like "bd") to a mapping between keywords and buffer number ...
pub type SampleSet = HashMap<String, Vec<(HashSet<String>, usize)>>;

// reflect event hierarchy here, like, Tuned, Param, Sample, Noise ?
pub enum BuiltInEvent {
    Level(EventOperation),
    Reverb(EventOperation),
    //LpQ(EventOperation),
    //LpDist(EventOperation),
    Sine(EventOperation),
    Saw(EventOperation),
    Square(EventOperation),
}

pub enum BuiltInDynamicParameter {
    Bounce,
    //Brownian,
    //Oscil,
    //Env,
    // RandRange
}

pub enum BuiltInGenProc {
    Pear,
    // PPear,
    // Apple,
    
}

/// As this doesn't strive to be a turing-complete lisp, we'll start with the basic
/// megra operations, learning and inferring, plus the built-in events
pub enum BuiltIn {
    Learn,
    Infer,    
    Rule,
    Clear,
    Silence,
    LoadSample,
    SyncContext,
    Parameter(BuiltInDynamicParameter),
    SoundEvent(BuiltInEvent),
    ModEvent(BuiltInEvent),
    GenProc(BuiltInGenProc),
}

pub enum Command {
    Clear,
    LoadSample((String, Vec<String>, String)) // set (events), keyword, path
}

pub enum Atom { // atom might not be the right word any longer 
    Float(f32),
    Description(String), // pfa descriptions
    Keyword(String),
    Symbol(String),
    Boolean(bool),
    BuiltIn(BuiltIn),
    MarkovSequenceGenerator(MarkovSequenceGenerator),
    Event(Event),
    Rule(Rule),
    Command(Command),
    SyncContext(SyncContext),
    Generator(Generator),
    GeneratorProcessor(Box<dyn GeneratorProcessor>),
    GeneratorProcessorList(Vec<Box<dyn GeneratorProcessor>>),
    Parameter(Parameter),
    Nothing
}

pub enum Expr {
    Constant(Atom),
    Custom(String),
    Application(Box<Expr>, Vec<Expr>),    
}


fn parse_builtin<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {    
    alt((	
	map(tag("learn"), |_| BuiltIn::Learn),
	map(tag("infer"), |_| BuiltIn::Infer),
	map(tag("rule"), |_| BuiltIn::Rule),	
	map(tag("clear"), |_| BuiltIn::Clear),
	map(tag("load-sample"), |_| BuiltIn::LoadSample),
	map(tag("sx"), |_| BuiltIn::SyncContext),
	map(tag("silence"), |_| BuiltIn::Silence),
	map(tag("~"), |_| BuiltIn::Silence),
	map(tag("bounce"), |_| BuiltIn::Parameter(BuiltInDynamicParameter::Bounce)),
	map(tag("sine"), |_| BuiltIn::SoundEvent(BuiltInEvent::Sine(EventOperation::Replace))),
	map(tag("saw"), |_| BuiltIn::SoundEvent(BuiltInEvent::Saw(EventOperation::Replace))),
	map(tag("sqr"), |_| BuiltIn::SoundEvent(BuiltInEvent::Square(EventOperation::Replace))),
	map(tag("lvl"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Replace))),	
	map(tag("lvl-add"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Add))),
	map(tag("lvl-mul"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Multiply))),
	map(tag("lvl-sub"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Subtract))),
	map(tag("lvl-div"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Divide))),
	map(tag("rev"), |_| BuiltIn::ModEvent(BuiltInEvent::Reverb(EventOperation::Replace))),	
	map(tag("pear"), |_| BuiltIn::GenProc(BuiltInGenProc::Pear)),
    ))(i)
}

fn parse_custom<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {    
    map(
	context("custom_fun", cut(alphanumeric1)),
	|fun_str: &str| Expr::Custom(fun_str.to_string()),
    )(i)
}

/// Our boolean values are also constant, so we can do it the same way
fn parse_bool<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    alt((
	map(tag("#t"), |_| Atom::Boolean(true)),
	map(tag("#f"), |_| Atom::Boolean(false)),
    ))(i)
}

fn parse_keyword<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    map(
	context("keyword", preceded(tag(":"), cut(alphanumeric1))),
	|sym_str: &str| Atom::Keyword(sym_str.to_string()),
    )(i)
}

fn parse_symbol<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    map(
	context("symbol", preceded(tag("'"), cut(alphanumeric1))),
	|sym_str: &str| Atom::Symbol(sym_str.to_string()),
    )(i)
}

fn parse_float<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    map_res(recognize(float), |digit_str: &str| {
	digit_str.parse::<f32>().map(Atom::Float)
    })(i)
}

pub fn valid_char(chr: char) -> bool {
    return
	chr == '~' ||
	chr == '.' ||
	chr == '_' ||
	chr == '/' ||
	chr == '-' ||
	is_alphanumeric(chr as u8) ||
	is_space(chr as u8)
}

fn parse_string<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    map(delimited(
        tag("\""),
        take_while(valid_char),
        tag("\"")), |desc_str: &str| {
	Atom::Description(desc_str.to_string())
    })(i)
}

fn parse_atom<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    alt((	
	parse_bool,
	map(parse_builtin, Atom::BuiltIn),
	parse_float, // parse after builtin, otherwise the beginning of "infer" would be parsed as "inf" (the float val)
	parse_keyword,
	parse_symbol,
	parse_string,
    ))(i)
}

fn parse_constant<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(parse_atom, |atom| Expr::Constant(atom))(i)
}

/// Unlike the previous functions, this function doesn't take or consume input, instead it
/// takes a parsing function and returns a new parsing function.
fn s_exp<'a, O1, F>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>
where
    F: Parser<&'a str, O1, VerboseError<&'a str>>,
{
    delimited(
	char('('),
	preceded(multispace0, inner),
	context("closing paren", cut(preceded(multispace0, char(')')))),
    )
}

/// We can now use our new combinator to define the rest of the `Expr`s.
///
/// Starting with function application, we can see how the parser mirrors our data
/// definitions: our definition is `Application(Box<Expr>, Vec<Expr>)`, so we know
/// that we need to parse an expression and then parse 0 or more expressions, all
/// wrapped in an S-expression.
///
/// `tuple` is used to sequence parsers together, so we can translate this directly
/// and then map over it to transform the output into an `Expr::Application`
fn parse_application<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    let application_inner = map(tuple((alt((parse_expr, parse_custom)), many0(parse_expr))), |(head, tail)| {
	Expr::Application(Box::new(head), tail)
    });
    // finally, we wrap it in an s-expression
    s_exp(application_inner)(i)
}

/// We tie them all together again, making a top-level expression parser!
fn parse_expr<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    preceded(
	multispace0,
	alt((parse_constant, parse_application)),
    )(i)
}

fn get_float_from_expr(e: &Expr) -> Option<f32> {
    match e {
	Expr::Constant(Atom::Float(n)) => Some(*n),
	_ => None
    }  
}

fn get_bool_from_expr(e: &Expr) -> Option<bool> {
    match e {
	Expr::Constant(Atom::Boolean(b)) => Some(*b),
	_ => None
    }	    
}

fn get_string_from_expr(e: &Expr) -> Option<String> {
    match e {
	Expr::Constant(Atom::Description(s)) => Some(s.to_string()),
	Expr::Constant(Atom::Symbol(s)) => Some(s.to_string()),
	_ => None
    }     
}

fn get_keyword_params(params: &mut HashMap<SynthParameter, Box<Parameter>>, tail_drain: &mut std::vec::Drain<Expr>) {
    while let Some(Expr::Constant(Atom::Keyword(k))) = tail_drain.next() {
	params.insert(map_parameter(&k), Box::new(get_next_param(tail_drain, 0.0)));
    }
}

fn get_next_param(tail_drain: &mut std::vec::Drain<Expr>, default: f32) -> Parameter {
    match tail_drain.next() {
	Some(Expr::Constant(Atom::Float(n))) => Parameter::with_value(n),
	Some(Expr::Constant(Atom::Parameter(pl))) => pl,
	_ => Parameter::with_value(default)
    }
}

fn handle_learn(tail: &mut Vec<Expr>) -> Atom {
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

fn handle_infer(tail: &mut Vec<Expr>) -> Atom {
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

fn handle_load_sample(tail: &mut Vec<Expr>) -> Atom {

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

fn handle_builtin_sound_event(event_type: &BuiltInEvent, tail: &mut Vec<Expr>) -> Atom {
    
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


fn handle_sample(tail: &mut Vec<Expr>, bufnum: usize, set: &String, keywords: &HashSet<String>) -> Atom {
    
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

fn handle_builtin_dynamic_parameter(par: &BuiltInDynamicParameter, tail: &mut Vec<Expr>) -> Atom {
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

fn handle_rule(tail: &mut Vec<Expr>) -> Atom {
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

fn handle_sync_context(tail: &mut Vec<Expr>) -> Atom {
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

fn handle_builtin_mod_event(event_type: &BuiltInEvent, tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    let mut ev = match event_type {
	BuiltInEvent::Level(o) => Event::with_name_and_operation("lvl".to_string(), *o),	
	BuiltInEvent::Reverb(o) => Event::with_name_and_operation("rev".to_string(), *o),	
	_ => Event::with_name("lvl".to_string()),
    };

    let param_key = match event_type {
	BuiltInEvent::Level(_) => SynthParameter::Level,
	BuiltInEvent::Reverb(_) => SynthParameter::ReverbMix,
	_ => SynthParameter::Level,
    };

    ev.params.insert(param_key, Box::new(get_next_param(&mut tail_drain, 0.0)));
    
    Atom::Event (ev)
}

fn collect_gen_proc(proc_type: &BuiltInGenProc, tail: &mut Vec<Expr>) -> Box<dyn GeneratorProcessor + Send> {
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
fn handle_builtin_gen_proc(proc_type: &BuiltInGenProc, tail: &mut Vec<Expr>) -> Atom {
        
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

/// This function tries to reduce the AST.
/// This has to return an Expression rather than an Atom because quoted s_expressions
/// can't be reduced
fn eval_expression(e: Expr, sample_set: &SampleSet) -> Option<Expr> {
    match e {
	// Constants and quoted s-expressions are our base-case
	Expr::Constant(_) => Some(e),
	Expr::Custom(_) => Some(e),	
	Expr::Application(head, tail) => {

	    let reduced_head = eval_expression(*head, sample_set)?;

	    let mut reduced_tail = tail
		.into_iter()
		.map(|expr| eval_expression(expr, sample_set))
		.collect::<Option<Vec<Expr>>>()?;

	    match reduced_head {
		Expr::Constant(Atom::BuiltIn(bi)) => {
		    Some(Expr::Constant(match bi {
			BuiltIn::Clear => Atom::Command(Command::Clear),
			BuiltIn::LoadSample => handle_load_sample(&mut reduced_tail),			
			BuiltIn::Silence => Atom::Event(Event::with_name("silence".to_string())),			
			BuiltIn::Rule => handle_rule(&mut reduced_tail),
			BuiltIn::Learn => handle_learn(&mut reduced_tail),
			BuiltIn::Infer => handle_infer(&mut reduced_tail),			
			BuiltIn::SyncContext => handle_sync_context(&mut reduced_tail),
			BuiltIn::Parameter(par) => handle_builtin_dynamic_parameter(&par, &mut reduced_tail),
			BuiltIn::SoundEvent(ev) => handle_builtin_sound_event(&ev, &mut reduced_tail),
			BuiltIn::ModEvent(ev) => handle_builtin_mod_event(&ev, &mut reduced_tail),
			BuiltIn::GenProc(g) => handle_builtin_gen_proc(&g, &mut reduced_tail)	
		    }))
		},
		Expr::Custom(s) => {
		    if let Some(sample_info) = sample_set.get(&s) {
			// just choose first sample for now ...
			Some(Expr::Constant(handle_sample(&mut reduced_tail, sample_info[0].1, &s, &sample_info[0].0)))
		    } else {
			None
		    }
		}, // return custom function,
		_ => {
		    println!("something else");
		    None
		}		    
	    }	    	     
	}
    }
}

/// And we add one more top-level function to tie everything together, letting
/// us call eval on a string directly
pub fn eval_from_str(src: &str, sample_set: &SampleSet) -> Result<Expr, String> {
    parse_expr(src)
	.map_err(|e: nom::Err<VerboseError<&str>>| format!("{:#?}", e))
	.and_then(|(_, exp)| eval_expression(exp, sample_set).ok_or("Eval failed".to_string()))
}


