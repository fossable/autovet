//! This module attempts to discover all reachable syscalls in a program by decoding
//! instructions and analyzing the execution flow.
//!
//! - When a conditional jump is encountered, the analyzer forks and follows both paths.
//! - If an instruction is visited twice in a particular thread, the thread completes to
//!   avoid infinite loops.
//!

use autovet_core::Syscall;
use autovet_core::SyscallType;
use iced_x86::{Decoder, DecoderOptions, Instruction, Mnemonic, OpKind, Register};
use log::{debug, trace};
use simple_error::bail;
use std::default::Default;
use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Read;
use std::os::unix::prelude::FileExt;
use std::process::Command;

#[derive(Default, Clone)]
struct RegisterState {
	pub rax: u64,
	pub rbx: u64,
	pub rcx: u64,
	pub rdx: u64,
	pub rsi: u64,
	pub rdi: u64,
	pub rbp: u64,
	pub rsp: u64,
	pub rip: usize,
	pub r8: u64,
	pub r9: u64,
	pub r10: u64,
	pub r11: u64,
	pub r12: u64,
	pub r13: u64,
	pub r14: u64,
	pub r15: u64,

	pub stack: Vec<u64>,
}

impl RegisterState {
	fn set(&mut self, register: Register, value: u64) {
		trace!("Setting {:?} to {}", register, value);
		match register {
			Register::RAX | Register::EAX => self.rax = value,
			Register::RBX | Register::EBX => self.rbx = value,
			Register::RCX | Register::ECX => self.rcx = value,
			Register::RDX | Register::EDX => self.rdx = value,
			Register::RSI | Register::ESI => self.rsi = value,
			Register::RDI | Register::EDI => self.rdi = value,
			Register::RBP | Register::EBP => self.rbp = value,
			Register::RSP | Register::ESP => self.rsp = value,
			_ => todo!(),
		}
	}

	fn get(&self, register: Register) -> u64 {
		match register {
			Register::RAX => self.rax,
			Register::RBX => self.rbx,
			Register::RCX => self.rcx,
			Register::RDX => self.rdx,
			Register::RSI => self.rsi,
			Register::RDI => self.rdi,
			Register::RBP => self.rbp,
			Register::RSP => self.rsp,
			Register::R8 => self.r8,
			Register::R9 => self.r9,
			Register::R10 => self.r10,
			Register::R11 => self.r11,
			Register::R12 => self.r12,
			Register::R13 => self.r13,
			Register::R14 => self.r14,
			Register::R15 => self.r15,
			_ => todo!(),
		}
	}
}

/// Extract the "entrypoint" field from readelf output.
fn parse_readelf_entrypoint(output: &str) -> Result<u64, Box<dyn Error>> {
	for line in Cursor::new(output).lines() {
		let line = line?;
		let line = line.trim();
		let fields: Vec<&str> = line.split_whitespace().collect();

		if fields.len() == 4
			&& fields[0] == "Entry"
			&& fields[1] == "point"
			&& fields[2] == "address:"
		{
			return Ok(u64::from_str_radix(fields[3].trim_start_matches("0x"), 16)?);
		}
	}
	bail!("No entry point found");
}

/// Extract the size and offset of the text section from objdump output.
fn parse_objdump_size_offset(output: &str) -> Result<(u64, u64), Box<dyn Error>> {
	for line in Cursor::new(output).lines() {
		let line = line?;
		let line = line.trim();
		let fields: Vec<&str> = line.split_whitespace().collect();
		if fields.len() == 7 && fields[1] == ".text" {
			return Ok((
				u64::from_str_radix(fields[2], 16)?,
				u64::from_str_radix(fields[5], 16)?,
			));
		}
	}
	bail!("No text segment found");
}

pub fn extract_syscalls(path: &str) -> Result<Vec<Syscall>, Box<dyn Error>> {
	// Get entrypoint with readelf
	let readelf = Command::new("readelf").arg("-h").arg(path).output()?;
	let entry_point = parse_readelf_entrypoint(std::str::from_utf8(&readelf.stdout)?)?;

	// Get code section size and offset with objdump
	let objdump = Command::new("objdump").arg("-h").arg(path).output()?;
	let (text_size, text_offset) =
		parse_objdump_size_offset(std::str::from_utf8(&objdump.stdout)?)?;

	trace!(
		".text size: {}, .text offset: {}, entrypoint: {}",
		text_size,
		text_offset,
		entry_point
	);

	// Read code section
	let mut file = File::open(path)?;
	//let mut code = vec![0u8; text_size as usize + text_offset as usize];
	//file.read_exact(&mut code)?;
	let mut code = vec![0u8; text_size as usize];
	file.read_exact_at(&mut code, text_offset)?;

	// Allocate a boolean for each instruction to track whether it has been visited
	let mut instructions: Vec<(Instruction, bool)> =
		Decoder::with_ip(64, &code, entry_point, DecoderOptions::NONE)
			.iter()
			.map(|i| (i, false))
			.collect();

	let mut state = RegisterState::default();
	state.rip = entry_point as usize - text_offset as usize;

	let mut syscalls: Vec<Syscall> = Vec::new();
	emulate(&mut state, &mut instructions, &mut syscalls);

	println!("Extracted {} syscalls", syscalls.len());
	return Ok(syscalls);
}

