use std::fmt;
use std::path::PathBuf;

pub enum StatsError {
    UnknownFlag(String),
    PathNotFound(PathBuf),
    PermissionDenied(PathBuf),
    Stat(PathBuf, std::io::Error),
    ReadLink(PathBuf, std::io::Error),
}

impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatsError::UnknownFlag(flag) => write!(f, "unknown flag: {flag}"),
            StatsError::PathNotFound(p) => write!(f, "no such file or directory: {}", p.display()),
            StatsError::PermissionDenied(p) => write!(f, "permission denied: {}", p.display()),
            StatsError::Stat(p, e) => write!(f, "cannot stat {}: {e}", p.display()),
            StatsError::ReadLink(p, e) => {
                write!(f, "cannot read symlink target for {}: {e}", p.display())
            }
        }
    }
}
