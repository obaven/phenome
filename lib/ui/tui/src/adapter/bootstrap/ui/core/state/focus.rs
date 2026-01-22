#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum FocusTarget {
    #[default]
    Tree,
    Status,
}

impl FocusTarget {
    pub fn toggle(self) -> Self {
        match self {
            FocusTarget::Tree => FocusTarget::Status,
            FocusTarget::Status => FocusTarget::Tree,
        }
    }
}
