//! Implements STRH (Store Register Halfword) instruction.

use super::Encoding::{self, T1, T2, T3};
use super::{
    indexing_args, other, undefined, unpredictable, AddOrSub, DecodeHelper, Instruction, Qualifier,
};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::arm::Effect;
use crate::qualifier_wide_match;
use crate::{
    arith::{shift_c, Shift},
    arm::{ArmProcessor, RunError},
    decoder::DecodeError,
    it_state::ItState,
    registers::RegisterIndex,
};
use core::panic;

/// STRH (immediate) instruction.
pub struct StrhImm {
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset from Rn.
    imm32: u32,
    /// True to store with indexing.
    index: bool,
    /// True to add offset, false to subtract.
    add: bool,
    /// True to write new offset value back to Rn.
    wback: bool,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for StrhImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "10000xxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110001010xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "111110000010xxxxxxxx1xxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                imm32: ins.imm5(6) << 1,
                index: true,
                add: true,
                wback: false,
                encoding,
            },
            T2 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                undefined(rt.is_pc())?;
                unpredictable(rt.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    imm32: ins.imm12(0),
                    index: true,
                    add: true,
                    wback: false,
                    encoding,
                }
            }
            T3 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                let (p, u, w) = ins.puw();
                other(p && u && !w)?; // STRHT
                undefined(rn.is_pc() || (!p && !w))?;
                unpredictable(rt.is_sp_or_pc() || (w && rt == rn))?;
                Self {
                    rt,
                    rn,
                    imm32: ins.imm8(0),
                    index: p,
                    add: u,
                    wback: w,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let rn = proc[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        proc.write_u16_unaligned(address, proc[self.rt] as u16)?;
        if self.wback {
            proc.set(self.rn, offset_addr)
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "strh".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, false, self.index, self.add, self.wback,)
        )
    }
}

/// STRH (register) instruction.
///
/// Store Register Halfword.
pub struct StrhReg {
    /// Source register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Rm shift amount.
    shift: u8,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for StrhReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0101001xxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110000010xxxxxxxx000000xxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: 0,
                encoding,
            },
            T2 => {
                let rn = ins.reg4(16);
                undefined(rn.is_pc())?;
                let rt = ins.reg4(12);
                let rm = ins.reg4(0);
                unpredictable(rt.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    rm,
                    shift: ins.imm2(4) as u8,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let carry_in = proc.registers.psr.c();
        let offset = shift_c(proc[self.rm], Shift::lsl(self.shift as u32), carry_in).0;
        let address = proc[self.rn].wrapping_add(offset);
        proc.write_u16_unaligned(address, proc[self.rt] as u16)?;
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "strh".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T2)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, [{}, {}{}]",
            self.rt,
            self.rn,
            self.rm,
            Shift::lsl(self.shift as u32).arg_string()
        )
    }
}
