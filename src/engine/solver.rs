use std::collections::HashSet;

use ahash::RandomState;
use smallvec::SmallVec;

use crate::engine::game_state::{GameState, Move as EngineMove, Undo as EngineUndo};

const NODE_LIMIT: usize = 250_000;

type SolverUndo = EngineUndo<u8>;
type SolverMove = EngineMove<u8>;

impl From<EngineUndo> for SolverUndo {
    fn from(value: EngineUndo) -> Self {
        SolverUndo {
            applied: value.applied.into(),
            moved_count: value.moved_count,
            flipped_tableau_top: value.flipped_tableau_top,
        }
    }
}

impl From<SolverUndo> for EngineUndo {
    fn from(value: SolverUndo) -> Self {
        EngineUndo {
            applied: value.applied.into(),
            moved_count: value.moved_count,
            flipped_tableau_top: value.flipped_tableau_top,
        }
    }
}

impl From<EngineMove> for SolverMove {
    fn from(value: EngineMove) -> Self {
        match value {
            EngineMove::Draw => SolverMove::Draw,
            EngineMove::Recycle => SolverMove::Recycle,
            EngineMove::WasteToFoundation { foundation_idx } => SolverMove::WasteToFoundation {
                foundation_idx: foundation_idx as u8,
            },
            EngineMove::WasteToTableau { tableau_idx } => SolverMove::WasteToTableau {
                tableau_idx: tableau_idx as u8,
            },
            EngineMove::TableauToFoundation {
                tableau_idx,
                foundation_idx,
            } => SolverMove::TableauToFoundation {
                tableau_idx: tableau_idx as u8,
                foundation_idx: foundation_idx as u8,
            },
            EngineMove::FoundationToTableau {
                foundation_idx,
                tableau_idx,
            } => SolverMove::FoundationToTableau {
                foundation_idx: foundation_idx as u8,
                tableau_idx: tableau_idx as u8,
            },
            EngineMove::TableauToTableau {
                src_tableau_idx,
                card_idx,
                dst_tableau_idx,
            } => SolverMove::TableauToTableau {
                src_tableau_idx: src_tableau_idx as u8,
                card_idx: card_idx as u8,
                dst_tableau_idx: dst_tableau_idx as u8,
            },
        }
    }
}

impl From<SolverMove> for EngineMove {
    fn from(value: SolverMove) -> Self {
        match value {
            SolverMove::Draw => EngineMove::Draw,
            SolverMove::Recycle => EngineMove::Recycle,
            SolverMove::WasteToFoundation { foundation_idx } => EngineMove::WasteToFoundation {
                foundation_idx: foundation_idx as usize,
            },
            SolverMove::WasteToTableau { tableau_idx } => EngineMove::WasteToTableau {
                tableau_idx: tableau_idx as usize,
            },
            SolverMove::TableauToFoundation {
                tableau_idx,
                foundation_idx,
            } => EngineMove::TableauToFoundation {
                tableau_idx: tableau_idx as usize,
                foundation_idx: foundation_idx as usize,
            },
            SolverMove::FoundationToTableau {
                foundation_idx,
                tableau_idx,
            } => EngineMove::FoundationToTableau {
                foundation_idx: foundation_idx as usize,
                tableau_idx: tableau_idx as usize,
            },
            SolverMove::TableauToTableau {
                src_tableau_idx,
                card_idx,
                dst_tableau_idx,
            } => EngineMove::TableauToTableau {
                src_tableau_idx: src_tableau_idx as usize,
                card_idx: card_idx as usize,
                dst_tableau_idx: dst_tableau_idx as usize,
            },
        }
    }
}

impl GameState {
    pub(crate) fn fill_with_legal_solver_moves(&self, moves: &mut SmallVec<[SolverMove; 64]>) {
        self.visit_legal_moves(|mv| {
            moves.push(mv.into());
            true
        });
    }
}

struct Frame {
    moves: SmallVec<[SolverMove; 64]>,
    idx: usize,
    undo: Option<SolverUndo>,
}

pub(crate) struct Solver {
    visited: HashSet<u64, RandomState>,
    hash_builder: RandomState,
    depth: usize,
}

impl Solver {
    fn new() -> Self {
        let hash_builder = RandomState::new();
        Self {
            visited: HashSet::with_capacity_and_hasher(NODE_LIMIT * 2, hash_builder.clone()),
            depth: 0,
            hash_builder,
        }
    }

    pub(crate) fn is_solvable(initial: &GameState) -> bool {
        let mut solver = Self::new();
        solver.solve_iterative(initial)
    }

    fn fingerprint_state(&self, state: &GameState) -> u64 {
        self.hash_builder.hash_one(state)
    }

    fn solve_iterative(&mut self, initial: &GameState) -> bool {
        enum Next {
            Backtrack(Option<SolverUndo>),
            TryMove(SolverMove),
        }

        let mut state = initial.clone();
        let mut stack = Vec::with_capacity(256);

        let initial_fingerprint = self.fingerprint_state(initial);
        _ = self.visited.insert(initial_fingerprint);
        self.depth = 1;

        let mut moves = SmallVec::new();
        state.fill_with_legal_solver_moves(&mut moves);
        stack.push(Frame {
            moves,
            idx: 0,
            undo: None,
        });

        while !stack.is_empty() {
            if state.is_won() {
                return true;
            }

            if self.depth >= NODE_LIMIT {
                return false;
            }

            let next = {
                let frame = stack.last_mut().expect("stack should not be empty");
                if frame.idx >= frame.moves.len() {
                    Next::Backtrack(frame.undo)
                } else {
                    let mv = frame.moves[frame.idx];
                    frame.idx += 1;
                    Next::TryMove(mv)
                }
            };

            match next {
                Next::Backtrack(undo) => {
                    let _ = stack.pop();
                    if let Some(undo) = undo
                        && state.apply_undo(undo.into()).is_err()
                    {
                        return false;
                    }
                }
                Next::TryMove(mv) => {
                    let undo = match state.apply_move(mv.into()) {
                        Ok(u) => u,
                        Err(_) => continue,
                    };

                    let fingerprint = self.fingerprint_state(&state);
                    if self.visited.insert(fingerprint) {
                        self.depth += 1;

                        let mut moves = SmallVec::new();
                        state.fill_with_legal_solver_moves(&mut moves);
                        stack.push(Frame {
                            moves,
                            idx: 0,
                            undo: Some(undo.into()),
                        });
                    } else if state.apply_undo(undo).is_err() {
                        return false;
                    }
                }
            }
        }

        false
    }
}
