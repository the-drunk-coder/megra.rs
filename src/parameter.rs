use std::boxed::Box;

pub trait Modifier {
    fn evaluate(&mut self, input: f32) -> f32;
}

pub struct BounceModifier {
    pub min: Parameter,
    pub max: Parameter,
    pub steps: Parameter,
    pub step_count: f32,
}

impl Modifier for BounceModifier {         
    fn evaluate(&mut self, _: f32) -> f32 {
	// why doesn't rust has a hashable float ?????
        
        let steps_raw:f32 = self.steps.evaluate();
	let dec_inc:f32 = 360.0 / steps_raw;
	let min_raw:f32 = self.min.evaluate();
	let max_raw:f32 = self.max.evaluate();
        let range_raw:f32 = max_raw - min_raw;
        
        let degree:f32 = (dec_inc * (self.step_count % steps_raw)) % 360.0;
        let abs_sin:f32 = degree.to_radians().sin().abs();
        
        let cur:f32 = min_raw + (abs_sin * range_raw);
	
        self.step_count += 1.0; 
        
        cur
    }
}

pub struct Parameter {
    pub val: f32,
    pub modifier: Option<Box<dyn Modifier + Send>>,
}

impl Parameter {
    pub fn with_value(val: f32) -> Self {
	Parameter {
	    val: val,
	    modifier: None
	}
    }
    
    pub fn evaluate(&mut self) -> f32 {
	if let Some(m) = &mut self.modifier {
	    m.evaluate(self.val)
	} else {
	    self.val
	}
    }
}

