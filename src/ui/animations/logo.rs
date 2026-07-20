use ratatui::style::{Color, Style};
use tachyonfx::{
    Effect, Interpolation,
    fx::{self, EvolveSymbolSet},
    pattern::{RadialPattern, SweepPattern},
};

use crate::ui::animations::shine;

const LOGO_SHINE_DELAY_MS: u32 = 2_000;
const LOGO_SHINE_DURATION_MS: u32 = 2_000;
const LOGO_SHINE_BAND: f32 = 4.0;

pub(crate) struct LogoAnimations;
impl LogoAnimations {
    pub(crate) fn random_transition() -> Effect {
        let effects = [
            fx::coalesce_from(Style::default(), (1500, Interpolation::ExpoInOut)),
            fx::evolve_into(
                EvolveSymbolSet::BlocksHorizontal,
                (1000, Interpolation::SineOut),
            )
            .with_pattern(SweepPattern::left_to_right(32)),
            fx::evolve_into(EvolveSymbolSet::Shaded, (900, Interpolation::SineOut))
                .with_pattern(RadialPattern::center().with_transition_width(6.0)),
            fx::evolve_into(EvolveSymbolSet::Quadrants, (1100, Interpolation::SineOut))
                .with_pattern(SweepPattern::left_to_right(28)),
            fx::parallel(&[
                fx::coalesce_from(Style::default(), (1200, Interpolation::ExpoInOut))
                    .with_pattern(RadialPattern::center().with_transition_width(8.0)),
            ]),
        ];

        let i = rand::random_range(0..effects.len());

        effects[i].clone()
    }

    pub(crate) fn idle(color: Color) -> Effect {
        fx::sequence(&[
            fx::sleep(LOGO_SHINE_DELAY_MS),
            shine(color, LOGO_SHINE_DURATION_MS, LOGO_SHINE_BAND),
        ])
    }
}
