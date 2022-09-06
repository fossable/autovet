use crate::cmd::Commands;
use autovet_core::Syscall;
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::io::BufRead;
use std::io::Cursor;
use std::process::Command;
use std::process::Stdio;

lazy_static! {
	static ref STRACE_LINE: Regex = Regex::new(r"\[([0-9a-f]+)\] ([a-z0-9]+)\((.*)\) = ").unwrap();
}

/// Reduce the number of syscalls by eliminating duplicates and wildcarding similar calls.
fn reduce(syscalls: Vec<Syscall>) -> Vec<Syscall> {
	let mut reduced: Vec<Syscall> = Vec::new();

	for syscall in syscalls.into_iter() {
		match reduced.iter_mut().find(|s| s.name == syscall.name) {
			Some(s) => {
				for i in 0..s.arguments.len() {
					// TODO check for substring
					if s.arguments[i] != syscall.arguments[i] {
						s.arguments[i] = String::from("*");
					}
				}
			}
			None => {
				reduced.push(syscall);
			}
		}
	}

	return reduced;
}

fn syscalls_to_yaml(syscalls: Vec<Syscall>) {}

fn parse_syscall(line: &str) -> Option<Syscall> {
	if let Some(groups) = STRACE_LINE.captures(&line) {
		if let (Some(address), Some(name), Some(args)) =
			(groups.get(1), groups.get(2), groups.get(3))
		{
			let args = args.as_str();
			let mut arguments: Vec<String> = Vec::new();

			// Parse the syscall parameters primitively instead of with a regex because commas can be enclosed in quotes or curly braces which complicates things
			{
				let mut quote = false;
				let mut brace = false;
				let mut i = 0;
				let mut j = 0;

				for c in args.chars() {
					match c {
						',' => {
							if !quote && !brace {
								arguments.push(args[i..j].trim().to_string());
								i = j + 1;
							}
						}
						'"' => quote = !quote,
						'{' => {
							if !quote {
								brace = true;
							}
						}
						'}' => {
							if !quote {
								brace = false;
							}
						}
						_ => (),
					}
					j += 1;
				}

				// Add the last argument
				arguments.push(args[i..].trim().to_string());
			}

			return Some(Syscall {
				name: name.as_str().to_string(),
				address: u64::from_str_radix(address.as_str(), 16).unwrap(),
				arguments,
			});
		}
	}

	None
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_parse_syscall() {
		assert_eq!(parse_syscall("[00007fee0bbbbd1b] execve(\"/usr/bin/grep\", [\"grep\"], 0x7ffff9fde4a8 /* 35 vars */) = 0"), Some(Syscall{name: String::from("execve"), address: 0x00007fee0bbbbd1bu64, arguments: vec![String::from("\"/usr/bin/grep\""), String::from("[\"grep\"]"), String::from("0x7ffff9fde4a8 /* 35 vars */")]}));
		assert_eq!(parse_syscall("[00007f6ceda8951e] newfstatat(3, \"\", {st_mode=S_IFREG|0755, st_size=481072, ...}, AT_EMPTY_PATH) = 0"), Some(Syscall{name: String::from("newfstatat"), address: 0x00007f6ceda8951eu64, arguments: vec![String::from("3"), String::from("\"\""), String::from("{st_mode=S_IFREG|0755, st_size=481072, ...}"), String::from("AT_EMPTY_PATH")]}));
	}
}

pub fn run(cmd: Commands) -> Result<(), Box<dyn Error>> {
	match cmd {
		Commands::Test { executable } => {
			let theme = ColorfulTheme {
				values_style: Style::new().yellow().dim(),
				..ColorfulTheme::default()
			};

			// Try to automatically determine to what channel the package belongs
			// TODO

			loop {
				let args: String = Input::with_theme(&theme)
					.with_prompt("Enter program arguments (or CTRL-D to stop)")
					.interact()?;

				// Run the test under strace
				let output = Command::new("strace")
					.arg("--follow-forks")
					.arg("--instruction-pointer")
					.arg(&executable)
					.args(args.split(" ").into_iter())
					.stdout(Stdio::null())
					.output()
					.expect("Failed to invoke strace");

				let mut syscalls: Vec<Syscall> = Vec::new();

				for line in Cursor::new(output.stderr).lines() {
					if let Some(syscall) = parse_syscall(&line.unwrap()) {
						syscalls.push(syscall);
					}
				}

				println!("Captured {} syscalls", syscalls.len());

				let reduced = reduce(syscalls);

				println!("Reduced: {:?}", reduced);
			}

			Ok(())
		}
		_ => panic!(),
	}
}
