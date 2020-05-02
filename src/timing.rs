use std::collections::VecDeque;
use std::time::{Duration, Instant};


pub struct Governer {
    frames_per_second: u32,
    frame_duration: Duration,
    frame_times_q: VecDeque<Instant>,
}

impl Governer {
    pub fn new(frames_per_second: u32) -> Governer {
        let frame_ns: u64 = 1_000_000_000 / (frames_per_second as u64);
        let frame_duration = Duration::from_nanos(frame_ns);
        let mut frame_times_q: VecDeque<Instant> = VecDeque::new();
        frame_times_q.push_back(Instant::now());

        Governer {
            frames_per_second,
            frame_duration,
            frame_times_q,
        }
    }

    pub fn end_frame(&mut self) {
        let num_frames_in_q = self.frame_times_q.len();
        let expected_duration = self.frame_duration * (num_frames_in_q as u32);

        let now = Instant::now();
        let actual_duration = {
            // Put this in a block so that oldest_frame_start gets dropped.
            // It contains a reference to the queue, which we need to drop
            // so we can modify it later.
            let oldest_frame_start = self.frame_times_q.back().expect("Queue is not empty");
            now.duration_since(*oldest_frame_start)
        };

        self.frame_times_q.push_front(now);

        // Sleep if we're ahead.
        match expected_duration.checked_sub(actual_duration) {
            Some(remaining_time) => std::thread::sleep(remaining_time),
            None => (),
        }

        self.frame_times_q.truncate(self.frames_per_second as usize);
    }
}
