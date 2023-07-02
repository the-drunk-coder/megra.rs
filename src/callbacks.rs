use dashmap::DashMap;

use crate::parser::EvaluatedExpr;

#[derive(Hash, PartialEq, Eq)]
pub enum CallbackKey {
    MidiNote(u8),
    OscAddr(String),
}

pub type CallbackMap = DashMap<CallbackKey, EvaluatedExpr>;
