use std::time::Instant;

use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use tachyonfx::{Effect, EffectManager, EffectTimer, Interpolation, fx};

pub(crate) mod game;
pub(crate) mod logo;

pub(crate) enum Animator {
    Enabled {
        effects: EffectManager<()>,
        last_frame: Instant,
    },
    Disabled,
}

impl Animator {
    pub(crate) fn new(disable_animations: bool) -> Self {
        if disable_animations {
            return Animator::Disabled;
        }

        Animator::Enabled {
            effects: EffectManager::default(),
            last_frame: Instant::now(),
        }
    }

    pub(crate) fn add_effect(&mut self, effect: Effect) {
        let Animator::Enabled {
            effects,
            last_frame: _,
        } = self
        else {
            return;
        };

        effects.add_effect(effect);
    }

    #[allow(dead_code)]
    pub(crate) fn is_running(&self) -> bool {
        let Animator::Enabled {
            effects,
            last_frame: _,
        } = self
        else {
            return false;
        };
        effects.is_running()
    }

    pub(crate) fn process_effects(&mut self, buf: &mut Buffer, area: Rect) {
        let Animator::Enabled {
            effects,
            last_frame,
        } = self
        else {
            return;
        };

        let elapsed = last_frame.elapsed();
        *last_frame = Instant::now();

        effects.process_effects(elapsed.into(), buf, area);
    }
}

pub(crate) fn shine(color: Color, duration_ms: u32, band: f32) -> Effect {
    fx::effect_fn(
        (),
        EffectTimer::from_ms(duration_ms, Interpolation::Linear),
        move |_, ctx, cells| {
            let progress = ctx.alpha();
            let width = ctx.area.width.max(1) as f32;
            let height = ctx.area.height.max(1) as f32;
            let shine_pos = progress * (width + height + band);

            cells.for_each_cell(|pos, cell| {
                if cell.symbol() == " " {
                    return;
                }

                let col = (pos.x - ctx.area.x) as f32;
                let row = (pos.y - ctx.area.y) as f32;
                let diagonal = col + row;

                if diagonal >= shine_pos && diagonal < shine_pos + band {
                    cell.set_fg(color);
                }
            });
        },
    )
}
