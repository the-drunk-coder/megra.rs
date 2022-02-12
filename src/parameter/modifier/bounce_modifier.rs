use crate::parameter::modifier::Modifier;
use crate::parameter::Parameter;

#[derive(Clone)]
pub struct BounceModifier {
    pub min: Parameter,
    pub max: Parameter,
    pub steps: Parameter,
    pub step_count: f32,
}

impl Modifier for BounceModifier {
    fn evaluate(&mut self, _: f32) -> f32 {
        let steps_raw: f32 = self.steps.evaluate();
        let dec_inc: f32 = 360.0 / steps_raw;
        let min_raw: f32 = self.min.evaluate();
        let max_raw: f32 = self.max.evaluate();
        let range_raw: f32 = max_raw - min_raw;

        let degree: f32 = (dec_inc * (self.step_count % steps_raw)) % 360.0;
        let abs_sin: f32 = degree.to_radians().sin().abs();

        let cur: f32 = min_raw + (abs_sin * range_raw);

        self.step_count += 1.0;

        cur
    }

    fn shake(&mut self, factor: f32) {
        self.min.shake(factor);
        self.max.shake(factor);
        self.steps.shake(factor);
    }
}
