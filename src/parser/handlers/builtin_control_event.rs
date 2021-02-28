use crate::builtin_types::*;
use crate::event::*;
use std::collections::HashSet;

pub fn handle(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut sync_contexts = Vec::new();

    while let Some(Expr::Constant(Atom::SyncContext(s))) = tail_drain.next() {
        sync_contexts.push(s);
    }

    Atom::ControlEvent(ControlEvent {
        tags: HashSet::new(),
        ctx: if sync_contexts.is_empty() {
            None
        } else {
            Some(sync_contexts)
        },
    })
}
