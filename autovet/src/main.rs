use crate::cmd::Commands;
use clap::Parser;
use std::error::Error;

pub mod cmd;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CommandLine {
    #[clap(subcommand)]
    command: Commands,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let command_line = CommandLine::parse();
    env_logger::init_from_env(env_logger::Env::new());

    // Dispatch command
    match &command_line.command {
        Commands::Test { .. } => crate::cmd::test::run(command_line.command),
        Commands::Pacman { .. } => crate::cmd::pacman::run(command_line.command),
        _ => Ok(()),
    }
}
