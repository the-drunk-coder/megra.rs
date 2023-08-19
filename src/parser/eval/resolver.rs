use std::collections::HashMap;

use crate::{
    builtin_types::{
        Comparable, GlobalVariables, LazyArithmetic, LazyVal, TypedEntity, VariableId,
    },
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

fn resolve_float(i: VariableId, globals: &std::sync::Arc<GlobalVariables>, default: f32) -> f32 {
    if let Some(thing) = globals.get(&i) {
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

pub fn resolve_lazy(ar: LazyArithmetic, globals: &std::sync::Arc<GlobalVariables>) -> f32 {
    match ar {
        LazyArithmetic::Add(mut args) => {
            // ADD ADD ADD
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, globals, 0.0),
                LazyVal::Arith(a) => resolve_lazy(a, globals),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum += v,
                    LazyVal::Id(i) => accum += resolve_float(i, globals, 0.0),
                    LazyVal::Arith(a) => accum += resolve_lazy(a, globals),
                }
            }
            accum
        }
        LazyArithmetic::Sub(mut args) => {
            // SUB SUB SUB
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, globals, 0.0),
                LazyVal::Arith(a) => resolve_lazy(a, globals),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum -= v,
                    LazyVal::Id(i) => accum -= resolve_float(i, globals, 0.0),
                    LazyVal::Arith(a) => accum -= resolve_lazy(a, globals),
                }
            }
            accum
        }
        LazyArithmetic::Mul(mut args) => {
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, globals, 1.0),
                LazyVal::Arith(a) => resolve_lazy(a, globals),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum *= v,
                    LazyVal::Id(i) => accum *= resolve_float(i, globals, 1.0),
                    LazyVal::Arith(a) => accum *= resolve_lazy(a, globals),
                }
            }
            accum
        }
        LazyArithmetic::Div(mut args) => {
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, globals, 1.0),
                LazyVal::Arith(a) => resolve_lazy(a, globals),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum /= v,
                    LazyVal::Id(i) => accum /= resolve_float(i, globals, 1.0),
                    LazyVal::Arith(a) => accum /= resolve_lazy(a, globals),
                }
            }
            accum
        }
        LazyArithmetic::Modulo(mut args) => {
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, globals, f32::INFINITY),
                LazyVal::Arith(a) => resolve_lazy(a, globals),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum %= v,
                    LazyVal::Id(i) => accum %= resolve_float(i, globals, f32::INFINITY),
                    LazyVal::Arith(a) => accum %= resolve_lazy(a, globals),
                }
            }
            accum
        }
        LazyArithmetic::Pow(mut args) => {
            let mut accum = match args.remove(0) {
                LazyVal::Val(v) => v,
                LazyVal::Id(i) => resolve_float(i, globals, 1.0),
                LazyVal::Arith(a) => resolve_lazy(a, globals),
            };

            for x in args {
                match x {
                    LazyVal::Val(v) => accum = accum.powf(v),
                    LazyVal::Id(i) => accum = accum.powf(resolve_float(i, globals, 1.0)),
                    LazyVal::Arith(a) => accum = accum.powf(resolve_lazy(a, globals)),
                }
            }
            accum
        }
    }
}

pub fn resolve_globals(tail: &mut [EvaluatedExpr], globals: &std::sync::Arc<GlobalVariables>) {
    for x in tail.iter_mut() {
        if let EvaluatedExpr::Identifier(i) = x {
            if let Some(thing) = globals.get(&VariableId::Custom(i.clone())) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        } else if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(i))) = x {
            if let Some(thing) = globals.get(&VariableId::Symbol(i.clone())) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        } else if let EvaluatedExpr::Typed(TypedEntity::LazyArithmetic(l)) = x {
            *x = EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(resolve_lazy(
                l.clone(),
                globals,
            ))));
        }
    }
}

pub fn resolve_locals(tail: &mut [EvaluatedExpr], globals: HashMap<String, TypedEntity>) {
    for x in tail.iter_mut() {
        if let EvaluatedExpr::Identifier(i) = x {
            if let Some(thing) = globals.get(i) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        } else if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(i))) = x {
            if let Some(thing) = globals.get(i) {
                *x = EvaluatedExpr::Typed(thing.clone());
            }
        }
    }
}
