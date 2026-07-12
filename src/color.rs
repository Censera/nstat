use std::fmt;
use std::io::IsTerminal;

/// Whether to emit ANSI color codes at all. Decided once, at startup,
/// from three inputs in priority order: explicit flag, NO_COLOR, then
/// whether stdout is actually a terminal.
pub struct Palette {
    enabled: bool,
}

pub struct Painted<'a> {
    text: &'a str,
    code: &'a str,
    enabled: bool,
}

impl<'a> fmt::Display for Painted<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.enabled {
            write!(f, "\x1b[{}m{}\x1b[0m", self.code, self.text)
        } else {
            f.write_str(self.text)
        }
    }
}

impl Palette {
    pub fn decide(no_color_flag: bool) -> Palette {
        let enabled = !no_color_flag
            && std::env::var_os("NO_COLOR").is_none()
            && std::io::stdout().is_terminal();
        Palette { enabled }
    }

    pub fn label<'a>(&self, s: &'a str) -> Painted<'a> {
        self.wrap(s, "0")
    }
    pub fn value<'a>(&self, s: &'a str) -> Painted<'a> {
        self.wrap(s, "1")
    }
    pub fn identity<'a>(&self, s: &'a str) -> Painted<'a> {
        self.wrap(s, "3")
    }
    pub fn stat<'a>(&self, s: &'a str) -> Painted<'a> {
        self.wrap(s, "32")
    }
    pub fn perm<'a>(&self, s: &'a str) -> Painted<'a> {
        self.wrap(s, "32")
    }
    pub fn time<'a>(&self, s: &'a str) -> Painted<'a> {
        self.wrap(s, "32")
    }

    fn wrap<'a>(&self, text: &'a str, code: &'a str) -> Painted<'a> {
        Painted {
            text,
            code,
            enabled: self.enabled,
        }
    }
}
