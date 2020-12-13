use std::boxed::Box;
use rand::Rng;

pub trait Modifier: ModifierClone {
    fn evaluate(&mut self, input: f32) -> f32;
    fn shake(&mut self, factor: f32);
}
pub trait ModifierClone {
    fn clone_box(&self) -> Box<dyn Modifier + Send>;
}

impl<T> ModifierClone for T
where
    T: 'static + Modifier + Clone + Send,
{
    fn clone_box(&self) -> Box<dyn Modifier + Send> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Modifier + Send> {
    fn clone(&self) -> Box<dyn Modifier + Send> {
        self.clone_box()
    }
}

#[derive(Clone)]
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

    fn shake(&mut self, factor: f32) {
	self.min.shake(factor);
	self.max.shake(factor);
	self.steps.shake(factor);
    }
}

#[derive(Clone)]
pub struct Parameter {
    pub val: f32,
    pub static_val: f32,
    pub modifier: Option<Box<dyn Modifier + Send>>,
}

impl Parameter {
    pub fn with_value(val: f32) -> Self {
	Parameter {
	    val: val,
	    static_val: val,
	    modifier: None
	}
    }
    
    pub fn evaluate(&mut self) -> f32 {
	if let Some(m) = &mut self.modifier {
	    self.static_val = m.evaluate(self.val);
	    self.static_val
	} else {
	    self.val
	}
    }

    pub fn shake(&mut self, mut factor: f32) {
	factor = factor.clamp(0.0, 1.0);
	let mut rng = rand::thread_rng();
	// heuristic ... from old megra ... not sure what i thought back then, let's see ...
	let rand = (factor * (1000.0 - rng.gen_range(0.0, 2000.0))) * (self.val / 1000.0); 
	self.val += rand;
	if let Some(m) = self.modifier.as_mut() {
	    m.shake(factor);
	}
    }
}

// TEST TEST TEST 
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    
    #[test]
    fn test_shake() {
	for _ in 0..20 {
	    let mut a = Parameter::with_value(1000.0);
	    a.shake(0.5);
	    println!("val after shake: {}", a.evaluate());
	    assert!(a.evaluate() != 1000.0);
	    assert!(a.evaluate() >= 500.0);
	    assert!(a.evaluate() <= 1500.0);	    	
	}	
    }
}
