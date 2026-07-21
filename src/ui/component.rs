use std::time::Instant;

use ratatui::{Frame, layout::Rect};

use crate::ui::event::AppEvent;

pub(crate) enum UpdateResult<A> {
    Unchanged,
    Changed,
    Action(A),
}

impl<A> UpdateResult<A> {
    pub(crate) fn map<B>(self, f: impl FnOnce(A) -> B) -> UpdateResult<B> {
        match self {
            Self::Unchanged => UpdateResult::Unchanged,
            Self::Changed => UpdateResult::Changed,
            Self::Action(action) => UpdateResult::Action(f(action)),
        }
    }

    pub(crate) fn and_then<B>(self, f: impl FnOnce(A) -> UpdateResult<B>) -> UpdateResult<B> {
        match self {
            Self::Unchanged => UpdateResult::Unchanged,
            Self::Changed => UpdateResult::Changed,
            Self::Action(action) => f(action),
        }
    }
}

impl<A> From<Option<A>> for UpdateResult<A> {
    fn from(value: Option<A>) -> Self {
        value.map_or(Self::Unchanged, Self::Action)
    }
}

impl<A> From<bool> for UpdateResult<A> {
    fn from(value: bool) -> Self {
        if value {
            Self::Changed
        } else {
            Self::Unchanged
        }
    }
}

pub(crate) trait Component<A> {
    fn render(&mut self, frame: &mut Frame, area: Rect);

    fn update(&mut self, _event: AppEvent) -> UpdateResult<A> {
        UpdateResult::Unchanged
    }

    fn next_tick(&mut self, _now: Instant) -> Option<Instant> {
        None
    }
}
