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

/// These are the basic building blocks of our casual lisp language.
/// You might notice that there's no lists in this lisp ... not sure
/// what to call it in that case ...
#[derive(Debug)]
pub enum Atom {
    Float(f32),
    String(String),
    Keyword(String),
    Symbol(String),
    Boolean(bool),
    Function(String),
}

/// Expression Type
#[derive(Debug)]
pub enum Expr {
    Comment,
    Constant(Atom),
    Application(Box<Expr>, Vec<Expr>),
}

#[derive(Debug)]
pub enum EvaluatedExpr {
    Float(f32),
    Symbol(String),
    Keyword(String),
    String(String),
    Boolean(bool),
    FunctionName(String),
}

pub type FunctionMap = HashMap<String, fn(&mut Vec<EvaluatedExpr>) -> Option<EvaluatedExpr>>;

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
        || is_alphanumeric(chr as u8)
        || is_space(chr as u8)
        || is_newline(chr as u8)
}

/// valid chars for a function name, symbol or keyword
fn valid_function_name_char(chr: char) -> bool {
    chr == '_' || chr == '-' || is_alphanumeric(chr as u8)
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
    ))(i)
}

/// keywords are language constructs that start with a ':'
fn parse_keyword(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(
        context(
            "keyword",
            preceded(tag(":"), take_while(valid_function_name_char)),
        ),
        |sym_str: &str| Atom::Keyword(sym_str.to_string()),
    )(i)
}

/// keywords are language constructs that start with a single quote
fn parse_symbol(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(
        context(
            "symbol",
            preceded(tag("'"), take_while(valid_function_name_char)),
        ),
        |sym_str: &str| Atom::Symbol(sym_str.to_string()),
    )(i)
}

/// keywords are language constructs that start with a single quote
fn parse_function(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map(
        context("function", take_while(valid_function_name_char)),
        |sym_str: &str| Atom::Function(sym_str.to_string()),
    )(i)
}

/// floating point numbers ... all numbers currently are ...
fn parse_float(i: &str) -> IResult<&str, Atom, VerboseError<&str>> {
    map_res(recognize(float), |digit_str: &str| {
        digit_str.parse::<f32>().map(Atom::Float)
    })(i)
}

/// parse all the atoms
fn parse_constant(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(
        alt((
            parse_boolean,
            parse_float,
            parse_keyword,
            parse_symbol,
            parse_string,
            parse_function,
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
        tuple((parse_expr, many0(preceded(multispace1, parse_expr)))),
        |(head, tail)| Expr::Application(Box::new(head), tail),
    );
    // finally, we wrap it in an s-expression
    s_exp(application_inner)(i)
}

fn parse_comment(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(preceded(tag(";"), take_while(|ch| ch != '\n')), |_| {
        Expr::Comment
    })(i)
}

/// We tie them all together again, making a top-level expression parser!
/// This one generates the abstract syntax tree
pub fn parse_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    alt((parse_application, parse_constant, parse_comment))(i)
}

/// This one reduces the abstract syntax tree ...
pub fn eval_expression(e: &Expr, functions: &FunctionMap) -> Option<EvaluatedExpr> {
    match e {
        Expr::Comment => None, // ignore comments
        Expr::Constant(c) => Some(match c {
            Atom::Float(f) => EvaluatedExpr::Float(*f),
            Atom::Symbol(s) => EvaluatedExpr::Symbol(s.to_string()),
            Atom::Keyword(k) => EvaluatedExpr::Keyword(k.to_string()),
            Atom::String(s) => EvaluatedExpr::String(s.to_string()),
            Atom::Boolean(b) => EvaluatedExpr::Boolean(*b),
            Atom::Function(f) => EvaluatedExpr::FunctionName(f.to_string()),
        }),
        Expr::Application(head, tail) => {
            if let Some(EvaluatedExpr::FunctionName(f)) = eval_expression(&*head, functions) {
                // check if we have this function ...
                if functions.contains_key(&f) {
                    let mut reduced_tail = tail
                        .iter()
                        .map(|expr| eval_expression(expr, functions))
                        .collect::<Option<Vec<EvaluatedExpr>>>()?;
                    functions[&f](&mut reduced_tail)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

pub fn eval_from_str(src: &str, functions: &FunctionMap) -> Result<EvaluatedExpr, String> {
    parse_expr(src)
        .map_err(|e: nom::Err<VerboseError<&str>>| format!("{:#?}", e))
        .and_then(|(_, exp)| {
            eval_expression(&exp, functions).ok_or_else(|| "eval failed".to_string())
        })
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_all() {
        let snippet = "(text 'tar :lvl 1.0 :global #t :relate #f :boost (bounce 0 400))";

        let mut functions = FunctionMap::new();

        functions.insert("text".to_string(), |tail| {
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

        functions.insert("bounce".to_string(), |tail| {
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

        match eval_from_str(snippet, &functions) {
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
            if let Expr::Constant(Atom::Function(function_name)) = *head {
                assert!(function_name == "text");
            } else {
                assert!(false)
            }

            // SYMBOLS
            if let Expr::Constant(Atom::Symbol(s)) = &tail[0] {
                assert!(s == "tar");
            } else {
                assert!(false);
            }

            // KEYWORDS
            if let Expr::Constant(Atom::Keyword(k)) = &tail[1] {
                assert!(k == "lvl");
            } else {
                assert!(false);
            }

            if let Expr::Constant(Atom::Keyword(k)) = &tail[3] {
                assert!(k == "global");
            } else {
                assert!(false);
            }

            if let Expr::Constant(Atom::Keyword(k)) = &tail[5] {
                assert!(k == "relate");
            } else {
                assert!(false);
            }

            if let Expr::Constant(Atom::Keyword(k)) = &tail[7] {
                assert!(k == "boost");
            } else {
                assert!(false);
            }

            // BOOLEANS
            if let Expr::Constant(Atom::Boolean(b)) = &tail[4] {
                assert!(b);
            } else {
                assert!(false);
            }

            if let Expr::Constant(Atom::Boolean(b)) = &tail[6] {
                assert!(!b);
            } else {
                assert!(false);
            }

            // FLOAT
            if let Expr::Constant(Atom::Float(f)) = &tail[2] {
                assert!(*f == 1.0);
            } else {
                assert!(false);
            }

            // APPLICATION
            if let Expr::Application(head2, tail2) = &tail[8] {
                if let Expr::Constant(Atom::Function(function_name2)) = &**head2 {
                    assert!(function_name2 == "bounce")
                } else {
                    assert!(false)
                }
                // FLOAT
                if let Expr::Constant(Atom::Float(f)) = &tail2[0] {
                    assert!(*f == 0.0);
                } else {
                    assert!(false);
                }
                // FLOAT
                if let Expr::Constant(Atom::Float(f)) = &tail2[1] {
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
