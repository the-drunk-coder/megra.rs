use crate::parameter::modifier::Modifier;
use crate::parameter::DynVal;
use rand::Rng;

#[derive(Clone)]
pub struct RandRangeModifier {
    pub min: DynVal,
    pub max: DynVal,
}

impl RandRangeModifier {
    pub fn from_data(min: DynVal, max: DynVal) -> Self {
        RandRangeModifier { min, max }
    }
}

impl Modifier for RandRangeModifier {
    fn evaluate(&mut self, _: f32) -> f32 {
        let min = self.min.evaluate_numerical();
        let max = self.max.evaluate_numerical();
        let mut rng = rand::thread_rng();
        if (min - max).abs() < f32::EPSILON {
            // min == max
            max
        } else if min > max {
            rng.gen_range(max..min)
        } else {
            rng.gen_range(min..max)
        }
    }

    fn shake(&mut self, factor: f32) {
        self.min.shake(factor);
        self.max.shake(factor);
    }
}
