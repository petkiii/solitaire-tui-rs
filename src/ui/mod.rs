use ratatui::layout::{Position, Rect};

mod action;
mod animations;
pub(crate) mod app;
mod component;
mod components;
pub(crate) mod config;
pub(crate) mod event;
mod keymap;
mod screens;
mod theme;
mod widgets;

trait RectExt {
    /// Resizes the rect by moving its top edge, keeping its bottom edge fixed.
    ///
    /// ```text
    /// grow                         shrink
    /// after  +--------+            before +--------+
    ///        | added  |                   | removed|
    /// before +--------+            after  +--------+
    ///        | rect   |                   | rect   |
    ///        +--------+                   +--------+
    ///          same bottom                  same bottom
    /// ```
    fn resize_height_from_top(self, new_height: u16) -> Self;
}

impl RectExt for Rect {
    fn resize_height_from_top(self, new_height: u16) -> Self {
        Self {
            y: self.bottom() - new_height,
            height: new_height,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Hit<T> {
    area: Rect,
    kind: T,
}

fn hit_test<T: Copy>(hits: &[Hit<T>], x: u16, y: u16) -> Option<Hit<T>> {
    let position = Position::new(x, y);
    hits.iter().find(|hit| hit.area.contains(position)).copied()
}
