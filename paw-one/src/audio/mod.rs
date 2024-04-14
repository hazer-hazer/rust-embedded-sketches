pub mod sai;

pub trait AudioOutput {
    fn write(&mut self, data: &[u32]) -> Result<(), ()>;
}
