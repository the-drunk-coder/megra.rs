use anyhow::{bail, Result};

use crate::builtin_types::*;
use std::sync;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

use super::multiplyer::spread_gens;

pub fn generator_list(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut gen_list = Vec::new();

    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    for c in tail_drain {
        match c {
            EvaluatedExpr::Typed(TypedEntity::Generator(g)) => {
                gen_list.push(g);
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorList(mut gl)) => {
                gen_list.append(&mut gl);
            }
            _ => {
                bail!("u can't list this ...");
            }
        }
    }

    Ok(EvaluatedExpr::Typed(TypedEntity::GeneratorList(gen_list)))
}

pub fn spread_list(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    out_mode: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut gen_list = Vec::new();

    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    for c in tail_drain {
        match c {
            EvaluatedExpr::Typed(TypedEntity::Generator(g)) => {
                gen_list.push(g);
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorList(mut gl)) => {
                gen_list.append(&mut gl);
            }
            _ => {
                bail!("u can't spread this ...");
            }
        }
    }

    spread_gens(&mut gen_list, &out_mode);

    Ok(EvaluatedExpr::Typed(TypedEntity::GeneratorList(gen_list)))
}
