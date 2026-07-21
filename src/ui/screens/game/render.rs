use std::cmp;

use ::ratatui::prelude::*;
use ratatui::widgets::Block;

use crate::ui::{
    RectExt,
    component::Component,
    screens::game::{
        BoardHit, BoardHitKind, CardSize, Game, GameError, Hint, HintTarget, MARGIN,
        MIN_CARD_HEIGHT, MIN_CARD_WIDTH,
    },
    widgets::card::{CardBackWidget, CardWidget, DragStackWidget, EmptyWidget},
};

impl Game {
    fn render_board(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Block::default().style(self.theme.board), area);

        let card_width = self.card_size.width;
        let card_height = self.card_size.height;

        let area = if self.theme.center_board {
            area.centered_horizontally(Constraint::Length(Self::width_needed(
                &self.theme,
                card_width,
            )))
        } else {
            area
        };

        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(card_height), Constraint::Min(0)])
            .spacing(self.theme.row_spacing)
            .margin(MARGIN)
            .areas(area);

        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(card_width); 7])
            .spacing(self.theme.col_spacing)
            .areas::<7>(top);

        self.render_stock(frame, top_cols[0]);
        self.render_waste(frame, top_cols[1]);

        for (foundation_idx, col_idx) in (3..7).enumerate() {
            self.render_foundation(frame, top_cols[col_idx], foundation_idx);
        }

        let tableau_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(card_width); 7])
            .spacing(self.theme.col_spacing)
            .areas::<7>(bottom);

        #[allow(clippy::needless_range_loop)]
        for i in 0..7 {
            self.render_tableau(frame, tableau_cols[i], i);
        }
    }

    fn render_stock(&mut self, frame: &mut Frame, area: Rect) {
        let state = self.engine.state();
        let hinted = self.hint_matches(HintTarget::Stock);
        let selected = self
            .board_hit_kind
            .is_some_and(|h| matches!(h, BoardHitKind::EmptyStock | BoardHitKind::Stock))
            || hinted;

        if state.stock_len() == 0 {
            frame.render_widget(EmptyWidget::new(selected, self.theme.clone()), area);
            self.board_hits.push(BoardHit {
                area,
                kind: BoardHitKind::EmptyStock,
            });
        } else {
            frame.render_widget(CardBackWidget::new(selected, self.theme.clone()), area);
            self.board_hits.push(BoardHit {
                area,
                kind: BoardHitKind::Stock,
            });
        }
    }

    fn render_waste(&mut self, frame: &mut Frame, area: Rect) {
        let state = self.engine.state();
        let dragging_waste = self
            .drag
            .as_ref()
            .is_some_and(|d| d.started && matches!(d.hit_kind, BoardHitKind::WasteCard));

        if dragging_waste {
            let waste_len = state.waste_len();
            if waste_len >= 2 {
                let card = state.waste()[waste_len - 2];
                frame.render_widget(CardWidget::new(card, false, self.theme.clone()), area);
            } else {
                frame.render_widget(EmptyWidget::new(false, self.theme.clone()), area);
            }

            return;
        }

        match state.waste_top() {
            None => frame.render_widget(EmptyWidget::new(false, self.theme.clone()), area),
            Some(&card) => {
                let hinted = self.hint_matches(HintTarget::Waste);
                let selected = self
                    .board_hit_kind
                    .is_some_and(|h| matches!(h, BoardHitKind::WasteCard))
                    || hinted;
                frame.render_widget(CardWidget::new(card, selected, self.theme.clone()), area);
                self.board_hits.push(BoardHit {
                    area,
                    kind: BoardHitKind::WasteCard,
                });
            }
        };
    }

    fn render_foundation(&mut self, frame: &mut Frame, area: Rect, foundation_idx: usize) {
        let state = self.engine.state();
        let dragging_this_foundation = self.drag.as_ref().is_some_and(|d| {
            d.started
                && matches!(d.hit_kind, BoardHitKind::FoundationCard(idx) if idx == foundation_idx)
        });

        if dragging_this_foundation {
            let foundation_len = state.foundation_len(foundation_idx);
            if foundation_len >= 2 {
                let card = state.foundation(foundation_idx)[foundation_len - 2];
                frame.render_widget(CardWidget::new(card, false, self.theme.clone()), area);
            } else {
                frame.render_widget(EmptyWidget::new(false, self.theme.clone()), area);
            }
            return;
        }
        let hinted = self.hint_matches(HintTarget::Foundation(foundation_idx));

        match state.foundation_top(foundation_idx) {
            None => {
                frame.render_widget(EmptyWidget::new(hinted, self.theme.clone()), area);
                self.board_hits.push(BoardHit {
                    area,
                    kind: BoardHitKind::EmptyFoundation(foundation_idx),
                });
            }
            Some(&card) => {
                let selected = self.board_hit_kind.is_some_and(
                    |h| matches!(h, BoardHitKind::FoundationCard(idx) if idx == foundation_idx),
                ) || hinted;
                frame.render_widget(CardWidget::new(card, selected, self.theme.clone()), area);
                self.board_hits.push(BoardHit {
                    area,
                    kind: BoardHitKind::FoundationCard(foundation_idx),
                });
            }
        };
    }

    fn render_tableau(&mut self, frame: &mut Frame, area: Rect, tableau_idx: usize) {
        let state = self.engine.state();
        let mut cards = state.tableau(tableau_idx);

        if cards.is_empty() {
            let hinted = self.hint_matches(HintTarget::EmptyTableau { tableau_idx });
            frame.render_widget(
                EmptyWidget::new(hinted, self.theme.clone()),
                Rect::new(area.x, area.y, self.card_size.width, self.card_size.height),
            );
            self.board_hits.push(BoardHit {
                area,
                kind: BoardHitKind::EmptyTableau(tableau_idx),
            });
            return;
        }

        let hide_from = self.drag.as_ref().and_then(|d| {
            if !d.started {
                return None;
            }
            match d.hit_kind {
                BoardHitKind::TableauCard {
                    tableau_idx: t,
                    card_idx,
                } if t == tableau_idx => Some(card_idx),
                _ => None,
            }
        });

        if let Some(hide_from) = hide_from {
            cards = &cards[..hide_from];
        }

        if cards.is_empty() {
            // render empty card when all cards are being moved from current tableau
            frame.render_widget(
                EmptyWidget::new(false, self.theme.clone()),
                Rect::new(area.x, area.y, self.card_size.width, self.card_size.height),
            );
            return;
        }

        let last_idx = cards.len() - 1;
        for (card_idx, &card) in cards.iter().enumerate() {
            let hinted = self.hint_matches(HintTarget::Tableau {
                tableau_idx,
                card_idx,
            });

            let selected = self.board_hit_kind.is_some_and(|h| {
                matches!(
                    h,
                    BoardHitKind::TableauCard { tableau_idx: hit_tableau_idx, card_idx: hit_card_idx }
                        if hit_tableau_idx == tableau_idx && hit_card_idx == card_idx
                )
            }) || hinted;

            let card_height = if card_idx == last_idx {
                self.card_size.height
            } else {
                1
            };
            let y = area.y + (card_idx as u16);
            let card_area = Rect::new(area.x, y, self.card_size.width, card_height);

            frame.render_widget(
                CardWidget::new(card, selected, self.theme.clone()),
                card_area,
            );
            if card.revealed() {
                self.board_hits.push(BoardHit {
                    area: card_area,
                    kind: BoardHitKind::TableauCard {
                        tableau_idx,
                        card_idx,
                    },
                });
            }
        }
    }

    fn render_drag_overlay(&mut self, frame: &mut Frame, board_area: Rect) {
        let Some(drag) = self.drag.as_ref() else {
            return;
        };

        if !drag.started || drag.cards.is_empty() {
            return;
        }

        let desired_x = drag.cursor.0.saturating_sub(drag.offset.0);
        let desired_y = drag.cursor.1.saturating_sub(drag.offset.1);

        let stack_height = (drag.cards.len() as u16).saturating_sub(1) + self.card_size.height;

        // clamp to board area
        let max_x = board_area
            .x
            .saturating_add(board_area.width)
            .saturating_sub(self.card_size.width);
        let max_y = board_area
            .y
            .saturating_add(board_area.height)
            .saturating_sub(stack_height);
        let x = cmp::min(cmp::max(desired_x, board_area.x), max_x);
        let y = cmp::min(cmp::max(desired_y, board_area.y), max_y);

        let overlay_area = Rect::new(x, y, self.card_size.width, stack_height);

        frame.render_widget(
            DragStackWidget::new(&drag.cards, self.theme.clone()),
            overlay_area,
        );
    }

    fn render_hud(&mut self, frame: &mut Frame<'_>, hud: Rect) {
        let full_hud = Line::from(vec![
            Span::styled("Time: ", self.theme.hud_label),
            Span::styled(self.timer.format(), self.theme.hud_value),
            Span::raw(" • "),
            Span::styled("Moves: ", self.theme.hud_label),
            Span::styled(self.engine.moves().to_string(), self.theme.hud_value),
            Span::raw(" • "),
            Span::styled("Undos: ", self.theme.hud_label),
            Span::styled(self.engine.undos().to_string(), self.theme.hud_value),
        ])
        .style(self.theme.hud)
        .centered();

        if full_hud.width() as u16 <= hud.width {
            frame.render_widget(full_hud, hud);
            return;
        }

        // compact form
        let compact_hud = Line::from(vec![
            Span::styled(self.timer.format(), self.theme.hud_value),
            Span::raw(" • "),
            Span::styled(format!("{}m", self.engine.moves()), self.theme.hud_value),
            Span::raw(" • "),
            Span::styled(format!("{}u", self.engine.undos()), self.theme.hud_value),
        ])
        .style(self.theme.hud)
        .centered();

        frame.render_widget(compact_hud, hud);
    }

    fn render_error(&self, frame: &mut Frame<'_>, help: Rect, game_error: &GameError) {
        let error_line = Line::from(game_error.err.to_string())
            .style(self.theme.hud_error)
            .centered();

        frame.render_widget(error_line, help);
    }

    pub(super) fn render_internal(&mut self, frame: &mut Frame, area: Rect) {
        self.board_hits.clear();

        let [board, help, hud] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(area);

        // card size
        if self.card_layout_dirty {
            self.compute_card_size(board);
            self.card_layout_dirty = false;
        }

        // board
        self.render_board(frame, board);

        // drag
        self.render_drag_overlay(frame, board);

        // hud
        self.render_hud(frame, hud);

        // help or error
        if let Some(game_error) = &self.error {
            self.render_error(frame, help, game_error);
        } else {
            // help line wrapping should not affect card size calculations
            // we get 1 help line from layout split then we sneakily take away from board
            let help_height = self.help.height(help.width);
            let help = help.resize_height_from_top(help_height);
            self.help.render(frame, help);
        }

        // win ovarlay
        if self.won
            && let Some(win_overlay) = self.win_overlay.as_mut()
        {
            win_overlay.render(frame, board);
        }

        // should come after win_overlay but before pause_overlay
        self.animator.process_effects(frame.buffer_mut(), area);

        // pause overlay
        if let Some(pause) = self.pause_overlay.as_mut() {
            pause.render(frame, board);
        }
    }

    fn compute_card_size(&mut self, board_area: Rect) {
        // apply margin
        let avail_width = board_area.width.saturating_sub(2 * MARGIN);
        let avail_height = board_area.height.saturating_sub(2 * MARGIN);

        for card_height in (MIN_CARD_HEIGHT..=(avail_height / 2)).rev() {
            if card_height % 2 != 1 {
                continue;
            }

            let mut card_width =
                (card_height * self.theme.card_ratio_nom).div_ceil(self.theme.card_ratio_denom);
            card_width = card_width.max(MIN_CARD_WIDTH);

            let needed_width = 7 * card_width + 6 * self.theme.col_spacing;
            if needed_width > avail_width {
                continue;
            }

            let max_tableau_height = 17 + card_height;
            let needed_height = card_height + self.theme.row_spacing + max_tableau_height;
            if needed_height > avail_height {
                continue;
            }

            self.card_size = CardSize {
                width: card_width,
                height: card_height,
            };
            return;
        }

        self.card_size = CardSize {
            width: MIN_CARD_WIDTH,
            height: MIN_CARD_HEIGHT,
        };
    }

    fn hint_matches(&self, target: HintTarget) -> bool {
        // if hint is some and target1 or target2 matches target
        self.hint.as_ref().is_some_and(
            |Hint {
                 targets,
                 displayed_at: _,
             }| {
                let (target1, target2) = targets;
                *target1 == target || target2.is_some_and(|t| t == target)
            },
        )
    }
}
