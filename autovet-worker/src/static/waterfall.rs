//! This module attempts to discover all reachable syscalls in a program by decoding
//! instructions and analyzing the execution flow.
//!
//! - When a conditional jump is encountered, the analyzer forks and follows both paths.
//! - If an instruction is visited twice in a particular thread, the thread completes to
//!   avoid infinite loops.
//!

use iced_x86::{Decoder, DecoderOptions, Instruction, Mnemonic, OpKind, Register};
use std::default::Default;

#[derive(Default)]
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
		match register {
			Register::RAX | Register::EAX => self.rax = value,
			Register::RBX | Register::EBX => self.rax = value,
			Register::RCX | Register::ECX => self.rax = value,
			Register::RDX | Register::EDX => self.rax = value,
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
	let mut instructions: Vec<(Instruction, bool)> = Decoder::new(64, code, DecoderOptions::NONE)
		.iter()
		.map(|i| (i, false))
		.collect();

	let mut state = RegisterState::default();

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
					state.set(ins.op_register(1), state.get(ins.op_register(0)))
				}
				(OpKind::Immediate64, OpKind::Register) => {
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
				// TODO set RIP
			}
			Mnemonic::Syscall => {
				// Determine what syscall this is
				/*let syscall = match SyscallType::from_repr(state.rax) {
					SyscallType::Close => Syscall{
						name: "close",
						address: instruction.ip(),
						arguments: vec![state.rdi],
					},
					_ => todo!(),
				};*/
			}
			_ => {}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use iced_x86::code_asm::*;

	fn run_instructions(a: CodeAssembler, state: &RegisterState) {
		let mut decoder = Decoder::new(64, a.assemble(0)?, DecoderOptions::NONE);
		let mut ins = Instruction::default();

		while decoder.can_decode() {
			decoder.decode_out(&mut ins);
			// TODO
		}
	}

	#[test]
	fn test_mov() {
		let mut a = CodeAssembler::new(64)?;
		a.mov(rax, rbx)?;

		let mut state = RegisterState::default();
		state.rax = 1337;
		state.rbx = 890;

		assert_eq!(state.rax, state.rbx);
	}
}
