use std::path::PathBuf;

use crate::error::NstatError;

/// Resolved run configuration.
pub struct Config {
    pub paths: Vec<PathBuf>,
    pub explain: bool,
    pub no_color: bool,
    pub no_follow: bool,
    pub help: bool,
}

pub fn parse(argv: impl Iterator<Item = String>) -> Result<Config, NstatError> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut explain = false;
    let mut no_color = false;
    let mut no_follow = false;
    let mut help = false;

    for arg in argv {
        match arg.as_str() {
            "-h" | "--help" => help = true,
            "--explain" => explain = true,
            "--no-color" => no_color = true,
            "--no-follow" | "-l" => no_follow = true,
            flag if flag.starts_with('-') && flag.len() > 1 => {
                return Err(NstatError::UnknownFlag(flag.to_string()))
            }
            value => paths.push(PathBuf::from(value)),
        }
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    Ok(Config {
        paths,
        explain,
        no_color,
        no_follow,
        help,
    })
}

pub fn print_help() {
    println!(
        r#"nstat (neostat)
  lightweight system stats & file tracking engine

USAGE:
  nstat [OPTIONS] [PATH]...

OPTIONS:
  -h, --help       Print this help message
      --explain    Provide detailed metric breakdowns
      --no-color   Disable ANSI terminal colors
  -l, --no-follow  Do not follow symlinks

ARGS:
  <PATH>...        Directories or files to analyze [default: .]"#
    );
}
