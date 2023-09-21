use crate::parameter::{
    modifier::bounce_modifier::BounceModifier, modifier::brownian_modifier::BrownianModifier,
    modifier::envelope_modifier::EnvelopeModifier, modifier::randrange_modifier::RandRangeModifier,
    DynVal,
};

use crate::builtin_types::{Comparable, TypedEntity};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{GlobalVariables, OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync;

// Some helpers
fn get_keyword_params(
    tail_drain: &mut std::vec::Drain<EvaluatedExpr>,
) -> HashMap<String, EvaluatedExpr> {
    let mut params = HashMap::new();
    while let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        if let Some(c) = tail_drain.next() {
            params.insert(k, c);
        }
    }
    params
}

fn find_keyword_param(
    raw_params: &HashMap<String, EvaluatedExpr>,
    key: &str,
    default: f32,
) -> DynVal {
    if let Some(b) = raw_params.get(key) {
        match b {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                DynVal::with_value(*f)
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p.clone(),
            _ => DynVal::with_value(default),
        }
    } else {
        DynVal::with_value(default)
    }
}

fn find_keyword_bool(
    raw_params: &HashMap<String, EvaluatedExpr>,
    key: &str,
    default: bool,
) -> bool {
    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(b)))) =
        raw_params.get(key)
    {
        *b
    } else {
        default
    }
}

fn get_next_param(tail_drain: &mut std::vec::Drain<EvaluatedExpr>, default: f32) -> DynVal {
    if let Some(b) = tail_drain.next() {
        match b {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                DynVal::with_value(f)
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
            _ => DynVal::with_value(default),
        }
    } else {
        DynVal::with_value(default)
    }
}

pub fn bounce(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    let min = get_next_param(&mut tail_drain, 0.0);
    let max = get_next_param(&mut tail_drain, 0.0);

    let keyword_params = get_keyword_params(&mut tail_drain);
    let steps = find_keyword_param(&keyword_params, "steps", 128.0);

    //println!("{:?} {:?} {:?}", min, max, steps);

    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(DynVal {
        val: 0.0,
        static_val: 0.0,
        modifier: Some(Box::new(BounceModifier {
            min,
            max,
            steps,
            step_count: 0.0,
        })),
    })))
}

pub fn brownian(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    let min = get_next_param(&mut tail_drain, 0.0);
    let max = get_next_param(&mut tail_drain, 0.0);

    let keyword_params = get_keyword_params(&mut tail_drain);
    let current = find_keyword_param(
        &keyword_params,
        "start",
        max.clone().evaluate_numerical() - min.clone().evaluate_numerical() / 2.0,
    )
    .evaluate_numerical();
    let step_size = find_keyword_param(&keyword_params, "step", 0.1);
    let wrap = find_keyword_bool(&keyword_params, "wrap", true);

    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(DynVal {
        val: 0.0,
        static_val: 0.0,
        modifier: Some(Box::new(BrownianModifier {
            min,
            max,
            step_size,
            current,
            wrap,
        })),
    })))
}

pub fn env(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    let mut collect_steps = false;
    let mut collect_values = false;

    let mut values = Vec::new();
    let mut steps = Vec::new();
    let mut repeat = false;

    while let Some(c) = tail_drain.next() {
        if collect_steps {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                    steps.push(DynVal::with_value(f))
                }
                EvaluatedExpr::Typed(TypedEntity::Parameter(ref p)) => steps.push(p.clone()),
                _ => {
                    collect_steps = false;
                }
            }
        }
        if collect_values {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                    values.push(DynVal::with_value(f))
                }
                EvaluatedExpr::Typed(TypedEntity::Parameter(ref p)) => values.push(p.clone()),
                _ => {
                    collect_values = false;
                }
            }
        }
        if let EvaluatedExpr::Keyword(k) = c {
            match k.as_str() {
                "v" => {
                    collect_values = true;
                }
                "values" => {
                    collect_values = true;
                }
                "s" => {
                    collect_steps = true;
                }
                "steps" => {
                    collect_steps = true;
                }
                "repeat" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.next()
                    {
                        repeat = b;
                    }
                }
                _ => {} // ignore
            }
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(DynVal {
        val: 0.0,
        static_val: 0.0,
        modifier: Some(Box::new(EnvelopeModifier::from_data(
            &values, &steps, repeat,
        ))),
    })))
}

pub fn fade(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    let from = get_next_param(&mut tail_drain, 0.0);
    let to = get_next_param(&mut tail_drain, 0.0);

    let mut values = Vec::new();
    let mut steps = Vec::new();

    values.push(from);
    values.push(to);

    let keyword_params = get_keyword_params(&mut tail_drain);
    steps.push(find_keyword_param(&keyword_params, "steps", 128.0));

    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(DynVal {
        val: 0.0,
        static_val: 0.0,
        modifier: Some(Box::new(EnvelopeModifier::from_data(
            &values, &steps, false,
        ))),
    })))
}

pub fn randrange(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    let min = get_next_param(&mut tail_drain, 0.0);
    let max = get_next_param(&mut tail_drain, 0.0);

    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(DynVal {
        val: 0.0,
        static_val: 0.0,
        modifier: Some(Box::new(RandRangeModifier::from_data(min, max))),
    })))
}
