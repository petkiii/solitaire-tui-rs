



# solitaire-tui-rs

Mouse-first Klondike solitaire for the terminal, built with Rust and [Ratatui](https://ratatui.rs).  
Inspired by tty-solitaire and [solitaire-tui](https://github.com/brianstrauch/solitaire-tui).

[SHOWCASE](https://github.com/user-attachments/assets/188d2234-686e-462c-81b9-d44a0c4f2c8e)

## Installation

Requires [Rust](https://rust-lang.org/tools/install/).

### Install with Cargo

```sh
cargo install --locked --git https://github.com/petkiii/solitaire-tui-rs.git
```

Then run:
`solitaire-tui-rs`

### Build from source

```sh
git clone https://github.com/petkiii/solitaire-tui-rs.git
cd solitaire-tui-rs
cargo run -r
```

## Usage

Launch interactive menu:

```sh
solitaire-tui-rs
```

Start a game immediately, skipping the menu:

```sh
# Start a random game
solitaire-tui-rs -r

# Start a solvable game
solitaire-tui-rs -s
```

## Features

- Draw 1 Klondike with mouse-driven card dragging gameplay
- Random deals and solver-assisted deal generation
- Undo, hints, and autofinish
- One-click automove
- Timer and move tracking
- Gruvbox and tty-solitaire themes

## Roadmap

- [ ] Keyboard-only gameplay
- [ ] Win, hint, and autofinish animations
- [ ] New solver
- [ ] Seeded runs
- [ ] Lazy rendering
- [ ] More themes

## Configuration

Configuration is loaded from `~/.config/solitaire-tui-rs/config.toml`.

```toml
disable_animations = false
theme_kind = "gruvbox" # gruvbox | tty-solitaire
show_fps = false
```

## Note

Handcrafted artisan code.
