use std::rc::Rc;

use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Margin, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear},
};
use tachyonfx::RefRect;

use crate::ui::{
    action::Action,
    component::{Component, UpdateResult},
    components::interactive_help::{HelpItem, InteractiveHelpComponent},
    event::AppEvent,
    keymap::GAME_KEYMAP,
    theme::OverlayStyles,
};

pub(crate) struct WinOverlay {
    time: String,
    moves: u32,
    area: RefRect,
    help: InteractiveHelpComponent<Action>,
    styles: Rc<OverlayStyles>,
}

impl WinOverlay {
    pub(crate) fn new(styles: Rc<OverlayStyles>, time: String, moves: u32) -> Self {
        let help_items = vec![
            HelpItem::new(
                GAME_KEYMAP.new_solvable.label(),
                "new solvable",
                Action::NewSolvableGame,
            ),
            HelpItem::new(
                GAME_KEYMAP.new_random.label(),
                "new random",
                Action::NewRandomGame,
            ),
            HelpItem::new(GAME_KEYMAP.go_to_menu.label(), "menu", Action::GoToMenu),
        ];
        let help = InteractiveHelpComponent::new(styles.help.clone(), help_items);

        Self {
            time,
            moves,
            styles,
            area: RefRect::default(),
            help,
        }
    }

    pub(crate) fn area(&self) -> RefRect {
        self.area.clone()
    }
}

impl Component<Action> for WinOverlay {
    fn update(&mut self, event: AppEvent) -> UpdateResult<Action> {
        if let AppEvent::Input(CrosstermEvent::Key(key)) = event
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                _ if GAME_KEYMAP.new_solvable.matches(key) => {
                    return UpdateResult::Action(Action::NewSolvableGame);
                }
                _ if GAME_KEYMAP.new_random.matches(key) => {
                    return UpdateResult::Action(Action::NewRandomGame);
                }
                _ if GAME_KEYMAP.go_to_menu.matches(key) || GAME_KEYMAP.pause.matches(key) => {
                    return UpdateResult::Action(Action::GoToMenu);
                }
                _ => {}
            }
        };

        self.help.update(event)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect)
    where
        Self: Sized,
    {
        let text = Text::from(vec![
            Line::from("You won!").style(self.styles.title).centered(),
            Line::default(),
            Line::from(vec![
                Span::styled("Time  ", self.styles.label),
                Span::styled(&self.time, self.styles.text),
                Span::raw("   "),
                Span::styled("Moves ", self.styles.label),
                Span::styled(self.moves.to_string(), self.styles.text),
            ])
            .centered(),
            Line::default(),
        ]);

        const BORDER: u16 = 2;
        let width = (46 + BORDER).min(area.width);
        let help_height = self.help.height(width - BORDER);
        let height = text.height() as u16 + BORDER + help_height;
        let area = area.centered(Constraint::Length(width), Constraint::Length(height));
        self.area.set(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.styles.border)
            .style(self.styles.surface);
        let inner = block.inner(area);

        const HORIZONTAL_PADDING: u16 = 1;
        let text_area = inner.inner(Margin::new(HORIZONTAL_PADDING, 0));
        let help_area = Rect {
            y: inner.bottom() - help_height,
            height: help_height,
            ..inner
        };

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        frame.render_widget(text, text_area);
        self.help.render(frame, help_area);
    }
}
