use std::time::Instant;

pub struct FpsCounter {
    last_time: Instant,
    frame_count: u32,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            last_time: Instant::now(),
            frame_count: 0,
        }
    }

    pub fn update(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time);

        if elapsed.as_secs_f32() >= 1.0 {
            let fps = self.frame_count as f32 / elapsed.as_secs_f32();
            println!("FPS: {:.1}", fps);
            self.frame_count = 0;
            self.last_time = now;
        }
    }
}