/// Emulate the given instructions with the given register state and collect syscalls encountered.
fn emulate(
	state: &mut RegisterState,
	instructions: &mut Vec<(Instruction, bool)>,
	syscalls: &mut Vec<Syscall>,
) {
	while state.rip < instructions.len() {
		// If we already visited this instruction, then we should stop to prevent infinite loops
		if instructions[state.rip].1 {
			break;
		} else {
			instructions[state.rip].1 = true;
		}

		// Get the next decoded instruction
		let ins = instructions[state.rip].0;
		state.rip = ins.next_ip() as usize;

		trace!("Executing instruction: {:?}", ins);

		// Simulate the instruction
		match ins.mnemonic() {
			Mnemonic::Mov => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => {
					state.set(ins.op_register(0), state.get(ins.op_register(1)));
				}
				(OpKind::Register, OpKind::Immediate64) => {
					state.set(ins.op_register(0), ins.immediate(1))
				}
				_ => todo!(),
			},
			Mnemonic::Add => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => {
					state.set(
						ins.op_register(0),
						state.get(ins.op_register(0)) + state.get(ins.op_register(1)),
					);
				}
				(OpKind::Register, OpKind::Immediate64) => state.set(
					ins.op_register(0),
					state.get(ins.op_register(0)) + ins.immediate(1),
				),
				_ => todo!(),
			},
			Mnemonic::Sub => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => {
					state.set(
						ins.op_register(0),
						state.get(ins.op_register(0)) - state.get(ins.op_register(1)),
					);
				}
				(OpKind::Register, OpKind::Immediate64) => state.set(
					ins.op_register(0),
					state.get(ins.op_register(0)) - ins.immediate(1),
				),
				_ => todo!(),
			},
			Mnemonic::And => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => state.set(
					ins.op_register(0),
					state.get(ins.op_register(1)) & state.get(ins.op_register(0)),
				),
				_ => todo!(),
			},
			Mnemonic::Or => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => state.set(
					ins.op_register(0),
					state.get(ins.op_register(1)) | state.get(ins.op_register(0)),
				),
				_ => todo!(),
			},
			Mnemonic::Xor => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => state.set(
					ins.op_register(0),
					state.get(ins.op_register(1)) ^ state.get(ins.op_register(0)),
				),
				_ => todo!(),
			},
			Mnemonic::Not => match ins.op_kind(0) {
				OpKind::Register => state.set(ins.op_register(0), !state.get(ins.op_register(0))),
				_ => todo!(),
			},
			Mnemonic::Jmp
			| Mnemonic::Je
			| Mnemonic::Jl
			| Mnemonic::Jg
			| Mnemonic::Jne
			| Mnemonic::Jle
			| Mnemonic::Jge
			| Mnemonic::Ja
			| Mnemonic::Jae
			| Mnemonic::Jb
			| Mnemonic::Jbe
			| Mnemonic::Js
			| Mnemonic::Jns => {
				// TODO disallow jumps into data

				// Take all jumps regardless of the condition
				let mut tmp_state = state.clone();
				// TODO set RIP
				emulate(&mut tmp_state, instructions, syscalls);
			}
			Mnemonic::Push => match ins.op_kind(0) {
				OpKind::Register => state.stack.push(state.get(ins.op_register(0))),
				_ => todo!(),
			},
			Mnemonic::Pop => match ins.op_kind(0) {
				OpKind::Register => {
					let value = state.stack.pop().unwrap();
					state.set(ins.op_register(0), value);
				}
				_ => todo!(),
			},
			Mnemonic::Test => {
				// Just sets condition codes, so we can ignore
			}
			Mnemonic::Nop => {
				// Do nothing
			}
			Mnemonic::Call => {
				// Push return address
				// TODO
				// Jump to a function
				// TODO
			}
			Mnemonic::Syscall => {
				trace!("Discovered syscall: {}", state.rax);

				// Determine what syscall this is
				syscalls.push(match SyscallType::from_repr(state.rax) {
					Some(SyscallType::Close) => Syscall {
						name: String::from("close"),
						address: ins.ip(),
						arguments: vec![],
					},
					_ => todo!(),
				});
			}
			_ => trace!("[{}] unknown instruction", state.rip),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use iced_x86::code_asm::*;
	use std::error::Error;

	fn test_emulate(mut a: CodeAssembler, state: &mut RegisterState) -> Vec<Syscall> {
		let bytes = a.assemble(0).unwrap();
		let mut instructions: Vec<(Instruction, bool)> =
			Decoder::new(64, &bytes, DecoderOptions::NONE)
				.iter()
				.map(|i| (i, false))
				.collect();

		// Ignore syscalls
		let mut syscalls: Vec<Syscall> = Vec::new();

		super::emulate(state, &mut instructions, &mut syscalls);
		return syscalls;
	}

	#[test]
	fn mov() -> Result<(), Box<dyn Error>> {
		let mut a = CodeAssembler::new(64)?;
		a.mov(rax, rbx)?;
		a.mov(rbx, 0x100u64)?;

		let mut state = RegisterState::default();
		state.rax = 0x1337;
		state.rbx = 0x890;

		test_emulate(a, &mut state);

		assert_eq!(state.rbx, 0x100);
		assert_eq!(state.rax, 0x890);
		Ok(())
	}

	#[test]
	fn and() -> Result<(), Box<dyn Error>> {
		let mut a = CodeAssembler::new(64)?;
		a.and(rax, rbx)?;

		let mut state = RegisterState::default();
		state.rax = 0x1337;
		state.rbx = 0x890;

		test_emulate(a, &mut state);

		assert_eq!(state.rax, 0x890 & 0x1337);
		assert_eq!(state.rbx, 0x890);
		Ok(())
	}

	#[test]
	fn syscall_close() -> Result<(), Box<dyn Error>> {
		let mut a = CodeAssembler::new(64)?;
		a.syscall()?;

		let mut state = RegisterState::default();
		state.rax = 3;

		let syscalls = test_emulate(a, &mut state);

		assert_eq!("close", syscalls.first().unwrap().name);
		Ok(())
	}
}
