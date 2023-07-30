use std::collections::HashMap;

use crate::{
    builtin_types::{Comparable, LazyArithmetic, LazyVal, TypedEntity, VariableId, VariableStore},
    parser::EvaluatedExpr,
};

pub fn needs_resolve(tail: &[EvaluatedExpr]) -> bool {
    let mut resolve = false;
    for x in tail.iter() {
        if let EvaluatedExpr::Identifier(_) = x {
            resolve = true;
        }
    }
    resolve
}

fn resolve_float(i: VariableId, var_store: &std::sync::Arc<VariableStore>, default: f32) -> f32 {
    if let Some(thing) = var_store.get(&i) {
        match thing.value() {
            TypedEntity::Comparable(Comparable::Float(f)) => *f,
            TypedEntity::Comparable(Comparable::Double(f)) => *f as f32,
            TypedEntity::Comparable(Comparable::Int32(f)) => *f as f32,
            TypedEntity::Comparable(Comparable::Int64(f)) => *f as f32,
            _ => default,
        }
    } else {
        default
    }
}

pub fn resolve_lazy(ar: LazyArithmetic, var_store: &std::sync::Arc<VariableStore>) -> f32 {
    match ar {
        LazyArithmetic::Add(mut args) => {
            // ADD ADD ADD
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, var_store, 0.0),
                LazyVal::Arith(a) => resolve_lazy(a, var_store),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum += v,
                    LazyVal::Id(i) => accum += resolve_float(i, var_store, 0.0),
                    LazyVal::Arith(a) => accum += resolve_lazy(a, var_store),
                }
            }
            accum
        }
        LazyArithmetic::Sub(mut args) => {
            // SUB SUB SUB
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, var_store, 0.0),
                LazyVal::Arith(a) => resolve_lazy(a, var_store),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum -= v,
                    LazyVal::Id(i) => accum -= resolve_float(i, var_store, 0.0),
                    LazyVal::Arith(a) => accum -= resolve_lazy(a, var_store),
                }
            }
            accum
        }
        LazyArithmetic::Mul(mut args) => {
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, var_store, 1.0),
                LazyVal::Arith(a) => resolve_lazy(a, var_store),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum *= v,
                    LazyVal::Id(i) => accum *= resolve_float(i, var_store, 1.0),
                    LazyVal::Arith(a) => accum *= resolve_lazy(a, var_store),
                }
            }
            accum
        }
        LazyArithmetic::Div(mut args) => {
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, var_store, 1.0),
                LazyVal::Arith(a) => resolve_lazy(a, var_store),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum /= v,
                    LazyVal::Id(i) => accum /= resolve_float(i, var_store, 1.0),
                    LazyVal::Arith(a) => accum /= resolve_lazy(a, var_store),
                }
            }
            accum
        }
        LazyArithmetic::Modulo(mut args) => {
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, var_store, f32::INFINITY),
                LazyVal::Arith(a) => resolve_lazy(a, var_store),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum %= v,
                    LazyVal::Id(i) => accum %= resolve_float(i, var_store, f32::INFINITY),
                    LazyVal::Arith(a) => accum %= resolve_lazy(a, var_store),
                }
            }
            accum
        }
        LazyArithmetic::Pow(mut args) => {
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, var_store, 1.0),
                LazyVal::Arith(a) => resolve_lazy(a, var_store),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum = accum.powf(v),
                    LazyVal::Id(i) => accum = accum.powf(resolve_float(i, var_store, 1.0)),
                    LazyVal::Arith(a) => accum = accum.powf(resolve_lazy(a, var_store)),
                }
            }
            accum
        }
    }
}

pub fn resolve_globals(tail: &mut [EvaluatedExpr], var_store: &std::sync::Arc<VariableStore>) {
    for x in tail.iter_mut() {
        if let EvaluatedExpr::Identifier(i) = x {
            if let Some(thing) = var_store.get(&VariableId::Custom(i.clone())) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        } else if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(i))) = x {
            if let Some(thing) = var_store.get(&VariableId::Symbol(i.clone())) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        } else if let EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(l)) = x {
            *x = EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(resolve_lazy(
                l.clone(),
                var_store,
            ))));
        }
    }
}

pub fn resolve_locals(tail: &mut [EvaluatedExpr], var_store: HashMap<String, TypedEntity>) {
    for x in tail.iter_mut() {
        if let EvaluatedExpr::Identifier(i) = x {
            if let Some(thing) = var_store.get(i) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        } else if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(i))) = x {
            if let Some(thing) = var_store.get(i) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        }
    }
}
