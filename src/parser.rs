use dashmap::DashMap;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, multispace0, multispace1},
    character::{is_alphanumeric, is_newline, is_space},
    combinator::{cut, map, map_res, recognize},
    error::{context, ErrorKind, VerboseError, VerboseErrorKind},
    multi::many0,
    number::complete::float,
    sequence::{delimited, preceded, tuple},
    Err, IResult, Parser,
};

use regex::Regex;
use std::collections::HashMap;
use std::{fmt, sync};

use crate::{
    builtin_types::{Comparable, Comparator, VariableId},
    session::SyncContext,
};
use crate::{Command, GlobalVariables, OutputMode, SampleAndWavematrixSet, TypedEntity};

pub mod eval;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArgumentCollector {
    All, // collects all arguments to be passed on ...
    Rest, // collect "unclaimed" arguments
         //Positional, // collect all positional arguments
         //Keyword // collect keyword arguments
}

/// These are the basic building blocks of our casual lisp language.
/// You might notice that there's no lists in this lisp ... not sure
/// what to call it in that case ...
#[derive(Debug, Clone)]
pub enum Atom {
    Float(f32),
    String(String),
    Keyword(String),
    Symbol(String),
    Boolean(bool),
    Identifier(String),
    ArgumentCollector(ArgumentCollector),
}

/// Expression Type
#[derive(Debug, Clone)]
pub enum Expr {
    FunctionDefinition,
    VariableDefinition,
    Constant(Atom),
    Application(Box<Expr>, Vec<Expr>),
    Definition(Box<Expr>, Vec<Expr>),
}

#[derive(Clone)]
pub enum EvaluatedExpr {
    // keywords and identifiers are untyped language constructs
    Keyword(String),
    Identifier(String),
    // commands, sync contexts and progns are also
    // top-level language constructs, as well as definitions
    // (see below)
    Command(Command),
    SyncContext(SyncContext),
    Progn(Vec<EvaluatedExpr>),
    Match(Box<EvaluatedExpr>, Vec<(EvaluatedExpr, EvaluatedExpr)>),
    // I don't really have an idea how to make functions,
    // so for now I'll just store the non-evaluated Exprs
    // and reduce them once the user calls the function ...
    // that might make them macros rather than functions?
    // Not sure ...
    FunctionDefinition(String, Vec<String>, Vec<Expr>),
    VariableDefinition(VariableId, TypedEntity),
    // everything else is a typed entity
    Typed(TypedEntity),
    // only for collecting arguments
    // the list field exists only to be flattened
    EvaluatedExprList(Vec<EvaluatedExpr>),
    Comparator(Comparator),
}

impl fmt::Debug for EvaluatedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvaluatedExpr::Typed(_) => write!(f, "EvaluatedExpr::Typed(_)"),
            EvaluatedExpr::Identifier(fna) => write!(f, "EvaluatedExpr::Identifier({fna})"),
            EvaluatedExpr::Command(_) => write!(f, "EvaluatedExpr::Command"),
            EvaluatedExpr::Keyword(k) => write!(f, "EvaluatedExpr::Keyword({k})"),
            EvaluatedExpr::SyncContext(_) => write!(f, "EvaluatedExpr::SyncContext(_)"),
            EvaluatedExpr::Progn(_) => write!(f, "EvaluatedExpr::Progn"),
            EvaluatedExpr::Match(_, _) => write!(f, "EvaluatedExpr::Match"),
            EvaluatedExpr::FunctionDefinition(_, _, _) => {
                write!(f, "EvaluatedExpr::FunctionDefinition")
            }
            EvaluatedExpr::VariableDefinition(_, _) => {
                write!(f, "EvaluatedExpr::VariableDefinition")
            }
            EvaluatedExpr::EvaluatedExprList(_) => {
                write!(f, "EvaluatedExpr::EvaluatedExprList")
            }
            EvaluatedExpr::Comparator(_) => {
                write!(f, "EvaluatedExpr::Comparator")
            }
        }
    }
}

