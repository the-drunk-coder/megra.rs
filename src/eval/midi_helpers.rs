use anyhow::{anyhow, bail, Result};

use crate::builtin_types::{Comparable, TypedEntity};
use crate::music_theory::{from_string, to_freq};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{GlobalVariables, OutputMode, SampleAndWavematrixSet};

use std::sync;

pub fn mtof(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(note)))) =
        tail_drain.next()
    {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(base)))) =
            tail_drain.next()
        {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Float(base * f32::powf(2.0, (note - 69.0) / 12.0)),
            )))
        } else {
            Err(anyhow!("mtof - both arguments need to be numbers"))
        }
    } else {
        Err(anyhow!("mtof - both arguments need to be numbers"))
    }
}

/// midi to megra-internal notation
pub fn mtosym(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(note)))) =
        tail_drain.next()
    {
        let pclass = match note as usize % 12 {
            0 => "c",
            1 => "cs",
            2 => "d",
            3 => "ds",
            4 => "e",
            5 => "f",
            6 => "fs",
            7 => "g",
            8 => "gs",
            9 => "a",
            10 => "as",
            11 => "b",
            _ => {
                bail!("mtosym - complete failure")
            }
        };

        let oct = (note / 12.0).floor() as usize;

        let pstring = format!("{}{}", pclass, oct);

        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Symbol(pstring),
        )))
    } else {
        Err(anyhow!(
            "mtosym - only midi numbers can be converted to note symbol"
        ))
    }
}

/// midi to vexflow-style notation
pub fn mtovex(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(note)))) =
        tail_drain.next()
    {
        let pclass = match note as usize % 12 {
            0 => "c",
            1 => "c#",
            2 => "d",
            3 => "d#",
            4 => "e",
            5 => "f",
            6 => "f#",
            7 => "g",
            8 => "g#",
            9 => "a",
            10 => "a#",
            11 => "b",
            _ => {
                unreachable!()
            }
        };

        let oct = (note / 12.0).floor() as usize;

        let pstring = format!("{}/{}", pclass, oct);

        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Symbol(pstring),
        )))
    } else {
        Err(anyhow!(
            "mtovex - only midi numbers can be converted to vexflow symbol"
        ))
    }
}

pub fn veltodyn(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(vel)))) =
        tail_drain.next()
    {
        let dynsym = if vel < 45.0 {
            "p"
        } else if vel > 45.0 && vel < 65.0 {
            "mp"
        } else if vel > 65.0 && vel < 85.0 {
            "mf"
        } else if vel > 85.0 && vel < 105.0 {
            "f"
        } else {
            "ff"
        };

        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Symbol(dynsym.to_string()),
        )))
    } else {
        Err(anyhow!(
            "veltodyn - only numbers can be converted to dynamic symbol"
        ))
    }
}

pub fn symtofreq(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let note_str = tail
        .drain(1..)
        .next()
        .ok_or(anyhow!("symtofreq - empty arg"))?;
    if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) = note_str {
        let note = from_string(&s)?;
        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Float(to_freq(note, crate::music_theory::Tuning::EqualTemperament)),
        )))
    } else {
        Err(anyhow!(
            "symtofreq - only symbols can be converted to strings"
        ))
    }
}
