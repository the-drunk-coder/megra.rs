mod handlers;
mod parse_parameter_events;
mod parser_helpers;

use parking_lot::Mutex;
use std::sync;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char, multispace0},
    character::{is_alphanumeric, is_space},
    combinator::{cut, map, map_res, recognize},
    error::{context, VerboseError},
    multi::many0,
    number::complete::float,
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};

use crate::builtin_types::*;
use crate::event::*;
use crate::sample_set::SampleSet;
use crate::session::OutputMode;

use parse_parameter_events::*;

fn parse_comment<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(preceded(tag(";"), take_while(|ch| ch != '\n')), |_| {
        Expr::Comment
    })(i)
}

fn parse_builtin<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        parse_constructors,
        parse_commands,
        map(tag("sx"), |_| BuiltIn::SyncContext),
        map(tag("ctrl"), |_| BuiltIn::ControlEvent),
        parse_generator_modifier_functions, // needs to come before events so it can catch relax before rel(ease)
        parse_events,
        parse_dynamic_parameters,
        parse_generator_processors,
        parse_multiplexer,
    ))(i)
}

fn parse_commands<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("clear"), |_| BuiltIn::Command(BuiltInCommand::Clear)),
        map(tag("tmod"), |_| BuiltIn::Command(BuiltInCommand::Tmod)),
        map(tag("global-resources"), |_| {
            BuiltIn::Command(BuiltInCommand::GlobRes)
        }),
        map(tag("delay"), |_| BuiltIn::Command(BuiltInCommand::Delay)),
        map(tag("reverb"), |_| BuiltIn::Command(BuiltInCommand::Reverb)),
        map(tag("load-sets"), |_| {
            BuiltIn::Command(BuiltInCommand::LoadSampleSets)
        }),
        map(tag("load-set"), |_| {
            BuiltIn::Command(BuiltInCommand::LoadSampleSet)
        }),
        map(tag("load-sample"), |_| {
            BuiltIn::Command(BuiltInCommand::LoadSample)
        }),
        map(tag("defpart"), |_| {
            BuiltIn::Command(BuiltInCommand::LoadPart)
        }),
        map(tag("export-dot"), |_| {
            BuiltIn::Command(BuiltInCommand::ExportDot)
        }),
        map(tag("freeze"), |_| {
            BuiltIn::Command(BuiltInCommand::FreezeBuffer)
        }),
        map(tag("once"), |_| BuiltIn::Command(BuiltInCommand::Once)),
    ))(i)
}

pub fn parse_dynamic_parameters<'a>(
    i: &'a str,
) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("env"), |_| {
            BuiltIn::Parameter(BuiltInDynamicParameter::Envelope)
        }),
        map(tag("fade"), |_| {
            BuiltIn::Parameter(BuiltInDynamicParameter::Fade)
        }),
        map(tag("bounce"), |_| {
            BuiltIn::Parameter(BuiltInDynamicParameter::Bounce)
        }),
        map(tag("brownian"), |_| {
            BuiltIn::Parameter(BuiltInDynamicParameter::Brownian)
        }),
        map(tag("randr"), |_| {
            BuiltIn::Parameter(BuiltInDynamicParameter::RandRange)
        }),
    ))(i)
}

fn parse_multiplexer<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("xdup"), |_| {
            BuiltIn::Multiplexer(BuiltInMultiplexer::XDup)
        }),
        map(tag("xspread"), |_| {
            BuiltIn::Multiplexer(BuiltInMultiplexer::XSpread)
        }),
    ))(i)
}

fn parse_constructors<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("learn"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Learn)
        }),
        map(tag("infer"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Infer)
        }),
        map(tag("rule"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Rule)
        }),
        map(tag("nuc"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Nucleus)
        }),
        map(tag("cyc"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Cycle)
        }),
        map(tag("fully"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Fully)
        }),
        map(tag("flower"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Flower)
        }),
        map(tag("friendship"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Friendship)
        }),
        map(tag("chop"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Chop)
        }),
	map(tag("stages"), |_| {
            BuiltIn::Constructor(BuiltInConstructor::Stages)
        }),
    ))(i)
}

fn parse_generator_processors<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("pear"), |_| BuiltIn::GenProc(BuiltInGenProc::Pear)),
        map(tag("inh"), |_| BuiltIn::GenProc(BuiltInGenProc::Inhibit)),
        map(tag("exh"), |_| BuiltIn::GenProc(BuiltInGenProc::Exhibit)),
        map(tag("inexh"), |_| {
            BuiltIn::GenProc(BuiltInGenProc::InExhibit)
        }),
        map(tag("apple"), |_| BuiltIn::GenProc(BuiltInGenProc::Apple)),
        map(tag("every"), |_| BuiltIn::GenProc(BuiltInGenProc::Every)),
        map(tag("life"), |_| BuiltIn::GenProc(BuiltInGenProc::Lifemodel)),
        map(tag("cmp"), |_| BuiltIn::Compose), // putting this here for now
    ))(i)
}

