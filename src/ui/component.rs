use ratatui::{Frame, crossterm::event::Event, layout::Rect};

pub(crate) trait Component<A> {
    fn update(&mut self, event: Event) -> Option<A>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
