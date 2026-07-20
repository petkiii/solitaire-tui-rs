use std::{
    io::{self},
    panic,
};

use anyhow::Result;
use clap::{ArgGroup, Parser};
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};

use crate::ui::{app::App, config::Config};

mod engine;
mod ui;

#[derive(Debug, Clone, Copy)]
enum StartKind {
    Menu,
    Random,
    Solvable,
}

#[derive(Parser)]
#[command(
    version,
    about = "Mouse-first Klondike solitaire in your terminal",
    long_about = None,
    group(
        ArgGroup::new("start")
            .args(["random", "solvable"])
            .multiple(false)
    )
)]
struct Cli {
    /// Start random game immediately
    #[arg(short = 'r', long = "random")]
    random: bool,

    /// Start solvable game immediately
    #[arg(short = 's', long = "solvable")]
    solvable: bool,
}

impl Cli {
    fn start_kind(&self) -> StartKind {
        if self.random {
            StartKind::Random
        } else if self.solvable {
            StartKind::Solvable
        } else {
            StartKind::Menu
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start_kind = cli.start_kind();

    let config = Config::load()?;
    let terminal = ratatui::init();
    install_panic_hook();
    execute!(io::stdout(), EnableMouseCapture)?;

    let result = App::new(config, start_kind).run(terminal);

    restore();

    result
}

fn restore() {
    _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
}

fn install_panic_hook() {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore();
        prev(info);
    }));
}