fn parse_generator_modifier_functions<'a>(
    i: &'a str,
) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("haste"), |_| {
            BuiltIn::GenModFun(BuiltInGenModFun::Haste)
        }),
        map(tag("relax"), |_| {
            BuiltIn::GenModFun(BuiltInGenModFun::Relax)
        }),
        map(tag("grow"), |_| BuiltIn::GenModFun(BuiltInGenModFun::Grow)),
        map(tag("shrink"), |_| {
            BuiltIn::GenModFun(BuiltInGenModFun::Shrink)
        }),
        map(tag("blur"), |_| BuiltIn::GenModFun(BuiltInGenModFun::Blur)),
        map(tag("sharpen"), |_| {
            BuiltIn::GenModFun(BuiltInGenModFun::Sharpen)
        }),
        map(tag("shake"), |_| {
            BuiltIn::GenModFun(BuiltInGenModFun::Shake)
        }),
        map(tag("skip"), |_| BuiltIn::GenModFun(BuiltInGenModFun::Skip)),
        map(tag("rewind"), |_| {
            BuiltIn::GenModFun(BuiltInGenModFun::Rewind)
        }),
        map(tag("rnd"), |_| BuiltIn::GenModFun(BuiltInGenModFun::Rnd)),
        map(tag("rep"), |_| BuiltIn::GenModFun(BuiltInGenModFun::Rep)),
    ))(i)
}

fn parse_synth_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("sine"), |_| {
            BuiltIn::SoundEvent(BuiltInSoundEvent::Sine(EventOperation::Replace))
        }),
	map(tag("tri"), |_| {
            BuiltIn::SoundEvent(BuiltInSoundEvent::Tri(EventOperation::Replace))
        }),
        map(tag("cub"), |_| {
            BuiltIn::SoundEvent(BuiltInSoundEvent::Cub(EventOperation::Replace))
        }),
        map(tag("saw"), |_| {
            BuiltIn::SoundEvent(BuiltInSoundEvent::Saw(EventOperation::Replace))
        }),
	map(tag("risset"), |_| {
            BuiltIn::SoundEvent(BuiltInSoundEvent::RissetBell(EventOperation::Replace))
        }),
        map(tag("sqr"), |_| {
            BuiltIn::SoundEvent(BuiltInSoundEvent::Square(EventOperation::Replace))
        }),
        map(tag("silence"), |_| BuiltIn::Silence),
        map(tag("~"), |_| BuiltIn::Silence),
    ))(i)
}

pub fn parse_events<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        parse_pitch_frequency_event,
        parse_level_event,
        parse_synth_event,
        parse_reverb_event,
        parse_duration_event,
        parse_attack_event,
        parse_release_event,
        parse_sustain_event,
        parse_channel_position_event,
        parse_delay_event,
        parse_lp_freq_event,
        parse_lp_q_event,
        parse_lp_dist_event,
        parse_pf_freq_event,
        parse_pf_q_event,
        parse_pf_gain_event,
        parse_pw_event,
        parse_playback_start_event,
        parse_playback_rate_event,
    ))(i)
}

pub fn parse_custom<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    map(
        context("custom_fun", cut(take_while(valid_fun_name_char))),
        |fun_str: &str| Expr::Custom(fun_str.to_string()),
    )(i)
}

/// Our boolean values are also constant, so we can do it the same way
pub fn parse_bool<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    alt((
        map(tag("#t"), |_| Atom::Boolean(true)),
        map(tag("#f"), |_| Atom::Boolean(false)),
    ))(i)
}

fn parse_keyword<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    map(
        context(
            "keyword",
            preceded(tag(":"), take_while(valid_fun_name_char)),
        ),
        |sym_str: &str| Atom::Keyword(sym_str.to_string()),
    )(i)
}

pub fn parse_symbol<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    map(
        context(
            "symbol",
            preceded(tag("'"), take_while(valid_fun_name_char)),
        ),
        |sym_str: &str| Atom::Symbol(sym_str.to_string()),
    )(i)
}

pub fn parse_float<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    map_res(recognize(float), |digit_str: &str| {
        digit_str.parse::<f32>().map(Atom::Float)
    })(i)
}

