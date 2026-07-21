use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event as CrosstermEvent, KeyEventKind},
    layout::{
        Constraint::{self},
        Rect,
    },
    text::{Line, Text},
};

use crate::{
    StartKind,
    ui::{
        action::Action,
        component::{Component, UpdateResult},
        config::Config,
        event::{self, AppEvent, Event},
        keymap::GLOBAL_KEYMAP,
        screens::{Screen, game::Game},
        theme::Theme,
    },
};

pub(crate) struct App {
    running: bool,
    screen: Screen,
    config: Rc<Config>,
    theme: Rc<Theme>,
    fps: u64,
    fps_frames: u64,
    fps_window_started: Instant,
}

impl App {
    pub(crate) fn new(config: Config, start_kind: StartKind) -> Self {
        let config = Rc::new(config);
        let theme = Rc::new(Theme::new(config.theme_kind));

        let screen = match start_kind {
            StartKind::Menu => Screen::new_menu(config.clone(), theme.clone()),
            StartKind::Random => Screen::new_random_game(config.clone(), theme.clone()),
            StartKind::Solvable => Screen::new_solvable_game(config.clone(), theme.clone()),
        };

        Self {
            running: false,
            screen,
            config,
            theme,
            fps: 0,
            fps_frames: 0,
            fps_window_started: Instant::now(),
        }
    }

    pub(crate) fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        if self.running {
            return Err(anyhow!("already running"));
        }

        self.running = true;

        const TARGET_FPS: u64 = 60;
        let frame_time = Duration::from_micros(1_000_000 / TARGET_FPS);

        terminal.draw(|frame| self.draw(frame))?;

        self.run_lazy(terminal, frame_time)
    }

    fn run_lazy(&mut self, mut terminal: DefaultTerminal, frame_time: Duration) -> Result<()> {
        let mut next_frame = Instant::now();
        let mut redraw = true;

        while self.running {
            let next_tick = self.screen.next_tick(Instant::now());
            let next_render = if redraw || self.screen.is_animating() {
                Some(next_frame)
            } else {
                None
            };
            let event = event::next_event(next_render, next_tick)?;

            match event {
                Event::Render => {
                    let now = Instant::now();
                    terminal.draw(|frame| self.draw(frame))?;

                    if self.config.show_fps {
                        self.update_fps();
                    }

                    redraw = false;
                    next_frame = now + frame_time;
                }
                Event::AppEvent(app_event) => redraw |= self.update(app_event)?,
            }
        }

        Ok(())
    }

    fn update(&mut self, event: AppEvent) -> Result<bool> {
        let mut resized = false;

        if let AppEvent::Input(crossterm_event) = &event {
            match crossterm_event {
                CrosstermEvent::Key(key)
                    if key.kind == KeyEventKind::Press && (GLOBAL_KEYMAP.quit.matches(*key)) =>
                {
                    self.apply_action(Action::Quit);
                    return Ok(true);
                }
                CrosstermEvent::Resize(_, _) => resized = true,
                _ => {}
            }
        };

        match self.screen.update(event) {
            UpdateResult::Unchanged => Ok(resized),
            UpdateResult::Changed => Ok(true),
            UpdateResult::Action(action) => {
                self.apply_action(action);
                Ok(true)
            }
        }
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::GoToMenu => {
                self.screen = Screen::new_menu(self.config.clone(), self.theme.clone())
            }
            Action::NewRandomGame => {
                self.screen = Screen::new_random_game(self.config.clone(), self.theme.clone())
            }
            Action::NewSolvableGame => {
                self.screen = Screen::new_solvable_game(self.config.clone(), self.theme.clone())
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if self.config.show_fps {
            frame.render_widget(
                Line::from(format!("FPS: {}", self.fps)).centered(),
                Rect {
                    x: 0,
                    y: 0,
                    width: area.width,
                    height: 1,
                },
            );
        }

        let (req_width, req_height) = Game::min_board_size(&self.theme);
        if area.width < req_width || area.height < req_height {
            let mut text = Text::from("Terminal is too small.");
            text.push_line(format!("Current W = {}, H = {}", area.width, area.height));
            text.push_line(format!("Required W = {}, H = {}", req_width, req_height));
            frame.render_widget(
                text.centered(),
                area.centered_vertically(Constraint::Length(3)),
            );
            return;
        }

        self.screen.render(frame, area);
    }

    fn update_fps(&mut self) {
        self.fps_frames += 1;

        let now = Instant::now();
        let elapsed = now.duration_since(self.fps_window_started);

        if elapsed >= Duration::from_secs(1) {
            self.fps = (self.fps_frames as f64 / elapsed.as_secs_f64()).round() as u64;
            self.fps_frames = 0;
            self.fps_window_started = now;
        }
    }
}
