use std::time::{Instant, Duration};

#[derive(Debug, Default)]
pub struct Stopwatch {
    last_start: Option<Instant>,
    total: Duration
}

impl Stopwatch {
    pub fn start() -> Stopwatch {
        Stopwatch {
            last_start: Instant::now().into(),
            total: <_>::default()
        }
    }

    pub fn pause(&mut self) -> Duration {
        self.last_start
            .take()
            .map(|start| self.total += start.elapsed());
        self.total
    }

    pub fn resume(&mut self) {
        self.last_start = Instant::now().into()
    }

    pub fn elapsed(&self) -> Duration {
        self.total + self
            .last_start
            .as_ref()
            .map(Instant::elapsed)
            .unwrap_or_default()
    }
}