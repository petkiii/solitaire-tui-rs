use std::time::Instant;

use anyhow::{Error, Result, anyhow};
use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};

use crate::{
    engine::{
        Engine,
        card::Card,
        game_state::{AutoMove, Move},
    },
    ui::{
        action::Action,
        animations::game::GameAnimations,
        component::Component,
        components::{
            pause_overlay::{PauseAction, PauseOverlay},
            win_overlay::WinOverlay,
        },
        hit_test,
        keymap::GAME_KEYMAP,
        screens::game::{
            AUTOFINISH_STEP, Autofinish, BoardHit, BoardHitKind, Drag, Game, GameError, HelpAction,
            Hint, HintTarget,
        },
    },
};

impl Game {
    fn get_dragged_cards(&self, hit: BoardHit) -> Option<Vec<Card>> {
        let state = self.engine.state();

        match hit.kind {
            BoardHitKind::WasteCard => state.waste_top().cloned().map(|c| vec![c]),
            BoardHitKind::FoundationCard(foundation_idx) => state
                .foundation_top(foundation_idx)
                .cloned()
                .map(|c| vec![c]),
            BoardHitKind::TableauCard {
                tableau_idx,
                card_idx,
            } => {
                let cards = state.tableau(tableau_idx);
                let dragged_cards = &cards[card_idx..];
                if dragged_cards.first().is_some_and(|c| !c.revealed()) {
                    return None;
                }

                Some(dragged_cards.to_vec())
            }
            _ => None,
        }
    }

    fn check_for_win(&mut self) {
        if !self.won && self.engine.state().is_won() {
            self.trigger_win();
        }
    }

    fn trigger_win(&mut self) {
        self.won = true;
        self.timer.pause();
        self.help.set_visible(HelpAction::Undo, false);
        self.help.set_visible(HelpAction::Hint, false);
        self.help.set_visible(HelpAction::Menu, false);

        let win_overlay = WinOverlay::new(
            self.theme.overlay.clone(),
            self.timer.format(),
            self.engine.moves(),
        );
        self.animator.add_effect(GameAnimations::win_panel_fade(
            win_overlay.area(),
            self.theme.loading_overlay,
        ));
        self.win_overlay = Some(win_overlay);

        self.animator
            .add_effect(GameAnimations::win_shine(self.theme.shine_color));
    }

    #[cfg(debug_assertions)]
    fn load_autofinish_ready(&mut self) {
        use crate::engine::game_state::GameState;

        self.engine = Engine::from_state(GameState::autofinish_ready());
        self.board_hit_kind = None;
        self.drag = None;
        self.hint = None;
        self.error = None;
        self.autofinish = None;
        self.won = false;
        self.check_for_autofinish();
    }

    fn check_for_autofinish(&mut self) {
        self.autofinish_available =
            self.autofinish.is_none() && !self.won && self.engine.state().all_tableau_revealed();
        self.help
            .set_visible(HelpAction::Autofinish, self.autofinish_available);
    }

    fn show_pause(&mut self) {
        self.pause_overlay = Some(PauseOverlay::new(self.theme.overlay.clone()));
        self.timer.pause();
    }

    fn show_hint(&mut self) {
        let Some(mv) = self.engine.state().hint_move() else {
            self.error = Some(GameError {
                err: anyhow!("no useful moves available"),
                displayed_at: Instant::now(),
            });
            return;
        };

        let targets = self.hint_targets(mv);
        self.hint = Some(Hint {
            targets,
            displayed_at: Instant::now(),
        });
    }

