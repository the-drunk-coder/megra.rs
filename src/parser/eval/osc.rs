use crate::builtin_types::*;
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use std::sync;

pub fn osc_define_sender(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();
    let sender_name = if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
        s
    } else {
        return None;
    };
    let host_name = if let Some(EvaluatedExpr::String(s)) = tail_drain.next() {
        s
    } else {
        return None;
    };

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
        Command::OscDefineClient(sender_name, host_name),
    )))
}

pub fn osc_send(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    let sender_name = if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
        s
    } else {
        return None;
    };
    let addr = if let Some(EvaluatedExpr::String(s)) = tail_drain.next() {
        s
    } else {
        return None;
    };

    let mut args = Vec::new();
    while let Some(thing) = tail_drain.next() {
        match thing {
            EvaluatedExpr::Float(f) => args.push(TypedVariable::Number(f)),
            EvaluatedExpr::Symbol(s) => args.push(TypedVariable::Symbol(s)),
            EvaluatedExpr::String(s) => args.push(TypedVariable::String(s)),
            _ => {}
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Command(
        Command::OscSendMessage(sender_name, addr, args),
    )))
}
