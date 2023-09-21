use crate::builtin_types::*;
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use std::sync;

pub fn osc_define_sender(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();
    let sender_name =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
            tail_drain.next()
        {
            s
        } else {
            return None;
        };
    let host_name =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) =
            tail_drain.next()
        {
            s
        } else {
            return None;
        };

    Some(EvaluatedExpr::Command(Command::OscDefineClient(
        sender_name,
        host_name,
    )))
}

pub fn osc_send(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    let sender_name =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
            tail_drain.next()
        {
            s
        } else {
            return None;
        };
    let addr = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) =
        tail_drain.next()
    {
        s
    } else {
        return None;
    };

    let mut args = Vec::new();
    for thing in tail_drain {
        match thing {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                args.push(TypedEntity::Comparable(Comparable::Float(f)))
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Double(f))) => {
                args.push(TypedEntity::Comparable(Comparable::Double(f)))
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Int32(f))) => {
                args.push(TypedEntity::Comparable(Comparable::Int32(f)))
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Int64(f))) => {
                args.push(TypedEntity::Comparable(Comparable::Int64(f)))
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
                args.push(TypedEntity::Comparable(Comparable::Symbol(s)))
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s))) => {
                args.push(TypedEntity::Comparable(Comparable::String(s)))
            }
            _ => {}
        }
    }

    Some(EvaluatedExpr::Command(Command::OscSendMessage(
        sender_name,
        addr,
        args,
    )))
}

pub fn osc_start_receiver(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    let host_name =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) =
            tail_drain.next()
        {
            s
        } else {
            return None;
        };

    Some(EvaluatedExpr::Command(Command::OscStartReceiver(host_name)))
}
