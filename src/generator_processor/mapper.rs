use std::{cell::RefCell, sync::Arc};

use crate::{
    eval::{eval_usr_fun_evaluated_tail, EvaluatedExpr, FunctionMap, LocalVariables},
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
    TypedEntity,
};

use super::GeneratorProcessor;

#[derive(Clone)]
pub struct MapperProcessor {
    pub fun: String,
    // if false, it's an actual "mapper",
    // if true, the incoming events are kept (i.e. if the mapping)
    // function doesn't return anything and is just there for
    // side effects ...
    pub keep: bool,
}

impl GeneratorProcessor for MapperProcessor {
    fn process_events(
        &mut self,
        events: &mut Vec<crate::event::InterpretableEvent>,
        globals: &std::sync::Arc<crate::GlobalVariables>,
        functions: &Arc<FunctionMap>,
        sample_set: SampleAndWavematrixSet,
        out_mode: OutputMode,
    ) {
        if functions.usr_lib.contains_key(&self.fun) {
            let (fun_arg_names, fun_expr) = functions.usr_lib.get(&self.fun).unwrap().clone();
            //println!("EVAL MAPPER FUN {}", self.fun);
            let processed_events = eval_usr_fun_evaluated_tail(
                fun_arg_names,
                fun_expr,
                events
                    .iter()
                    .map(|ev| EvaluatedExpr::Typed(TypedEntity::StaticEvent(ev.clone())))
                    .collect(),
                functions,
                globals,
                std::rc::Rc::new(RefCell::new(LocalVariables::new())),
                sample_set,
                out_mode,
            );

            if !self.keep {
                events.clear();
            }

            if let Ok(pe) = processed_events {
                match pe {
                    crate::eval::EvaluatedExpr::Typed(TypedEntity::SoundEvent(mut ev)) => {
                        events.push(crate::event::InterpretableEvent::Sound(
                            ev.get_static(globals),
                        ));
                    }
                    crate::eval::EvaluatedExpr::Typed(TypedEntity::ControlEvent(ev)) => {
                        events.push(crate::event::InterpretableEvent::Control(ev));
                    }
                    crate::eval::EvaluatedExpr::Typed(TypedEntity::StaticEvent(ev)) => {
                        events.push(ev);
                    }
                    _ => {}
                }
            }
        }
    }
}
