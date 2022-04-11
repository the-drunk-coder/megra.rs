use crate::parameter::modifier::Modifier;
use crate::parameter::Parameter;

#[derive(Clone)]
pub struct EnvelopeModifier {
    pub values: Vec<Parameter>,
    pub steps: Vec<Parameter>,
    pub current_steps: usize,
    pub current_from: f32,
    pub current_to: f32,
    pub done: bool,
    pub repeat: bool,
    step_count: usize,
    value_idx: usize,
    steps_idx: usize,
}

impl EnvelopeModifier {
    pub fn from_data(values: &[Parameter], steps: &[Parameter], repeat: bool) -> Self {
        let mut env = EnvelopeModifier {
            values: values.to_vec(),
            steps: steps.to_vec(),
            current_steps: 0,
            current_from: 0.0,
            current_to: 0.0,
            done: false,
            repeat,
            step_count: 0,
            value_idx: 1,
            steps_idx: 1,
        };

        if let Some(cur_step) = env.steps.get_mut(0) {
            env.current_steps = cur_step.evaluate_numerical() as usize;
        } else {
            env.done = true;
        }

        if let Some(cur_from) = env.values.get_mut(0) {
            env.current_from = cur_from.evaluate_numerical();
        } else {
            env.done = true;
        }

        if let Some(cur_to) = env.values.get_mut(1) {
            env.current_to = cur_to.evaluate_numerical();
        } else {
            env.done = true;
        }

        env
    }
}

impl Modifier for EnvelopeModifier {
    fn evaluate(&mut self, _: f32) -> f32 {
        if self.step_count >= self.current_steps {
            if let Some(cur_step) = self.steps.get_mut(self.steps_idx) {
                self.current_steps = cur_step.evaluate_numerical() as usize;
                self.steps_idx += 1;
            } else if self.repeat {
                if let Some(cur_step) = self.steps.get_mut(0) {
                    self.current_steps = cur_step.evaluate_numerical() as usize;
                    self.steps_idx = 1;
                } else {
                    self.done = true;
                }
            } else {
                self.done = true;
            }

            if let Some(cur_from) = self.values.get_mut(self.value_idx) {
                self.current_from = cur_from.evaluate_numerical();
                self.value_idx += 1;
            } else if self.repeat {
                if let Some(cur_from) = self.values.get_mut(0) {
                    self.current_from = cur_from.evaluate_numerical();
                } else {
                    self.done = true;
                }
            } else {
                self.done = true;
            }

            if let Some(cur_to) = self.values.get_mut(self.value_idx) {
                self.current_to = cur_to.evaluate_numerical();
            } else if self.repeat {
                if let Some(cur_to) = self.values.get_mut(0) {
                    self.current_to = cur_to.evaluate_numerical();
                    self.value_idx = 0;
                } else {
                    self.done = true;
                }
            } else {
                self.done = true;
            }

            self.step_count = 0;
        }
        //println!("from: {} to: {} step: {} done: {}", self.current_from, self.current_to, self.current_steps, self.done);

        let val = if (self.current_from - self.current_to).abs() < f32::EPSILON || self.done {
            // self.current_to == self.current_from
            self.current_to
        } else {
            let current_range = self.current_to - self.current_from;
            let degree_increment = if self.current_steps <= 1 {
                90.0
            } else {
                90.0 / (self.current_steps - 1) as f32
            };
            let degree = degree_increment * self.step_count as f32;
            self.current_from + degree.to_radians().sin() * current_range
        };

        self.step_count += 1;
        val
    }

    fn shake(&mut self, factor: f32) {
        for val in self.values.iter_mut() {
            val.shake(factor);
        }
        for step in self.steps.iter_mut() {
            step.shake(factor);
        }
    }
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_envelope_no_repeat() {
        let steps = vec![Parameter::with_value(10.0), Parameter::with_value(10.0)];
        let values = vec![
            Parameter::with_value(0.0),
            Parameter::with_value(10.0),
            Parameter::with_value(0.0),
        ];

        let mut env = EnvelopeModifier::from_data(&values, &steps, false);

        let mut count = 0;
        let mut val = env.evaluate()(0.0);
        println!("count: {} val: {}", count, val);
        assert_approx_eq::assert_approx_eq!(val, 0.0, 0.00001);
        count += 1;
        for _ in 0..9 {
            println!("count: {} val: {}", count, env.evaluate()(0.0));
            count += 1;
        }
        val = env.evaluate()(0.0);
        println!("count: {} val: {}", count, val);
        assert_approx_eq::assert_approx_eq!(val, 10.0, 0.00001);
        count += 1;
        for _ in 0..9 {
            println!("count: {} val: {}", count, env.evaluate()(0.0));
            count += 1;
        }
        val = env.evaluate()(0.0);
        println!("count: {} val: {}", count, val);
        assert_approx_eq::assert_approx_eq!(val, 0.0, 0.00001);
        count += 1;
        for _ in 0..9 {
            println!("count: {} val: {}", count, env.evaluate()(0.0));
            count += 1;
        }
        assert_approx_eq::assert_approx_eq!(env.evaluate()(0.0), 0.0, 0.00001);
    }

    #[test]
    fn test_envelope_repeat() {
        let steps = vec![
            Parameter::with_value(10.0),
            Parameter::with_value(5.0),
            Parameter::with_value(10.0),
        ];
        let values = vec![
            Parameter::with_value(0.0),
            Parameter::with_value(10.0),
            Parameter::with_value(5.0),
        ];

        let mut env = EnvelopeModifier::from_data(&values, &steps, true);

        let mut count = 0;
        let mut val = env.evaluate()(0.0);
        println!("count: {} val: {}", count, val);
        assert_approx_eq::assert_approx_eq!(val, 0.0, 0.00001);
        count += 1;
        for _ in 0..9 {
            println!("count: {} val: {}", count, env.evaluate()(0.0));
            count += 1;
        }
        val = env.evaluate()(0.0);
        println!("count: {} val: {}", count, val);
        assert_approx_eq::assert_approx_eq!(val, 10.0, 0.00001);
        count += 1;
        for _ in 0..4 {
            println!("count: {} val: {}", count, env.evaluate()(0.0));
            count += 1;
        }
        val = env.evaluate()(0.0);
        println!("count: {} val: {}", count, val);
        assert_approx_eq::assert_approx_eq!(val, 5.0, 0.00001);
        count += 1;
        for _ in 0..9 {
            println!("count: {} val: {}", count, env.evaluate()(0.0));
            count += 1;
        }
        assert_approx_eq::assert_approx_eq!(env.evaluate()(0.0), 0.0, 0.00001);
    }
}
