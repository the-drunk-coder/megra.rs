use anyhow::Result;

use crate::builtin_types::TypedEntity;
use crate::eval::{EvaluatedExpr, FunctionMap};
use crate::event::*;
use crate::{GlobalVariables, OutputMode, SampleAndWavematrixSet};

use std::collections::BTreeSet;
use std::sync;

pub fn control(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut sync_contexts = Vec::new();
    let mut commands = Vec::new();

    for c in tail.drain(..) {
        match c {
            EvaluatedExpr::SyncContext(s) => {
                sync_contexts.push(s);
            }
            EvaluatedExpr::Command(c) => {
                commands.push(c);
            }
            _ => {} // not controllable
        }
    }

    Ok(EvaluatedExpr::Typed(TypedEntity::ControlEvent(
        ControlEvent {
            tags: BTreeSet::new(),
            ctx: if sync_contexts.is_empty() {
                None
            } else {
                Some(sync_contexts)
            },
            cmd: if commands.is_empty() {
                None
            } else {
                Some(commands)
            },
        },
    )))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_eval_ctrl() {
        let snippet = "(ctrl (sx 'ba #t (nuc 'ad (saw 200))))";
        let functions = FunctionMap::new();
        let sample_set = SampleAndWavematrixSet::new();

        functions
            .std_lib
            .insert("saw".to_string(), crate::eval::events::sound::sound);
        functions
            .std_lib
            .insert("ctrl".to_string(), crate::eval::events::control::control);
        functions.std_lib.insert(
            "sx".to_string(),
            crate::eval::session::sync_context::sync_context,
        );
        functions
            .std_lib
            .insert("nuc".to_string(), crate::eval::constructors::nuc::nuc);

        let globals = sync::Arc::new(GlobalVariables::new());

        match crate::eval::parse_and_eval_from_str(
            snippet,
            &functions,
            &globals,
            sample_set,
            OutputMode::Stereo,
        ) {
            Ok(res) => {
                assert!(matches!(
                    res,
                    EvaluatedExpr::Typed(TypedEntity::ControlEvent(_))
                ));
            }
            Err(e) => {
                println!("err {e}");
                panic!();
            }
        }
    }
}
