//! Implements LDRSH (Load Register Signed Halfword) instruction.

use super::Encoding::{self, T1, T2};
use super::{other, undefined, unpredictable, AddOrSub, DecodeHelper, Instruction, Qualifier};
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::qualifier_wide_match;
use crate::{
    align::Align,
    arith::{shift_c, Shift},
    core::ItState,
    core::{Processor, Effect, RunError},
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::indexing_args,
    registers::RegisterIndex,
};

/// LDRSH (immediate) instruction.
///
/// Load Register Signed Halfword.
pub struct LdrshImm {
    /// Destination register.
    pub rt: RegisterIndex,
    /// Base register.
    pub rn: RegisterIndex,
    /// Offset from Rn.
    pub imm32: u32,
    /// True to load with indexing.
    pub index: bool,
    /// True to add offset, false to subtract.
    pub add: bool,
    /// True to write new offset value back to Rn.
    pub wback: bool,
    /// Encoding.
    pub encoding: Encoding,
}

impl Instruction for LdrshImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "111110011011xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110010011xxxxxxxx1xxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match encoding {
            T1 => {
                let rn = ins.reg4(16);
                other(rn.is_pc())?; // LDRSH (literal)
                let rt = ins.reg4(12);
                other(rt.is_pc())?; // Unallocated memory hints
                unpredictable(rt.is_sp())?;
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
            T2 => {
                let rn = ins.reg4(16);
                other(rn.is_pc())?; // LDRSH (literal)
                let rt = ins.reg4(12);
                let (p, u, w) = ins.puw();
                other(rt.is_pc() && p & !u && !w)?; // Unallocated memory hints
                other(p && u && !w)?; // LDRSHT
                undefined(!p && !w)?;
                unpredictable(rt.is_sp() || (w && rn == rt))?;
                unpredictable(rt.is_pc() && (!p || u || w))?;
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

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        let rn = proc[self.rn];
        let offset_addr = rn.wrapping_add_or_sub(self.imm32, self.add);
        let address = if self.index { offset_addr } else { rn };
        let data = proc.read_u16_unaligned(address)?;
        if self.wback {
            proc.set(self.rn, offset_addr);
        }
        proc.set(self.rt, ((data as i16) as i32) as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrsh".into()
    }

    fn qualifier(&self) -> Qualifier {
        qualifier_wide_match!(self.encoding, T1)
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(self.rn, self.imm32, false, self.index, self.add, self.wback)
        )
    }
}

/// LDRSH (literal) instruction.
///
/// Load Register Signed Halfword.
pub struct LdrshLit {
    /// Destination register.
    rt: RegisterIndex,
    /// Label offset.
    imm32: u32,
    /// True to add offset, false to subtract.
    add: bool,
}

impl Instruction for LdrshLit {
    fn patterns() -> &'static [Pattern] {
        &[Pattern {
            encoding: T1,
            versions: &[V7M, V7EM, V8M],
            expression: "11111001x0111111xxxxxxxxxxxxxxxx",
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
        let address = base.wrapping_add_or_sub(self.imm32, self.add);
        let data = proc.read_u16_unaligned(address)?;
        proc.set(self.rt, ((data as i16) as i32) as u32);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrsh".into()
    }

    fn qualifier(&self) -> Qualifier {
        // llvm-objdump add .w qualifier despite Arm Architecture Reference Manual doesn't.
        Qualifier::Wide
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "{}, {}",
            self.rt,
            indexing_args(RegisterIndex::Pc, self.imm32, false, true, self.add, false)
        )
    }
}

/// LDRSH (register) instruction.
///
/// Load Register Signed Halfword.
pub struct LdrshReg {
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

impl Instruction for LdrshReg {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "0101111xxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111110010011xxxxxxxx000000xxxxxx",
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
                let rn = ins.reg4(16);
                other(rn.is_pc())?; // LDRSH (literal)
                let rt = ins.reg4(12);
                other(rt.is_pc())?; // Unallocated memory hints
                let rm = ins.reg4(0);
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
        // From the specification, INDEX is always true, ADD is always true and WBACK always false,
        // so the implementation has been simplified.
        let (offset, _) = shift_c(proc[self.rm], self.shift, proc.registers.psr.c());
        let rn = proc[self.rn];
        let address = rn.wrapping_add(offset);
        let data = ((proc.read_u16_unaligned(address)? as i16) as i32) as u32;
        proc.set(self.rt, data);
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "ldrsh".into()
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

#[cfg(test)]
mod tests {
    use super::{LdrshImm, LdrshLit, LdrshReg};
    use crate::{
        arith::Shift,
        core::{Processor, Config},
        instructions::{Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };

    #[test]
    fn test_ldrsh_imm() {
        let mut proc = Processor::new(Config::v8m());
        proc.map_ram(0x1000, 4).unwrap();
        proc.write_u32le_iface(0x1000, 0x9234d678).unwrap();

        proc.registers.r1 = 0x1000 - 16;
        let mut ins = LdrshImm {
            rn: RegisterIndex::R1,
            rt: RegisterIndex::R0,
            imm32: 16,
            index: true,
            add: true,
            wback: true,
            encoding: DontCare,
        };
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffd678);
        assert_eq!(proc.registers.r1, 0x1000);

        proc.registers.r0 = 0;
        proc.registers.r1 = 0x1000 + 16;
        ins.add = false;
        ins.wback = false;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffd678);
        assert_eq!(proc.registers.r1, 0x1000 + 16);

        proc.registers.r0 = 0;
        proc.registers.r1 = 0x1000;
        ins.index = false;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffd678);
    }

    #[test]
    fn test_ldrsh_lit() {
        let mut proc = Processor::new(Config::v8m());
        proc.map_ram(0x1000, 4).unwrap();
        proc.write_u32le_iface(0x1000, 0x9234d678).unwrap();

        let ins = LdrshLit {
            rt: RegisterIndex::R0,
            imm32: 128,
            add: true,
        };

        proc.set_pc(0x1000 - 128);
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffd678);
    }

    #[test]
    fn test_ldrsh_reg() {
        let mut proc = Processor::new(Config::v8m());
        proc.map_ram(0x1000, 4).unwrap();
        proc.write_u32le_iface(0x1000, 0x9234d678).unwrap();

        let mut ins = LdrshReg {
            rt: RegisterIndex::R0,
            rn: RegisterIndex::R1,
            rm: RegisterIndex::R2,
            shift: Shift::lsl(0),
            encoding: DontCare,
        };

        proc.registers.r1 = 0x1000;
        proc.registers.r2 = 0;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffffd678);
        proc.registers.r2 = 2;
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffff9234);
        proc.registers.r2 = 1;
        ins.shift = Shift::lsl(1);
        ins.execute(&mut proc).unwrap();
        assert_eq!(proc.registers.r0, 0xffff9234);
    }
}
