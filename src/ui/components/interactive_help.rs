use std::{mem, rc::Rc};

use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::{
    Frame,
    crossterm::event::Event as CrosstermEvent,
    layout::Rect,
    text::{Line, Span},
};

use crate::ui::{
    Hit,
    component::{Component, UpdateResult},
    event::AppEvent,
    hit_test,
    theme::HelpStyles,
};

const ITEM_SEPARATOR: &str = " • ";
const ITEM_SEPARATOR_WIDTH: u16 = 3;
const KEY_SEPARATOR: &str = "/";
const KEY_SEPARATOR_WIDTH: u16 = 1;
const HORIZONTAL_MARGIN: u16 = 1;

#[derive(Debug)]
pub(crate) struct HelpItem<T> {
    pub(crate) keys: Vec<&'static str>,
    pub(crate) text: &'static str,
    pub(crate) action: T,
    pub(crate) visible: bool,
}

impl<T> HelpItem<T> {
    pub(crate) fn new(key: &'static str, text: &'static str, action: T) -> Self {
        Self::with_keys(vec![key], text, action)
    }

    pub(crate) fn with_keys(keys: Vec<&'static str>, text: &'static str, action: T) -> Self {
        Self {
            keys,
            text,
            action,
            visible: true,
        }
    }

    pub(crate) fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    fn len(&self) -> u16 {
        let keys_width = self
            .keys
            .iter()
            .map(|key| key.chars().count() as u16)
            .sum::<u16>();
        let key_separators_width = (self.keys.len() as u16 - 1) * KEY_SEPARATOR_WIDTH;

        keys_width + key_separators_width + self.text.len() as u16 + 1
    }
}

#[derive(Debug)]
struct HelpRow {
    items: Vec<usize>,
    width: u16,
}

#[derive(Debug)]
pub(crate) struct InteractiveHelpComponent<T> {
    items: Vec<HelpItem<T>>,
    rows: Vec<HelpRow>,
    hits: Vec<Hit<T>>,
    hovered_action: Option<T>,
    styles: Rc<HelpStyles>,
    layout_width: Option<u16>,
    layout_dirty: bool,
}

impl<T> InteractiveHelpComponent<T> {
    pub(crate) fn new(styles: Rc<HelpStyles>, items: Vec<HelpItem<T>>) -> Self {
        let item_count = items.len();

        Self {
            items,
            rows: Vec::with_capacity(item_count),
            hits: Vec::with_capacity(item_count),
            hovered_action: None,
            styles,
            layout_width: None,
            layout_dirty: true,
        }
    }

    pub(crate) fn height(&mut self, avail_width: u16) -> u16 {
        self.ensure_layout(avail_width);
        self.rows.len() as u16
    }

    fn ensure_layout(&mut self, avail_width: u16) {
        if self.layout_dirty || self.layout_width != Some(avail_width) {
            self.compute_rows(avail_width);
            self.layout_width = Some(avail_width);
            self.layout_dirty = false;
        }
    }

    fn compute_rows(&mut self, avail_width: u16) {
        let avail_width = avail_width - HORIZONTAL_MARGIN * 2;
        self.rows.clear();

        let mut row = Vec::<usize>::with_capacity(self.items.len());
        let mut row_width = 0;

        for (item_idx, help) in self.items.iter().enumerate().rev() {
            if !help.visible {
                continue;
            }

            let help_width = help.len();
            let mut separator_width = if row.is_empty() {
                0
            } else {
                ITEM_SEPARATOR_WIDTH
            };

            if !row.is_empty() && row_width + help_width + separator_width > avail_width {
                self.rows.push(HelpRow {
                    items: mem::take(&mut row),
                    width: row_width,
                });
                separator_width = 0;
                row_width = 0;
            }

            row.insert(0, item_idx);
            row_width += help_width + separator_width;
        }

        self.rows.push(HelpRow {
            items: row,
            width: row_width,
        });
        self.rows.reverse();
    }
}

impl<T> InteractiveHelpComponent<T>
where
    T: PartialEq + Copy,
{
    pub(crate) fn set_visible(&mut self, action: T, visible: bool) {
        let Some(item) = self.items.iter_mut().find(|item| item.action == action) else {
            return;
        };

        if item.visible != visible {
            item.visible = visible;
            self.layout_dirty = true;
        }
    }
}

impl<T> Component<T> for InteractiveHelpComponent<T>
where
    T: PartialEq + Copy,
{
    fn update(&mut self, event: AppEvent) -> UpdateResult<T> {
        let AppEvent::Input(CrosstermEvent::Mouse(mouse)) = event else {
            return UpdateResult::Unchanged;
        };

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                hit_test(&self.hits, mouse.column, mouse.row)
                    .map(|hit| hit.kind)
                    .into()
            }
            MouseEventKind::Moved => {
                let action = hit_test(&self.hits, mouse.column, mouse.row).map(|hit| hit.kind);
                let changed = self.hovered_action != action;
                self.hovered_action = action;

                changed.into()
            }
            _ => UpdateResult::Unchanged,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        debug_assert!(area.height == self.rows.len() as u16);

        self.hits.clear();

        self.ensure_layout(area.width);

        for (area, row) in area.rows().zip(&self.rows) {
            let row_x = area.x + (area.width.saturating_sub(row.width)) / 2;
            let mut x = row_x;
            let mut spans = Vec::with_capacity(row.items.len() * 4);

            for (i, &item_idx) in row.items.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(ITEM_SEPARATOR));
                    x += ITEM_SEPARATOR_WIDTH;
                }

                let help = &self.items[item_idx];
                let help_len = help.len();

                self.hits.push(Hit {
                    area: Rect {
                        x,
                        y: area.y,
                        width: help_len,
                        height: 1,
                    },
                    kind: help.action,
                });

                let style = if self.hovered_action == Some(help.action) {
                    self.styles.hovered
                } else {
                    self.styles.text
                };

                for (key_idx, key) in help.keys.iter().enumerate() {
                    if key_idx > 0 {
                        spans.push(Span::styled(KEY_SEPARATOR, self.styles.key));
                    }
                    spans.push(Span::styled(*key, self.styles.key));
                }
                spans.push(Span::styled(" ", self.styles.text));
                spans.push(Span::styled(help.text, style));

                x += help_len;
            }

            let line = Line::from(spans).style(self.styles.text).centered();
            frame.render_widget(line, area);
        }
    }
}
