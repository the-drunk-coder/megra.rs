use std::{fs, path};

use crate::{interpreter, session::Session};

pub fn find_closing_paren(text: &str, mut pos: usize) -> Option<usize> {
    let mut par_lvl = 1;

    // spool forward to current position
    for next_char in text.chars().skip(pos) {
        if next_char == '(' {
            par_lvl += 1;
        } else if next_char == ')' {
            par_lvl -= 1;
        }
        pos += 1;
        if par_lvl == 0 {
            return Some(pos);
        }
    }
    None
}

// this is used elsewhere (livecode_text_edit)
pub fn find_opening_paren(text: &str, pos: usize) -> Option<usize> {
    let rev_pos = text.chars().count() - pos;

    // well, should be reverse par level ...
    let mut par_lvl = 1;
    let mut count = 0;
    for next_char in text.chars().rev().skip(rev_pos) {
        if next_char == '(' {
            par_lvl -= 1;
        } else if next_char == ')' {
            par_lvl += 1;
        }
        count += 1;
        if par_lvl == 0 {
            return Some(pos - count);
        }
    }
    None
}

/// take a string and segment it into expressions
pub fn segment_expressions(text: String) -> Vec<String> {
    let mut expr_bounds = Vec::new();
    let mut last_close = 0;
    let mut ignore = false;

    for (pos, next_char) in text.chars().enumerate() {
        if pos == last_close {
            ignore = false;
        }

        if !ignore && next_char == '(' {
            if let Some(close) = find_closing_paren(&text, pos + 1) {
                expr_bounds.push((pos, close));
                ignore = true;
                last_close = close - 1;
            }
        }
    }

    let mut single_exprs = Vec::new();
    for (open, close) in expr_bounds {
        single_exprs.push(text[open..close].to_string());
    }
    single_exprs
}

pub fn parse_file<const BUFSIZE: usize, const NCHAN: usize>(
    path: String,
    session: &Session<BUFSIZE, NCHAN>,
    base_dir: String,
) {
    let mut p = path::Path::new(&path);
    let skpath = format!("{base_dir}/sketchbook/{path}");
    // see if file exists
    if !p.exists() {
        println!("can't find file {path}, check whether it exists in sketchbook");
        // if not, check if file exists in sketchbook

        p = path::Path::new(&skpath);
        if !p.exists() {
            println!("can't find file {skpath} either, aborting");
            return;
        }
    }

    match fs::read_to_string(p) {
        Ok(s) => {
            let expressions = segment_expressions(s);

            for expr in expressions {
                let res = {
                    crate::eval::parse_and_eval_from_str(
                        &expr,
                        &session.functions,
                        &session.globals,
                        session.sample_set.clone(),
                        session.output_mode,
                    )
                };

                if let Ok(res) = res {
                    interpreter::interpret(res, session.clone(), base_dir.clone());
                }
            }
        }
        Err(e) => {
            println!("couldn't parse file {path:?} - {e}");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_segmentation() {
        let a = ";; hi
         (sx 'ba #t (nuc 'hi))

(gog 'pasf (h  fas) ( hol) (((sdafs))))

(progn (ha) (ho))";

        let single_exprs = segment_expressions(a.to_string());

        println!("{single_exprs:?}");

        assert!(single_exprs.len() == 3);
    }
}
