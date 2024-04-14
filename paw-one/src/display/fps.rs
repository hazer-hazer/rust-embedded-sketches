use embassy_stm32::time::Hertz;
use embassy_time::Instant;

pub struct FPS {
    last_instant: Instant,
}

impl FPS {
    pub fn new() -> Self {
        Self {
            last_instant: Instant::now(),
        }
    }

    pub fn value(&mut self) -> f32 {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_instant);
        let fps = 1_000.0 / elapsed.as_millis() as f32;

        self.last_instant = now;

        fps
    }
}