    fn hint_targets(&self, mv: Move) -> (HintTarget, Option<HintTarget>) {
        let state = self.engine.state();
        let tableau_top = |tableau_idx: usize| {
            let tableau = state.tableau(tableau_idx);

            if tableau.is_empty() {
                return HintTarget::EmptyTableau { tableau_idx };
            }

            HintTarget::Tableau {
                tableau_idx,
                card_idx: tableau.len() - 1,
            }
        };

        match mv {
            Move::Draw | Move::Recycle => (HintTarget::Stock, None),
            Move::WasteToFoundation { foundation_idx } => (
                HintTarget::Waste,
                Some(HintTarget::Foundation(foundation_idx)),
            ),
            Move::WasteToTableau { tableau_idx } => {
                (HintTarget::Waste, Some(tableau_top(tableau_idx)))
            }
            Move::TableauToFoundation {
                tableau_idx,
                foundation_idx,
            } => (
                tableau_top(tableau_idx),
                Some(HintTarget::Foundation(foundation_idx)),
            ),
            Move::FoundationToTableau {
                foundation_idx,
                tableau_idx,
            } => (
                HintTarget::Foundation(foundation_idx),
                Some(tableau_top(tableau_idx)),
            ),
            Move::TableauToTableau {
                src_tableau_idx,
                card_idx,
                dst_tableau_idx,
            } => (
                HintTarget::Tableau {
                    tableau_idx: src_tableau_idx,
                    card_idx,
                },
                Some(tableau_top(dst_tableau_idx)),
            ),
        }
    }

    fn start_autofinish(&mut self) {
        let Some(moves) = self.engine.state().autofinish_moves() else {
            return;
        };

        // reset state before autofinish start
        self.board_hit_kind = None;
        self.drag = None;
        self.hint = None;

        self.autofinish = Some(Autofinish {
            moves: moves.into(),
            last_step: Instant::now(),
        });

        // reset autofinish
        self.check_for_autofinish();
    }

    // TODO: implement event based tick system
    pub(super) fn step_autofinish(&mut self) {
        if self.autofinish.is_none() {
            return;
        }

        let step = self.autofinish.as_mut().and_then(|af| {
            if af.last_step.elapsed() >= AUTOFINISH_STEP {
                af.last_step = Instant::now();
                af.moves.pop_front()
            } else {
                None
            }
        });
        let Some(mv) = step else {
            return;
        };

        if self.engine.apply_move(mv).is_err() {
            self.autofinish = None;
        }

        self.check_for_win();

        if self
            .autofinish
            .as_ref()
            .is_some_and(|af| af.moves.is_empty())
        {
            self.autofinish = None;
        }
    }

    fn handle_mouse_down(&mut self, mouse: MouseEvent) {
        let board_hit = hit_test(&self.board_hits, mouse.column, mouse.row);
        let Some(board_hit) = board_hit else {
            return;
        };

        self.board_hit_kind = match board_hit.kind {
            BoardHitKind::EmptyFoundation(_) | BoardHitKind::EmptyTableau(_) => None,
            _ => {
                // start timer at first mouse hit
                self.timer.start();
                // remove hint on mouse hit
                self.hint = None;

                Some(board_hit.kind)
            }
        };

        let Some(cards) = self.get_dragged_cards(board_hit) else {
            return;
        };

        let offset_x = mouse.column.saturating_sub(board_hit.area.x);
        let offset_y = mouse.row.saturating_sub(board_hit.area.y);
        let cursor = (mouse.column, mouse.row);

        self.drag = Some(Drag {
            hit_kind: board_hit.kind,
            cards,
            initial_cursor: cursor,
            cursor,
            offset: (offset_x, offset_y),
            started: false,
        });
    }

    fn handle_mouse_drag(&mut self, mouse: MouseEvent) {
        let Some(drag) = self.drag.as_mut() else {
            return;
        };

        if drag.initial_cursor == (mouse.column, mouse.row) {
            return;
        }

        drag.cursor = (mouse.column, mouse.row);
        drag.started = true;
    }

    fn handle_mouse_up(&mut self, mouse: MouseEvent) {
        let drag = self.drag.take();

        let Some(mouse_down_hit_kind) = self.board_hit_kind.take() else {
            return;
        };

        let hit = hit_test(&self.board_hits, mouse.column, mouse.row);
        let Some(mouse_up_hit_kind) = hit.map(|hit| hit.kind) else {
            return;
        };

        let res = if drag.is_some_and(|d| d.started) {
            self.apply_drop(mouse_down_hit_kind, mouse_up_hit_kind)
        } else {
            self.apply_click(mouse_down_hit_kind, mouse_up_hit_kind)
        };

        if let Err(err) = res {
            self.error = Some(GameError {
                err,
                displayed_at: Instant::now(),
            })
        } else {
            self.error = None;
            self.check_for_win();
            self.check_for_autofinish();
        }
    }

