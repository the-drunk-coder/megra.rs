pub mod modifier;
use modifier::*;

use rand::Rng;
use std::boxed::Box;
use std::fmt::*;

use ruffbox_synth::building_blocks::{SynthParameterValue, ValOp};

#[derive(Clone)]
pub enum ParameterValue {
    Scalar(Parameter),
    Vector(Vec<Parameter>),
    Matrix(Vec<Vec<Parameter>>),
    Lfo(Parameter, Parameter, Parameter, Parameter, Parameter, ValOp), // init, freq, amp, add, op
    LFSaw(Parameter, Parameter, Parameter, Parameter, ValOp),          // init, freq, amp, add, op
    LFTri(Parameter, Parameter, Parameter, Parameter, ValOp),          // init, freq, amp, add, op
    LFSquare(Parameter, Parameter, Parameter, Parameter, Parameter, ValOp), // init, freq, pw, amp, add, op
}

#[derive(Clone)]
pub struct Parameter {
    pub val: f32,
    pub static_val: f32,
    pub modifier: Option<Box<dyn Modifier + Send + Sync>>,
}

impl Debug for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Parameter")
            .field("current", &self.val)
            .field("static", &self.static_val)
            .finish()
    }
}

impl Parameter {
    pub fn with_value(val: f32) -> Self {
        Parameter {
            val,
            static_val: val,
            modifier: None,
        }
    }

    pub fn evaluate_val_f32(&mut self) -> SynthParameterValue {
        SynthParameterValue::ScalarF32(if let Some(m) = &mut self.modifier {
            self.static_val = m.evaluate(self.val);
            self.static_val
        } else {
            self.val
        })
    }

    pub fn evaluate_val_usize(&mut self) -> SynthParameterValue {
        SynthParameterValue::ScalarUsize(if let Some(m) = &mut self.modifier {
            self.static_val = m.evaluate(self.val);
            self.static_val as usize
        } else {
            self.val as usize
        })
    }

    pub fn evaluate_numerical(&mut self) -> f32 {
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
        let rand = (factor * (1000.0 - rng.gen_range(0.0..2000.0))) * (self.val / 1000.0);
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
            println!("val after shake: {}", a.evaluate_numerical());
            assert!(a.evaluate_numerical() != 1000.0);
            assert!(a.evaluate_numerical() >= 500.0);
            assert!(a.evaluate_numerical() <= 1500.0);
        }
    }
}
