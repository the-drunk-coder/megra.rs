use nom::{
    branch::alt,
    bytes::complete::tag,            
    combinator::map,
    error::VerboseError,
    multi::{separated_list0, separated_list1},
    sequence::{preceded, separated_pair},
    IResult,
};

use crate::builtin_types::*;
use crate::parser::*;
use crate::session::OutputMode;
use crate::sample_set::SampleSet;

///////////////////////////
//  CYC NOTATION PARSER  //
///////////////////////////


fn parse_cyc_event<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {        
    map(map(parse_events, Atom::BuiltIn), |atom| Expr::Constant(atom))(i)
}

/// atoms within the cyc sublanguage
fn parse_cyc_atom<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    alt((	
	parse_bool,
	parse_float, // parse after builtin, otherwise the beginning of "infer" would be parsed as "inf" (the float val)
	parse_symbol,
    ))(i)
}

fn parse_cyc_constant<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(parse_cyc_atom, |atom| Expr::Constant(atom))(i)
}

fn parse_cyc_application<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    alt((map(separated_pair(alt((parse_cyc_event, parse_custom)), tag(":"), separated_list0(tag(":"), parse_cyc_constant)),
	     |(head, tail)| {
		 Expr::Application(Box::new(head), tail)
	     }),
	 map(alt((parse_cyc_event, parse_custom)), |head| { Expr::Application(Box::new(head), Vec::new())})
	 ))(i)    
}

fn parse_cyc_param<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    preceded(tag(":"), parse_cyc_constant)(i)
}

/// We tie them all together again, making a top-level expression parser!
fn parse_cyc_expr<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    alt((parse_cyc_param, parse_cyc_application))(i)
}

fn parse_cyc<'a>(i: &'a str) -> IResult<&'a str, Vec<Expr>, VerboseError<&'a str>> {
    separated_list1(tag(" "), parse_cyc_expr)(i)
}

/// eval cyc substrings ...
pub fn eval_cyc_from_str(src: &str, sample_set: &SampleSet, parts_store: &PartsStore, out_mode: OutputMode) -> Result<Vec<Option<Expr>>, String> {
    parse_cyc(src)
	.map_err(|e: nom::Err<VerboseError<&str>>| format!("{:#?}", e))
	.and_then(|(_, exps)| Ok(exps.into_iter().map(|exp| eval_expression(exp, sample_set, parts_store, out_mode)).collect()))
}


// TEST TEST TEST 
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;            
    
    #[test]
    fn test_basic_cyc_elem() {
	let sample_set = SampleSet::new();
	let parts_store = PartsStore::new();
	
	match eval_cyc_from_str("saw:200", &sample_set, &parts_store, OutputMode::Stereo) {
	    Ok(o) => {
		match &o[0] {
		    Some(Expr::Constant(Atom::SoundEvent(_))) => assert!(true),
		    _ => {
			assert!(false)
		    }
		}
	    },
	    Err(_) => assert!(false),
	}
    }

    #[test]
    fn test_basic_cyc() {
	let sample_set = SampleSet::new();
	let parts_store = PartsStore::new();
	
	match eval_cyc_from_str("saw:200 ~ ~ ~", &sample_set, &parts_store, OutputMode::Stereo) {
	    Ok(o) => {
		assert!(o.len() == 4);
		
		match &o[0] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "saw"),
		    _ => assert!(false)					    
		}

		match &o[1] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
		    _ => assert!(false)
		}

		match &o[2] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
		    _ => assert!(false)
		}
		
		match &o[3] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
		    _ => assert!(false)
		}
	    },
	    Err(_) => assert!(false),
	}
    }

    #[test]
    fn test_basic_cyc_noparam() {
	let sample_set = SampleSet::new();
	let parts_store = PartsStore::new();
	
	match eval_cyc_from_str("saw ~ ~ ~", &sample_set, &parts_store, OutputMode::Stereo) {
	    Ok(o) => {
		assert!(o.len() == 4);
		
		match &o[0] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "saw"),
		    _ => assert!(false)					    
		}

		match &o[1] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
		    _ => assert!(false)
		}

		match &o[2] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
		    _ => assert!(false)
		}
		
		match &o[3] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
		    _ => assert!(false)
		}
	    },
	    Err(_) => assert!(false),
	}
    }

    #[test]
    fn test_param_only() {
	let sample_set = SampleSet::new();
	let parts_store = PartsStore::new();
	
	match eval_cyc_from_str(":200 ~ ~ ~", &sample_set, &parts_store, OutputMode::Stereo) {
	    Ok(o) => {
		match &o[0] {
		    Some(Expr::Constant(Atom::Float(f))) => assert!(*f == 200.0),
		    _ => assert!(false)					    
		}

		match &o[1] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
		    _ => assert!(false)
		}

		match &o[2] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
		    _ => assert!(false)
		}
		
		match &o[3] {
		    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
		    _ => assert!(false)
		}
	    },
	    Err(_) => assert!(false),
	}
    }
}



