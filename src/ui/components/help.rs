use std::{mem, rc::Rc};

use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
};

use crate::ui::{component::Component, theme::HelpStyles};

const ITEM_SEPARATOR: &str = " • ";
const ITEM_SEPARATOR_WIDTH: u16 = 3;
const KEY_SEPARATOR: &str = "/";
const KEY_SEPARATOR_WIDTH: u16 = 1;
const HORIZONTAL_MARGIN: u16 = 1;

#[derive(Debug)]
pub(crate) struct HelpItem {
    keys: &'static [&'static str],
    text: &'static str,
}

impl HelpItem {
    pub(crate) const fn new(keys: &'static [&'static str], text: &'static str) -> Self {
        Self { keys, text }
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
}

#[derive(Debug)]
pub(crate) struct HelpComponent {
    items: &'static [HelpItem],
    rows: Vec<HelpRow>,
    styles: Rc<HelpStyles>,
    layout_width: Option<u16>,
    layout_dirty: bool,
}

impl HelpComponent {
    pub(crate) fn new(theme: Rc<HelpStyles>, items: &'static [HelpItem]) -> Self {
        let item_count = items.len();

        Self {
            items,
            rows: Vec::with_capacity(item_count),
            styles: theme,
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
            let help_width = help.len();
            let mut separator_width = if row.is_empty() {
                0
            } else {
                ITEM_SEPARATOR_WIDTH
            };

            if !row.is_empty() && row_width + help_width + separator_width > avail_width {
                self.rows.push(HelpRow {
                    items: mem::take(&mut row),
                });
                separator_width = 0;
                row_width = 0;
            }

            row.insert(0, item_idx);
            row_width += help_width + separator_width;
        }

        self.rows.push(HelpRow { items: row });
        self.rows.reverse();
    }

    fn render_internal(&mut self, frame: &mut Frame, area: Rect) {
        self.ensure_layout(area.width);
        debug_assert!(area.height == self.rows.len() as u16);

        for (area, row) in area.rows().zip(&self.rows) {
            let mut spans = Vec::with_capacity(row.items.len() * 4);

            for (i, &item_idx) in row.items.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(ITEM_SEPARATOR));
                }

                let help = &self.items[item_idx];

                for (key_idx, key) in help.keys.iter().enumerate() {
                    if key_idx > 0 {
                        spans.push(Span::styled(KEY_SEPARATOR, self.styles.text));
                    }
                    spans.push(Span::styled(*key, self.styles.key));
                }
                spans.push(Span::raw(" "));
                spans.push(Span::styled(help.text, self.styles.text));
            }

            let line = Line::from(spans).style(self.styles.text).centered();
            frame.render_widget(line, area);
        }
    }
}

impl Component<()> for HelpComponent {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.render_internal(frame, area);
    }
}
