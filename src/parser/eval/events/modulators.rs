use crate::parameter::{Parameter, ParameterValue};
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{GlobalParameters, OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use ruffbox_synth::building_blocks::ValOp;
use std::sync;

pub fn lfo_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = Parameter::with_value(1.0);
    let mut freq = Parameter::with_value(1.0);
    let mut range = Parameter::with_value(1.0);
    let mut op = ValOp::Replace;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "range" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => range = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => range = p,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
                        match s.as_str() {
                            "add" => op = ValOp::Add,
                            "sub" => op = ValOp::Subtract,
                            "div" => op = ValOp::Divide,
                            "mul" => op = ValOp::Multiply,
                            _ => op = ValOp::Replace,
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::Lfo(init, freq, range, op),
    )))
}
