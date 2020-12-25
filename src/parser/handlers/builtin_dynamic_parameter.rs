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
		    // positional args ...
		    let min = get_next_param(&mut tail_drain, 0.0);    
		    let max = get_next_param(&mut tail_drain, 0.0);		    
		    let steps = get_next_keyword_param("steps".to_string(),
						       &mut tail_drain,
						       128.0);

		    Box::new(BounceModifier {                        
			min: min,
			max: max,            
			steps: steps,
			step_count: (0.0).into(),
		    })
		}
		BuiltInDynamicParameter::Brownian => {

		    let min = get_next_param(&mut tail_drain, 0.0);    
		    let max = get_next_param(&mut tail_drain, 0.0);

		    let raw_params = get_raw_keyword_params(&mut tail_drain);

		    let current = find_keyword_float_value(&raw_params, "start".to_string(), max.clone().evaluate() - min.clone().evaluate() / 2.0);
		    let step_size = find_keyword_float_param(&raw_params, "step".to_string(), 0.1);
		    let wrap = find_keyword_bool_value(&raw_params, "wrap".to_string(), true);

		    Box::new(BrownianModifier {                        
			min: min,
			max: max,            
			step_size: step_size,
			wrap: wrap,
			current: current
		    })
		}
	    }	    
	)
    })
}
