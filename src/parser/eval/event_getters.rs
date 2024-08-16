use std::sync;

use ruffbox_synth::building_blocks::SynthParameterValue;

use crate::{
    event::InterpretableEvent,
    event_helpers::map_parameter,
    parser::{EvaluatedExpr, FunctionMap},
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
    Comparable, GlobalVariables, TypedEntity,
};

use super::resolver::resolve_globals;

pub fn event_tag(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    let tag_num = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
        tail_drain.next()
    {
        f as usize
    } else {
        return None;
    };

    match tail_drain.next() {
        // only numeric values so far ...
        Some(EvaluatedExpr::Typed(TypedEntity::StaticEvent(InterpretableEvent::Sound(ev)))) => {
            if let Some(t) = ev.tags.into_iter().nth(tag_num) {
                Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                    Comparable::String(t),
                )))
            } else {
                None
            }
        }
        Some(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev))) => {
            if let Some(t) = ev.tags.into_iter().nth(tag_num) {
                Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                    Comparable::String(t),
                )))
            } else {
                None
            }
        }
        Some(EvaluatedExpr::Typed(TypedEntity::ControlEvent(ev))) => {
            if let Some(t) = ev.tags.into_iter().nth(tag_num) {
                Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                    Comparable::String(t),
                )))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn event_param(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
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
            return None;
        }
    };

    match tail_drain.next() {
        // only numeric values so far ...
        Some(EvaluatedExpr::Typed(TypedEntity::StaticEvent(InterpretableEvent::Sound(ev)))) => {
            match ev.params.get(&par_addr) {
                Some(SynthParameterValue::ScalarF32(n)) => Some(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Float(*n)),
                )),
                Some(SynthParameterValue::ScalarU32(n)) => Some(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Int32(*n as i32)),
                )),
                Some(SynthParameterValue::ScalarUsize(n)) => Some(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Int64(*n as i64)),
                )),
                Some(SynthParameterValue::Symbolic(s)) => Some(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::String(s.clone())),
                )),
                _ => None,
            }
        }
        Some(EvaluatedExpr::Typed(TypedEntity::SoundEvent(mut ev))) => {
            let stat_ev = ev.get_static(globals);
            match stat_ev.params.get(&par_addr) {
                Some(SynthParameterValue::ScalarF32(n)) => Some(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Float(*n)),
                )),
                Some(SynthParameterValue::ScalarU32(n)) => Some(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Int32(*n as i32)),
                )),
                Some(SynthParameterValue::ScalarUsize(n)) => Some(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::Int64(*n as i64)),
                )),
                Some(SynthParameterValue::Symbolic(s)) => Some(EvaluatedExpr::Typed(
                    TypedEntity::Comparable(Comparable::String(s.clone())),
                )),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn event_name(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    match tail_drain.next() {
        // only numeric values so far ...
        Some(EvaluatedExpr::Typed(TypedEntity::StaticEvent(InterpretableEvent::Sound(ev)))) => {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::String(ev.name),
            )))
        }
        Some(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev))) => Some(EvaluatedExpr::Typed(
            TypedEntity::Comparable(Comparable::String(ev.name)),
        )),
        Some(EvaluatedExpr::Typed(TypedEntity::ControlEvent(_))) => Some(EvaluatedExpr::Typed(
            TypedEntity::Comparable(Comparable::String("ctrl".to_string())),
        )),
        _ => None,
    }
}
