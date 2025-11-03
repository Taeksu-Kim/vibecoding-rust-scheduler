use clap::Parser;
use scheduler::cli::{execute_command, Cli};

fn main() {
    let cli = Cli::parse();

    if let Err(e) = execute_command(cli.command) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
