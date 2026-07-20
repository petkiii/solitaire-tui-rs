use std::rc::Rc;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Widget},
};

use crate::{engine::card::Card, ui::theme::Theme};

pub(crate) struct CardWidget {
    card: Card,
    selected: bool,
    theme: Rc<Theme>,
}

impl CardWidget {
    pub fn new(card: Card, selected: bool, theme: Rc<Theme>) -> Self {
        Self {
            card,
            selected,
            theme,
        }
    }
}

impl Widget for CardWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.card.revealed() {
            CardBackWidget::new(self.selected, self.theme.clone()).render(area, buf);
            return;
        }

        let inner = render_border(
            area,
            self.selected,
            buf,
            self.theme.card_border,
            self.theme.card_border_highlight,
            self.theme.card_border_type,
        );

        fill(inner, buf, ' ', self.theme.card_fill);

        let fg = if self.card.is_red() {
            self.theme.card_red
        } else {
            self.theme.card_black
        };

        let rank_str = self.card.rank_str();
        let suit_str = self.card.suit_str();
        let rank_len = rank_str.len() as u16;
        let suit_len = suit_str.chars().count() as u16;
        let label_len = rank_len + suit_len;

        // top left
        buf.set_string(area.x + 1, area.y, rank_str, fg);
        buf.set_string(area.x + 1 + rank_len, area.y, suit_str, fg);

        if area.height <= 1 {
            return;
        }

        // bottom right
        let x = area.x + area.width.saturating_sub(label_len + 1);
        let y = area.y + area.height.saturating_sub(1);
        buf.set_string(x, y, suit_str, fg);
        buf.set_string(x + suit_len, y, rank_str, fg);

        if !self.theme.card_center_label || area.width < 8 || area.height < 6 {
            return;
        }

        let center_x = (inner.x + inner.width / 2) - 1;
        let center_y = inner.y + inner.height / 2;
        buf.set_string(center_x, center_y, rank_str, fg);
        buf.set_string(center_x + rank_len, center_y, suit_str, fg);
    }
}

pub(crate) struct CardBackWidget {
    hinted: bool,
    theme: Rc<Theme>,
}

impl CardBackWidget {
    pub fn new(hinted: bool, theme: Rc<Theme>) -> Self {
        Self { hinted, theme }
    }
}

impl Widget for CardBackWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = render_border(
            area,
            self.hinted,
            buf,
            self.theme.card_back_border,
            self.theme.card_back_border_highlight,
            self.theme.card_border_type,
        );
        fill(inner, buf, '/', self.theme.card_back);
    }
}

pub(crate) struct EmptyWidget {
    hinted: bool,
    theme: Rc<Theme>,
}

impl EmptyWidget {
    pub fn new(hinted: bool, theme: Rc<Theme>) -> Self {
        Self { hinted, theme }
    }
}

impl Widget for EmptyWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.hinted {
            self.theme.empty_slot.patch(self.theme.empty_slot_highlight)
        } else {
            self.theme.empty_slot
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(self.theme.card_border_type)
            .border_style(style)
            .style(self.theme.board);
        block.render(area, buf);
    }
}

fn fill(area: Rect, buf: &mut Buffer, ch: char, style: Style) {
    for y in 0..area.height {
        for x in 0..area.width {
            buf.cell_mut((area.x + x, area.y + y))
                .unwrap()
                .set_char(ch)
                .set_style(style);
        }
    }
}

fn render_border(
    area: Rect,
    selected: bool,
    buf: &mut Buffer,
    border: Style,
    highlight: Style,
    border_type: BorderType,
) -> Rect {
    let style = if selected {
        border.patch(highlight)
    } else {
        border
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(style);

    let inner = block.inner(area);
    block.render(area, buf);

    inner
}

pub(crate) struct DragStackWidget<'a> {
    cards: &'a [Card],

    theme: Rc<Theme>,
}

impl<'a> DragStackWidget<'a> {
    pub fn new(cards: &'a [Card], theme: Rc<Theme>) -> Self {
        Self { cards, theme }
    }
}

impl<'a> Widget for DragStackWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.cards.is_empty() {
            return;
        }

        let cards_len = self.cards.len();
        let full_card_height = area.height - cards_len as u16 + 1;

        let last_idx = cards_len - 1;
        for (i, &card) in self.cards.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y.saturating_add(area.height) {
                break;
            }
            let height = if i == last_idx { full_card_height } else { 1 };
            let card_area = Rect::new(area.x, y, area.width, height);

            // selected=true so the dragged stack is visually distinct
            CardWidget::new(card, true, self.theme.clone()).render(card_area, buf);
        }
    }
}
