use std::{
    collections::VecDeque,
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::Error;
use crossterm::event::Event;
use ratatui::{Frame, layout::Rect};

use crate::{
    engine::{Engine, card::Card, game_state::Move},
    ui::{
        Hit,
        action::Action,
        animations::{Animator, game::GameAnimations},
        component::Component,
        components::{
            interactive_help::{HelpItem, InteractiveHelpComponent},
            pause_overlay::PauseOverlay,
            win_overlay::WinOverlay,
        },
        config::Config,
        keymap::{GAME_KEYMAP, GLOBAL_KEYMAP},
        screens::game::timer::Timer,
        theme::Theme,
    },
};

mod render;
mod timer;
mod update;

const MARGIN: u16 = 1;
const MIN_CARD_WIDTH: u16 = 4;
const MIN_CARD_HEIGHT: u16 = 3;
const ERROR_TTL: Duration = Duration::from_secs(1);
const HINT_TTL: Duration = Duration::from_secs(1);
const AUTOFINISH_STEP: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HelpAction {
    Undo,
    Hint,
    Menu,
    Quit,
    Autofinish,
}

#[derive(Debug, Clone, Copy)]
enum BoardHitKind {
    Stock,
    EmptyStock,
    WasteCard,
    TableauCard { tableau_idx: usize, card_idx: usize },
    FoundationCard(usize),
    EmptyFoundation(usize),
    EmptyTableau(usize),
}

type BoardHit = Hit<BoardHitKind>;

#[derive(Debug)]
struct Drag {
    hit_kind: BoardHitKind,
    cards: Vec<Card>,
    // only use-case is to prevent dragging from start before moving the mouse, therefore allowing
    initial_cursor: (u16, u16),
    cursor: (u16, u16),
    offset: (u16, u16),
    started: bool,
}

#[derive(Debug)]
struct CardSize {
    width: u16,
    height: u16,
}

#[derive(Debug)]
struct Autofinish {
    moves: VecDeque<Move>,
    last_step: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum HintTarget {
    Stock,
    Waste,
    Tableau { tableau_idx: usize, card_idx: usize },
    EmptyTableau { tableau_idx: usize },
    Foundation(usize),
}

struct Hint {
    targets: (HintTarget, Option<HintTarget>),
    displayed_at: Instant,
}

struct GameError {
    err: Error,
    displayed_at: Instant,
}

pub(crate) struct Game {
    engine: Engine,

    // hits
    board_hits: Vec<BoardHit>,
    board_hit_kind: Option<BoardHitKind>,

    // state
    hint: Option<Hint>,
    error: Option<GameError>,
    drag: Option<Drag>,
    autofinish_available: bool,
    // when this is some, it means that autofinish is in progress
    autofinish: Option<Autofinish>,
    won: bool,

    // layout
    card_size: CardSize,
    card_layout_dirty: bool,

    // misc
    help: InteractiveHelpComponent<HelpAction>,
    win_overlay: Option<WinOverlay>,
    pause_overlay: Option<PauseOverlay>,
    timer: Timer,

    // theme and anim
    theme: Rc<Theme>,
    animator: Animator,
}

impl Game {
    pub(crate) fn new(config: Rc<Config>, theme: Rc<Theme>) -> Self {
        Self::with_solvable(config.as_ref(), theme, false)
    }

    pub(crate) fn new_solvable(config: Rc<Config>, theme: Rc<Theme>) -> Self {
        Self::with_solvable(config.as_ref(), theme, true)
    }

    pub(crate) fn width_needed(theme: &Theme, card_width: u16) -> u16 {
        7 * card_width + 6 * theme.col_spacing + MARGIN * 2
    }

    pub(crate) fn min_board_size(theme: &Theme) -> (u16, u16) {
        // +17: all cards stacked, +2: Help line + HUD line
        (
            Self::width_needed(theme, MIN_CARD_WIDTH),
            MIN_CARD_HEIGHT + theme.row_spacing + 17 + MIN_CARD_HEIGHT + MARGIN * 2 + 2,
        )
    }

    fn with_solvable(config: &Config, theme: Rc<Theme>, solvable: bool) -> Self {
        let engine = if solvable {
            Engine::new_solvable()
        } else {
            Engine::new()
        };

        let help_items = vec![
            HelpItem::new(GAME_KEYMAP.undo.label(), "undo", HelpAction::Undo),
            HelpItem::new(GAME_KEYMAP.hint.label(), "hint", HelpAction::Hint),
            HelpItem::new(
                GAME_KEYMAP.autofinish.label(),
                "autofinish",
                HelpAction::Autofinish,
            )
            .hidden(),
            HelpItem::new(GAME_KEYMAP.pause.label(), "menu", HelpAction::Menu),
            HelpItem::new(GLOBAL_KEYMAP.quit.label(), "quit", HelpAction::Quit),
        ];

        let mut animator = Animator::new(config.disable_animations);
        animator.add_effect(GameAnimations::loading(theme.loading_overlay));

        Self {
            engine,
            board_hit_kind: None,
            board_hits: Vec::new(),
            error: None,
            drag: None,
            card_size: CardSize {
                width: MIN_CARD_WIDTH,
                height: MIN_CARD_HEIGHT,
            },
            card_layout_dirty: true,
            timer: Timer::new(),
            won: false,
            pause_overlay: None,
            help: InteractiveHelpComponent::new(theme.game_help.clone(), help_items),
            hint: None,
            autofinish_available: false,
            autofinish: None,
            theme,
            animator,
            win_overlay: None,
        }
    }
}

impl Component<Action> for Game {
    fn update(&mut self, event: Event) -> Option<Action> {
        self.update_internal(event)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.render_internal(frame, area);
    }
}