// std_lib are hard-coded,
// usr_lib is for user-defined functions ...
pub struct FunctionMap {
    pub usr_lib: DashMap<String, (Vec<String>, Vec<Expr>)>,
    pub std_lib: DashMap<
        String,
        fn(
            &FunctionMap,
            &mut Vec<EvaluatedExpr>,
            &sync::Arc<GlobalVariables>,
            SampleAndWavematrixSet,
            OutputMode,
        ) -> Option<EvaluatedExpr>,
    >,
}

impl FunctionMap {
    pub fn new() -> Self {
        FunctionMap {
            std_lib: DashMap::new(),
            usr_lib: DashMap::new(),
        }
    }
}

/// valid chars for a string
fn valid_string_char(chr: char) -> bool {
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
        || chr == '#'
        || is_alphanumeric(chr as u8)
        || is_space(chr as u8)
        || is_newline(chr as u8)
}

/// valid chars for a function name, symbol or keyword
pub fn valid_identifier_name_char(chr: char) -> bool {
    chr == '_'
        || chr == '~'
        || chr == '-'
        || chr == '>'
        || chr == '<'
        || chr == '='
        || is_alphanumeric(chr as u8)
}

/// parse a string, which is enclosed in double quotes
fn parse_string(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(
        delimited(tag("\""), take_while(valid_string_char), tag("\"")),
        |desc_str: &str| Atom::String(desc_str.to_string()),
    )(i)
}

/// booleans have a # prefix
fn parse_boolean(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    alt((
        map(tag("#t"), |_| Atom::Boolean(true)),
        map(tag("#f"), |_| Atom::Boolean(false)),
        map(tag("()"), |_| Atom::Boolean(false)),
    ))(i)
}

/// arg collectors have a @ prefix
fn parse_arg_collector(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    alt((
        map(tag("@rest"), |_| {
            Atom::ArgumentCollector(ArgumentCollector::Rest)
        }),
        map(tag("@all"), |_| {
            Atom::ArgumentCollector(ArgumentCollector::All)
        }),
    ))(i)
}

fn parse_definition(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    alt((
        map(tag("fun"), |_| Expr::FunctionDefinition),
        map(tag("callback"), |_| Expr::FunctionDefinition),
        map(tag("let"), |_| Expr::VariableDefinition),
        map(tag("defpart"), |_| Expr::VariableDefinition),
    ))(i)
}

/// keywords are language constructs that start with a ':'
fn parse_keyword(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(
        context(
            "keyword",
            preceded(tag(":"), take_while(valid_identifier_name_char)),
        ),
        |sym_str: &str| Atom::Keyword(sym_str.to_string()),
    )(i)
}

/// keywords are language constructs that start with a single quote
pub fn parse_symbol(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(
        context(
            "symbol",
            preceded(tag("'"), take_while(valid_identifier_name_char)),
        ),
        |sym_str: &str| Atom::Symbol(sym_str.to_string()),
    )(i)
}

/// function names are language constructs that contain allowed function name chars
fn parse_identifier(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(
        context("identifer", take_while1(valid_identifier_name_char)),
        |sym_str: &str| Atom::Identifier(sym_str.to_string()),
    )(i)
}

/// floating point numbers ... all numbers currently are ...
pub fn parse_float(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    // manually disallowing "infinity" because it doesn't make much sense here
    // and clashes with "infer", which led to an error ...
    if i.starts_with("inf") {
        Err(Err::Error(VerboseError {
            errors: vec![(
                "infinity disallowed",
                VerboseErrorKind::Nom(ErrorKind::Float),
            )],
        }))
    } else {
        map_res(recognize(float), |digit_str: &str| {
            digit_str.parse::<f32>().map(Atom::Float)
        })(i)
    }
}

