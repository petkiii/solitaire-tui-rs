use crate::engine::game_state::{GameState, Move};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HintScore {
    Reveal = 100,
    SafeFoundation = 90,
    WasteToTableau = 60,
    UnsafeFoundation = 40,
    Draw = 10,
    Never = 0,
}

impl GameState {
    pub(crate) fn hint_move(&self) -> Option<Move> {
        let mut best: Option<(HintScore, Move)> = None;
        self.visit_legal_moves(|mv| {
            let score = self.hint_score(mv);
            if score != HintScore::Never && best.is_none_or(|(x, _)| score > x) {
                best = Some((score, mv));
            }
            score != HintScore::Reveal
        });

        best.map(|(_, mv)| mv)
    }

    fn hint_score(&self, mv: Move) -> HintScore {
        match mv {
            Move::TableauToTableau {
                src_tableau_idx,
                card_idx,
                ..
            } => {
                // only worth hinting when it flips the face-down card below
                let src_tableau = &self.tableau(src_tableau_idx);
                let reveals = card_idx > 0 && !src_tableau[card_idx - 1].revealed;
                if reveals {
                    HintScore::Reveal
                } else {
                    HintScore::Never
                }
            }
            Move::TableauToFoundation { tableau_idx, .. } => {
                let tableau = &self.tableau(tableau_idx);
                let tableau_len = tableau.len();
                let card = tableau.last().unwrap();
                let reveals = tableau_len >= 2 && !tableau[tableau_len - 2].revealed;
                if reveals {
                    HintScore::Reveal
                } else if self.is_safe_to_foundation(card) {
                    HintScore::SafeFoundation
                } else {
                    HintScore::UnsafeFoundation
                }
            }
            Move::WasteToFoundation { .. } => {
                let card = self.waste_top().unwrap();
                if self.is_safe_to_foundation(card) {
                    HintScore::SafeFoundation
                } else {
                    HintScore::UnsafeFoundation
                }
            }
            Move::WasteToTableau { .. } => HintScore::WasteToTableau,
            Move::FoundationToTableau { .. } => HintScore::Never,
            Move::Draw | Move::Recycle => HintScore::Draw,
        }
    }
}
