use std::path::PathBuf;

use crate::error::NstatError;

/// Resolved run configuration.
pub struct Config {
    pub path: PathBuf,
    pub explain: bool,
    pub no_color: bool,
    pub no_follow: bool,
}

pub fn parse(argv: impl Iterator<Item = String>) -> Result<Config, NstatError> {
    let mut path: Option<PathBuf> = None;
    let mut explain = false;
    let mut no_color = false;
    let mut no_follow = false;

    for arg in argv {
        match arg.as_str() {
            "--explain" => explain = true,
            "--no-color" => no_color = true,
            "--no-follow" | "-l" => no_follow = true,
            flag if flag.starts_with('-') && flag.len() > 1 => {
                return Err(NstatError::UnknownFlag(flag.to_string()))
            }
            value => path = Some(PathBuf::from(value)),
        }
    }

    Ok(Config {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        explain,
        no_color,
        no_follow,
    })
}
