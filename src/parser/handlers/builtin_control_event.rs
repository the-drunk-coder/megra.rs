use crate::builtin_types::*;
use crate::event::*;
use std::collections::HashSet;

pub fn handle(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut sync_contexts = Vec::new();
    let mut commands = Vec::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::SyncContext(s) => {
                sync_contexts.push(s);
            }
            Atom::Command(c) => {
                commands.push(c);
            }
            _ => {} // not controllable
        }
    }

    Atom::ControlEvent(ControlEvent {
        tags: HashSet::new(),
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
    })
}
