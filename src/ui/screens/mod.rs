use std::{rc::Rc, time::Instant};

use ratatui::{Frame, layout::Rect};

use crate::ui::{
    action::Action,
    component::{Component, UpdateResult},
    config::Config,
    event::AppEvent,
    screens::{game::Game, main_menu::MainMenu},
    theme::Theme,
};

pub(crate) mod game;
pub(crate) mod main_menu;

pub(crate) enum Screen {
    MainMenu(MainMenu),
    Game(Game),
}

impl Screen {
    pub(crate) fn new_menu(config: Rc<Config>, theme: Rc<Theme>) -> Self {
        Screen::MainMenu(MainMenu::new(config, theme))
    }

    pub(crate) fn new_random_game(config: Rc<Config>, theme: Rc<Theme>) -> Self {
        Screen::Game(Game::new(config, theme))
    }

    pub(crate) fn new_solvable_game(config: Rc<Config>, theme: Rc<Theme>) -> Self {
        Screen::Game(Game::new_solvable(config, theme))
    }

    pub(crate) fn is_animating(&self) -> bool {
        match self {
            Screen::MainMenu(main_menu) => main_menu.is_animating(),
            Screen::Game(game) => game.is_animating(),
        }
    }
}

impl Component<Action> for Screen {
    fn update(&mut self, event: AppEvent) -> UpdateResult<Action> {
        match self {
            Screen::MainMenu(menu) => menu.update(event),
            Screen::Game(game) => game.update(event),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self {
            Screen::MainMenu(menu) => menu.render(frame, area),
            Screen::Game(game) => game.render(frame, area),
        }
    }

    fn next_tick(&mut self, now: Instant) -> Option<Instant> {
        match self {
            Screen::MainMenu(_) => None,
            Screen::Game(game) => game.next_tick(now),
        }
    }
}
