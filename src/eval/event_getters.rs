use std::sync;

use anyhow::{anyhow, bail, Result};
use ruffbox_synth::building_blocks::SynthParameterValue;

use crate::{
    eval::{EvaluatedExpr, FunctionMap},
    event::InterpretableEvent,
    event_helpers::map_parameter,
    parameter::ParameterAddress,
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
    Comparable, GlobalVariables, TypedEntity,
};

use super::resolver::{resolve_globals, resolve_lazy};

pub fn event_tag(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    let tag_num = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
        tail_drain.next()
    {
        f as usize
    } else {
        bail!("event-tag - no tag number specified")
    };

    match tail_drain.next() {
        // only numeric values so far ...
        Some(EvaluatedExpr::Typed(TypedEntity::StaticEvent(InterpretableEvent::Sound(ev)))) => ev
            .tags
            .into_iter()
            .nth(tag_num)
            .map(|t| EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(t))))
            .ok_or(anyhow!("event-tag - can't get event tag")),
        Some(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev))) => ev
            .tags
            .into_iter()
            .nth(tag_num)
            .map(|t| EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(t))))
            .ok_or(anyhow!("event-tag - can't get event tag")),
        Some(EvaluatedExpr::Typed(TypedEntity::ControlEvent(ev))) => ev
            .tags
            .into_iter()
            .nth(tag_num)
            .map(|t| EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(t))))
            .ok_or(anyhow!("event-tag - can't get event tag")),
        _ => Err(anyhow!("event-tag - can't get event tag")),
    }
}

pub fn event_param(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    let par_addr = match tail_drain.next() {
        Some(EvaluatedExpr::Keyword(k)) => map_parameter(&k),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
            map_parameter(&s)
        }
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) => {
            map_parameter(&s)
        }
        Some(EvaluatedExpr::Identifier(i)) => map_parameter(&i),
        _ => {
            bail!("event-param - can't extract event param")
        }
    };

    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::StaticEvent(InterpretableEvent::Sound(ev)))) => {
            match ev.params.get(&par_addr) {
                Some(SynthParameterValue::ScalarF32(n)) => Ok(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Float(*n)),
                )),
                Some(SynthParameterValue::ScalarU32(n)) => Ok(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Int32(*n as i32)),
                )),
                Some(SynthParameterValue::ScalarUsize(n)) => Ok(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Int64(*n as i64)),
                )),
                Some(SynthParameterValue::Symbolic(s)) => Ok(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::String(s.clone())),
                )),
                _ => Err(anyhow!("can't extract event param")),
            }
        }
        Some(EvaluatedExpr::Typed(TypedEntity::SoundEvent(mut iev))) => {
            let ev = iev.get_static(globals);
            match ev.params.get(&par_addr) {
                Some(SynthParameterValue::ScalarF32(n)) => Ok(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Float(*n)),
                )),
                Some(SynthParameterValue::ScalarU32(n)) => Ok(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Int32(*n as i32)),
                )),
                Some(SynthParameterValue::ScalarUsize(n)) => Ok(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Int64(*n as i64)),
                )),
                Some(SynthParameterValue::Symbolic(s)) => Ok(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::String(s.clone())),
                )),
                _ => Err(anyhow!("can't extract event param")),
            }
        }
        _ => Err(anyhow!("can't extract event param")),
    }
}

pub fn event_name(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    match tail_drain.next() {
        // only numeric values so far ...
        Some(EvaluatedExpr::Typed(TypedEntity::StaticEvent(InterpretableEvent::Sound(ev)))) => Ok(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(ev.name))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev))) => Ok(EvaluatedExpr::Typed(
            TypedEntity::Comparable(Comparable::String(ev.name)),
        )),
        Some(EvaluatedExpr::Typed(TypedEntity::ControlEvent(_))) => Ok(EvaluatedExpr::Typed(
            TypedEntity::Comparable(Comparable::String("ctrl".to_string())),
        )),
        _ => Err(anyhow!("event-name - can't extract name")),
    }
}
