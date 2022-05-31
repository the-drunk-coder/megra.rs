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
    let mut eff_phase = Parameter::with_value(0.0);
    let mut amp = Parameter::with_value(1.0);
    let mut add = Parameter::with_value(0.0);
    let mut op = ValOp::Replace;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "phase" | "p" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => eff_phase = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => eff_phase = p,
                            _ => {}
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => amp = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => amp = p,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => add = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => add = p,
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
        ParameterValue::Lfo(init, freq, eff_phase, amp, add, op),
    )))
}

pub fn lfsaw_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = Parameter::with_value(1.0);
    let mut freq = Parameter::with_value(1.0);
    let mut amp = Parameter::with_value(1.0);
    let mut add = Parameter::with_value(0.0);
    let mut op = ValOp::Replace;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => amp = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => amp = p,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => add = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => add = p,
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
        ParameterValue::LFSaw(init, freq, amp, add, op),
    )))
}

pub fn lftri_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = Parameter::with_value(1.0);
    let mut freq = Parameter::with_value(1.0);
    let mut amp = Parameter::with_value(1.0);
    let mut add = Parameter::with_value(0.0);
    let mut op = ValOp::Replace;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => amp = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => amp = p,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => add = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => add = p,
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
        ParameterValue::LFTri(init, freq, amp, add, op),
    )))
}

pub fn lfsquare_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = Parameter::with_value(1.0);
    let mut freq = Parameter::with_value(1.0);
    let mut amp = Parameter::with_value(1.0);
    let mut add = Parameter::with_value(0.0);
    let mut pw = Parameter::with_value(0.5);
    let mut op = ValOp::Replace;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "pw" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(p) => pw = Parameter::with_value(p),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => pw = p,
                            _ => {}
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => amp = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => amp = p,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => add = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => add = p,
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
        ParameterValue::LFSquare(init, freq, pw, amp, add, op),
    )))
}
