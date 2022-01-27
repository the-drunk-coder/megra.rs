use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char, multispace0, multispace1},
    character::{is_alphanumeric, is_newline, is_space},
    combinator::{cut, map, map_res, recognize},
    error::{context, VerboseError},
    multi::many0,
    number::complete::float,
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};
use std::collections::HashMap;
use std::sync;
use crate::GlobalParameters;
use crate::generator::Generator;

mod reduce_nuc;

/// These are the basic building blocks of our casual lisp language.
/// You might notice that there's no lists in this lisp ... not sure
/// what to call it in that case ...
#[derive(Debug)]
enum Atom2 {
    Float(f32),
    String(String),
    Keyword(String),
    Symbol(String),
    Boolean(bool),
    Function(String),
}

/// Expression Type
#[derive(Debug)]
enum Expr2 {
    Comment,
    Constant(Atom2),
    Application(Box<Expr2>, Vec<Expr2>),
}

pub enum BuiltIn2 {    
    Generator(Generator),
}

pub enum EvaluatedExpr {
    Float(f32),
    Symbol(String),
    Keyword(String),
    String(String),
    Boolean(bool),
    FunctionName(String),
    BuiltIn(BuiltIn2)
}

pub type FunctionMap = HashMap<String, fn(&mut Vec<EvaluatedExpr>, &sync::Arc<GlobalParameters>) -> Option<EvaluatedExpr>>;

/// valid chars for a string
fn valid_string_char2(chr: char) -> bool {
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
        || is_newline(chr as u8)
}

/// valid chars for a function name, symbol or keyword
fn valid_function_name_char2(chr: char) -> bool {
    chr == '_' || chr == '-' || is_alphanumeric(chr as u8)
}

/// parse a string, which is enclosed in double quotes
fn parse_string2(i: &str) -> IResult<&str, Atom2, VerboseError<&str>> {
    map(
        delimited(tag("\""), take_while(valid_string_char2), tag("\"")),
        |desc_str: &str| Atom2::String(desc_str.to_string()),
    )(i)
}

/// booleans have a # prefix
fn parse_boolean2(i: &str) -> IResult<&str, Atom2, VerboseError<&str>> {
    alt((
        map(tag("#t"), |_| Atom2::Boolean(true)),
        map(tag("#f"), |_| Atom2::Boolean(false)),
    ))(i)
}

/// keywords are language constructs that start with a ':'
fn parse_keyword2(i: &str) -> IResult<&str, Atom2, VerboseError<&str>> {
    map(
        context(
            "keyword",
            preceded(tag(":"), take_while(valid_function_name_char2)),
        ),
        |sym_str: &str| Atom2::Keyword(sym_str.to_string()),
    )(i)
}

/// keywords are language constructs that start with a single quote
fn parse_symbol2(i: &str) -> IResult<&str, Atom2, VerboseError<&str>> {
    map(
        context(
            "symbol",
            preceded(tag("'"), take_while(valid_function_name_char2)),
        ),
        |sym_str: &str| Atom2::Symbol(sym_str.to_string()),
    )(i)
}

/// keywords are language constructs that start with a single quote
fn parse_function2(i: &str) -> IResult<&str, Atom2, VerboseError<&str>> {
    map(
        context("function", take_while(valid_function_name_char2)),
        |sym_str: &str| Atom2::Function(sym_str.to_string()),
    )(i)
}

/// floating point numbers ... all numbers currently are ...
fn parse_float2(i: &str) -> IResult<&str, Atom2, VerboseError<&str>> {
    map_res(recognize(float), |digit_str: &str| {
        digit_str.parse::<f32>().map(Atom2::Float)
    })(i)
}

