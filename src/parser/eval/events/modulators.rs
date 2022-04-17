use crate::parameter::{Parameter, ParameterValue};
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{GlobalParameters, OutputMode, SampleSet};
use parking_lot::Mutex;
use ruffbox_synth::building_blocks::ValOp;
use std::sync;

pub fn lfo_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let freq = if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
        f
    } else {
        5.0
    };

    let range = if let Some(EvaluatedExpr::Float(r)) = tail_drain.next() {
        r
    } else {
        2.0
    };

    let op = if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
        //println!("{}", s);
        match s.as_str() {
            "add" => ValOp::Add,
            "sub" => ValOp::Subtract,
            "div" => ValOp::Divide,
            "mul" => ValOp::Multiply,
            _ => ValOp::Replace,
        }
    } else {
        ValOp::Replace
    };

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::Lfo(
            Parameter::with_value(freq),
            Parameter::with_value(range),
            op,
        ),
    )))
}
