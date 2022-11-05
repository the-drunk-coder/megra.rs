use crate::builtin_types::*;
use crate::event::*;
use crate::parser::*;
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::OutputMode;

use parking_lot::Mutex;
use std::{collections::HashMap, sync};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char, multispace0},
    character::is_alphanumeric,
    combinator::{cut, map},
    error::{context, VerboseError},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

pub enum CycleParameter {
    Number(f32),
    Symbol(String),
}

// "inner" item
pub enum CycleItem {
    Duration(f32),
    Event((String, Vec<CycleItem>)),
    Parameter(CycleParameter),
    NamedParameter((String, CycleParameter)),
    Nothing,
}

// "outer" item that'll be passed to the calling function
pub enum CycleResult {
    SoundEvent(Event),
    ControlEvent(ControlEvent),
    Duration(f32),
}

///////////////////////////
//  CYC NOTATION PARSER  //
///////////////////////////

fn parse_cyc_parameter<'a>(i: &'a str) -> IResult<&'a str, CycleItem, VerboseError<&'a str>> {
    alt((parse_cyc_symbol, parse_cyc_float))(i)
}

fn parse_cyc_named_parameter<'a>(i: &'a str) -> IResult<&'a str, CycleItem, VerboseError<&'a str>> {
    map(
        separated_pair(
            map(
                context(
                    "custom_cycle_fun",
                    cut(take_while(valid_function_name_char)),
                ),
                |fun_str: &str| fun_str.to_string(),
            ),
            tag("="),
            alt((parse_cyc_symbol, parse_cyc_float)),
        ),
        |(head, tail)| {
            if let CycleItem::Parameter(p) = tail {
                CycleItem::NamedParameter((head, p))
            } else {
                CycleItem::Nothing
            }
        },
    )(i)
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

/// valid chars for a function name
fn valid_cycle_fun_name_char(chr: char) -> bool {
    chr == '_' || chr == '~' || chr == '-' || is_alphanumeric(chr as u8)
}

fn parse_cyc_application<'a>(i: &'a str) -> IResult<&'a str, CycleItem, VerboseError<&'a str>> {
    alt((
        map(
            separated_pair(
                map(
                    context(
                        "custom_cycle_fun",
                        cut(take_while(valid_function_name_char)),
                    ),
                    |fun_str: &str| fun_str.to_string(),
                ),
                tag(":"),
                separated_list0(
                    tag(":"),
                    alt((parse_cyc_parameter, parse_cyc_named_parameter)),
                ),
            ),
            |(head, tail)| CycleItem::Event((head, tail)),
        ),
        map(
            context(
                "custom_cycle_fun",
                cut(take_while(valid_cycle_fun_name_char)),
            ),
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
                separated_list1(tag(" "), alt((parse_cyc_parameter, parse_cyc_application))),
            ),
            preceded(multispace0, char(']')),
        ),
        separated_list1(
            tag(":"),
            alt((parse_cyc_parameter, parse_cyc_named_parameter)),
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
    functions: &FunctionMap,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    out_mode: OutputMode,
    template_events: &[String],
    event_mappings: &HashMap<String, Vec<SourceEvent>>,
    global_parameters: &sync::Arc<GlobalParameters>,
) -> Vec<Vec<CycleResult>> {
    let items = parse_cyc(src.trim()).map_err(|e: nom::Err<VerboseError<&str>>| {
        let ret = format!("{:#?}", e);
        println!("{}", ret);
        ret
    });

    match items {
        Ok((_, mut i)) => {
            let mut results = Vec::new();

            for mut inner in i.drain(..) {
                // iterate through cycle positions ...
                let mut cycle_position = Vec::new();
                let mut template_params = Vec::new(); // collect params for templates ..
                for item in inner.drain(..) {
                    match item {
                        CycleItem::Duration(d) => {
                            cycle_position.push(CycleResult::Duration(d));
                        }
                        CycleItem::Event((mut name, pars)) => {
                            // now this might seem odd, but to re-align the positional arguments i'm just reassembling the string
                            // and use the regular parser ... not super elegant, but hey ...
                            for par in pars.iter() {
                                match par {
                                    CycleItem::Parameter(CycleParameter::Number(f)) => {
                                        name = name + " " + &f.to_string();
                                    }
                                    CycleItem::Parameter(CycleParameter::Symbol(s)) => {
                                        name = name + " \'" + s;
                                    }
                                    CycleItem::NamedParameter((
                                        pname,
                                        CycleParameter::Symbol(s),
                                    )) => {
                                        name = name + &format!(" :{} \'{} ", pname, s);
                                    }
                                    CycleItem::NamedParameter((
                                        pname,
                                        CycleParameter::Number(f),
                                    )) => {
                                        name = name + &format!(" :{} {} ", pname, f);
                                    }
                                    _ => {
                                        println!("ignore cycle event param")
                                    }
                                }
                            }
                            // in brackets so it's recognized as a "function"
                            name = format!("({})", name);
                            //println!("{}", name);
                            match parse_expr(name.trim()) {
                                Ok((_, expr)) => {
                                    if let Some(EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e))) =
                                        eval_expression(
                                            &expr,
                                            functions,
                                            global_parameters,
                                            sample_set,
                                            out_mode,
                                        )
                                    {
                                        //println!("ev {}", e.name);
                                        cycle_position.push(CycleResult::SoundEvent(e));
                                    } else {
                                        println!("couldn't eval cycle expr");
                                    }
                                }
                                Err(_) => {
                                    println!("couldn't parse re-assembled cycle event")
                                }
                            }
                        }
                        CycleItem::Parameter(CycleParameter::Number(f)) => {
                            template_params.push(CycleItem::Parameter(CycleParameter::Number(f)));
                        }
                        CycleItem::Parameter(CycleParameter::Symbol(s)) => {
                            if let Some(evs) = event_mappings.get(&s) {
                                // mappings have precedence ...
                                for ev in evs {
                                    match ev {
                                        SourceEvent::Sound(s) => {
                                            cycle_position.push(CycleResult::SoundEvent(s.clone()))
                                        }
                                        SourceEvent::Control(c) => cycle_position
                                            .push(CycleResult::ControlEvent(c.clone())),
                                    }
                                }
                            } else {
                                template_params
                                    .push(CycleItem::Parameter(CycleParameter::Symbol(s)));
                            }
                        }
                        CycleItem::NamedParameter((pname, param)) => {
                            template_params.push(CycleItem::NamedParameter((pname, param)));
                        }
                        _ => {
                            println!("nothing to be done ...")
                        }
                    }
                }
                if !template_events.is_empty() && !template_params.is_empty() {
                    for t_ev in template_events.iter() {
                        let mut ev_name = t_ev.clone();
                        for t_par in template_params.iter() {
                            match t_par {
                                CycleItem::Parameter(CycleParameter::Number(f)) => {
                                    ev_name = ev_name + " " + &f.to_string();
                                }
                                CycleItem::Parameter(CycleParameter::Symbol(s)) => {
                                    ev_name = ev_name + " \'" + s;
                                }
                                CycleItem::NamedParameter((pname, CycleParameter::Number(f))) => {
                                    ev_name = ev_name + &format!(" :{} {} ", pname, f);
                                }
                                CycleItem::NamedParameter((pname, CycleParameter::Symbol(s))) => {
                                    ev_name = ev_name + &format!(" :{} \'{} ", pname, s);
                                }
                                _ => {}
                            }
                        }
                        // brackets so it's recognized as a "function"
                        ev_name = format!("({})", ev_name);
                        match parse_expr(ev_name.trim()) {
                            Ok((_, expr)) => {
                                if let Some(EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e))) =
                                    eval_expression(
                                        &expr,
                                        functions,
                                        global_parameters,
                                        sample_set,
                                        out_mode,
                                    )
                                {
                                    cycle_position.push(CycleResult::SoundEvent(e));
                                } else {
                                    println!("couldn't eval cycle expr");
                                }
                            }
                            Err(_) => {
                                println!("couldn't parse re-assembled cycle event")
                            }
                        }
                    }
                }
                //println!("r {} p {}", results.len(), cycle_position.len());
                results.push(cycle_position);
            }
            results
        }
        Err(_) => Vec::new(),
    }
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use dashmap::DashMap;
    use std::collections::HashSet;

    #[test]
    fn test_basic_cyc2_float() {
        match parse_cyc_float("100 b") {
            Ok(o) => {
                println!("{:?}", o.0)
            }
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }

    #[test]
    fn test_basic_cyc2_elem() {
        match parse_cyc("[saw:200]") {
            Ok((_, o)) => match &o[0][0] {
                CycleItem::Event((_, _)) => assert!(true),
                _ => {
                    panic!()
                }
            },
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_basic_cyc2() {
        match parse_cyc("saw:200 ~ ~ ~") {
            Ok((_, o)) => {
                assert!(o.len() == 4);

                match &o[0][0] {
                    CycleItem::Event((s, _)) => assert!(s == "saw"),
                    _ => panic!(),
                }

                match &o[1][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }
            }
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_basic_cyc2_noparam() {
        match parse_cyc("saw ~ ~ ~") {
            Ok((_, o)) => {
                assert!(o.len() == 4);

                match &o[0][0] {
                    CycleItem::Event((s, _)) => assert!(s == "saw"),
                    _ => panic!(),
                }

                match &o[1][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }
            }
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_basic_cyc2_noparam_dur() {
        match parse_cyc("saw /100 saw ~ ~") {
            Ok((_, o)) => {
                assert!(o.len() == 5);

                match &o[0][0] {
                    CycleItem::Event((s, _)) => assert!(s == "saw"),
                    _ => panic!(),
                }

                match &o[1][0] {
                    CycleItem::Duration(d) => assert!(*d == 100.0),
                    _ => panic!(),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == "saw"),
                    _ => panic!(),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }

                match &o[4][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }
            }
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_symbol_param_only() {
        match parse_cyc("'boat ~ ~ ~") {
            Ok((_, o)) => {
                match &o[0][0] {
                    CycleItem::Parameter(CycleParameter::Symbol(s)) => assert!(s == "boat"),
                    _ => panic!(),
                }

                match &o[1][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }
            }
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_float_param_only() {
        match parse_cyc("200 ~ ~ ~") {
            Ok((_, o)) => {
                match &o[0][0] {
                    CycleItem::Parameter(CycleParameter::Number(f)) => assert!(*f == 200.0),
                    _ => panic!(),
                }

                match &o[1][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }

                match &o[2][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }

                match &o[3][0] {
                    CycleItem::Event((s, _)) => assert!(s == "~"),
                    _ => panic!(),
                }
            }
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_basic_cyc2_eval_noparam() {
        let sample_set = sync::Arc::new(Mutex::new(SampleAndWavematrixSet::new()));

        // mock sample
        let mut keys = HashSet::new();
        keys.insert("a3".to_string());
        sample_set.lock().insert("piano".to_string(), keys, 3, 100);

        let template_events = Vec::new();
        let event_mappings = HashMap::new();

        let mut fmap: FunctionMap = FunctionMap::new();
        fmap.fmap.insert(
            "piano".to_string(),
            crate::parser::eval::events::sound::sound,
        );
        fmap.fmap
            .insert("saw".to_string(), crate::parser::eval::events::sound::sound);
        fmap.fmap
            .insert("~".to_string(), crate::parser::eval::events::sound::sound);

        let o = eval_cyc_from_str(
            "saw /100 saw:400 ~ ~ [saw:100 saw:500] ~ piano:'a3 piano:'a3:lpf=100",
            &fmap,
            &sample_set,
            OutputMode::Stereo,
            &template_events,
            &event_mappings,
            &sync::Arc::new(DashMap::new()),
        );
        println!("return length: {}", o.len());

        assert!(o.len() == 9);

        match &o[0][0] {
            CycleResult::SoundEvent(e) => assert!(e.name == "saw"),
            _ => panic!(),
        }

        match &o[1][0] {
            CycleResult::Duration(d) => assert!(*d == 100.0_f32),
            _ => panic!(),
        }

        match &o[2][0] {
            CycleResult::SoundEvent(e) => assert!(e.name == "saw"),
            _ => panic!(),
        }

        match &o[3][0] {
            CycleResult::SoundEvent(e) => assert!(e.name == "silence"),
            _ => panic!(),
        }

        match &o[4][0] {
            CycleResult::SoundEvent(e) => assert!(e.name == "silence"),
            _ => panic!(),
        }

        match &o[5][0] {
            CycleResult::SoundEvent(e) => assert!(e.name == "saw"),
            _ => panic!(),
        }

        match &o[5][1] {
            CycleResult::SoundEvent(e) => assert!(e.name == "saw"),
            _ => panic!(),
        }

        match &o[6][0] {
            CycleResult::SoundEvent(e) => assert!(e.name == "silence"),
            _ => panic!(),
        }

        match &o[7][0] {
            CycleResult::SoundEvent(e) => assert!(e.name == "sampler"),
            _ => panic!(),
        }

        match &o[8][0] {
            CycleResult::SoundEvent(e) => assert!(e.name == "sampler"),
            _ => panic!(),
        }
    }
}
