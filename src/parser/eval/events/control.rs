use crate::event::*;
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{GlobalParameters, OutputMode, SampleSet};
use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::sync;

pub fn control(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut sync_contexts = Vec::new();
    let mut commands = Vec::new();

    for c in tail.drain(..) {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::SyncContext(s)) => {
                sync_contexts.push(s);
            }
            EvaluatedExpr::BuiltIn(BuiltIn::Command(c)) => {
                commands.push(c);
            }
            _ => {} // not controllable
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(
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
        let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));

        functions.insert("saw".to_string(), eval::events::sound::sound);
        functions.insert("ctrl".to_string(), eval::events::control::control);
        functions.insert("sx".to_string(), eval::session::sync_context::sync_context);
        functions.insert("nuc".to_string(), eval::constructors::nuc::nuc);

        let globals = sync::Arc::new(GlobalParameters::new());

        match eval_from_str(snippet, &functions, &globals, &sample_set) {
            Ok(res) => {
                assert!(matches!(
                    res,
                    EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(_))
                ));
            }
            Err(e) => {
                println!("err {}", e);
                assert!(false)
            }
        }
    }
}
