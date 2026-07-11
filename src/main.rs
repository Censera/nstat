mod args;
mod color;
mod error;
mod info;
mod output;

use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("nstat: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), error::NstatError> {
    let config = args::parse(std::env::args().skip(1))?;
    let pal = color::Palette::decide(config.no_color);
    let info = info::gather(&config.path, config.no_follow)?;
    print!("{}", output::render(&info, config.explain, &pal));
    Ok(())
}
