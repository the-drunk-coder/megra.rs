use crate::builtin_types::*;
use crate::generator::Generator;
use crate::parser::eval::resolver::resolve_globals;
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::session::SyncContext;
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::sync;

pub fn sync_context(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    // eval-time resolve
    // ignore function name
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    // name is the first symbol
    let name = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
        tail_drain.next()
    {
        s
    } else {
        "".to_string()
    };

    let active =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(b)))) =
            tail_drain.next()
        {
            b
        } else {
            false
        };

    if !active {
        return Some(EvaluatedExpr::SyncContext(SyncContext {
            name,
            generators: Vec::new(),
            sync_to: None,
            active: false,
            shift: 0,
            block_tags: BTreeSet::new(),
            solo_tags: BTreeSet::new(),
        }));
    }

    let mut gens: Vec<Generator> = Vec::new();
    let mut sync_to = None;
    let mut shift: i32 = 0;
    let mut collect_block_tags: bool = false;
    let mut collect_solo_tags: bool = false;
    let mut block_tags: BTreeSet<String> = BTreeSet::new();
    let mut solo_tags: BTreeSet<String> = BTreeSet::new();

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => {
                match k.as_str() {
                    "sync" => {
                        collect_solo_tags = false;
                        collect_block_tags = false;
                        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(
                            sync,
                        ))) = tail_drain.next().unwrap()
                        {
                            sync_to = Some(sync);
                        }
                    }
                    "shift" => {
                        collect_solo_tags = false;
                        collect_block_tags = false;
                        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) =
                            tail_drain.next().unwrap()
                        {
                            shift = f as i32;
                        }
                    }
                    "solo" => {
                        collect_block_tags = false;
                        collect_solo_tags = true;
                    }
                    "block" => {
                        collect_solo_tags = false;
                        collect_block_tags = true;
                    }
                    _ => {} // ignore
                }
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
                if collect_solo_tags {
                    solo_tags.insert(s);
                } else if collect_block_tags {
                    block_tags.insert(s);
                }
            }
            EvaluatedExpr::Typed(TypedEntity::Generator(mut k)) => {
                collect_solo_tags = false;
                collect_block_tags = false;
                k.id_tags.insert(name.clone());
                gens.push(k);
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorList(mut kl)) => {
                collect_solo_tags = false;
                collect_block_tags = false;
                for k in kl.iter_mut() {
                    k.id_tags.insert(name.clone());
                }
                gens.append(&mut kl);
            }
            _ => println! {"ignored"},
        }
    }

    Some(EvaluatedExpr::SyncContext(SyncContext {
        name,
        generators: gens,
        sync_to,
        active: true,
        shift,
        block_tags,
        solo_tags,
    }))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parser::*;

    #[test]
    fn test_eval_sx() {
        let snippet = "(sx 'ga #t (nuc 'da (bd)))";
        let mut functions = FunctionMap::new();
        let sample_set = sync::Arc::new(Mutex::new(SampleAndWavematrixSet::new()));

        functions
            .std_lib
            .insert("sx".to_string(), eval::session::sync_context::sync_context);
        functions
            .std_lib
            .insert("nuc".to_string(), eval::constructors::nuc::nuc);
        functions.std_lib.insert("bd".to_string(), |_, _, _, _, _| {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::String("bd".to_string()),
            )))
        });

        let globals = sync::Arc::new(GlobalVariables::new());

        match eval_from_str(
            snippet,
            &functions,
            &globals,
            &sample_set,
            OutputMode::Stereo,
        ) {
            Ok(res) => {
                assert!(matches!(res, EvaluatedExpr::SyncContext(_)));
            }
            Err(e) => {
                println!("err {e}");
                panic!()
            }
        }
    }
}
