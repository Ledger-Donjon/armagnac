//! Implements LDRH (Load Register Halfword) instruction.

use super::Encoding::{self, T1, T2, T3};
use super::Qualifier;
use super::{ldr::LdrImm, other, undefined, AddOrSub, Instruction};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::instructions::indexing_args;
use crate::qualifier_wide_match;
use crate::{
    align::Align,
    arith::{shift_c, Shift},
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{unpredictable, DecodeHelper},
    registers::RegisterIndex,
};

/// LDRH (immediate) instruction.
///
/// Derived from [LdrImm] since it shares many properties.
pub struct LdrhImm(LdrImm);

impl Instruction for LdrhImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "10001xxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110001011xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "111110000011xxxxxxxx1xxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self(LdrImm {
                rn: ins.reg3(3),
                rt: ins.reg3(0),
                imm32: ins.imm5(6) << 1,
                index: true,
                add: true,
                wback: false,
                encoding,
            }),
            T2 => {
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                other(rt.is_pc())?; // Unallocated memory hints
                other(rn.is_pc())?; // LDRH (literal)
                unpredictable(rt.is_sp())?;
                Self(LdrImm {
                    rn,
                    rt,
                    imm32: ins.imm12(0),
                    index: true,
                    add: true,
                    wback: false,
                    encoding,
                })
            }
            T3 => {
                let rn = ins.reg4(16);
                let rt = ins.reg4(12);
                let (p, u, w) = ins.puw();
                other(rn.is_pc())?; // LDRH (literal)
                other(rt.is_pc() && p && !u && !w)?; // Unallocated memory hints
                other(p && u && !w)?; // LDRHT
                undefined(!p && !w)?;
                unpredictable(rt.is_sp_or_pc() || (w && rn == rt))?;
                Self(LdrImm {
                    rn,
                    rt,
                    imm32: ins.imm8(0),
                    index: p,
                    add: u,
                    wback: w,
                    encoding,
                })
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rn = proc[self.0.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.0.imm32, self.0.add);
        let addr = if self.0.index { offset_addr } else { rn };
        let data = proc.read_u16_unaligned(addr)?;
        if self.0.wback {
            proc.set(self.0.rn, offset_addr);
        }
        if self.0.rt.is_pc() {
            if addr & 3 == 0 {
                todo!();
            } else {
                return Err(RunError::InstructionUnpredictable);
            }
        } else {
            proc.set(self.0.rt, data as u32)
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrh".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.0.encoding, T2)
    }

    fn args(&self, pc: u32) -> String {
        self.0.args(pc)
    }
}

/// LDRH (literal) instruction.
pub struct LdrhLit {
    /// Destination register.
    rt: RegisterIndex,
    /// Label offset.
    imm32: u32,
    /// True to add offset, false to subtract.
    add: bool,
}

impl Instruction for LdrhLit {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11111000x0111111xxxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rt = ins.reg4(12);
        other(rt.is_pc())?; // Unallocated memory hints
        unpredictable(rt.is_sp())?;
        Ok(Self {
            rt,
            imm32: ins.imm12(0),
            add: ins.bit(23),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let base = proc.pc().align(4);
        let addr = base.wrapping_add_or_sub(self.imm32, self.add);
        let data = proc.read_u16_unaligned(addr)?;
        if self.rt.is_pc() {
            if addr & 3 == 0 {
                todo!();
            } else {
                return Err(RunError::InstructionUnpredictable);
            }
        } else {
            proc.set(self.rt, data as u32)
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrh".into()
    }

    fn qualifier(&self) -> Qualifier {
        // llvm-objdump add .w qualifier despite Arm Architecture Reference Manual doesn't.
        Qualifier::Wide
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(RegisterIndex::Pc, self.imm32, true, true, self.add, false)
        )
    }
}

/// LDRH (register) instruction.
pub struct LdrhReg {
    /// Destination register.
    rt: RegisterIndex,
    /// Base value register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Shift applied to Rm.
    shift: Shift,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for LdrhReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0101101xxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110000011xxxxxxxx000000xxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: Shift::lsl(0),
                encoding,
            },
            T2 => {
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                let rm = ins.reg4(0);
                other(rn.is_pc())?; // LDRH (literal)
                other(rt.is_pc())?; // Unallocated memory hints
                unpredictable(rt.is_sp() || rm.is_sp_or_pc())?;
                Self {
                    rt,
                    rn,
                    rm,
                    shift: Shift::lsl(ins.imm2(4)),
                    encoding,
                }
            }
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let carry_in = proc.registers.psr.c();
        let (offset, _) = shift_c(proc[self.rm], self.shift, carry_in);
        let address = proc[self.rn].wrapping_add(offset);
        let data = proc.read_u16_unaligned(address)?;
        proc.set(self.rt, data as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrh".into()
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
            self.shift.arg_string()
        )
    }
}
