use crate::builtin_types::*;
use crate::parser::parser_helpers::*;
use crate::parameter::{
    Parameter,
    modifier::bounce_modifier::BounceModifier,
    modifier::brownian_modifier::BrownianModifier,
    modifier::envelope_modifier::EnvelopeModifier
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

		BuiltInDynamicParameter::Envelope => {
		    let mut collect_steps = false;
		    let mut collect_values = false;

		    let mut values = Vec::new();
		    let mut steps = Vec::new();
		    let mut repeat = false;
		    
		    while let Some(Expr::Constant(c)) = tail_drain.next() {
			if collect_steps {
			    match c {
				Atom::Float(f) => {steps.push(Parameter::with_value(f))},
				Atom::Parameter(ref p) => steps.push(p.clone()),
				_ => {collect_steps = false;}
			    }
			}
			if collect_values {
			    match c {
				Atom::Float(f) => {values.push(Parameter::with_value(f))},
				Atom::Parameter(ref p) => values.push(p.clone()),
				_ => {collect_values = false;}
			    }
			}
			match c {
			    Atom::Keyword(k) => {
				match k.as_str() {
				    "v" => {
					collect_values = true;
				    },
				    "values" => {
					collect_values = true;
				    },
				    "s" => {
					collect_steps = true;
				    },
				    "steps" => {
					collect_steps = true;
				    },
				    "repeat" => {
					if let Some(b) = get_bool_from_expr_opt(&tail_drain.next()) {
					    repeat = b;
					}
				    },
				    _ => {} // ignore
				}
			    }
			    _ => {}
			}
		    }
		    Box::new(EnvelopeModifier::from_data(&values, &steps, repeat))
		},
		BuiltInDynamicParameter::Fade => {
		    let from = get_next_param(&mut tail_drain, 0.0);    
		    let to = get_next_param(&mut tail_drain, 0.0);

		    let mut values = Vec::new();
		    let mut steps = Vec::new();

		    values.push(from);
		    values.push(to);
		    
		    if let Some(Expr::Constant(Atom::Keyword(k))) = tail_drain.next() {						 
			match k.as_str() {				    
			    "steps" => {
				if let Some(f) = get_float_from_expr_opt(&tail_drain.next()) {
				    steps.push(Parameter::with_value(f));
				}				
			    },				    
			    _ => {} // ignore
			}
		    }		    

		    // if steps isn't specified, use default of 128
		    if steps.is_empty() {
			steps.push(Parameter::with_value(128.0));
		    }
		    
		    Box::new(EnvelopeModifier::from_data(&values, &steps, false))
		}
	    }	    
	)
    })
}
