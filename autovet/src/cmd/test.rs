use crate::cmd::Commands;
use regex::Regex;
use std::error::Error;
use std::io::BufRead;
use std::io::Cursor;
use std::process::Command;
use std::process::Stdio;

struct Syscall {
    pub name: String,
    pub address: String,
    pub arguments: Vec<String>,
}

pub fn run(cmd: Commands) -> Result<(), Box<dyn Error>> {
    match cmd {
        Commands::Test { executable } => {
            // Try to automatically determine to what package channel the executable belongs
            // TODO

            // Run the test under strace
            let output = Command::new("strace")
                .arg("--follow-forks")
                .arg("--instruction-pointer")
                .stdout(Stdio::null())
                .output()
                .expect("Failed to run test");

            let mut syscalls: Vec<Syscall> = Vec::new();

            let re = Regex::new(r"\[(0-9a-f)+\] ([a-z0-9]+)\((.*)\) = ").unwrap();
            for line in Cursor::new(output.stderr).lines() {
                let line = line.unwrap();
                let groups = re.captures(&line).unwrap();

                if let (Some(address), Some(name), Some(args)) =
                    (groups.get(1), groups.get(2), groups.get(3))
                {
                    let mut arguments: Vec<String> = Vec::new();

                    // TODO
                }
            }
            Ok(())
        }
        _ => panic!(),
    }
}
