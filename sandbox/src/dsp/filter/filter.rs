pub trait Filter {
    fn reset(&mut self) {}
    fn filter(&mut self, sample: f32) -> f32;
}
