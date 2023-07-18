use crate::builtin_types::{Comparable, TypedEntity};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet, VariableStore};

use parking_lot::Mutex;
use std::sync;

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
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

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
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

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
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

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
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

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
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

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
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

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

pub fn mtof(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
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
