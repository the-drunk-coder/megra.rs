use crate::parameter::Parameter;
use crate::parameter::modifier::Modifier;
use rand::Rng;

#[derive(Clone)]
pub struct RandRangeModifier {
    pub min: Parameter,
    pub max: Parameter,        
}

impl RandRangeModifier {
    pub fn from_data(min: Parameter, max: Parameter) -> Self {
	RandRangeModifier {
	    min: min,
	    max: max
	}
    }	
}

impl Modifier for RandRangeModifier {         
    fn evaluate(&mut self, _: f32) -> f32 {	
	let min = self.min.evaluate();
	let max = self.max.evaluate();
	let mut rng = rand::thread_rng();
	if min == max {
	    max
	} else if min > max {
	    rng.gen_range(max, min)
	} else {
	    rng.gen_range(min, max)
	}	    	
    }

    fn shake(&mut self, factor: f32) {	
	self.min.shake(factor);	
	self.max.shake(factor);	
    }
}
