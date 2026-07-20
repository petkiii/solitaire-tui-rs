#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Quit,
    GoToMenu,
    NewRandomGame,
    NewSolvableGame,
}
