use std::collections::HashMap;

use crate::{
    builtin_types::{TypedEntity, VariableId, VariableStore},
    parser::EvaluatedExpr,
};

pub fn resolve_globals(tail: &mut [EvaluatedExpr], var_store: &std::sync::Arc<VariableStore>) {
    for x in tail.iter_mut() {
        if let EvaluatedExpr::Identifier(i) = x {
            if let Some(thing) = var_store.get(&VariableId::Custom(i.clone())) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        } else if let EvaluatedExpr::Typed(TypedEntity::Symbol(i)) = x {
            if let Some(thing) = var_store.get(&VariableId::Symbol(i.clone())) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        }
    }
}

pub fn resolve_locals(tail: &mut [EvaluatedExpr], var_store: HashMap<String, TypedEntity>) {
    for x in tail.iter_mut() {
        if let EvaluatedExpr::Identifier(i) = x {
            if let Some(thing) = var_store.get(i) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        } else if let EvaluatedExpr::Typed(TypedEntity::Symbol(i)) = x {
            if let Some(thing) = var_store.get(i) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        }
    }
}
