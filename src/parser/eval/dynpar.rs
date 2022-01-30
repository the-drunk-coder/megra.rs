use crate::parameter::{
    modifier::bounce_modifier::BounceModifier, modifier::brownian_modifier::BrownianModifier,
    modifier::envelope_modifier::EnvelopeModifier, modifier::randrange_modifier::RandRangeModifier,
    Parameter,
};

use std::sync;
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleSet};
use parking_lot::Mutex;

pub fn bounce(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let min = if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
	f
    } else {
	0.0
    };

    let max = if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
	f
    } else {
	0.0
    };
    
    let steps = if let Some(EvaluatedExpr::Keyword("steps")) = tail_drain.next() {
	if let Some(EvaluatedExpr::Float(f)) {
	    f
	} else {
	    128.0
	}
    } else {
	128.0
    }
    
    Some(EvaluatedExpr::BuiltIn(BuiltIn::DynamicParameter(Box::new(BounceModifier {
                    min,
                    max,
                    steps,
                    step_count: (0.0),
                }))))
}
