use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    character::{is_alphanumeric, is_space},
    combinator::{cut, map, map_res},
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

/// maps an event type (like "bd") to a mapping between keywords and buffer number ...
pub type SampleSet = HashMap<String, Vec<(HashSet<String>, usize)>>;

/// As this doesn't strive to be a turing-complete lisp, we'll start with the basic
/// megra operations, learning and inferring, plus the built-in events
pub enum BuiltIn {
    Learn,
    Infer,
    Sine,
    Saw,
    Rule,
    Clear,
    LoadSample,
}

pub enum Command {
    Clear,
    LoadSample((String, Vec<String>, String)) // set (events), keyword, path
}

pub enum Atom {
    Num(i32),
    Description(String), // pfa descriptions
    Keyword(String),
    Symbol(String),
    Boolean(bool),
    BuiltIn(BuiltIn),
    MarkovSequenceGenerator(MarkovSequenceGenerator),
    Event(Event),
    Rule(Rule),
    Command(Command),
}

pub enum Expr {
    Constant(Atom),
    Custom(String),
    /// (func-name arg1 arg2)
    Application(Box<Expr>, Vec<Expr>),
}


fn parse_builtin<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    // alt gives us the result of first parser that succeeds, of the series of
    // parsers we give it
    alt((	
	// map lets us process the parsed output, in this case we know what we parsed,
	// so we ignore the input and return the BuiltIn directly
	map(tag("learn"), |_| BuiltIn::Learn),
	map(tag("infer"), |_| BuiltIn::Infer),
	map(tag("rule"), |_| BuiltIn::Rule),
	map(tag("sine"), |_| BuiltIn::Sine),
	map(tag("saw"), |_| BuiltIn::Saw),
	map(tag("clear"), |_| BuiltIn::Clear),
	map(tag("load-sample"), |_| BuiltIn::LoadSample),
    ))(i)
}

fn parse_custom<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {    
    map(
	context("custom_fun", cut(alpha1)),
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

fn parse_num<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    alt((
	map_res(digit1, |digit_str: &str| {
	    digit_str.parse::<i32>().map(Atom::Num)
	}),
	map(preceded(tag("-"), digit1), |digit_str: &str| {
	    Atom::Num(-1 * digit_str.parse::<i32>().unwrap())
	}),
    ))(i)
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
	parse_num,
	parse_bool,
	map(parse_builtin, Atom::BuiltIn),	
	parse_keyword,
	parse_symbol,
	parse_string,
    ))(i)
}

/// We then add the Expr layer on top
fn parse_constant<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(parse_atom, |atom| Expr::Constant(atom))(i)
}
/// Before continuing, we need a helper function to parse lists.
/// A list starts with `(` and ends with a matching `)`.
/// By putting whitespace and newline parsing here, we can avoid having to worry about it
/// in much of the rest of the parser.
///
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

/// And that's it!
/// We can now parse our entire lisp language.
///
/// But in order to make it a little more interesting, we can hack together
/// a little interpreter to take an Expr, which is really an
/// [Abstract Syntax Tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree) (AST),
/// and give us something back

/// To start we define a couple of helper functions
fn get_num_from_expr(e: &Expr) -> Option<i32> {
    if let Expr::Constant(Atom::Num(n)) = e {
	Some(*n)
    } else {
	None
    }
}

fn get_bool_from_expr(e: &Expr) -> Option<bool> {
    if let Expr::Constant(Atom::Boolean(b)) = e {
	Some(*b)
    } else {
	None
    }
}

