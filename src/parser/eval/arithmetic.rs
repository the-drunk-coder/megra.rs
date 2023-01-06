use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{GlobalParameters, OutputMode, SampleAndWavematrixSet};

use parking_lot::Mutex;
use std::sync;

// some simple arithmetic functions, to bring megra a bit closer to
// a regular lisp ...

pub fn add(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let mut result = 0.0;
    for n in tail_drain {
        if let EvaluatedExpr::Float(f) = n {
            result += f;
        }
    }
    Some(EvaluatedExpr::Float(result))
}

pub fn sub(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let mut result = if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
        f
    } else {
        0.0
    };

    for n in tail_drain {
        if let EvaluatedExpr::Float(f) = n {
            result -= f;
        }
    }

    Some(EvaluatedExpr::Float(result))
}

pub fn mul(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let mut result = if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
        f
    } else {
        0.0
    };

    for n in tail_drain {
        if let EvaluatedExpr::Float(f) = n {
            result *= f;
        }
    }

    Some(EvaluatedExpr::Float(result))
}

pub fn div(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let mut result = if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
        f
    } else {
        0.0
    };

    for n in tail_drain {
        if let EvaluatedExpr::Float(f) = n {
            result /= f;
        }
    }

    Some(EvaluatedExpr::Float(result))
}

pub fn modulo(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Float(a)) = tail_drain.next() {
        if let Some(EvaluatedExpr::Float(b)) = tail_drain.next() {
            Some(EvaluatedExpr::Float(a % b))
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
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Float(a)) = tail_drain.next() {
        if let Some(EvaluatedExpr::Float(b)) = tail_drain.next() {
            Some(EvaluatedExpr::Float(a.powf(b)))
        } else {
            None
        }
    } else {
        None
    }
}
