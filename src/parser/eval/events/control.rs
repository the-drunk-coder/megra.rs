use crate::builtin_types::TypedEntity;
use crate::event::*;
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet, VariableStore};
use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::sync;

pub fn control(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
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

    Some(EvaluatedExpr::Typed(TypedEntity::ControlEvent(
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
    use crate::parser::*;

    #[test]
    fn test_eval_ctrl() {
        let snippet = "(ctrl (sx 'ba #t (nuc 'ad (saw 200))))";
        let mut functions = FunctionMap::new();
        let sample_set = sync::Arc::new(Mutex::new(SampleAndWavematrixSet::new()));

        functions
            .std_lib
            .insert("saw".to_string(), eval::events::sound::sound);
        functions
            .std_lib
            .insert("ctrl".to_string(), eval::events::control::control);
        functions
            .std_lib
            .insert("sx".to_string(), eval::session::sync_context::sync_context);
        functions
            .std_lib
            .insert("nuc".to_string(), eval::constructors::nuc::nuc);

        let globals = sync::Arc::new(VariableStore::new());

        match eval_from_str(
            snippet,
            &functions,
            &globals,
            &sample_set,
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
                assert!(false)
            }
        }
    }
}
