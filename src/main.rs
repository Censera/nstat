mod args;
mod color;
mod error;
mod info;
mod output;

use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("nstat: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Returns Ok(true) if all paths were successfully processed,
/// or Ok(false) if one or more paths failed to sta.
fn run() -> Result<bool, error::NstatError> {
    let config = args::parse(std::env::args().skip(1))?;
    let pal = color::Palette::decide(config.no_color);

    if config.help {
        args::print_help();
        return Ok(true);
    }

    let mut all_success = true;
    let path_count = config.paths.len();

    for (i, path) in config.paths.iter().enumerate() {
        match info::gather(path, config.no_follow) {
            Ok(info) => {
                print!("{}", output::render(&info, config.explain, &pal));
                if i < path_count - 1 {
                    println!();
                }
            }
            Err(e) => {
                eprintln!("nstat: {e}");
                all_success = false;
            }
        }
    }

    Ok(all_success)
}
