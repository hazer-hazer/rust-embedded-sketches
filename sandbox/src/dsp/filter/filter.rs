pub trait Filter {
    fn reset(&mut self) {}
    fn filter(&mut self, input: f32) -> f32;
}
