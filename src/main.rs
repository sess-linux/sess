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
        Some(Command::Start {
            name,
            auto_save,
            no_auto_save,
        }) => commands::start(name, auto_save, no_auto_save),
        Some(Command::Save { name, force }) => commands::save(name, force),
        Some(Command::List { json }) => commands::list(json),
        Some(Command::Open { name, force }) => commands::open(name, force),
        Some(Command::Switch) => commands::switch(),
        Some(Command::Attach { name }) => commands::attach(name),
        Some(Command::Close { name }) => commands::close(name),
        Some(Command::Delete { name }) => commands::delete(name),
        Some(Command::Rename { from, to }) => commands::rename(from, to),
        Some(Command::Duplicate { from, to }) => commands::duplicate(from, to),
        Some(Command::Status) => commands::status(),
        Some(Command::Doctor { fix }) => commands::doctor(fix),
        Some(Command::AutoSave {
            name,
            stop,
            interval,
        }) => commands::auto_save(name, stop, interval),
        Some(Command::Prune) => commands::prune(),
        Some(Command::KillSession { name }) => commands::kill_session(name),
        Some(Command::KillWindow { target }) => commands::kill_window(target),
        Some(Command::KillServer) => commands::kill_server(),
        Some(Command::AutoSaveLoop { name, interval }) => commands::auto_save_loop(name, interval),
        None => run_picker(),
    }
}

/// Shared by `sess` (no subcommand) and `sess switch` — both open the same
/// interactive picker, per the architecture's existing convention of not
/// duplicating behavior across entry points.
pub fn run_picker() -> anyhow::Result<()> {
    match tui::run()? {
        tui::PickerResult::Open(name) => commands::open(name, false),
        tui::PickerResult::Quit => Ok(()),
    }
}
