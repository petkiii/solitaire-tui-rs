use std::slice;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub(crate) enum Binding {
    Single(Key),
    Multi(&'static [Key]),
}

impl Binding {
    pub fn matches(&self, event: KeyEvent) -> bool {
        self.keys().iter().any(|key| key.matches(event))
    }

    pub fn keys(&self) -> &[Key] {
        match self {
            Self::Single(key) => slice::from_ref(key),
            Self::Multi(keys) => keys,
        }
    }

    #[allow(dead_code)]
    pub fn labels(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.keys().iter().map(|key| key.label)
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::Single(key) => key.label,
            Self::Multi(keys) => keys[0].label,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
    label: &'static str,
}

impl Key {
    pub const fn new(code: KeyCode, label: &'static str) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::NONE,
            label,
        }
    }

    pub const fn with_modifier(
        code: KeyCode,
        modifiers: KeyModifiers,
        label: &'static str,
    ) -> Self {
        Self {
            code,
            modifiers,
            label,
        }
    }

    pub fn matches(self, key: KeyEvent) -> bool {
        self.code == key.code && self.modifiers == key.modifiers
    }

    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        self.label
    }
}

#[derive(Debug)]
pub(crate) struct GlobalKeymap {
    pub quit: Binding,
}

#[derive(Debug)]
pub(crate) struct MenuKeymap {
    pub previous: Binding,
    pub next: Binding,
    pub confirm: Binding,
}

#[derive(Debug)]
pub(crate) struct GameKeymap {
    pub pause: Binding,
    pub undo: Binding,
    pub hint: Binding,
    pub autofinish: Binding,
    // pause and win overlay
    pub new_solvable: Binding,
    pub new_random: Binding,
    pub go_to_menu: Binding,
}

pub(crate) const GLOBAL_KEYMAP: GlobalKeymap = GlobalKeymap {
    quit: Binding::Multi(&[
        Key::new(KeyCode::Char('q'), "q"),
        Key::with_modifier(KeyCode::Char('c'), KeyModifiers::CONTROL, "ctrl+c"),
    ]),
};
pub(crate) const GAME_KEYMAP: GameKeymap = GameKeymap {
    pause: Binding::Single(Key::new(KeyCode::Esc, "esc")),
    undo: Binding::Single(Key::new(KeyCode::Char('u'), "u")),
    hint: Binding::Single(Key::new(KeyCode::Char('h'), "h")),
    autofinish: Binding::Multi(&[
        Key::new(KeyCode::Char('f'), "f"),
        Key::new(KeyCode::Char(' '), "space"),
    ]),
    new_solvable: Binding::Single(Key::new(KeyCode::Char('n'), "n")),
    new_random: Binding::Single(Key::new(KeyCode::Char('r'), "r")),
    go_to_menu: Binding::Single(Key::new(KeyCode::Char('m'), "m")),
};
pub(crate) const MENU_KEYMAP: MenuKeymap = MenuKeymap {
    previous: Binding::Multi(&[
        Key::new(KeyCode::Up, "↑"),
        Key::new(KeyCode::Char('k'), "k"),
    ]),
    next: Binding::Multi(&[
        Key::new(KeyCode::Down, "↓"),
        Key::new(KeyCode::Char('j'), "j"),
    ]),
    confirm: Binding::Single(Key::new(KeyCode::Enter, "enter")),
};