/// parse all the atoms
fn parse_constant2(i: &str) -> IResult<&str, Expr2, VerboseError<&str>> {
    map(
        alt((
            parse_boolean2,
            parse_float2,
            parse_keyword2,
            parse_symbol2,
            parse_string2,
            parse_function2,
        )),
        Expr2::Constant,
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
fn parse_application2(i: &str) -> IResult<&str, Expr2, VerboseError<&str>> {
    let application_inner = map(
        tuple((parse_expr2, many0(preceded(multispace1, parse_expr2)))),
        |(head, tail)| Expr2::Application(Box::new(head), tail),
    );
    // finally, we wrap it in an s-expression
    s_exp(application_inner)(i)
}

fn parse_comment2(i: &str) -> IResult<&str, Expr2, VerboseError<&str>> {
    map(preceded(tag(";"), take_while(|ch| ch != '\n')), |_| {
        Expr2::Comment
    })(i)
}

/// We tie them all together again, making a top-level expression parser!
/// This one generates the abstract syntax tree
fn parse_expr2(i: &str) -> IResult<&str, Expr2, VerboseError<&str>> {
    alt((parse_application2, parse_constant2, parse_comment2))(i)
}

/// This one reduces the abstract syntax tree ...
fn eval_expression2(e: &Expr2,
		    functions: &FunctionMap,
		    globals: &sync::Arc<GlobalParameters>) -> Option<EvaluatedExpr> {
    match e {
        Expr2::Comment => None, // ignore comments
        Expr2::Constant(c) => Some(match c {
            Atom2::Float(f) => EvaluatedExpr::Float(*f),
            Atom2::Symbol(s) => EvaluatedExpr::Symbol(s.to_string()),
            Atom2::Keyword(k) => EvaluatedExpr::Keyword(k.to_string()),
            Atom2::String(s) => EvaluatedExpr::String(s.to_string()),
            Atom2::Boolean(b) => EvaluatedExpr::Boolean(*b),
            Atom2::Function(f) => EvaluatedExpr::FunctionName(f.to_string()),
        }),
        Expr2::Application(head, tail) => {
            if let Some(EvaluatedExpr::FunctionName(f)) = eval_expression2(&*head, functions, globals) {
                // check if we have this function ...
                if functions.contains_key(&f) {
                    let mut reduced_tail = tail
                        .iter()
                        .map(|expr| eval_expression2(expr, functions, globals))
                        .collect::<Option<Vec<EvaluatedExpr>>>()?;
                    functions[&f](&mut reduced_tail, globals)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

pub fn eval_from_str2(src: &str,
		      functions: &FunctionMap,
		      globals: &sync::Arc<GlobalParameters>) -> Result<EvaluatedExpr, String> {
    parse_expr2(src)
        .map_err(|e: nom::Err<VerboseError<&str>>| format!("{:#?}", e))
        .and_then(|(_, exp)| {
            eval_expression2(&exp, functions, globals).ok_or_else(|| "eval failed".to_string())
        })
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_parse_nuc() {
	let snippet = "(nuc 'da (bd))";
	let mut functions = FunctionMap::new();
	
	functions.insert("nuc".to_string(), reduce_nuc::reduce_nuc);
	functions.insert("bd".to_string(), |_,_| Some(EvaluatedExpr::String("bd".to_string())));
			 
	let globals = sync::Arc::new(GlobalParameters::new());
	
	match eval_from_str2(snippet, &functions, &globals) {
            Ok(res) => {
                assert!(matches!(res, EvaluatedExpr::BuiltIn(BuiltIn2::Generator(_))));
            }
            Err(e) => {
                println!("err {}", e);
                assert!(false)
            }
        }
    }
    
    #[test]
    fn test_parse_all() {
        let snippet = "(text 'tar :lvl 1.0 :global #t :relate #f :boost (bounce 0 400))";
	
        let mut functions = FunctionMap::new();
	let mut globals = sync::Arc::new(GlobalParameters::new());

        functions.insert("text".to_string(), |tail, _| {
            // SYMBOLS
            if let EvaluatedExpr::Symbol(s) = &tail[0] {
                assert!(s == "tar");
            } else {
                assert!(false);
            }

            // KEYWORDS
            if let EvaluatedExpr::Keyword(k) = &tail[1] {
                assert!(k == "lvl");
            } else {
                assert!(false);
            }

            if let EvaluatedExpr::Keyword(k) = &tail[3] {
                assert!(k == "global");
            } else {
                assert!(false);
            }

            if let EvaluatedExpr::Keyword(k) = &tail[5] {
                assert!(k == "relate");
            } else {
                assert!(false);
            }

            if let EvaluatedExpr::Keyword(k) = &tail[7] {
                assert!(k == "boost");
            } else {
                assert!(false);
            }

            // BOOLEANS
            if let EvaluatedExpr::Boolean(b) = &tail[4] {
                assert!(b);
            } else {
                assert!(false);
            }

            if let EvaluatedExpr::Boolean(b) = &tail[6] {
                assert!(!b);
            } else {
                assert!(false);
            }

            // FLOAT
            if let EvaluatedExpr::Float(f) = &tail[2] {
                assert!(*f == 1.0);
            } else {
                assert!(false);
            }

            Some(EvaluatedExpr::Boolean(true))
        });

        functions.insert("bounce".to_string(), |tail, _| {
            if let EvaluatedExpr::Float(f) = &tail[0] {
                assert!(*f == 0.0);
            } else {
                assert!(false);
            }
            if let EvaluatedExpr::Float(f) = &tail[1] {
                assert!(*f == 400.0);
            } else {
                assert!(false);
            }

            Some(EvaluatedExpr::Boolean(true))
        });

        match eval_from_str2(snippet, &functions, &globals) {
            Ok(res) => {
                assert!(matches!(res, EvaluatedExpr::Boolean(true)))
            }
            Err(e) => {
                println!("err {}", e);
                assert!(false)
            }
        }
    }

    #[test]
    fn test_parse_float() {
        assert!(matches!(parse_float2("0.0"), Ok(("", Atom2::Float(_)))));
        assert!(matches!(parse_float2("1.0"), Ok(("", Atom2::Float(_)))));
        assert!(matches!(parse_float2("-1.0"), Ok(("", Atom2::Float(_)))));
    }

    #[test]
    fn test_parse_symbol() {
        assert!(matches!(parse_symbol2("'test"), Ok(("", Atom2::Symbol(_)))));
        assert!(!matches!(parse_symbol2(":test"), Ok(("", Atom2::Symbol(_)))));
    }

    #[test]
    fn test_parse_keyword() {
        assert!(matches!(parse_keyword2(":test"), Ok(("", Atom2::Keyword(_)))));
    }

    #[test]
    fn test_parse_string() {
        assert!(matches!(
            parse_string2("\"test\""),
            Ok(("", Atom2::String(_)))
        ));
    }

    #[test]
    fn test_parse_boolean() {
        assert!(matches!(parse_boolean2("#t"), Ok(("", Atom2::Boolean(true)))));
        assert!(matches!(
            parse_boolean2("#f"),
            Ok(("", Atom2::Boolean(false)))
        ));
    }

    #[test]
    fn test_parse_atom_constant() {
        assert!(matches!(
            parse_constant2("#t"),
            Ok(("", Expr2::Constant(Atom2::Boolean(true))))
        ));
        assert!(matches!(
            parse_constant2("#f"),
            Ok(("", Expr2::Constant(Atom2::Boolean(false))))
        ));
        assert!(matches!(
            parse_constant2("'test"),
            Ok(("", Expr2::Constant(Atom2::Symbol(_))))
        ));
        assert!(matches!(
            parse_constant2(":test"),
            Ok(("", Expr2::Constant(Atom2::Keyword(_))))
        ));
        assert!(matches!(
            parse_constant2("\"test\""),
            Ok(("", Expr2::Constant(Atom2::String(_))))
        ));
    }

    #[test]
    fn test_parse_expr() {
        assert!(matches!(
            parse_expr2("#t"),
            Ok(("", Expr2::Constant(Atom2::Boolean(true))))
        ));
        assert!(matches!(
            parse_expr2("#f"),
            Ok(("", Expr2::Constant(Atom2::Boolean(false))))
        ));
        assert!(matches!(
            parse_expr2("'test"),
            Ok(("", Expr2::Constant(Atom2::Symbol(_))))
        ));
        assert!(matches!(
            parse_expr2(":test"),
            Ok(("", Expr2::Constant(Atom2::Keyword(_))))
        ));
        assert!(matches!(
            parse_expr2("\"test\""),
            Ok(("", Expr2::Constant(Atom2::String(_))))
        ));
        assert!(matches!(
            parse_expr2("(#t)"),
            Ok(("", Expr2::Application(_, _)))
        ));
        assert!(matches!(
            parse_expr2("('test)"),
            Ok(("", Expr2::Application(_, _)))
        ));
        assert!(matches!(
            parse_expr2("(:test)"),
            Ok(("", Expr2::Application(_, _)))
        ));
        assert!(matches!(
            parse_expr2("(\"test\")"),
            Ok(("", Expr2::Application(_, _)))
        ));

        if let Ok(("", Expr2::Application(head, tail))) =
            parse_expr2("(text 'tar :lvl 1.0 :global #t :relate #f :boost (bounce 0 400))")
        {
            if let Expr2::Constant(Atom2::Function(function_name)) = *head {
                assert!(function_name == "text");
            } else {
                assert!(false)
            }

            // SYMBOLS
            if let Expr2::Constant(Atom2::Symbol(s)) = &tail[0] {
                assert!(s == "tar");
            } else {
                assert!(false);
            }

            // KEYWORDS
            if let Expr2::Constant(Atom2::Keyword(k)) = &tail[1] {
                assert!(k == "lvl");
            } else {
                assert!(false);
            }

            if let Expr2::Constant(Atom2::Keyword(k)) = &tail[3] {
                assert!(k == "global");
            } else {
                assert!(false);
            }

            if let Expr2::Constant(Atom2::Keyword(k)) = &tail[5] {
                assert!(k == "relate");
            } else {
                assert!(false);
            }

            if let Expr2::Constant(Atom2::Keyword(k)) = &tail[7] {
                assert!(k == "boost");
            } else {
                assert!(false);
            }

            // BOOLEANS
            if let Expr2::Constant(Atom2::Boolean(b)) = &tail[4] {
                assert!(b);
            } else {
                assert!(false);
            }

            if let Expr2::Constant(Atom2::Boolean(b)) = &tail[6] {
                assert!(!b);
            } else {
                assert!(false);
            }

            // FLOAT
            if let Expr2::Constant(Atom2::Float(f)) = &tail[2] {
                assert!(*f == 1.0);
            } else {
                assert!(false);
            }

            // APPLICATION
            if let Expr2::Application(head2, tail2) = &tail[8] {
                if let Expr2::Constant(Atom2::Function(function_name2)) = &**head2 {
                    assert!(function_name2 == "bounce")
                } else {
                    assert!(false)
                }
                // FLOAT
                if let Expr2::Constant(Atom2::Float(f)) = &tail2[0] {
                    assert!(*f == 0.0);
                } else {
                    assert!(false);
                }
                // FLOAT
                if let Expr2::Constant(Atom2::Float(f)) = &tail2[1] {
                    assert!(*f == 400.0);
                } else {
                    assert!(false);
                }
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }
}
