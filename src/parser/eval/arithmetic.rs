use crate::builtin_types::{Comparable, TypedEntity, LazyFun};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet, VariableStore};

use parking_lot::Mutex;
use std::sync;

use super::resolver::needs_resolution;

// some simple arithmetic functions, to bring megra a bit closer to
// a regular lisp ...

// now, with variables, if there's in-time evaluation, we'd need to return a function in case
// there's an identifier in there ... hmpf ...

pub fn add(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {

    if needs_resolution(&tail[1..]) {
        return Some(EvaluatedExpr::Typed(TypedEntity::LazyFun(LazyFun(tail.clone()))));
    }

    // ignore function name
    let tail_drain = tail.drain(1..);

    let mut result = 0.0;
    for n in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) = n {
            result += f;
        }
    }
    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn sub(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    
    if needs_resolution(&tail[1..]) {
        return Some(EvaluatedExpr::Typed(TypedEntity::LazyFun(LazyFun(tail.clone()))));
    }

    // ignore function name
    let mut tail_drain = tail.drain(1..);

    let mut result =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
            tail_drain.next()
        {
            f
        } else {
            0.0
        };

    for n in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) = n {
            result -= f;
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn mul(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    
    if needs_resolution(&tail[1..]) {
        return Some(EvaluatedExpr::Typed(TypedEntity::LazyFun(LazyFun(tail.clone()))));
    }

    // ignore function name
    let mut tail_drain = tail.drain(1..);

    let mut result =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
            tail_drain.next()
        {
            f
        } else {
            0.0
        };

    for n in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) = n {
            result *= f;
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn div(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    
    if needs_resolution(&tail[1..]) {
        return Some(EvaluatedExpr::Typed(TypedEntity::LazyFun(LazyFun(tail.clone()))));
    }

    // ignore function name
    let mut tail_drain = tail.drain(1..);

    let mut result =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
            tail_drain.next()
        {
            f
        } else {
            0.0
        };

    for n in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) = n {
            result /= f;
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn modulo(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    
    if needs_resolution(&tail[1..]) {
        return Some(EvaluatedExpr::Typed(TypedEntity::LazyFun(LazyFun(tail.clone()))));
    }

    // ignore function name
    let mut tail_drain = tail.drain(1..);

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(a)))) =
        tail_drain.next()
    {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(b)))) =
            tail_drain.next()
        {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Float(a % b),
            )))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn pow(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    
    if needs_resolution(&tail[1..]) {
        return Some(EvaluatedExpr::Typed(TypedEntity::LazyFun(LazyFun(tail.clone()))));
    }

    // ignore function name
    let mut tail_drain = tail.drain(1..);

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(a)))) =
        tail_drain.next()
    {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(b)))) =
            tail_drain.next()
        {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Float(a.powf(b)),
            )))
        } else {
            None
        }
    } else {
        None
    }
}
