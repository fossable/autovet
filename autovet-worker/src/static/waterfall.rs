//! This module attempts to discover all reachable syscalls in a program by decoding
//! instructions and analyzing the execution flow.
//! 
//! - When a conditional jump is encountered, the analyzer forks and follows both paths.
//! - If an instruction is visited twice in a particular thread, the thread completes to
//!   avoid infinite loops.
//! 
use autovet_core::{SyscallType, Syscall};
use iced_x86::{Decoder, DecoderOptions, Instruction, Mnemonic};
use std::default::Default;

#[derive(Default)]
struct RegisterState {
    pub rax: u64,
    pub rdi: u64,
    pub rsi: u64,
}

pub fn waterfall(code: &Vec<u8>, rip: u64) {
    let mut decoder =
        Decoder::with_ip(64, code, rip, DecoderOptions::NONE);
 
    let mut state = RegisterState::default();
    let mut instruction = Instruction::default();

    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);

        match instruction.mnemonic() {
            Mnemonic::Mov => {

            },
            Mnemonic::Syscall => {
                // Determine what syscall this is
                let syscall = match SyscallType::from_repr(state.rax) {
                    SyscallType::Close => Syscall{
                        name: "close",
                        address: instruction.ip(),
                        arguments: vec![state.rdi],
                    },
                    _ => todo!(),
                };
            },
            _ => {},
        }
    }
}