/// valid chars for a string
pub fn valid_char(chr: char) -> bool {
    chr == '~'
        || chr == '.'
        || chr == '\''
        || chr == '_'
        || chr == '/'
        || chr == '-'
        || chr == ':'
        || chr == '='
        || chr == '['
        || chr == ']'
        || is_alphanumeric(chr as u8)
        || is_space(chr as u8)
}

/// valid chars for a function name
pub fn valid_fun_name_char(chr: char) -> bool {
    chr == '_' || chr == '-' || is_alphanumeric(chr as u8)
}

fn parse_string<'a>(i: &'a str) -> IResult<&'a str, Atom, VerboseError<&'a str>> {
    map(
        delimited(tag("\""), take_while(valid_char), tag("\"")),
        |desc_str: &str| Atom::Description(desc_str.to_string()),
    )(i)
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
    map(parse_atom, Expr::Constant)(i)
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
    let application_inner = map(
        tuple((alt((parse_expr, parse_custom)), many0(parse_expr))),
        |(head, tail)| Expr::Application(Box::new(head), tail),
    );
    // finally, we wrap it in an s-expression
    s_exp(application_inner)(i)
}

/// We tie them all together again, making a top-level expression parser!
fn parse_expr<'a>(i: &'a str) -> IResult<&'a str, Expr, VerboseError<&'a str>> {
    preceded(
        multispace0,
        alt((parse_comment, parse_constant, parse_application)),
    )(i)
}

/// This function tries to reduce the AST.
/// This has to return an Expression rather than an Atom because quoted s_expressions
/// can't be reduced
pub fn eval_expression(
    e: Expr,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
) -> Option<Expr> {
    match e {
        // Constants and quoted s-expressions are our base-case
        Expr::Comment => Some(e),
        Expr::Constant(_) => Some(e),
        Expr::Custom(_) => Some(e),
        Expr::Application(head, tail) => {
            let reduced_head = eval_expression(*head, sample_set, out_mode)?;

            let mut reduced_tail = tail
                .into_iter()
                .map(|expr| eval_expression(expr, sample_set, out_mode))
                .collect::<Option<Vec<Expr>>>()?;

            // filter out reduced comments ...
            reduced_tail.retain(|x| !matches!(x, Expr::Comment));

            match reduced_head {
                Expr::Constant(Atom::BuiltIn(bi)) => Some(Expr::Constant(match bi {
                    BuiltIn::Command(cmd) => {
                        handlers::builtin_commands::handle(cmd, &mut reduced_tail)
                    }
                    BuiltIn::Compose => handlers::builtin_compose::handle(&mut reduced_tail),
                    BuiltIn::Silence => Atom::SoundEvent(Event::with_name("silence".to_string())),
                    BuiltIn::Constructor(con) => handlers::builtin_constructors::handle(
                        &con,
                        &mut reduced_tail,
                        sample_set,
                        out_mode,
                    ),
                    BuiltIn::SyncContext => {
                        handlers::builtin_sync_context::handle(&mut reduced_tail)
                    }
                    BuiltIn::Parameter(par) => {
                        handlers::builtin_dynamic_parameter::handle(&par, &mut reduced_tail)
                    }
                    BuiltIn::SoundEvent(ev) => {
                        handlers::builtin_sound_event::handle(&ev, &mut reduced_tail)
                    }
                    BuiltIn::ControlEvent => {
                        handlers::builtin_control_event::handle(&mut reduced_tail)
                    }
                    BuiltIn::ParameterEvent(ev) => {
                        handlers::builtin_parameter_event::handle(&ev, &mut reduced_tail)
                    }
                    BuiltIn::GenProc(g) => {
                        handlers::builtin_generator_processor::handle(&g, &mut reduced_tail)
                    }
                    BuiltIn::GenModFun(g) => {
                        handlers::builtin_generator_modifier_function::handle(&g, &mut reduced_tail)
                    }
                    BuiltIn::Multiplexer(m) => {
                        handlers::builtin_multiplexer::handle(&m, &mut reduced_tail, out_mode)
                    }
                })),
                Expr::Custom(s) => {
                    handlers::custom_sample_event::handle(&mut reduced_tail, s, &sample_set)
                }
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
pub fn eval_from_str(
    src: &str,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
) -> Result<Expr, String> {
    parse_expr(src)
        .map_err(|e: nom::Err<VerboseError<&str>>| format!("{:#?}", e))
        .and_then(|(_, exp)| {
            eval_expression(exp, sample_set, out_mode).ok_or_else(|| "Eval failed".to_string())
        })
}