fn get_string_from_expr(e: &Expr) -> Option<String> {
    if let Expr::Constant(Atom::Description(s)) = e {
	Some(s.to_string())
    } else if let Expr::Constant(Atom::Symbol(s)) = e {
	Some(s.to_string())
    } else {
	None
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
			if let Expr::Constant(Atom::Num(n)) = tail_drain.next().unwrap() {
			    dur = n;
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
    Atom::MarkovSequenceGenerator (MarkovSequenceGenerator {
	name: name,
	generator: pfa,
	event_mapping: event_mapping,
	duration_mapping: HashMap::new(),
	modified: false,
	symbol_ages: HashMap::new(),
	default_duration: dur as u64,
	last_transition: None,			    
    })
}

fn handle_infer(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    
    let mut event_mapping = HashMap::<char, Vec<Event>>::new();
    let mut rules = Vec::new();
    
    let mut collect_events = false;
    let mut collect_rules = false;
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
	
	if collect_rules {
	    if let Atom::Rule(s) = c {
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
			if let Expr::Constant(Atom::Num(n)) = tail_drain.next().unwrap() {
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
    Atom::MarkovSequenceGenerator (MarkovSequenceGenerator {
	name: name,
	generator: pfa,
	event_mapping: event_mapping,
	duration_mapping: HashMap::new(),
	modified: false,
	symbol_ages: HashMap::new(),
	default_duration: dur as u64,
	last_transition: None,			    
    })
}

fn handle_saw(tail: &mut Vec<Expr>) -> Atom {

    let mut tail_drain = tail.drain(..);
    
    let mut ev = Event::with_name("saw".to_string());
    ev.tags.push("saw".to_string());
    ev.params.insert("freq".to_string(), Box::new(Parameter::with_value(get_num_from_expr(&tail_drain.next().unwrap()).unwrap() as f32)));
    
    // set some defaults 2
    ev.params.insert("lvl".to_string(), Box::new(Parameter::with_value(0.3)));
    ev.params.insert("atk".to_string(), Box::new(Parameter::with_value(0.01)));
    ev.params.insert("sus".to_string(), Box::new(Parameter::with_value(0.1)));
    ev.params.insert("rel".to_string(), Box::new(Parameter::with_value(0.01)));
    
    while let Some(Expr::Constant(Atom::Keyword(k))) = tail_drain.next() {			    
	ev.params.insert(k, Box::new(Parameter::with_value(get_num_from_expr(&tail_drain.next().unwrap()).unwrap() as f32)));
    }
    
    Atom::Event (ev)
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

fn handle_sine(tail: &mut Vec<Expr>) -> Atom {
    
    let mut tail_drain = tail.drain(..);
    
    let mut ev = Event::with_name("sine".to_string());
    ev.tags.push("sine".to_string());

    // first arg is always freq ...
    ev.params.insert("freq".to_string(),Box::new(Parameter::with_value(get_num_from_expr(&tail_drain.next().unwrap()).unwrap() as f32)));

    // set some defaults 2
    ev.params.insert("lvl".to_string(), Box::new(Parameter::with_value(0.3)));
    ev.params.insert("atk".to_string(), Box::new(Parameter::with_value(0.01)));
    ev.params.insert("sus".to_string(), Box::new(Parameter::with_value(0.1)));
    ev.params.insert("rel".to_string(), Box::new(Parameter::with_value(0.01)));
    
    while let Some(Expr::Constant(Atom::Keyword(k))) = tail_drain.next() {			    
	ev.params.insert(k, Box::new(Parameter::with_value(get_num_from_expr(&tail_drain.next().unwrap()).unwrap() as f32)));
    }
    
    Atom::Event (ev)
}

fn handle_sample(tail: &mut Vec<Expr>, bufnum: usize) -> Atom {
    
    let mut tail_drain = tail.drain(..);
    
    let mut ev = Event::with_name("sampler".to_string());
    ev.tags.push("sampler".to_string());

    ev.params.insert("bufnum".to_string(), Box::new(Parameter::with_value(bufnum as f32)));
    
    // set some defaults 2
    ev.params.insert("lvl".to_string(), Box::new(Parameter::with_value(0.3)));
    ev.params.insert("atk".to_string(), Box::new(Parameter::with_value(0.01)));
    ev.params.insert("sus".to_string(), Box::new(Parameter::with_value(0.1)));
    ev.params.insert("rel".to_string(), Box::new(Parameter::with_value(0.01)));
    ev.params.insert("rate".to_string(), Box::new(Parameter::with_value(1.0)));
    ev.params.insert("lp-dist".to_string(), Box::new(Parameter::with_value(0.0)));
    ev.params.insert("start".to_string(), Box::new(Parameter::with_value(0.0)));
    
    while let Some(Expr::Constant(Atom::Keyword(k))) = tail_drain.next() {			    
	ev.params.insert(k, Box::new(Parameter::with_value(get_num_from_expr(&tail_drain.next().unwrap()).unwrap() as f32)));
    }
    
    Atom::Event (ev)
}

fn handle_rule(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let source_vec:Vec<char> = get_string_from_expr(&tail_drain.next().unwrap()).unwrap().chars().collect();
    let sym_vec:Vec<char> = get_string_from_expr(&tail_drain.next().unwrap()).unwrap().chars().collect();
    Atom::Rule(Rule {
	source: source_vec,
	symbol: sym_vec[0],
	probability: get_num_from_expr(&tail_drain.next().unwrap()).unwrap() as f32 / 100.0,
	duration: get_num_from_expr(&tail_drain.next().unwrap()).unwrap() as u64				
    })
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
			BuiltIn::Learn => handle_learn(&mut reduced_tail),
			BuiltIn::Infer => handle_infer(&mut reduced_tail),
			BuiltIn::Sine => handle_sine(&mut reduced_tail),
			BuiltIn::Saw => handle_saw(&mut reduced_tail),
			BuiltIn::Rule => handle_rule(&mut reduced_tail)		    
		    }))
		},
		Expr::Custom(s) => {
		    if let Some(sample_info) = sample_set.get(&s) {
			// just choose first sample for now ...
			Some(Expr::Constant(handle_sample(&mut reduced_tail, sample_info[0].1)))
		    } else {
			None
		    }
		}, // return custom function
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


