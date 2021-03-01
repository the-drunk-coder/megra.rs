use parking_lot::Mutex;
use std::sync;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::map,
    error::VerboseError,
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

use crate::builtin_types::*;
use crate::parser::*;
use crate::sample_set::SampleSet;
use crate::session::OutputMode;

///////////////////////////
//  CYC NOTATION PARSER  //
///////////////////////////

fn parse_cyc_event<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(map(parse_events, Atom::BuiltIn), Expr::Constant)(i)
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
    map(parse_cyc_atom, Expr::Constant)(i)
}

fn parse_cyc_symbol<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(parse_symbol, Expr::Constant)(i)
}

fn parse_cyc_float<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(parse_float, Expr::Constant)(i)
}

fn parse_cyc_application<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    alt((
        map(
            separated_pair(
                alt((parse_cyc_event, parse_custom)),
                tag(":"),
                separated_list0(tag(":"), parse_cyc_constant),
            ),
            |(head, tail)| Expr::Application(Box::new(head), tail),
        ),
        map(alt((parse_cyc_event, parse_custom)), |head| {
            Expr::Application(Box::new(head), Vec::new())
        }),
    ))(i)
}

fn parse_cyc_param<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    preceded(tag(":"), parse_cyc_constant)(i)
}

/// We tie them all together again, making a top-level expression parser!
fn parse_cyc_expr<'a>(i: &'a str) -> IResult<&'a str, Vec<Expr>, VerboseError<&'a str>> {
    alt((
        delimited(
            char('['),
            preceded(
                multispace0,
                separated_list1(
                    tag(" "),
                    alt((parse_cyc_symbol, parse_cyc_param, parse_cyc_application)),
                ),
            ),
            preceded(multispace0, char(']')),
        ),
        map(
            alt((
                parse_cyc_symbol,
                parse_cyc_float,
                parse_cyc_param,
                parse_cyc_application,
            )),
            |x| vec![x],
        ),
    ))(i)
}

fn parse_cyc<'a>(i: &'a str) -> IResult<&'a str, Vec<Vec<Expr>>, VerboseError<&'a str>> {
    separated_list1(tag(" "), parse_cyc_expr)(i)
}

/// eval cyc substrings ...
pub fn eval_cyc_from_str(
    src: &str,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
) -> Result<Vec<Vec<Option<Expr>>>, String> {
    parse_cyc(src.trim())
        .map_err(|e: nom::Err<VerboseError<&str>>| {
            let ret = format!("{:#?}", e);
            println!("{}", ret);
            ret
        })
        .map(|(_, exp_vecs)| {
            exp_vecs
                .into_iter()
                .map(|exp_vec| {
                    let mut eval_exps = Vec::new();
                    for exp in exp_vec.into_iter() {
                        eval_exps.push(eval_expression(exp, sample_set, out_mode));
                    }
                    eval_exps
                })
                .collect()
        })
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_basic_cyc_elem() {
        let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));

        match eval_cyc_from_str("[saw:200]", &sample_set, OutputMode::Stereo) {
            Ok(o) => match &o[0][0] {
                Some(Expr::Constant(Atom::SoundEvent(_))) => assert!(true),
                _ => {
                    assert!(false)
                }
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_basic_cyc() {
        let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));

        match eval_cyc_from_str("saw:200 ~ ~ ~", &sample_set, OutputMode::Stereo) {
            Ok(o) => {
                assert!(o.len() == 4);

                match &o[0][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "saw"),
                    _ => assert!(false),
                }

                match &o[1][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
                    _ => assert!(false),
                }

                match &o[2][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
                    _ => assert!(false),
                }

                match &o[3][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
                    _ => assert!(false),
                }
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_basic_cyc_noparam() {
        let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));

        match eval_cyc_from_str("saw ~ ~ ~", &sample_set, OutputMode::Stereo) {
            Ok(o) => {
                assert!(o.len() == 4);

                match &o[0][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "saw"),
                    _ => assert!(false),
                }

                match &o[1][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
                    _ => assert!(false),
                }

                match &o[2][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
                    _ => assert!(false),
                }

                match &o[3][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
                    _ => assert!(false),
                }
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_param_only() {
        let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));

        match eval_cyc_from_str(":200 ~ ~ ~", &sample_set, OutputMode::Stereo) {
            Ok(o) => {
                match &o[0][0] {
                    Some(Expr::Constant(Atom::Float(f))) => assert!(*f == 200.0),
                    _ => assert!(false),
                }

                match &o[1][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
                    _ => assert!(false),
                }

                match &o[2][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
                    _ => assert!(false),
                }

                match &o[3][0] {
                    Some(Expr::Constant(Atom::SoundEvent(e))) => assert!(e.name == "silence"),
                    _ => assert!(false),
                }
            }
            Err(_) => assert!(false),
        }
    }
}
