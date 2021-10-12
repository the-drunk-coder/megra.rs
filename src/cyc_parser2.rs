use parking_lot::Mutex;
use std::{sync, collections::HashMap};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char, multispace0},
    combinator::{cut, map},
    error::{context, VerboseError},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

use ruffbox_synth::ruffbox::synth::SynthParameter;

pub enum CycleParameter {
    Number(f32),
    Symbol(String)
}

// "inner" item
pub enum CycleItem {
    Duration(f32),
    Event((String, Vec<CycleItem>)),
    Parameter(CycleParameter),   
    Nothing
}

// "outer" item that'll be passed to the calling function
pub enum CycleResult {
    SoundEvent(Event),
    Duration(Event)
} 

use crate::builtin_types::*;
use crate::parameter::*;
use crate::parser::*;
use crate::event::*;
use crate::sample_set::SampleSet;
use crate::session::OutputMode;

///////////////////////////
//  CYC NOTATION PARSER  //
///////////////////////////

fn parse_cyc_parameter<'a>(i: &'a str) -> IResult<&'a str, CycleItem, VerboseError<&'a str>> {
    alt((parse_cyc_symbol, parse_cyc_float))(i)
}

fn parse_cyc_symbol<'a>(i: &'a str) -> IResult<&'a str, CycleItem, VerboseError<&'a str>> {
    map(parse_symbol, |s| {
	if let Atom::Symbol(val) = s {
	    CycleItem::Parameter(CycleParameter::Symbol(val))
	} else {
	    CycleItem::Nothing
	}	
    })(i)
}

fn parse_cyc_float<'a>(i: &'a str) -> IResult<&'a str, CycleItem, VerboseError<&'a str>> {
    map(parse_float, |f| {
	if let Atom::Float(val) = f {
	    CycleItem::Parameter(CycleParameter::Number(val))
	} else {
	    CycleItem::Nothing
	}	
    })(i)
}

fn parse_cyc_duration<'a>(i: &'a str) -> IResult<&'a str, CycleItem, VerboseError<&'a str>> {
    map(preceded(tag("/"), parse_float), |f| {
	if let Atom::Float(dur) = f {
	    CycleItem::Duration(dur)
	} else {
	    CycleItem::Nothing
	}	
    })(i)
}

fn parse_cyc_application<'a>(i: &'a str) -> IResult<&'a str, CycleItem, VerboseError<&'a str>> {
    alt((
        map(
            separated_pair(
                map(
		    context("custom_cycle_fun", cut(take_while(valid_fun_name_char))),
		    |fun_str: &str| fun_str.to_string(),
		),
                tag(":"),
                separated_list0(tag(":"), parse_cyc_parameter),
            ),
            |(head, tail)| CycleItem::Event((head, tail)),
        ),
        map(
	    context("custom_cycle_fun", cut(take_while(valid_fun_name_char))),
	    |fun_str: &str| CycleItem::Event((fun_str.to_string(), Vec::new())),
	),
    ))(i)
}

/// We tie them all together again, making a top-level expression parser!
fn parse_cyc_expr<'a>(i: &'a str) -> IResult<&'a str, Vec<CycleItem>, VerboseError<&'a str>> {
    alt((
        delimited(
            char('['),
            preceded(
                multispace0,
                separated_list1(
                    tag(" "),
                    alt((parse_cyc_parameter, parse_cyc_application)),
                ),
            ),
            preceded(multispace0, char(']')),
        ),
        map(
            alt((
                parse_cyc_parameter,
                parse_cyc_duration,
                parse_cyc_application,
            )),
            |x| vec![x],
        ),
    ))(i)
}

/// parse cycle to cycle items ...
fn parse_cyc<'a>(i: &'a str) -> IResult<&'a str, Vec<Vec<CycleItem>>, VerboseError<&'a str>> {
    separated_list1(tag(" "), parse_cyc_expr)(i)
}

