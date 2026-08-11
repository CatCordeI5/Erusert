use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EruModel {
    /// Full Strium - NOT RECOMMENDED. Full system access. Dangerous.
    FullStrium,
    /// Assistant/Friend - RECOMMENDED. Caring companion mode.
    Assistant,
    /// Terminal-Eru - For devs who want to edit the model / open-source coding helper.
    TerminalEru,
    /// Exathry - Fast-lite. Kinda recommended.
    Exathry,
    /// Avl-Eru - Fastest of all Eru models.
    AvlEru,
}

impl EruModel {
    pub fn from_key(key: char) -> Option<Self> {
        match key.to_ascii_lowercase() {
            'p' => Some(Self::FullStrium),
            'u' => Some(Self::Assistant),
            'q' => Some(Self::TerminalEru),
            'z' => Some(Self::Exathry),
            '2' => Some(Self::AvlEru),
            _ => None,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::FullStrium => "A. Full Strium (p)         NOT RECOMMENDED - full system access!",
            Self::Assistant => "B. Assistant/Friend (u)    RECOMMENDED - new friend yay :D",
            Self::TerminalEru => "C. Terminal-Eru (q)        For editing model / coding helper",
            Self::Exathry => "D. Exathry (z)             Kinda recommended - fast-lite",
            Self::AvlEru => "E. Avl-Eru (2)             Most fastest of all Eru models",
        }
    }

    pub fn download_size_mb(&self) -> u64 {
        match self {
            Self::FullStrium => 2400,
            Self::Assistant => 380,
            Self::TerminalEru => 520,
            Self::Exathry => 180,
            Self::AvlEru => 95,
        }
    }

    pub fn is_dangerous(&self) -> bool {
        matches!(self, Self::FullStrium)
    }
}
