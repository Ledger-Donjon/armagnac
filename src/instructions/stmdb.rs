//! Implements STMDB (Store Multiple Decrement Before) and STMFD (Store Multiple Full Descending)
//! instructions.

use super::Encoding::{self, T1, T2};
use super::Instruction;
use super::{
    ArmVersion::{V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{other, unpredictable, DecodeHelper, ItState},
    registers::{MainRegisterList, RegisterIndex},
};

/// STMDB instruction.
pub struct Stmdb {
    /// Base register.
    ///
    /// [RegisterIndex::Sp] in case of PUSH instruction.
    pub rn: RegisterIndex,
    /// Wether Rn is written back with a modified value.
    ///
    /// True in case of PUSH instruction.
    pub wback: bool,
    /// Registers to be pushed to the stack.
    pub registers: MainRegisterList,
}

impl Instruction for Stmdb {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "1110100100x0xxxx(0)x(0)xxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V8M],
                expression: "1011010xxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        match encoding {
            T1 => {
                let rn = ins.reg4(16);
                let registers = MainRegisterList::new((ins & 0x5fff) as u16);
                let wback = ins.bit(21);
                other(wback && rn.is_sp())?; // PUSH
                unpredictable(rn.is_pc() || registers.len() < 2)?;
                unpredictable(wback && registers.contains(&rn))?;
                Ok(Self {
                    rn,
                    wback,
                    registers,
                })
            }
            T2 => {
                let registers = MainRegisterList::new(((ins.imm1(8) << 14) | ins.imm8(0)) as u16);
                Ok(Self {
                    rn: RegisterIndex::Sp,
                    wback: true,
                    registers,
                })
            }
            _ => panic!(),
        }
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        // SP and PC cannot be pushed.
        // Instruction decoder should prevent this to happen
        debug_assert!(!self.registers.has_sp() && !self.registers.has_pc());
        let mut addr = proc[self.rn];
        for reg in self.registers.iter().rev() {
            addr = addr.wrapping_sub(4);
            proc.write_u32_aligned(addr, proc[reg])?;
        }
        if self.wback {
            proc.set(self.rn, addr);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "stmdb".into()
    }

    fn args(&self, _pc: u32) -> String {
        let wback = if self.wback { "!" } else { "" };
        format!("{}{}, {{{}}}", self.rn, wback, self.registers)
    }
}
