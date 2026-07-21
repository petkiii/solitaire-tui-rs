use std::rc::Rc;

use ratatui::{
    Frame,
    crossterm::event::{Event, KeyEventKind, MouseButton, MouseEventKind},
    layout::{Position, Rect},
    text::{Line, Text},
};
use tachyonfx::CenteredShrink;

use crate::ui::{
    component::{Component, UpdateResult},
    event::AppEvent,
    keymap::MENU_KEYMAP,
    theme::MenuStyles,
};

const HORIZONTAL_PADDING: u16 = 2;

#[derive(Clone, Copy)]
struct Hit {
    area: Rect,
    idx: usize,
}

pub(crate) struct MenuComponent<T>
where
    T: Copy + 'static,
{
    selected_index: usize,
    hits: Vec<Hit>,
    items: &'static [MenuItem<T>],
    styles: Rc<MenuStyles>,

    menu_height: u16,
    menu_width: u16,
}

pub(crate) struct MenuItem<T> {
    pub label: &'static str,
    pub action: T,
}

impl<T> MenuComponent<T>
where
    T: Copy + 'static,
{
    pub(crate) fn new(styles: Rc<MenuStyles>, items: &'static [MenuItem<T>]) -> Self {
        let menu_height = items.len();
        let max_item_len = items
            .iter()
            .map(|x| x.label.chars().count())
            .max()
            .expect("menu items slice was empty");
        Self {
            selected_index: 0,
            hits: Vec::with_capacity(items.len()),
            styles,
            items,
            menu_height: menu_height as u16,
            menu_width: max_item_len as u16 + HORIZONTAL_PADDING * 2,
        }
    }

    pub(crate) fn width(&self) -> u16 {
        self.menu_width
    }

    pub(crate) fn height(&self) -> u16 {
        self.menu_height
    }

    fn item_prev(&mut self) {
        self.selected_index = (self.selected_index + self.items.len() - 1) % self.items.len();
    }

    fn item_next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    fn item_enter(&self) -> T {
        self.items[self.selected_index].action
    }
}

impl<T> Component<T> for MenuComponent<T>
where
    T: Copy + 'static,
{
    fn update(&mut self, event: AppEvent) -> UpdateResult<T> {
        let AppEvent::Input(event) = event else {
            return UpdateResult::Unchanged;
        };

        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let mut changed = true;
                match key.code {
                    _ if MENU_KEYMAP.previous.matches(key) => self.item_prev(),
                    _ if MENU_KEYMAP.next.matches(key) => self.item_next(),
                    _ if MENU_KEYMAP.confirm.matches(key) => {
                        return UpdateResult::Action(self.item_enter());
                    }
                    _ => changed = false,
                }

                return changed.into();
            }
            Event::Mouse(mouse)
                if matches!(
                    mouse.kind,
                    MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Moved
                ) =>
            {
                let hit = self
                    .hits
                    .iter()
                    .find(|x| x.area.contains(Position::new(mouse.column, mouse.row)));

                if let Some(hit) = hit {
                    let prev_idx = self.selected_index;
                    self.selected_index = hit.idx;

                    if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                        return UpdateResult::Action(self.item_enter());
                    }

                    return (prev_idx != self.selected_index).into();
                }
            }
            _ => {}
        }

        UpdateResult::Unchanged
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.hits.clear();

        let inner = area.inner_centered(self.menu_width, self.menu_height);
        let lines = self.items.iter().enumerate().map(|(idx, item)| {
            self.hits.push(Hit {
                area: Rect {
                    x: inner.x,
                    y: inner.y + idx as u16,
                    width: inner.width,
                    height: 1,
                },
                idx,
            });

            let line = Line::from(item.label).centered();
            if idx == self.selected_index {
                line.style(self.styles.selected)
            } else {
                line.style(self.styles.normal)
            }
        });

        frame.render_widget(Text::from_iter(lines), inner);
    }
}
