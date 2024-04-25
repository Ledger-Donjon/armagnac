//! Implements LDRB (immediate), LDRB (literal) and LDRB (register) instructions.

use crate::{
    arm::{Arm7Processor, RunError},
    decoder::DecodeError,
    it_state::ItState,
};

use super::{ldr::LdrImm, other, undefined, unpredictable, AddOrSub, DecodeHelper, Instruction};

/// LDRB (immediate) instruction.
pub struct LdrbImm(LdrImm);

impl Instruction for LdrbImm {
    fn patterns() -> &'static [&'static str] {
        &[
            "01111xxxxxxxxxxx",
            "111110001001xxxxxxxxxxxxxxxxxxxx",
            "111110000001xxxxxxxx1xxxxxxxxxxx",
        ]
    }

    fn try_decode(tn: usize, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        Ok(match tn {
            1 => Self(LdrImm {
                rn: ins.reg3(3),
                rt: ins.reg3(0),
                imm32: ins.imm5(6),
                index: true,
                add: true,
                wback: false,
            }),
            2 => {
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
                })
            }
            3 => {
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
                    wback: w
                })
            },
            _ => panic!(),
        })
    }

    fn execute(&self, proc: &mut Arm7Processor) -> Result<bool, RunError> {
        let rn = proc.registers[self.0.rn].val();
        let offset_addr = rn.wrapping_add_or_sub(self.0.imm32, self.0.add);
        let addr = if self.0.index { offset_addr } else { rn };
        let data = proc.u8_at(addr)?;
        proc.registers[self.0.rt].set_val(data as u32);
        if self.0.wback {
            proc.registers[self.0.rn].set_val(offset_addr);
        }
        Ok(false)
    }

    fn name(&self) -> String {
        "ldrb".into()
    }

    fn args(&self, pc: u32) -> String {
        self.0.args(pc)
    }
}
