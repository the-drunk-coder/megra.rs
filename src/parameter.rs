use std::boxed::Box;

pub trait Modifier {
    fn evaluate(&mut self, input: f32) -> f32;
}

pub struct Parameter {
    val: f32,
    modifiers: Vec<Box<dyn Modifier + Send>>,
}

impl Parameter {
    pub fn with_value(val: f32) -> Self {
	Parameter {
	    val: val,
	    modifiers: Vec::new()
	}
    }
    
    pub fn evaluate(&mut self) -> f32 {
	let mut accum = self.val;
	for m in self.modifiers.iter_mut() {
	    accum = m.evaluate(accum);
	}
	self.val = accum;
	accum
    }
}

