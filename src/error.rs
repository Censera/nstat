use std::fmt;
use std::path::PathBuf;

pub enum NstatError {
    UnknownFlag(String),
    PathNotFound(PathBuf),
    PermissionDenied(PathBuf),
    Stat(PathBuf, std::io::Error),
    ReadLink(PathBuf, std::io::Error),
}

impl fmt::Display for NstatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NstatError::UnknownFlag(flag) => write!(f, "unknown flag: {flag}"),
            NstatError::PathNotFound(p) => write!(f, "no such file or directory: {}", p.display()),
            NstatError::PermissionDenied(p) => write!(f, "permission denied: {}", p.display()),
            NstatError::Stat(p, e) => write!(f, "cannot stat {}: {e}", p.display()),
            NstatError::ReadLink(p, e) => {
                write!(f, "cannot read symlink target for {}: {e}", p.display())
            }
        }
    }
}
