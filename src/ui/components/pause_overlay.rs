use std::rc::Rc;

use crossterm::event::KeyEventKind;
use ratatui::{
    Frame,
    crossterm::event::Event as CrosstermEvent,
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Clear},
};
use tachyonfx::CenteredShrink;

use crate::ui::{
    component::{Component, UpdateResult},
    components::menu::{MenuComponent, MenuItem},
    event::AppEvent,
    keymap::GAME_KEYMAP,
    theme::OverlayStyles,
};

const ITEMS: &[MenuItem<PauseAction>] = &[
    MenuItem {
        label: "Resume",
        action: PauseAction::Resume,
    },
    MenuItem {
        label: "New Random Game",
        action: PauseAction::NewRandomGame,
    },
    // MenuItem {
    //     label: "New Solvable Game",
    //     action: PauseAction::NewSolvableGame,
    // },
    MenuItem {
        label: "Main Menu",
        action: PauseAction::GoToMenu,
    },
    MenuItem {
        label: "Quit",
        action: PauseAction::Quit,
    },
];

#[derive(Clone, Copy)]
pub(crate) enum PauseAction {
    Resume,
    NewRandomGame,
    GoToMenu,
    Quit,
}

pub(crate) struct PauseOverlay {
    menu: MenuComponent<PauseAction>,
    styles: Rc<OverlayStyles>,
}

impl PauseOverlay {
    pub(crate) fn new(styles: Rc<OverlayStyles>) -> Self {
        Self {
            menu: MenuComponent::new(styles.menu.clone(), ITEMS),
            styles,
        }
    }
}

impl Component<PauseAction> for PauseOverlay {
    fn update(&mut self, app_event: AppEvent) -> UpdateResult<PauseAction> {
        if let AppEvent::Input(CrosstermEvent::Key(key)) = app_event
            && key.kind == KeyEventKind::Press
            && GAME_KEYMAP.pause.matches(key)
        {
            return UpdateResult::Action(PauseAction::Resume);
        };

        self.menu.update(app_event)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.styles.border)
            .title(Line::from(" Paused ").style(self.styles.title).centered())
            .style(self.styles.surface);

        // +2 horizontal and vertical borders
        let panel = area.inner_centered(self.menu.width() + 2, self.menu.height() + 2);
        let inner = block.inner(panel);

        frame.render_widget(Clear, panel);
        frame.render_widget(block, panel);
        self.menu.render(frame, inner);
    }
}
