use std::fs;

use anyhow::Result;
use serde::Deserialize;

use crate::ui::theme::ThemeKind;

#[derive(Clone, Copy, Deserialize)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) disable_animations: bool,
    pub(crate) theme_kind: ThemeKind,
    pub(crate) show_fps: bool,
}

impl Config {
    pub(crate) fn load() -> Result<Self> {
        const CONFIG_PATH: &str = "solitaire-tui-rs/config.toml";

        // config_dir() resolves to different directories on different systems
        // therefore ~/.config is manually added
        // first existing config gets resolved
        let system_config_path = dirs::config_dir().map(|c| c.join(CONFIG_PATH));
        let dot_config_path = dirs::home_dir().map(|h| h.join(".config").join(CONFIG_PATH));

        let path = [system_config_path, dot_config_path]
            .into_iter()
            .flatten()
            .find(|x| x.exists());

        let Some(path) = path else {
            return Ok(Self::default());
        };

        let contents = fs::read_to_string(&path)?;
        // FIXME: invalid config silently falls back to default
        let config = toml::from_str::<Config>(&contents).unwrap_or_default();

        Ok(config)
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Config {
            disable_animations: false,
            theme_kind: ThemeKind::default(),
            show_fps: false,
        }
    }
}
