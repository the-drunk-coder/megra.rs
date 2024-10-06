use anyhow::{anyhow, Result};

use crate::builtin_types::{Comparable, LazyArithmetic, LazyVal, TypedEntity};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{GlobalVariables, OutputMode, SampleAndWavematrixSet};

use std::sync;

use super::resolver::needs_resolve;

// some simple arithmetic functions, to bring megra a bit closer to
// a regular lisp ...

// now, with variables, if there's in-time evaluation, we'd need to return a function in case
// there's an identifier in there ... hmpf ...

fn collect_lazy_vals(tail: &mut Vec<EvaluatedExpr>) -> Vec<LazyVal> {
    let mut vals = Vec::new();
    let tail_drain = tail.drain(1..);
    for n in tail_drain {
        match n {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                vals.push(LazyVal::Val(f));
            }
            EvaluatedExpr::Identifier(i) => {
                vals.push(LazyVal::Id(crate::builtin_types::VariableId::Custom(i)));
            }
            EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(a)) => {
                vals.push(LazyVal::Arith(a));
            }
            _ => {}
        }
    }
    vals
}

pub fn add(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    if needs_resolve(&tail[1..]) {
        return Ok(EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(
            LazyArithmetic::Add(collect_lazy_vals(tail)),
        )));
    }

    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let mut result = 0.0;
    for n in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) = n {
            result += f;
        }
    }
    Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn sub(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    if needs_resolve(&tail[1..]) {
        return Ok(EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(
            LazyArithmetic::Sub(collect_lazy_vals(tail)),
        )));
    }

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

    Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn mul(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    if needs_resolve(&tail[1..]) {
        return Ok(EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(
            LazyArithmetic::Mul(collect_lazy_vals(tail)),
        )));
    }

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

    Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn div(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    if needs_resolve(&tail[1..]) {
        return Ok(EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(
            LazyArithmetic::Div(collect_lazy_vals(tail)),
        )));
    }

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

    Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn modulo(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    if needs_resolve(&tail[1..]) {
        return Ok(EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(
            LazyArithmetic::Modulo(collect_lazy_vals(tail)),
        )));
    }

    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(a)))) =
        tail_drain.next()
    {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(b)))) =
            tail_drain.next()
        {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Float(a % b),
            )))
        } else {
            Err(anyhow!("modulo - invalid arguments"))
        }
    } else {
        Err(anyhow!("modulo - invalid arguments"))
    }
}

pub fn pow(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    if needs_resolve(&tail[1..]) {
        return Ok(EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(
            LazyArithmetic::Pow(collect_lazy_vals(tail)),
        )));
    }

    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(a)))) =
        tail_drain.next()
    {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(b)))) =
            tail_drain.next()
        {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Float(a.powf(b)),
            )))
        } else {
            Err(anyhow!("pow - invalid arguments"))
        }
    } else {
        Err(anyhow!("pow - invalid arguments"))
    }
}

pub fn max(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    if needs_resolve(&tail[1..]) {
        return Ok(EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(
            LazyArithmetic::Mul(collect_lazy_vals(tail)),
        )));
    }

    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let mut result =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
            tail_drain.next()
        {
            f
        } else {
            f32::MIN
        };

    for n in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) = n {
            result = f32::max(result, f);
        }
    }

    Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn min(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    if needs_resolve(&tail[1..]) {
        return Ok(EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(
            LazyArithmetic::Mul(collect_lazy_vals(tail)),
        )));
    }

    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let mut result =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
            tail_drain.next()
        {
            f
        } else {
            f32::MAX
        };

    for n in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) = n {
            result = f32::min(result, f);
        }
    }

    Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Float(result),
    )))
}

pub fn round(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => Ok(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f.round()))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Double(d)))) => Ok(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Double(d.round()))),
        ),
        _ => Err(anyhow!("round needs float or double")),
    }
}

pub fn floor(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => Ok(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f.round()))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Double(d)))) => Ok(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Double(d.round()))),
        ),
        _ => Err(anyhow!("floor needs float or double")),
    }
}

pub fn ceil(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => Ok(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f.round()))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Double(d)))) => Ok(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Double(d.round()))),
        ),
        _ => Err(anyhow!("ceil needs float or double")),
    }
}
