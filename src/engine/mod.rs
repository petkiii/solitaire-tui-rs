use crate::engine::{
    errors::MoveError,
    game_state::{AutoMove, GameState, Move, Undo},
    solver::Solver,
};

pub(crate) mod card;
pub(crate) mod errors;
pub(crate) mod game_state;
pub(crate) mod hinting;
pub(crate) mod solver;

pub(crate) struct Engine {
    state: GameState,
    moves: u32,
    undos: Vec<Undo>,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            state: GameState::new_random(),
            moves: 0,
            undos: Vec::new(),
        }
    }
}

impl Engine {
    pub(crate) fn new() -> Self {
        let seed = if cfg!(debug_assertions) {
            1337
        } else {
            rand::random()
        };

        Self::from_state(GameState::from_seed(seed))
    }

    pub(crate) fn new_solvable() -> Self {
        // TODO: come up with a better solution
        for _ in 0..10 {
            let state = GameState::new_random();
            if Solver::is_solvable(&state) {
                return Self::from_state(state);
            }
        }

        Self::from_state(GameState::new_random())
    }

    pub(crate) fn from_state(state: GameState) -> Self {
        Self {
            state,
            moves: 0,
            undos: Vec::new(),
        }
    }

    pub(crate) fn apply_auto_move(&mut self, mv: AutoMove) -> Result<(), MoveError> {
        let resolved_mv = self.state.resolve_auto_move(mv)?;
        self.apply_move(resolved_mv)
    }

    pub(crate) fn apply_move(&mut self, mv: Move) -> Result<(), MoveError> {
        let res = self.state.apply_move(mv);
        match res {
            Ok(undo) => {
                self.moves += 1;
                self.undos.push(undo);

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) fn apply_undo(&mut self) -> Result<(), MoveError> {
        let Some(undo) = self.undos.pop() else {
            return Err(MoveError::NoUndos);
        };

        let res = self.state.apply_undo(undo);
        if res.is_ok() {
            self.moves += 1;
        }

        res
    }

    pub(crate) fn moves(&self) -> u32 {
        self.moves
    }

    pub(crate) fn undos(&self) -> usize {
        self.undos.len()
    }

    pub(crate) fn state(&self) -> &GameState {
        &self.state
    }
}
