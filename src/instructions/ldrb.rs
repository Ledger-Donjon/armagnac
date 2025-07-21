//! Implements LDRB (Load Register Byte) instruction.

use super::Encoding::{self, T1, T2, T3};
use super::Qualifier;
use super::{ldr::LdrImm, other, undefined, unpredictable, AddOrSub, DecodeHelper, Instruction};
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
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    registers::RegisterIndex,
};
use core::panic;

/// LDRB (immediate) instruction.
///
/// Load Register Byte (immediate).
pub struct LdrbImm(LdrImm);

impl Instruction for LdrbImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "01111xxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110001001xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T3,
                versions: &[V7M, V7EM, V8M],
                expression: "111110000001xxxxxxxx1xxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self(LdrImm {
                rn: ins.reg3(3),
                rt: ins.reg3(0),
                imm32: ins.imm5(6),
                index: true,
                add: true,
                wback: false,
                encoding,
            }),
            T2 => {
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                other(rt.is_pc())?; // PLD
                other(rn.is_pc())?; // LDRB (literal)
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
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                let (p, u, w) = ins.puw();
                other(rt.is_pc() && p)?; // PLD
                other(rn.is_pc())?; // LDRB (literal)
                other(p && u && !w)?; // LDRBT
                undefined(!p && !w)?;
                unpredictable(rt.is_sp_or_pc() || (w && rt == rn))?;
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
        let data = proc.read_u8(addr)?;
        proc.set(self.0.rt, data as u32);
        if self.0.wback {
            proc.set(self.0.rn, offset_addr);
        }
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrb".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.0.encoding, T2)
    }

    fn args(&self, pc: u32) -> String {
        self.0.args(pc)
    }
}

/// LDRB (register) instruction.
///
/// Load Register Byte (register).
pub struct LdrbReg {
    /// Destination register.
    rt: RegisterIndex,
    /// Base register.
    rn: RegisterIndex,
    /// Offset register.
    rm: RegisterIndex,
    /// Shift to be applied to Rm.
    shift: Shift,
    /// Encoding.
    encoding: Encoding,
}

impl Instruction for LdrbReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0101110xxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110000001xxxxxxxx000000xxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => Self {
                rt: ins.reg3(0),
                rn: ins.reg3(3),
                rm: ins.reg3(6),
                shift: Shift::lsl(0),
                encoding,
            },
            T2 => {
                let rm = ins.reg4(0);
                let rt = ins.reg4(12);
                let rn = ins.reg4(16);
                other(rt.is_pc())?; // PLD
                other(rn.is_pc())?; // LDR (literal)
                unpredictable(rm.is_sp_or_pc())?;
                unpredictable(rt.is_pc() && state.in_it_block_not_last())?;
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
        // From the specification, INDEX is always true, ADD is always true and WBACK always false,
        // so the implementation has been simplified.
        let (offset, _) = shift_c(proc[self.rm], self.shift, proc.registers.psr.c());
        let addr = proc[self.rn].wrapping_add(offset);
        let data = proc.read_u8(addr)?;
        proc.set(self.rt, data as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrb".into()
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

/// LDRB (literal) instruction.
///
/// Load Register Byte (literal).
pub struct LdrbLit {
    /// Destination register.
    rt: RegisterIndex,
    /// Label offset.
    imm32: u32,
    /// True to add offset, false to subtract.
    add: bool,
}

impl Instruction for LdrbLit {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11111000x0011111xxxxxxxxxxxxxxxx",
        }]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert_eq!(encoding, T1);
        let rt = ins.reg4(12);
        other(rt.is_pc())?; // PLD
        unpredictable(rt.is_sp())?;
        Ok(Self {
            rt,
            imm32: ins.imm12(0),
            add: ins.bit(23),
        })
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let base = proc.pc().align(4);
        let address = base.wrapping_add_or_sub(self.imm32, self.add);
        let data = proc.read_u8(address)?;
        proc.set(self.rt, data as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrb".into()
    }

    fn qualifier(&self) -> Qualifier {
        // llvm-objdump add .w qualifier despite Arm Architecture Reference Manual doesn't.
        Qualifier::Wide
    }

    fn args(&self, _pc: u32) -> String {
        //let address = pc.wrapping_add(4).align(4).wrapping_add(self.imm32);
        format!(
            "{}, {}",
            self.rt,
            indexing_args(RegisterIndex::Pc, self.imm32, true, true, self.add, false)
        )
    }
}
