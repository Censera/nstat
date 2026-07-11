use std::io::IsTerminal;

/// Whether to emit ANSI color codes at all. Decided once, at startup,
/// from three inputs in priority order: explicit flag, NO_COLOR, then
/// whether stdout is actually a terminal.
pub struct Palette {
    enabled: bool,
}

impl Palette {
    pub fn decide(no_color_flag: bool) -> Palette {
        let enabled = !no_color_flag
            && std::env::var_os("NO_COLOR").is_none()
            && std::io::stdout().is_terminal();
        Palette { enabled }
    }

    pub fn label(&self, s: &str) -> String {
        self.wrap(s, "0") // normal
    }

    pub fn value(&self, s: &str) -> String {
        self.wrap(s, "1") // bold
    }

    pub fn identity(&self, s: &str) -> String {
        self.wrap(s, "3") // italic
    }

    pub fn stat(&self, s: &str) -> String {
        self.wrap(s, "32") // green
    }

    pub fn perm(&self, s: &str) -> String {
        self.wrap(s, "32") // green
    }

    pub fn time(&self, s: &str) -> String {
        self.wrap(s, "32") // green
    }

    fn wrap(&self, s: &str, code: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
}
