use crate::builtin_types::{Comparable, TypedEntity};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{GlobalVariables, OutputMode, SampleAndWavematrixSet};

use std::sync;

pub fn concat(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let mut sym = false;
    let a = tail_drain.next();

    // first arg determines return type
    let mut accum = match a {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
            sym = true;
            s
        }
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) => s,
        _ => {
            return None; /* first part needs to be symbol or string */
        }
    };

    for x in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Comparable(c)) = x {
            match c {
                Comparable::Boolean(b) => {
                    accum.push_str(&b.to_string());
                }
                Comparable::Float(f) => {
                    accum.push_str(&f.to_string());
                }
                Comparable::Double(d) => {
                    accum.push_str(&d.to_string());
                }
                Comparable::Int32(i) => {
                    accum.push_str(&i.to_string());
                }
                Comparable::Int64(i) => {
                    accum.push_str(&i.to_string());
                }
                Comparable::String(s) => {
                    accum.push_str(&s);
                }
                Comparable::Symbol(s) => {
                    accum.push_str(&s);
                }
                Comparable::Character(c) => {
                    accum.push(c);
                }
            }
        }
    }

    if sym {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Symbol(accum),
        )))
    } else {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::String(accum),
        )))
    }
}
