use ratatui::layout::Rect;
use std::collections::HashSet;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalImageProtocol {
    Kitty,
    ITerm2,
    None,
}

impl TerminalImageProtocol {
    pub fn detect() -> Self {
        if let Some(protocol) = Self::from_env() {
            return protocol;
        }
        if env::var("KITTY_WINDOW_ID").is_ok()
            || env::var("TERM")
                .map(|term| term.contains("kitty"))
                .unwrap_or(false)
        {
            return Self::Kitty;
        }
        if env::var("ITERM_SESSION_ID").is_ok()
            || env::var("TERM_PROGRAM")
                .map(|term| term == "iTerm.app")
                .unwrap_or(false)
        {
            return Self::ITerm2;
        }
        Self::None
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Kitty => "Kitty",
            Self::ITerm2 => "iTerm2",
            Self::None => "none",
        }
    }

    fn from_env() -> Option<Self> {
        let value = env::var("ROTAPPO_TUI_GRAPHICS").ok()?;
        match value.to_lowercase().as_str() {
            "kitty" => Some(Self::Kitty),
            "iterm" | "iterm2" | "iterm.app" => Some(Self::ITerm2),
            "none" | "off" | "disabled" => Some(Self::None),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphRenderStatus {
    Idle,
    Pending,
    Rendered,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub struct GraphRenderRequest {
    pub area: Rect,
    pub dot: String,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub tail: usize,
    pub head: usize,
    pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Default, Clone)]
pub struct GraphDependencyPath {
    pub nodes: HashSet<usize>,
    pub edges: HashSet<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct GraphBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}
