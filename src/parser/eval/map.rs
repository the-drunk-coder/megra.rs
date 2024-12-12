use std::collections::HashMap;
use std::sync;

use anyhow::{anyhow, bail, Result};

use crate::builtin_types::*;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn map(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let tail_drain = tail.drain(..).skip(1);

    let mut pmap = HashMap::new();

    for p in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Pair(key, val)) = p {
            match *key {
                TypedEntity::Comparable(Comparable::String(k)) => {
                    pmap.insert(VariableId::Custom(k), *val);
                }
                TypedEntity::Comparable(Comparable::Symbol(k)) => {
                    pmap.insert(VariableId::Symbol(k), *val);
                }
                _ => { /* ignore */ }
            }
        }
    }

    Ok(EvaluatedExpr::Typed(TypedEntity::Map(pmap)))
}

pub fn insert(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(1..);

    let place = match tail_drain.next() {
        Some(EvaluatedExpr::Identifier(i)) => VariableId::Custom(i),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
            VariableId::Symbol(s)
        }
        _ => {
            bail!("insert - invalid map identifier")
        }
    };

    let key = match tail_drain.next() {
        //Some(EvaluatedExpr::Identifier(i)) => VariableId::Custom(i),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) => {
            VariableId::Custom(s)
        }
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
            VariableId::Symbol(s)
        }
        _ => {
            bail!("insert - invalid key")
        }
    };

    if let Some(EvaluatedExpr::Typed(t)) = tail_drain.next() {
        Ok(EvaluatedExpr::Command(Command::Insert(place, key, t)))
    } else {
        Err(anyhow!("insert - can't insert {key:?} into {place:?}"))
    }
}

pub fn get(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(1..);

    let place = match tail_drain.next() {
        Some(EvaluatedExpr::Identifier(i)) => VariableId::Custom(i),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
            VariableId::Symbol(s)
        }
        _ => {
            bail!("map get - invalid map identifier")
        }
    };

    if let Some(map) = globals.get(&place) {
        match map.value() {
            TypedEntity::Map(hash_map) => {
                let key = match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) => {
                        VariableId::Custom(s)
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
                        VariableId::Symbol(s)
                    }
                    _ => {
                        bail!("map get - invalid key")
                    }
                };
                if let Some(res) = hash_map.get(&key) {
                    Ok(EvaluatedExpr::Typed(res.clone()))
                } else {
                    bail!("map get - can't find entry")
                }
            }

            _ => {
                bail!("map get - place is not a map")
            }
        }
    } else {
        bail!("map get - invalid key")
    }
}