/// parse all the atoms
fn parse_constant(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(
        alt((
            parse_boolean,
            parse_arg_collector,
            parse_float,
            parse_keyword,
            parse_symbol,
            parse_string,
            parse_identifier,
        )),
        Expr::Constant,
    )(i)
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
fn parse_application(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let application_inner = map(
        tuple((
            parse_expr,
            many0(alt((
                preceded(multispace0, parse_application), // applications can follow one another without whitespace
                preceded(multispace1, parse_constant), // constants are delimited by at least one whitespace
            ))),
        )),
        |(head, tail)| match head {
            Expr::FunctionDefinition => Expr::Definition(Box::new(head), tail),
            Expr::VariableDefinition => Expr::Definition(Box::new(head), tail),
            _ => Expr::Application(Box::new(head), tail),
        },
    );
    // finally, we wrap it in an s-expression
    s_exp(application_inner)(i)
}

/// We tie them all together again, making a top-level expression parser!
/// This one generates the abstract syntax tree
pub fn parse_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    alt((parse_definition, parse_application, parse_constant))(i)
}

// evaluate as argument identifiers, or better, constants only, no applications
// or definitions
pub fn eval_as_arg(e: &Expr) -> Option<EvaluatedExpr> {
    match e {
        Expr::Constant(c) => Some(match c {
            Atom::Float(f) => EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(*f))),
            Atom::Symbol(s) => {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s.to_string())))
            }
            Atom::Keyword(k) => EvaluatedExpr::Keyword(k.to_string()),
            Atom::String(s) => {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s.to_string())))
            }
            Atom::Boolean(b) => {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(*b)))
            }
            Atom::Identifier(f) => {
                // eval local vars at eval time ???
                EvaluatedExpr::Identifier(f.to_string())
            }
            _ => return None,
        }),
        _ => None,
    }
}

#[derive(Debug)]
pub struct LocalVariables {
    pub pos_args: Option<HashMap<String, EvaluatedExpr>>,
    pub rest: Option<Vec<EvaluatedExpr>>,
}

pub fn eval_usr_fun(
    fun_arg_names: Vec<String>,
    fun_expr: Vec<Expr>,
    tail: &[Expr],
    functions: &FunctionMap,
    globals: &sync::Arc<GlobalVariables>,
    sample_set: SampleAndWavematrixSet,
    out_mode: OutputMode,
) -> Option<EvaluatedExpr> {
    if fun_arg_names.len() > tail.len() {
        // not enough arguments ... no general currying currently :(
        return None;
    }

    // FIRST, eval local args,
    // manual zip
    let mut local_args = HashMap::new();
    for (i, expr) in tail[..fun_arg_names.len()].iter().enumerate() {
        if let Some(res) =
            eval_expression(expr, functions, globals, None, sample_set.clone(), out_mode)
        {
            local_args.insert(fun_arg_names[i].clone(), res);
        }
    }

    let mut rest = Vec::new();
    for expr in tail[fun_arg_names.len()..].iter() {
        if let Some(res) =
            eval_expression(expr, functions, globals, None, sample_set.clone(), out_mode)
        {
            rest.push(res);
        }
    }

    let local_vars = LocalVariables {
        pos_args: if local_args.is_empty() {
            None
        } else {
            Some(local_args)
        },
        rest: if rest.is_empty() { None } else { Some(rest) },
    };

    // THIRD
    let mut fun_tail: Vec<EvaluatedExpr> = Vec::new();
    for expr in fun_expr.iter() {
        if let Some(e) = eval_expression(
            expr,
            functions,
            globals,
            Some(&local_vars),
            sample_set.clone(),
            out_mode,
        ) {
            // the list field exists only to be flattened
            if let EvaluatedExpr::EvaluatedExprList(mut l) = e {
                fun_tail.append(&mut l);
            } else {
                fun_tail.push(e);
            }
        } else {
            // jump out if expression can't be evaluated
            return None;
        }
    }

    // return last form result, cl-style
    fun_tail.pop()
}

