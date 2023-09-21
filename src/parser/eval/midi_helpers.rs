use crate::builtin_types::{Comparable, TypedEntity};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{GlobalVariables, OutputMode, SampleAndWavematrixSet};

use parking_lot::Mutex;
use std::sync;

pub fn mtof(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(note)))) =
        tail_drain.next()
    {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(base)))) =
            tail_drain.next()
        {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Float(base * f32::powf(2.0, (note - 69.0) / 12.0)),
            )))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn mtosym(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(note)))) =
        tail_drain.next()
    {
        let pclass = match note as usize % 12 {
            0 => "c",
            1 => "cs",
            2 => "d",
            3 => "ds",
            4 => "e",
            5 => "f",
            6 => "fs",
            7 => "g",
            8 => "gs",
            9 => "a",
            10 => "as",
            11 => "b",
            _ => {
                unreachable!()
            }
        };

        let oct = (note / 12.0).floor() as usize;

        let pstring = format!("{}{}", pclass, oct);

        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Symbol(pstring),
        )))
    } else {
        None
    }
}

pub fn veltodyn(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(vel)))) =
        tail_drain.next()
    {
        let dynsym = if vel < 45.0 {
            "p"
        } else if vel > 45.0 && vel < 65.0 {
            "mp"
        } else if vel > 65.0 && vel < 85.0 {
            "mf"
        } else if vel > 85.0 && vel < 105.0 {
            "f"
        } else {
            "ff"
        };

        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Symbol(dynsym.to_string()),
        )))
    } else {
        None
    }
}
