use ratatui::style::{Color, Style};
use tachyonfx::{
    Effect, Interpolation, RefRect,
    fx::{self},
};

use crate::ui::animations::shine;

const WIN_FADE_IN_MS: u32 = 800;
const WIN_SHINE_DELAY_MS: u32 = 800;
const WIN_SHINE_DURATION_MS: u32 = 1_800;
const WIN_SHINE_BAND: f32 = 6.0;

pub(crate) struct GameAnimations;

impl GameAnimations {
    pub(crate) fn loading(style: Style) -> Effect {
        fx::coalesce_from(style, (1000, Interpolation::ExpoInOut))
    }

    pub(crate) fn win_panel_fade(area: RefRect, fade_from: Style) -> Effect {
        fx::dynamic_area(
            area,
            fx::coalesce_from(fade_from, (WIN_FADE_IN_MS, Interpolation::ExpoInOut)),
        )
    }

    pub(crate) fn win_shine(color: Color) -> Effect {
        fx::repeating(fx::sequence(&[
            fx::sleep(WIN_SHINE_DELAY_MS),
            shine(color, WIN_SHINE_DURATION_MS, WIN_SHINE_BAND),
        ]))
    }

    // TODO: hint pulse animations
    //
    // pub(crate) fn hint_pulse(area: RefRect, color: Color) -> Effect {
    //     let pulse = fx::dynamic_area(
    //         area,
    //         fx::ping_pong(fx::fade_to_fg(
    //             color,
    //             (PULSE_DURATION_MS, Interpolation::SineInOut),
    //         )),
    //     );
    //     fx::repeat(pulse, RepeatMode::Times(3))
    // }
}