/// This one reduces the abstract syntax tree ...
/// does not resolve global variables at this point,
/// as there might be different points in time where it makes
/// sense to resolve things ...
pub fn eval_expression(
    e: &Expr,
    functions: &FunctionMap,
    globals: &sync::Arc<GlobalVariables>,
    locals: Option<&LocalVariables>,
    sample_set: SampleAndWavematrixSet,
    out_mode: OutputMode,
) -> Option<EvaluatedExpr> {
    match e {
        Expr::Constant(c) => Some(match c {
            Atom::Float(f) => EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(*f))),
            Atom::Symbol(s) => {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s.to_string())))
            }
            Atom::Keyword(k) => EvaluatedExpr::Keyword(k.to_string()),
            Atom::String(s) => {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s.to_string())))
            }
            Atom::Boolean(b) => {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(*b)))
            }
            // if we have an identifier, check whether we have
            // a matching local (positional) variable to resolve ...
            Atom::Identifier(f) => {
                // eval local vars at eval time ???
                if let Some(loc) = locals {
                    if let Some(pos_args) = &loc.pos_args {
                        if let Some(arg) = pos_args.get(f) {
                            arg.clone()
                        } else {
                            EvaluatedExpr::Identifier(f.to_string())
                        }
                    } else {
                        EvaluatedExpr::Identifier(f.to_string())
                    }
                } else {
                    EvaluatedExpr::Identifier(f.to_string())
                }
            }
            Atom::ArgumentCollector(argc) => {
                let mut coll = Vec::new();

                if let Some(loc) = locals {
                    match argc {
                        ArgumentCollector::Rest => {
                            if let Some(rest) = &loc.rest {
                                coll.append(&mut rest.clone());
                            }
                        }
                        ArgumentCollector::All => {
                            if let Some(pos_args) = &loc.pos_args {
                                for arg in pos_args.values() {
                                    coll.push(arg.clone())
                                }
                            }
                            if let Some(rest) = &loc.rest {
                                coll.append(&mut rest.clone());
                            }
                        }
                    }
                }

                // even if it's empty, as the @rest arg can be empty ...
                EvaluatedExpr::EvaluatedExprList(coll)
            }
        }),
        Expr::Application(head, tail) => {
            // local variables aren't evaluated for the head, which is supposed to be the identifier
            // in some context it might be nice to have the head procedurally generated as well,
            // but it'd cause a whole lotta trouble right now and I have no desire currently to use it ...
            // there's more explicit ways to do the whole thing ...
            if let Some(EvaluatedExpr::Identifier(f)) =
                eval_expression(head, functions, globals, None, sample_set.clone(), out_mode)
            {
                // check if we have this function ...
                if functions.std_lib.contains_key(&f) {
                    let mut reduced_tail: Vec<EvaluatedExpr> = Vec::new();
                    for expr in tail {
                        if let Some(e) = eval_expression(
                            expr,
                            functions,
                            globals,
                            locals,
                            sample_set.clone(),
                            out_mode,
                        ) {
                            // the list field exists only to be flattened
                            if let EvaluatedExpr::EvaluatedExprList(mut l) = e {
                                reduced_tail.append(&mut l);
                            } else {
                                reduced_tail.push(e);
                            }
                        } else if f == "match" {
                            // stupid temporary hack to make the match statement work in cases
                            // when an arm can't be evaluated, in which case it just evaluates to false.
                            // I'm pretty sure there's a much better, less sloppy solution to this, but
                            // I can't pinpoint it right now ...
                            reduced_tail.push(EvaluatedExpr::Typed(TypedEntity::Comparable(
                                Comparable::Boolean(false),
                            )))
                        } else {
                            // otherwise jump out if expression can't be evaluated
                            return None;
                        }
                    }

                    // push function name
                    reduced_tail.insert(0, EvaluatedExpr::Identifier(f.clone()));
                    functions.std_lib.get(&f).unwrap()(
                        functions,
                        &mut reduced_tail,
                        globals,
                        sample_set,
                        out_mode,
                    )
                } else if functions.usr_lib.contains_key(&f) {
                    let (fun_arg_names, fun_expr) = functions.usr_lib.get(&f).unwrap().clone();
                    eval_usr_fun(
                        fun_arg_names,
                        fun_expr,
                        &tail,
                        functions,
                        globals,
                        sample_set,
                        out_mode,
                    )
                } else {
                    None
                }
            } else {
                None
            }
        }
        Expr::Definition(head, tail) => match **head {
            Expr::FunctionDefinition => {
                let id = match eval_expression(
                    &tail[0],
                    functions,
                    globals,
                    None,
                    sample_set.clone(),
                    out_mode,
                ) {
                    Some(EvaluatedExpr::Identifier(i)) => Some(i),
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) => {
                        Some(s)
                    }
                    _ => None,
                };

                if let Some(i) = id {
                    // i hate this clone ...
                    let mut tail_clone = tail.clone();

                    // remove function name
                    tail_clone.remove(0);

                    let mut positional_args = Vec::new();
                    let mut rem_args = false;

                    // evaluate positional arguments ...
                    if let Some(Expr::Application(head, fun_tail)) = tail_clone.first() {
                        if let Some(EvaluatedExpr::Identifier(f)) = eval_as_arg(head) {
                            positional_args.push(f);
                        }
                        // reduce tail args ...
                        let reduced_tail = fun_tail
                            .iter()
                            .map(eval_as_arg)
                            .collect::<Option<Vec<EvaluatedExpr>>>()?;

                        for eexpr in reduced_tail {
                            if let EvaluatedExpr::Identifier(f) = eexpr {
                                positional_args.push(f);
                            }
                        }
                        rem_args = true;
                    }

                    if rem_args {
                        tail_clone.remove(0);
                    }

                    Some(EvaluatedExpr::FunctionDefinition(
                        i,
                        positional_args,
                        tail_clone,
                    ))
                } else {
                    None
                }
            }
            Expr::VariableDefinition => {
                let id = match eval_expression(
                    &tail[0],
                    functions,
                    globals,
                    None,
                    sample_set.clone(),
                    out_mode,
                ) {
                    Some(EvaluatedExpr::Identifier(i)) => VariableId::Custom(i),
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
                        // check whether it's a reserved symbol
                        if crate::parser::eval::events::sound::map_symbolic_param_value(&s)
                            .is_some()
                            || crate::music_theory::from_string(&s).is_some()
                        {
                            return None;
                        }
                        VariableId::Symbol(s)
                    }
                    _ => {
                        return None;
                    }
                };

                let mut reduced_tail: Vec<EvaluatedExpr> = Vec::new();
                for expr in tail {
                    if let Some(e) = eval_expression(
                        expr,
                        functions,
                        globals,
                        locals,
                        sample_set.clone(),
                        out_mode,
                    ) {
                        // the list field exists only to be flattened
                        if let EvaluatedExpr::EvaluatedExprList(mut l) = e {
                            reduced_tail.append(&mut l);
                        } else {
                            reduced_tail.push(e);
                        }
                    } else {
                        // jump out if expression can't be evaluated
                        return None;
                    }
                }

                if let Some(EvaluatedExpr::Typed(te)) = reduced_tail.pop() {
                    Some(EvaluatedExpr::VariableDefinition(id, te))
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
}
pub fn eval_from_str(
    src: &str,
    functions: &FunctionMap,
    globals: &sync::Arc<GlobalVariables>,
    sample_set: SampleAndWavematrixSet,
    out_mode: OutputMode,
) -> Result<EvaluatedExpr, String> {
    // preprocessing - remove all comments ...
    let re = Regex::new(r";[^\n]+\n").unwrap();
    let src_nocomment = re.replace_all(src, "\n");
    parse_expr(&src_nocomment)
        .map_err(|e: nom::Err<VerboseError<&str>>| format!("{e:#?}"))
        .and_then(|(_, exp)| {
            eval_expression(&exp, functions, globals, None, sample_set, out_mode)
                .ok_or_else(|| "eval failed".to_string())
        })
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_parse_eval() {
        let snippet = "(text 'tar :lvl 1.0 :global #t :relate #f :boost (bounce 0 400))";

        let mut functions = FunctionMap::new();
        let globals = sync::Arc::new(GlobalVariables::new());
        let sample_set = SampleAndWavematrixSet::new();

        functions
            .std_lib
            .insert("text".to_string(), |_, tail, _, _, _| {
                // SYMBOLS
                if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) =
                    &tail[1]
                {
                    assert!(s == "tar");
                } else {
                    panic!();
                }

                // KEYWORDS
                if let EvaluatedExpr::Keyword(k) = &tail[2] {
                    assert!(k == "lvl");
                } else {
                    panic!();
                }

                if let EvaluatedExpr::Keyword(k) = &tail[4] {
                    assert!(k == "global");
                } else {
                    panic!();
                }

                if let EvaluatedExpr::Keyword(k) = &tail[6] {
                    assert!(k == "relate");
                } else {
                    panic!();
                }

                if let EvaluatedExpr::Keyword(k) = &tail[8] {
                    assert!(k == "boost");
                } else {
                    panic!();
                }

                // BOOLEANS
                if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(b))) =
                    &tail[5]
                {
                    assert!(b);
                } else {
                    panic!();
                }

                if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(b))) =
                    &tail[7]
                {
                    assert!(!b);
                } else {
                    panic!();
                }

                // FLOA
                if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) =
                    &tail[3]
                {
                    assert!(*f == 1.0);
                } else {
                    panic!();
                }

                Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                    Comparable::Boolean(true),
                )))
            });

        functions
            .std_lib
            .insert("bounce".to_string(), |_, tail, _, _, _| {
                if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) =
                    &tail[1]
                {
                    assert!(*f == 0.0);
                } else {
                    panic!();
                }
                if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) =
                    &tail[2]
                {
                    assert!(*f == 400.0);
                } else {
                    panic!();
                }

                Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                    Comparable::Boolean(true),
                )))
            });

        match eval_from_str(
            snippet,
            &functions,
            &globals,
            sample_set,
            OutputMode::Stereo,
        ) {
            Ok(res) => {
                assert!(matches!(
                    res,
                    EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(true)))
                ))
            }
            Err(e) => {
                println!("err {e}");
                panic!()
            }
        }
    }

    #[test]
    fn test_parse_float() {
        assert!(matches!(parse_float("0.0"), Ok(("", Atom::Float(_)))));
        assert!(matches!(parse_float("1.0"), Ok(("", Atom::Float(_)))));
        assert!(matches!(parse_float("-1.0"), Ok(("", Atom::Float(_)))));
    }

    #[test]
    fn test_parse_symbol() {
        assert!(matches!(parse_symbol("'test"), Ok(("", Atom::Symbol(_)))));
        assert!(!matches!(parse_symbol(":test"), Ok(("", Atom::Symbol(_)))));
    }

    #[test]
    fn test_parse_keyword() {
        assert!(matches!(parse_keyword(":test"), Ok(("", Atom::Keyword(_)))));
    }

    #[test]
    fn test_parse_string() {
        assert!(matches!(
            parse_string("\"test\""),
            Ok(("", Atom::String(_)))
        ));
    }

    #[test]
    fn test_parse_boolean() {
        assert!(matches!(parse_boolean("#t"), Ok(("", Atom::Boolean(true)))));
        assert!(matches!(
            parse_boolean("#f"),
            Ok(("", Atom::Boolean(false)))
        ));
    }

    #[test]
    fn test_parse_atom_constant() {
        assert!(matches!(
            parse_constant("#t"),
            Ok(("", Expr::Constant(Atom::Boolean(true))))
        ));
        assert!(matches!(
            parse_constant("#f"),
            Ok(("", Expr::Constant(Atom::Boolean(false))))
        ));
        assert!(matches!(
            parse_constant("'test"),
            Ok(("", Expr::Constant(Atom::Symbol(_))))
        ));
        assert!(matches!(
            parse_constant(":test"),
            Ok(("", Expr::Constant(Atom::Keyword(_))))
        ));
        assert!(matches!(
            parse_constant("\"test\""),
            Ok(("", Expr::Constant(Atom::String(_))))
        ));
    }

    #[test]
    fn test_parse_expr() {
        assert!(matches!(
            parse_expr("#t"),
            Ok(("", Expr::Constant(Atom::Boolean(true))))
        ));
        assert!(matches!(
            parse_expr("#f"),
            Ok(("", Expr::Constant(Atom::Boolean(false))))
        ));
        assert!(matches!(
            parse_expr("'test"),
            Ok(("", Expr::Constant(Atom::Symbol(_))))
        ));
        assert!(matches!(
            parse_expr(":test"),
            Ok(("", Expr::Constant(Atom::Keyword(_))))
        ));
        assert!(matches!(
            parse_expr("\"test\""),
            Ok(("", Expr::Constant(Atom::String(_))))
        ));
        assert!(matches!(
            parse_expr("(#t)"),
            Ok(("", Expr::Application(_, _)))
        ));
        assert!(matches!(
            parse_expr("('test)"),
            Ok(("", Expr::Application(_, _)))
        ));
        assert!(matches!(
            parse_expr("(:test)"),
            Ok(("", Expr::Application(_, _)))
        ));
        assert!(matches!(
            parse_expr("(\"test\")"),
            Ok(("", Expr::Application(_, _)))
        ));

        if let Ok(("", Expr::Application(head, tail))) =
            parse_expr("(text 'tar :lvl 1.0 :global #t :relate #f :boost (bounce 0 400))")
        {
            if let Expr::Constant(Atom::Identifier(function_name)) = *head {
                assert!(function_name == "text");
            } else {
                panic!()
            }

            // SYMBOLS
            if let Expr::Constant(Atom::Symbol(s)) = &tail[0] {
                assert!(s == "tar");
            } else {
                panic!();
            }

            // KEYWORDS
            if let Expr::Constant(Atom::Keyword(k)) = &tail[1] {
                assert!(k == "lvl");
            } else {
                panic!();
            }

            if let Expr::Constant(Atom::Keyword(k)) = &tail[3] {
                assert!(k == "global");
            } else {
                panic!();
            }

            if let Expr::Constant(Atom::Keyword(k)) = &tail[5] {
                assert!(k == "relate");
            } else {
                panic!();
            }

            if let Expr::Constant(Atom::Keyword(k)) = &tail[7] {
                assert!(k == "boost");
            } else {
                panic!();
            }

            // BOOLEANS
            if let Expr::Constant(Atom::Boolean(b)) = &tail[4] {
                assert!(b);
            } else {
                panic!();
            }

            if let Expr::Constant(Atom::Boolean(b)) = &tail[6] {
                assert!(!b);
            } else {
                panic!();
            }

            // FLOAT
            if let Expr::Constant(Atom::Float(f)) = &tail[2] {
                assert!(*f == 1.0);
            } else {
                panic!();
            }

            // APPLICATION
            if let Expr::Application(head2, tail2) = &tail[8] {
                if let Expr::Constant(Atom::Identifier(function_name2)) = &**head2 {
                    assert!(function_name2 == "bounce")
                } else {
                    panic!()
                }
                // FLOAT
                if let Expr::Constant(Atom::Float(f)) = &tail2[0] {
                    assert!(*f == 0.0);
                } else {
                    panic!();
                }
                // FLOAT
                if let Expr::Constant(Atom::Float(f)) = &tail2[1] {
                    assert!(*f == 400.0);
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}
