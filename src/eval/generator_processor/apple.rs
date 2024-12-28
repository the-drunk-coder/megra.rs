use crate::builtin_types::*;
use crate::eval::EvaluatedExpr;
use crate::generator_processor::*;
use crate::parameter::DynVal;

pub fn collect_apple(tail: &mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send + Sync> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // skip function name

    let mut proc = AppleProcessor::new();

    let mut cur_prob = DynVal::with_value(100.0); // if nothing is specified, it's always or prob 100
    let mut gen_mod_funs = Vec::new();

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf),
            )) => {
                gen_mod_funs.push(gmf);
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorModifierList(mut ml)) => {
                for gpom in ml.drain(..) {
                    if let GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf) = gpom {
                        gen_mod_funs.push(gmf);
                    }
                }
            }
            EvaluatedExpr::Keyword(k) => {
                if k == "p" {
                    if !gen_mod_funs.is_empty() {
                        let mut new_mods = Vec::new();
                        new_mods.append(&mut gen_mod_funs);
                        proc.modifiers_to_be_applied
                            .push((cur_prob.clone(), new_mods));
                    }

                    // grab new probability
                    cur_prob = match tail_drain.next() {
                        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                            f,
                        )))) => DynVal::with_value(f),
                        Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => p,
                        _ => DynVal::with_value(1.0),
                    };
                }
            }
            _ => {}
        }
    }

    // save last context
    if !gen_mod_funs.is_empty() {
        proc.modifiers_to_be_applied.push((cur_prob, gen_mod_funs));
    }

    Box::new(proc)
}
