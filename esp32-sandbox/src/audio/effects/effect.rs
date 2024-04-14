pub trait Effect {
    fn process(&mut self, input: f32) -> f32;
}
