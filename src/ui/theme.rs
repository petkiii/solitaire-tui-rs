use std::rc::Rc;

use ratatui::{
    style::{Color, Modifier, Style},
    widgets::BorderType,
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ThemeKind {
    #[default]
    Gruvbox,
    TtySolitaire,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct HelpStyles {
    pub(crate) text: Style,
    pub(crate) key: Style,
    pub(crate) hovered: Style,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MenuStyles {
    pub(crate) normal: Style,
    pub(crate) selected: Style,
}

#[derive(Debug)]
pub(crate) struct OverlayStyles {
    pub(crate) surface: Style,
    pub(crate) border: Style,
    pub(crate) title: Style,
    pub(crate) text: Style,
    pub(crate) label: Style,
    pub(crate) menu: Rc<MenuStyles>,
    pub(crate) help: Rc<HelpStyles>,
}

#[derive(Debug, Clone)]
pub(crate) struct Theme {
    //
    pub board: Style,
    pub menu: Style,
    pub logo: Style,
    pub menu_items: Rc<MenuStyles>,
    pub menu_help: Rc<HelpStyles>,
    pub game_help: Rc<HelpStyles>,
    pub overlay: Rc<OverlayStyles>,

    // hud
    pub hud: Style,
    pub hud_label: Style,
    pub hud_value: Style,
    pub hud_error: Style,

    // card
    pub card_fill: Style,
    pub card_back: Style,
    pub card_red: Style,
    pub card_black: Style,
    pub card_border: Style,
    pub card_back_border: Style,
    pub card_border_highlight: Style,
    pub card_back_border_highlight: Style,
    pub empty_slot_highlight: Style,
    pub card_center_label: bool,

    pub empty_slot: Style,
    pub loading_overlay: Style,
    pub shine_color: Color,

    // misc
    pub card_ratio_nom: u16,
    pub card_ratio_denom: u16,
    pub center_board: bool,
    pub col_spacing: u16,
    pub row_spacing: u16,
    pub card_border_type: BorderType,
}

impl Theme {
    pub(crate) fn new(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Gruvbox => Self::gruvbox(),
            ThemeKind::TtySolitaire => Self::tty_solitaire(),
        }
    }

    fn gruvbox() -> Self {
        let bg = Color::Reset;
        let fg = Color::Rgb(0xeb, 0xdb, 0xb2);
        let muted = Color::Rgb(0x92, 0x83, 0x74);
        let hud_bg = Color::Rgb(0x3c, 0x38, 0x36);
        let accent = Color::Rgb(0xd7, 0x99, 0x21);
        let red = Color::Rgb(0xcc, 0x24, 0x1d);
        let black = fg;

        let menu_items = Rc::new(MenuStyles {
            normal: Style::default().fg(fg).bg(bg),
            selected: Style::default().fg(Color::Rgb(0x28, 0x28, 0x28)).bg(fg),
        });
        let help = Rc::new(HelpStyles {
            text: Style::default().fg(muted).bg(bg),
            key: Style::default().fg(accent).bg(bg),
            hovered: Style::default().fg(fg).bg(bg),
        });
        let overlay = Rc::new(OverlayStyles {
            surface: Style::default().bg(bg),
            border: Style::default().fg(fg).add_modifier(Modifier::BOLD),
            title: Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
            text: Style::default().fg(fg).bg(bg),
            label: Style::default().fg(accent).bg(bg),
            menu: menu_items.clone(),
            help: help.clone(),
        });

        Self {
            board: Style::default().bg(bg),
            menu: Style::default().bg(bg),
            logo: Style::default().fg(fg).add_modifier(Modifier::BOLD),
            menu_items,
            menu_help: help.clone(),
            game_help: help,
            overlay,
            // hud
            hud: Style::default().fg(fg).bg(hud_bg),
            hud_label: Style::default().fg(accent).bg(hud_bg),
            hud_value: Style::default().fg(fg).bg(hud_bg),
            hud_error: Style::default().fg(red).bg(bg),
            // card
            card_fill: Style::default().fg(muted).bg(bg),
            card_back: Style::default().fg(muted).bg(bg),
            card_red: Style::default().fg(red).bg(bg).add_modifier(Modifier::BOLD),
            card_black: Style::default()
                .fg(black)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
            card_border: Style::default().fg(fg).bg(bg),
            card_back_border: Style::default().fg(fg).bg(bg),
            card_border_highlight: Style::default().fg(accent),
            card_back_border_highlight: Style::default().fg(accent),
            empty_slot_highlight: Style::default().fg(accent),
            card_center_label: true,
            //
            empty_slot: Style::default().fg(muted).bg(bg),
            loading_overlay: Style::default().bg(bg),
            shine_color: Color::Rgb(0xff, 0x99, 0x00),
            // misc
            card_ratio_nom: 4,
            card_ratio_denom: 3,
            center_board: true,
            col_spacing: 0,
            row_spacing: 0,
            card_border_type: BorderType::Rounded,
        }
    }

    fn tty_solitaire() -> Self {
        // 7x5
        let bg = Color::Green;
        let fg = Color::Gray;
        let hud_bg = Color::DarkGray;
        let card_back_bg = Color::Blue;
        let card_front_bg = Color::Gray;
        let help_key = Color::Yellow;
        let red = Color::Red;
        let black = Color::Black;
        let overlay_bg = hud_bg;

        let hovered = Style::default()
            .fg(fg)
            .bg(card_back_bg)
            .add_modifier(Modifier::BOLD);
        let selected_menu = Style::default()
            .fg(black)
            .bg(help_key)
            .add_modifier(Modifier::BOLD);

        let menu_game_help = Rc::new(HelpStyles {
            text: Style::default().fg(fg).bg(bg),
            key: Style::default()
                .fg(help_key)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
            hovered,
        });
        let overlay = Rc::new(OverlayStyles {
            surface: Style::default().bg(overlay_bg),
            border: Style::default().fg(fg),
            title: Style::default()
                .fg(fg)
                .bg(overlay_bg)
                .add_modifier(Modifier::BOLD),
            text: Style::default().fg(fg).bg(overlay_bg),
            label: Style::default()
                .fg(help_key)
                .bg(overlay_bg)
                .add_modifier(Modifier::BOLD),
            menu: Rc::new(MenuStyles {
                normal: Style::default().fg(fg).bg(overlay_bg),
                selected: selected_menu,
            }),
            help: Rc::new(HelpStyles {
                text: Style::default().fg(fg).bg(overlay_bg),
                key: Style::default()
                    .fg(help_key)
                    .bg(overlay_bg)
                    .add_modifier(Modifier::BOLD),
                hovered,
            }),
        });

        Self {
            board: Style::default().bg(bg),
            menu: Style::default().bg(bg),
            logo: Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
            menu_items: Rc::new(MenuStyles {
                normal: Style::default().fg(fg).bg(bg),
                selected: selected_menu,
            }),
            menu_help: menu_game_help.clone(),
            game_help: menu_game_help,
            overlay,
            // hud
            hud: Style::default().fg(fg).bg(hud_bg),
            hud_label: Style::default()
                .fg(help_key)
                .bg(hud_bg)
                .add_modifier(Modifier::BOLD),
            hud_value: Style::default().fg(fg).bg(hud_bg),
            hud_error: Style::default().fg(fg).bg(red).add_modifier(Modifier::BOLD),
            card_fill: Style::default().fg(black).bg(card_front_bg),
            card_back: Style::default().fg(fg).bg(card_back_bg),
            card_red: Style::default().fg(red).bg(card_front_bg),
            card_black: Style::default().fg(black).bg(card_front_bg),
            card_border: Style::default().fg(fg).bg(card_front_bg),
            card_back_border: Style::default().fg(fg).bg(card_back_bg),
            card_border_highlight: Style::default().fg(help_key).add_modifier(Modifier::BOLD),
            card_back_border_highlight: Style::default().fg(help_key).add_modifier(Modifier::BOLD),
            empty_slot_highlight: Style::default().fg(help_key).add_modifier(Modifier::BOLD),
            card_center_label: false,
            empty_slot: Style::default().fg(fg).bg(bg),
            loading_overlay: Style::default().bg(bg),
            shine_color: Color::Blue,
            //
            card_ratio_nom: 7,
            card_ratio_denom: 5,
            center_board: false,
            col_spacing: 1,
            row_spacing: 1,
            card_border_type: BorderType::Thick,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(ThemeKind::default())
    }
}
