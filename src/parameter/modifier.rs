pub mod bounce_modifier;
pub mod brownian_modifier;
pub mod envelope_modifier;
pub mod randrange_modifier;

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


