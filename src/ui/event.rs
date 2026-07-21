use std::time::Instant;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent};

#[derive(Debug, Clone)]
pub(crate) enum AppEvent {
    Input(CrosstermEvent),
    Tick(Instant),
}
pub(super) enum Event {
    Render,
    AppEvent(AppEvent),
}

pub(super) fn next_event(next_frame: Option<Instant>, next_tick: Option<Instant>) -> Result<Event> {
    loop {
        let now = Instant::now();

        if next_tick.is_some_and(|t| now >= t) {
            return Ok(Event::AppEvent(AppEvent::Tick(now)));
        }

        if next_frame.is_some_and(|f| now >= f) {
            return Ok(Event::Render);
        }

        let timeout = [next_frame, next_tick].into_iter().flatten().min();

        match timeout {
            Some(timeout) => {
                if event::poll(timeout.saturating_duration_since(now))? {
                    return Ok(Event::AppEvent(AppEvent::Input(event::read()?)));
                }
            }
            None => return Ok(Event::AppEvent(AppEvent::Input(event::read()?))),
        }
    }
}