    fn apply_undo(&mut self) {
        // reset states before applying undo
        self.drag = None;
        self.board_hit_kind = None;
        self.hint = None;

        let res = self.engine.apply_undo();
        match res {
            Err(err) => {
                self.error = Some(GameError {
                    err: err.into(),
                    displayed_at: Instant::now(),
                });
            }
            Ok(_) => {
                self.error = None;
                self.check_for_autofinish();
            }
        }
    }

    fn apply_click(
        &mut self,
        mouse_down_hit_kind: BoardHitKind,
        mouse_up_hit_kind: BoardHitKind,
    ) -> Result<(), Error> {
        match (mouse_down_hit_kind, mouse_up_hit_kind) {
            (BoardHitKind::Stock, BoardHitKind::Stock) => {
                self.engine.apply_move(Move::Draw).map_err(Into::into)
            }
            (BoardHitKind::EmptyStock, BoardHitKind::EmptyStock) => {
                self.engine.apply_move(Move::Recycle).map_err(Into::into)
            }
            (BoardHitKind::WasteCard, BoardHitKind::WasteCard) => self
                .engine
                .apply_auto_move(AutoMove::WasteCard)
                .map_err(Into::into),
            (
                BoardHitKind::TableauCard {
                    tableau_idx: src_tableau_idx,
                    card_idx: src_card_idx,
                },
                BoardHitKind::TableauCard {
                    tableau_idx: dst_tableau_idx,
                    card_idx: dst_card_idx,
                },
            ) if src_tableau_idx == dst_tableau_idx && src_card_idx == dst_card_idx => self
                .engine
                .apply_auto_move(AutoMove::TableauCard {
                    tableau_idx: src_tableau_idx,
                    card_idx: src_card_idx,
                })
                .map_err(Into::into),
            (
                BoardHitKind::FoundationCard(src_foundation_idx),
                BoardHitKind::FoundationCard(dst_foundation_idx),
            ) if src_foundation_idx == dst_foundation_idx => self
                .engine
                .apply_auto_move(AutoMove::FoundationCard {
                    foundation_idx: src_foundation_idx,
                })
                .map_err(Into::into),
            _ => Err(anyhow!("invalid click")),
        }
    }

