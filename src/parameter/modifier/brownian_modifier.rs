use crate::parameter::modifier::Modifier;
use crate::parameter::DynVal;
use rand::Rng;

#[derive(Clone)]
pub struct BrownianModifier {
    pub min: DynVal,
    pub max: DynVal,
    pub step_size: DynVal,
    pub current: f32,
    pub wrap: bool,
}

impl Modifier for BrownianModifier {
    fn evaluate(&mut self, _: f32) -> f32 {
        // why doesn't rust has a hashable float ?????
        let mut rng = rand::thread_rng();
        // heuristic ... from old megra ... not sure what i thought back then, let's see ...
        let rand = rng.gen_range(0..2000);
        let step_size = self.step_size.evaluate_numerical();
        let min = self.min.evaluate_numerical();
        let max = self.max.evaluate_numerical();

        if rand < 1000 {
            self.current -= step_size;
        } else {
            self.current += step_size;
        }

        if !self.wrap {
            self.current = self.current.clamp(min, max);
        } else if self.current < min {
            let diff = min - self.current;
            self.current = max - diff;
        } else if self.current > max {
            let diff = self.current - max;
            self.current = min + diff;
        }

        self.current
    }

    fn shake(&mut self, factor: f32) {
        self.min.shake(factor);
        self.max.shake(factor);
        self.step_size.shake(factor);
    }
}
