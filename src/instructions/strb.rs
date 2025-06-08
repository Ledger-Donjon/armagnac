//! Implements STRB (Store Byte) instruction.

use super::Encoding::{self, T1, T2, T3};
use super::{
    indexing_args, other, undefined, unpredictable, AddOrSub, DecodeHelper, Instruction, Qualifier,
};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
use crate::{
    arith::{shift_c, Shift},
    core::{ArmProcessor, Effect, RunError},
    decoder::DecodeError,
    instructions::ItState,
    registers::RegisterIndex,
};

/// STRB (immediate) instruction.
pub struct StrbImm {
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

impl Instruction for StrbImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "01110xxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110001000xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "111110000000xxxxxxxx1xxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                imm32: (ins >> 6) & 0x1f,
                index: true,
                add: true,
                wback: false,
                encoding,
            },
            T2 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                undefined(rn.is_pc())?;
                unpredictable(rt.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    imm32: ins & 0xfff,
                    index: true,
                    add: true,
                    wback: false,
                    encoding,
                }
            }
            T3 => {
                let puw = (ins >> 8) & 7;
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                let wback = puw & 1 != 0;
                other(puw == 6)?; // STRBT
                undefined(rn.is_pc() || puw & 5 == 0)?;
                unpredictable(rt.is_sp_or_pc() || (wback && rn == rt))?;
                Self {
                    rt,
                    rn,
                    imm32: ins & 0xff,
                    index: puw & 4 != 0,
                    add: puw & 2 != 0,
                    wback,
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
        proc.write_u8(address, (proc[self.rt] & 0xff) as u8)?;
        if self.wback {
            proc.set(self.rn, offset_addr)
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "strb".into()
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

/// STRB (register) instruction.
pub struct StrbReg {
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

impl Instruction for StrbReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0101010xxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110000000xxxxxxxx000000xxxxxx",
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
                let rt = ins.reg4(12);
                let rm = ins.reg4(0);
                undefined(rn.is_pc())?;
                unpredictable(rt.is_sp_or_pc() || rm.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    rm,
                    shift: ((ins >> 4) & 3) as u8,
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let carry_in = proc.registers.psr.c();
        let shift = Shift::lsl(self.shift as u32);
        let (offset, _) = shift_c(proc[self.rm], shift, carry_in);
        let address = proc[self.rn].wrapping_add(offset);
        proc.write_u8(address, (proc[self.rt] & 0xff) as u8)?;
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "strb".into()
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
