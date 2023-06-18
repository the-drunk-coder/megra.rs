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
        }
    }
}
