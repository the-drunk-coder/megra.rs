use crate::parameter::Parameter;
use crate::parameter::modifier::Modifier;

#[derive(Clone)]
pub struct EnvelopeModifier {
    pub levels: Vec<Parameter>,
    pub steps: Vec<Parameter>,    
    pub last_steps: usize,
    pub current_steps: usize,
    pub current_from: f32,
    pub current_to: f32,
    pub done: bool,
    pub repeat: bool,
    step_count: usize,
}

impl EnvelopeModifier {
    pub fn from_data(levels: &Vec<Parameter>, steps: &Vec<Parameter>, repeat:bool) -> Self {
	EnvelopeModifier {
	    levels: levels.to_vec(),
	    steps: steps.to_vec(),
	    last_steps: 0,
	    current_steps: 0,
	    current_from: 0.0,
	    current_to: 0.0,
	    done: false,
	    repeat: repeat,
	    step_count: 0,
	}
    }	
}
impl Modifier for EnvelopeModifier {         
    fn evaluate(&mut self, _: f32) -> f32 {
	if self.current_from == self.current_to || self.done {
	    self.current_to
	} else {
	    let current_range = self.current_to - self.current_from;
	    let degree_increment = 90.0 / self.current_steps as f32;
	    0.0
	}
	
	
	    
    }

    fn shake(&mut self, factor: f32) {
	// later ...
    }
}