    fn apply_drop(
        &mut self,
        mouse_down_hit_kind: BoardHitKind,
        mouse_up_hit_kind: BoardHitKind,
    ) -> Result<(), Error> {
        match (mouse_down_hit_kind, mouse_up_hit_kind) {
            (BoardHitKind::Stock, BoardHitKind::Stock) => {
                self.engine.apply_move(Move::Draw).map_err(Into::into)
            }
            (
                BoardHitKind::WasteCard,
                BoardHitKind::TableauCard {
                    tableau_idx,
                    card_idx: _,
                },
            )
            | (BoardHitKind::WasteCard, BoardHitKind::EmptyTableau(tableau_idx)) => self
                .engine
                .apply_move(Move::WasteToTableau { tableau_idx })
                .map_err(Into::into),
            (BoardHitKind::WasteCard, BoardHitKind::FoundationCard(foundation_idx)) => self
                .engine
                .apply_move(Move::WasteToFoundation { foundation_idx })
                .map_err(Into::into),
            (BoardHitKind::WasteCard, BoardHitKind::EmptyFoundation(foundation_idx)) => self
                .engine
                .apply_move(Move::WasteToFoundation { foundation_idx })
                .map_err(Into::into),
            (
                BoardHitKind::TableauCard {
                    tableau_idx: src_tableau_idx,
                    card_idx,
                },
                BoardHitKind::TableauCard {
                    tableau_idx: dst_tableau_idx,
                    card_idx: _,
                },
            )
            | (
                BoardHitKind::TableauCard {
                    tableau_idx: src_tableau_idx,
                    card_idx,
                },
                BoardHitKind::EmptyTableau(dst_tableau_idx),
            ) => self
                .engine
                .apply_move(Move::TableauToTableau {
                    src_tableau_idx,
                    card_idx,
                    dst_tableau_idx,
                })
                .map_err(Into::into),
            (
                BoardHitKind::TableauCard {
                    tableau_idx,
                    card_idx,
                },
                BoardHitKind::FoundationCard(foundation_idx),
            )
            | (
                BoardHitKind::TableauCard {
                    tableau_idx,
                    card_idx,
                },
                BoardHitKind::EmptyFoundation(foundation_idx),
            ) => {
                if card_idx != self.engine.state().tableau(tableau_idx).len() - 1 {
                    return Err(anyhow!("only single card can be moved to foundation"));
                }

                self.engine
                    .apply_move(Move::TableauToFoundation {
                        tableau_idx,
                        foundation_idx,
                    })
                    .map_err(Into::into)
            }
            (
                BoardHitKind::FoundationCard(foundation_idx),
                BoardHitKind::TableauCard {
                    tableau_idx,
                    card_idx: _,
                },
            )
            | (
                BoardHitKind::FoundationCard(foundation_idx),
                BoardHitKind::EmptyTableau(tableau_idx),
            ) => self
                .engine
                .apply_move(Move::FoundationToTableau {
                    foundation_idx,
                    tableau_idx,
                })
                .map_err(Into::into),
            _ => Err(anyhow!("invalid hit state")),
        }
    }

    pub(super) fn update_internal(&mut self, event: Event) -> Option<Action> {
        if let Event::Resize(_, _) = event {
            self.card_layout_dirty = true;
        }

        // autofinish playback ignores input
        if self.autofinish.is_some() {
            return None;
        }

        // pause overlay captures all input while open
        if self.pause_overlay.is_some() {
            return self.update_pause_overlay(event);
        }

        // victory screen
        if self.won {
            return self.win_overlay.as_mut().and_then(|w| w.update(event));
        }

        // handle key and mouse events
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                _ if GAME_KEYMAP.pause.matches(key) => self.show_pause(),
                _ if GAME_KEYMAP.undo.matches(key) => self.apply_undo(),
                _ if GAME_KEYMAP.hint.matches(key) => self.show_hint(),
                _ if GAME_KEYMAP.autofinish.matches(key) && self.autofinish_available => {
                    self.start_autofinish()
                }
                #[cfg(debug_assertions)]
                _ if key.code == KeyCode::Char('r') => self.trigger_win(),
                #[cfg(debug_assertions)]
                _ if key.code == KeyCode::Char('t') => self.load_autofinish_ready(),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => self.handle_mouse_down(mouse),
                MouseEventKind::Drag(_) => self.handle_mouse_drag(mouse),
                MouseEventKind::Up(MouseButton::Left) => self.handle_mouse_up(mouse),
                _ => {}
            },
            _ => {}
        }

        // update help
        let help_kind = self.help.update(event)?;
        match help_kind {
            HelpAction::Undo => self.apply_undo(),
            HelpAction::Hint => self.show_hint(),
            HelpAction::Menu => self.show_pause(),
            HelpAction::Quit => return Some(Action::Quit),
            HelpAction::Autofinish => self.start_autofinish(),
        };

        None
    }

    fn update_pause_overlay(&mut self, event: Event) -> Option<Action> {
        let pause = self.pause_overlay.as_mut()?;
        let action = pause.update(event)?;
        match action {
            PauseAction::Resume => {
                self.pause_overlay = None;
                self.timer.resume();
                None
            }
            PauseAction::NewRandomGame => Some(Action::NewRandomGame),
            PauseAction::GoToMenu => Some(Action::GoToMenu),
            PauseAction::Quit => Some(Action::Quit),
        }
    }
}
