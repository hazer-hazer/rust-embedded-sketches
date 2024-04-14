pub struct MixerInsert {
    volume: f32,
    // panning: f32
}

pub struct Mixer {
    inserts: Vec<MixerInsert>,
    sample_rate: u32,
}

impl Mixer {
    pub fn new(sample_rate: u32) -> Self {
        let mut inserts = Vec::new();

        inserts.reserve(2);

        Self {
            inserts,
            sample_rate,
        }
    }
}
