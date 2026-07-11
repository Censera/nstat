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

    if config.help {
       print_help();
       return Ok(());
    }
    let info = info::gather(&config.path, config.no_follow)?;
    print!("{}", output::render(&info, config.explain, &pal));
    Ok(())
}

fn print_help() {
    println!(
        r#"nstat - lightweight system stats & file tracking engine

USAGE:
    nstat [OPTIONS] [PATH]

OPTIONS:
    -h, --help       Print this help message
    --explain        Provide detailed metric breakdowns
    --no-color       Disable ANSI terminal colors
    -l, --no-follow  Do not follow symlinks

ARGS:
    <PATH>           Directory or file to analyze [default: .]"#
    );
}
