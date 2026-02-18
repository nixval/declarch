#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOwner {
    Pacman,
    Homebrew,
    Scoop,
    Winget,
    Script,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct UpdateHint {
    pub current: String,
    pub latest: String,
    pub owner: InstallOwner,
}