/// adapt items to results ...
pub fn eval_cyc_from_str(
    src: &str,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
    template_events: &Vec<String>,
    event_mappings: &HashMap<String, Vec<Event>>
) -> Vec<Vec<CycleResult>> {
    let items = parse_cyc(src.trim())
        .map_err(|e: nom::Err<VerboseError<&str>>| {
            let ret = format!("{:#?}", e);
            println!("{}", ret);
            ret
        });
    
    match items {
	Ok((_, mut i)) => {	    
	    let mut results = Vec::new();
	    let mut item_drain = i.drain(..);
	    
	    while let Some(mut inner) = item_drain.next() { // iterate through cycle positions ...
		let mut inner_drain = inner.drain(..);
		let mut cycle_position = Vec::new();
		let mut template_params = Vec::new(); // collect params for templates ..
		while let Some(item) = inner_drain.next() {
		    match item {
			CycleItem::Duration(d) => {
			    let mut ev = Event::with_name("transition".to_string());
			    ev.params.insert(SynthParameter::Duration, Box::new(Parameter::with_value(d)));
			    cycle_position.push(CycleResult::Duration(ev));
			},
			CycleItem::Event((mut name, pars)) => {
			    // now this might seem odd, but to re-align the positional arguments i'm just reassembling the string
			    // and use the regular parser ... not super elegant, but hey ...
			    for par in pars.iter() {				
				match par {
				    CycleItem::Parameter(CycleParameter::Number(f)) => {
					name = name + " " + &f.to_string();
				    },
				    CycleItem::Parameter(CycleParameter::Symbol(s)) => {
					name = name + " " + s;
				    },
				    _ => { println!("ignore cycle event param") }
				}
				match parse_expr(&name.trim()) {
				    Ok((_, expr)) => {
					if let Some(Expr::Constant(Atom::SoundEvent(e))) = eval_expression(expr, sample_set, out_mode) {
					    cycle_position.push(CycleResult::SoundEvent(e));
					}
				    },
				    Err(_) => { println!("couldn't parse re-assembled cycle event") }
				}								
			    }			    
			},
			CycleItem::Parameter(CycleParameter::Number(f)) => {
			    template_params.push(CycleParameter::Number(f));
			},
			CycleItem::Parameter(CycleParameter::Symbol(s)) => {
			    if let Some(evs) = event_mappings.get(&s) { // mappings have precedence ...
				for ev in evs {
				    // sound event might not be correct here as this could be control events ...
				    cycle_position.push(CycleResult::SoundEvent(ev.clone()));
				}				    
			    } else {
				template_params.push(CycleParameter::Symbol(s));
			    }			    
			}			
			_ => {println!("nothing to be done ...")}
		    }
		}
		if !template_events.is_empty() && !template_params.is_empty() {
		    for t_ev in template_events.iter() {
			let mut ev_name = t_ev.clone();
			for t_par in template_params.iter() {
			    match t_par {
				CycleParameter::Number(f) => {
				    ev_name = ev_name + " " + &f.to_string();
				},
				CycleParameter::Symbol(s) => {
				    ev_name = ev_name + " " + s;
				}
			    }
			}
			match parse_expr(&ev_name.trim()) {
			    Ok((_, expr)) => {
				if let Some(Expr::Constant(Atom::SoundEvent(e))) = eval_expression(expr, sample_set, out_mode) {
				    cycle_position.push(CycleResult::SoundEvent(e));
				}
			    },
			    Err(_) => { println!("couldn't parse re-assembled cycle event") }
			}	
		    }
		}
		results.push(cycle_position);
	    }	    
	    results
	},
	Err(_) => Vec::new()
    }
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_basic_cyc2_float() {
	match parse_cyc_float("100 b") {
	    Ok(o) => {
		println!("{:?}", o.0)
	    },
	    Err(e) => {println!("{:?}", e)}
	}
    }
    
    #[test]
    fn test_basic_cyc_elem() {        
        match parse_cyc("[saw:200]") {
            Ok((_,o)) => match &o[0][0] {
                CycleItem::Event((_, _)) => assert!(true),
                _ => {
                    assert!(false)
                }
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_basic_cyc() {        
        match parse_cyc("saw:200 ~ ~ ~") {
            Ok((_, o)) => {
                assert!(o.len() == 4);

                match &o[0][0] {
                    CycleItem::Event((s, _)) => assert!(s == "saw"),
                    _ => assert!(false),
                }

                match &o[1][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == ""),
                    _ => assert!(false),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_basic_cyc_noparam() {        
        match parse_cyc("saw ~ ~ ~") {
            Ok((_, o)) => {
                assert!(o.len() == 4);

                match &o[0][0] {
                    CycleItem::Event((s, _)) => assert!(s == "saw"),
                    _ => assert!(false),
                }

                match &o[1][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == ""),
                    _ => assert!(false),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_basic_cyc_noparam_dur() {        
        match parse_cyc("saw /100 saw ~ ~") {
            Ok((_, o)) => {
                assert!(o.len() == 5);

		match &o[0][0] {
                    CycleItem::Event((s, _)) => assert!(s == "saw"),
                    _ => assert!(false),
                }

                match &o[1][0] {
                    CycleItem::Duration(d) => assert!(*d == 100.0),
                    _ => assert!(false),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == ""),
                    _ => assert!(false),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }
		
		match &o[4][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_symbol_param_only() {        
        match parse_cyc("'boat ~ ~ ~") {
            Ok((_, o)) => {
                match &o[0][0] {
                    CycleItem::Parameter(CycleParameter::Symbol(s)) => assert!(s == "boat"),
                    _ => assert!(false),
                }

                match &o[1][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == ""),
                    _ => assert!(false),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_float_param_only() {        
        match parse_cyc("200 ~ ~ ~") {
            Ok((_, o)) => {
                match &o[0][0] {
                    CycleItem::Parameter(CycleParameter::Number(f)) => assert!(*f == 200.0),
                    _ => assert!(false),
                }

                match &o[1][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == ""),
                    _ => assert!(false),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => assert!(false),
                }
            }
            Err(_) => assert!(false),
        }
    }
}
