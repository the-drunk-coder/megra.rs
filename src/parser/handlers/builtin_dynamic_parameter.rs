use crate::builtin_types::*;
use crate::parser::parser_helpers::*;
use crate::parameter::{
    Parameter,
    modifier::bounce_modifier::BounceModifier,
    modifier::brownian_modifier::BrownianModifier
};

pub fn handle(par: &BuiltInDynamicParameter, tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    Atom::Parameter(Parameter {
	val:0.0,
	static_val:0.0,
	modifier: Some(
	    match par {
		BuiltInDynamicParameter::Bounce => {
		    let min = get_next_param(&mut tail_drain, 0.0);    
		    let max = get_next_param(&mut tail_drain, 0.0);    
		    let steps = get_next_param(&mut tail_drain, 0.0);

		    Box::new(BounceModifier {                        
			min: min,
			max: max,            
			steps: steps,
			step_count: (0.0).into(),
		    })
		}
		BuiltInDynamicParameter::Brownian => {
		    let current = get_next_param(&mut tail_drain, 0.0).evaluate();
		    let min = get_next_param(&mut tail_drain, 0.0);    
		    let max = get_next_param(&mut tail_drain, 0.0);    
		    let step_size = get_next_param(&mut tail_drain, 0.0);

		    Box::new(BrownianModifier {                        
			min: min,
			max: max,            
			step_size: step_size,
			wrap: true,
			current: current
		    })
		}
	    }	    
	)
    })
}
