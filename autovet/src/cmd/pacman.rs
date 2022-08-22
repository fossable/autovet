use crate::cmd::Commands;
use std::{
	error::Error,
	io::{Read, Write},
	process::{Command, Stdio},
};

pub fn run(cmd: Commands) -> Result<(), Box<dyn Error>> {
	match cmd {
		Commands::Pacman { arguments } => {
			let mut pacman = Command::new("pacman")
				.args(arguments)
				.stdout(Stdio::piped())
				.spawn()
				.expect("Failed to invoke pacman");

			// Read one byte at a time so the progress bars look normal
			if let Some(stdout) = &mut pacman.stdout {
				let mut buffer = [0u8; 1];
				loop {
					match stdout.read_exact(&mut buffer) {
						Ok(()) => match &buffer {
							b"P" => {}
							_ => std::io::stdout().write_all(&buffer).unwrap(),
						},
						Err(_) => break,
					}
				}
			}

			Ok(())
		}
		_ => panic!(),
	}
}
