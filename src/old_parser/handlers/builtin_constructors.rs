use crate::builtin_types::*;
use crate::sample_set::SampleSet;
use crate::session::OutputMode;
use parking_lot::Mutex;
use std::sync;

mod construct_chop;
mod construct_cycle;
mod construct_flower;
mod construct_friendship;
mod construct_fully;
mod construct_infer;
mod construct_learn;
mod construct_linear;
mod construct_loop;
mod construct_nucleus;
mod construct_stages;

pub fn handle(
    constructor_type: &BuiltInConstructor,
    tail: &mut Vec<Expr>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
    global_parameters: &sync::Arc<GlobalParameters>,
) -> Atom {
    match constructor_type {
        BuiltInConstructor::Infer => construct_infer::construct_infer(tail, global_parameters),
        BuiltInConstructor::Learn => construct_learn::construct_learn(tail, global_parameters),
        BuiltInConstructor::Rule => construct_infer::construct_rule(tail),
        BuiltInConstructor::Nucleus => {
            construct_nucleus::construct_nucleus(tail, global_parameters)
        }
        BuiltInConstructor::Flower => construct_flower::construct_flower(tail, global_parameters),
        BuiltInConstructor::Friendship => {
            construct_friendship::construct_friendship(tail, global_parameters)
        }
        BuiltInConstructor::Fully => construct_fully::construct_fully(tail, global_parameters),
        BuiltInConstructor::Cycle => {
            construct_cycle::construct_cycle(tail, sample_set, out_mode, global_parameters)
        }
        BuiltInConstructor::Chop => construct_chop::construct_chop(tail, global_parameters),
        BuiltInConstructor::Stages => construct_stages::construct_stages(tail, global_parameters),
        BuiltInConstructor::Linear => construct_linear::construct_linear(tail, global_parameters),
        BuiltInConstructor::Loop => construct_loop::construct_loop(tail, global_parameters),
    }
}
