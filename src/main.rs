#![forbid(unsafe_code)]

use clap::Parser;
use principia_studio::cli::{run_cli_action, RootCli, TopCommand};
use principia_studio::gui::launch_gui;
use principia_studio::ModuleId;

fn main() {
    let cli = RootCli::parse();
    match cli.command {
        None => {
            if let Err(err) = launch_gui(ModuleId::N1PrimeSieve, cli.threads, cli.skip_hub) {
                eprintln!("GUI initialization error: {err}");
                std::process::exit(1);
            }
        }
        Some(TopCommand::Gui {
            module,
            threads,
            skip_hub,
        }) => {
            let mod_id = ModuleId::from_code(&module).unwrap_or(ModuleId::N1PrimeSieve);
            if let Err(err) = launch_gui(mod_id, threads.or(cli.threads), skip_hub || cli.skip_hub)
            {
                eprintln!("GUI initialization error: {err}");
                std::process::exit(1);
            }
        }
        Some(TopCommand::Cli { action }) => {
            if let Err(err) = run_cli_action(action) {
                eprintln!("CLI execution error: {err}");
                std::process::exit(1);
            }
        }
    }
}
