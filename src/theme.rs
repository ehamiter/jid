use gpui::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Dark,
    Midnight,
    Sepia,
    Ocean,
    Forest,
    Light,
}

impl ThemeMode {
    pub fn next(&self) -> Self {
        match self {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Sepia,
            ThemeMode::Sepia => ThemeMode::Ocean,
            ThemeMode::Ocean => ThemeMode::Forest,
            ThemeMode::Forest => ThemeMode::Midnight,
            ThemeMode::Midnight => ThemeMode::Dark,
        }
    }
}

#[derive(Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub background: Hsla,
    pub foreground: Hsla,
    pub muted: Hsla,
    pub selection: Hsla,
    pub focus_current: Hsla,
    pub focus_dimmed: Hsla,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            background: hsla(0.67, 0.08, 0.12, 1.0),
            foreground: hsla(0.17, 0.06, 0.82, 1.0),
            muted: hsla(0.17, 0.04, 0.45, 1.0),
            selection: hsla(0.58, 0.30, 0.35, 0.40),
            focus_current: hsla(0.17, 0.06, 0.82, 1.0),
            focus_dimmed: hsla(0.17, 0.04, 0.45, 1.0),
        }
    }

    pub fn midnight() -> Self {
        Self {
            mode: ThemeMode::Midnight,
            background: hsla(0.67, 0.05, 0.06, 1.0),
            foreground: hsla(0.0, 0.0, 0.30, 1.0),
            muted: hsla(0.0, 0.0, 0.20, 1.0),
            selection: hsla(0.67, 0.10, 0.15, 0.50),
            focus_current: hsla(0.0, 0.0, 0.22, 1.0),
            focus_dimmed: hsla(0.0, 0.0, 0.15, 1.0),
        }
    }

    pub fn sepia() -> Self {
        Self {
            mode: ThemeMode::Sepia,
            background: hsla(0.10, 0.25, 0.88, 1.0),
            foreground: hsla(0.08, 0.35, 0.25, 1.0),
            muted: hsla(0.08, 0.20, 0.50, 1.0),
            selection: hsla(0.10, 0.35, 0.70, 0.35),
            focus_current: hsla(0.08, 0.35, 0.25, 1.0),
            focus_dimmed: hsla(0.08, 0.20, 0.50, 1.0),
        }
    }

    pub fn ocean() -> Self {
        Self {
            mode: ThemeMode::Ocean,
            background: hsla(0.55, 0.15, 0.14, 1.0),
            foreground: hsla(0.52, 0.12, 0.78, 1.0),
            muted: hsla(0.52, 0.10, 0.45, 1.0),
            selection: hsla(0.50, 0.40, 0.40, 0.40),
            focus_current: hsla(0.52, 0.12, 0.78, 1.0),
            focus_dimmed: hsla(0.52, 0.10, 0.45, 1.0),
        }
    }

    pub fn forest() -> Self {
        Self {
            mode: ThemeMode::Forest,
            background: hsla(0.30, 0.12, 0.13, 1.0),
            foreground: hsla(0.25, 0.08, 0.80, 1.0),
            muted: hsla(0.25, 0.06, 0.45, 1.0),
            selection: hsla(0.35, 0.35, 0.35, 0.40),
            focus_current: hsla(0.25, 0.08, 0.80, 1.0),
            focus_dimmed: hsla(0.25, 0.06, 0.45, 1.0),
        }
    }

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            background: hsla(0.15, 0.10, 0.94, 1.0),
            foreground: hsla(0.17, 0.08, 0.25, 1.0),
            muted: hsla(0.17, 0.05, 0.55, 1.0),
            selection: hsla(0.58, 0.30, 0.75, 0.30),
            focus_current: hsla(0.17, 0.08, 0.25, 1.0),
            focus_dimmed: hsla(0.17, 0.05, 0.55, 1.0),
        }
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Midnight => Self::midnight(),
            ThemeMode::Sepia => Self::sepia(),
            ThemeMode::Ocean => Self::ocean(),
            ThemeMode::Forest => Self::forest(),
            ThemeMode::Light => Self::light(),
        }
    }

    pub fn toggled(&self) -> Self {
        Self::from_mode(self.mode.next())
    }
}
