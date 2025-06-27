//! Implements MRS (Move to Register from Special) instruction.

use super::Encoding::{self, T1};
use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    instructions::{unpredictable, DecodeHelper, ItState},
    registers::RegisterIndex,
};
use core::panic;

pub struct Mrs {
    /// Destination register.
    rd: RegisterIndex,
    /// Source special register.
    sysm: RegisterIndex,
}

impl Instruction for Mrs {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V6M, V7M, V7EM, V8M],
            expression: "11110011111(0)(1)(1)(1)(1)10(0)0xxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rd = ins.reg4(8);
        let sysm = ins & 0xff;
        let good_sysm = matches!(sysm, 0..=3 | 5..=9 | 16..=20);
        unpredictable(rd.is_sp_or_pc() || !good_sysm)?;
        Ok(Self {
            rd,
            sysm: RegisterIndex::new_sys(sysm),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let mut rd = 0;
        let sysm = self.sysm.index_sys();
        match sysm >> 3 {
            0b00000 => {
                if sysm & 1 == 1 {
                    rd |= proc.registers.psr.ipsr();
                }
                if sysm & 4 == 0 {
                    rd |= proc.registers.psr.apsr() & 0xf8000000;
                    // TODO DSP extension
                }
            }
            0b00001 => {
                if proc.is_privileged() {
                    match sysm & 7 {
                        0 => rd = proc.registers.msp,
                        1 => rd = proc.registers.psp,
                        _ => {}
                    }
                }
            }
            0b00010 => match sysm & 7 {
                0b000 => {
                    if proc.is_privileged() {
                        rd = proc.registers.primask.pm() as u32;
                    }
                }
                0b001 => todo!(),
                0b010 => todo!(),
                0b011 => todo!(),
                0b100 => rd = proc.registers.control.read() & 3,
                _ => {}
            },
            _ => panic!(),
        }
        proc.set(self.rd, rd);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "mrs".into()
    }

    fn args(&self, _pc: u32) -> String {
        format!("{}, {}", self.rd, self.sysm)
    }
}
