use std::{
    fmt::{self, Display},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub(crate) struct Timer {
    accumulated: Duration,
    started_at: Option<Instant>,
    started: bool,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Self {
            accumulated: Duration::ZERO,
            started_at: None,
            started: false,
        }
    }

    pub(crate) fn start(&mut self) {
        if !self.started && self.started_at.is_none() {
            self.started = true;
            self.started_at = Some(Instant::now());
        }
    }

    pub(crate) fn pause(&mut self) {
        if self.started
            && let Some(since) = self.started_at.take()
        {
            self.accumulated += since.elapsed();
        }
    }

    pub(crate) fn resume(&mut self) {
        if self.started && self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }
    }

    pub(crate) fn elapsed_at(&self, now: Instant) -> Duration {
        self.accumulated
            + self
                .started_at
                .map_or(Duration::ZERO, |started_at| now.duration_since(started_at))
    }

    pub(crate) fn next_second_tick(&self, now: Instant) -> Option<Instant> {
        self.started_at?;

        let elapsed = self.elapsed_at(now);
        let next_second = Duration::from_secs(elapsed.as_secs() + 1);

        Some(now + next_second.saturating_sub(elapsed))
    }

    pub(crate) fn elapsed(&self) -> Duration {
        self.elapsed_at(Instant::now())
    }

    #[allow(dead_code)]
    pub(crate) fn started(&self) -> bool {
        self.started
    }

    pub(crate) fn format(&self) -> String {
        let secs = self.elapsed().as_secs();
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.format())
    }
}
