mod cli;
mod core;
mod tui;

use clap::Parser;
use cli::{commands, Cli, Command};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Start { name }) => commands::start(name),
        Some(Command::Save { name, force }) => commands::save(name, force),
        Some(Command::List { json }) => commands::list(json),
        Some(Command::Open { name, force }) => commands::open(name, force),
        Some(Command::Attach { name }) => commands::attach(name),
        Some(Command::Delete { name }) => commands::delete(name),
        Some(Command::Rename { from, to }) => commands::rename(from, to),
        Some(Command::Duplicate { from, to }) => commands::duplicate(from, to),
        Some(Command::Status) => commands::status(),
        Some(Command::Prune) => commands::prune(),
        Some(Command::KillSession { name }) => commands::kill_session(name),
        Some(Command::KillWindow { target }) => commands::kill_window(target),
        Some(Command::KillServer) => commands::kill_server(),
        None => run_picker(),
    }
}

fn run_picker() -> anyhow::Result<()> {
    match tui::run()? {
        tui::PickerResult::Open(name) => commands::open(name, false),
        tui::PickerResult::Delete(name) => commands::delete(name),
        tui::PickerResult::Quit => Ok(()),
    }
}
