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
use log::trace;
use std::default::Default;

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
}

impl RegisterState {
	pub fn set(&mut self, register: Register, value: u64) {
		trace!("Setting {:?} to {}", register, value);
		match register {
			Register::RAX | Register::EAX => self.rax = value,
			Register::RBX | Register::EBX => self.rbx = value,
			Register::RCX | Register::ECX => self.rcx = value,
			Register::RDX | Register::EDX => self.rdx = value,
			_ => todo!(),
		}
	}

	pub fn get(&self, register: Register) -> u64 {
		match register {
			Register::RAX => self.rax,
			Register::RBX => self.rbx,
			Register::RCX => self.rcx,
			Register::RDX => self.rdx,
			Register::RSI => self.rsi,
			Register::RDI => self.rdi,
			Register::RBP => self.rbp,
			Register::RSP => self.rsp,
			_ => todo!(),
		}
	}
}

pub fn waterfall(code: &Vec<u8>, rip: u64) {
	// Allocate a boolean for each instruction to track whether it has been visited
	let mut instructions: Vec<(Instruction, bool)> = Decoder::new(64, code, DecoderOptions::NONE)
		.iter()
		.map(|i| (i, false))
		.collect();

	let mut state = RegisterState::default();
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
		state.rip += 1;

		// Simulate the instruction
		match ins.mnemonic() {
			Mnemonic::Mov => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => {
					trace!("[{}] register mov", state.rip);
					state.set(ins.op_register(1), state.get(ins.op_register(0)));
				}
				(OpKind::Immediate64, OpKind::Register) => {
					trace!("[{}] immediate mov", state.rip);
					state.set(ins.op_register(1), ins.immediate(0))
				}
				_ => todo!(),
			},
			Mnemonic::And => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => state.set(
					ins.op_register(1),
					state.get(ins.op_register(0)) & state.get(ins.op_register(1)),
				),
				_ => todo!(),
			},
			Mnemonic::Or => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => state.set(
					ins.op_register(1),
					state.get(ins.op_register(0)) | state.get(ins.op_register(1)),
				),
				_ => todo!(),
			},
			Mnemonic::Xor => match (ins.op_kind(0), ins.op_kind(1)) {
				(OpKind::Register, OpKind::Register) => state.set(
					ins.op_register(1),
					state.get(ins.op_register(0)) ^ state.get(ins.op_register(1)),
				),
				_ => todo!(),
			},
			Mnemonic::Not => match (ins.op_kind(0)) {
				(OpKind::Register) => state.set(ins.op_register(0), !state.get(ins.op_register(0))),
				_ => todo!(),
			},
			// Take all jumps regardless of the condition
			Mnemonic::Jmp | Mnemonic::Je | Mnemonic::Jne | Mnemonic::Jle => {
				// Duplicate state before the jump
				let mut tmp_state = state.clone();
				// TODO set RIP
				emulate(&mut tmp_state, instructions, syscalls);
			}
			Mnemonic::Syscall => {
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
		a.mov(rax, 0x100u64)?;

		let mut state = RegisterState::default();
		state.rax = 0x1337;
		state.rbx = 0x890;

		test_emulate(a, &mut state);

		assert_eq!(state.rax, 0x100);
		assert_eq!(state.rbx, 0x1337);
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
