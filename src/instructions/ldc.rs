//! Implements LDC and LDC2 (Load Coprocessor) instructions.

use super::{
    other, undefined, AddOrSub, DecodeHelper,
    Encoding::{self, T1, T2},
    Instruction, Pattern,
};
use crate::{
    align::Align,
    core::ItState,
    core::{
        ArmProcessor,
        ArmVersion::{V7EM, V7M, V8M},
        Effect, RunError,
    },
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{indexing_args, unpredictable},
    registers::RegisterIndex,
};
use core::panic;

/// LDC or LDC2 (immediate) instruction.
///
/// Load Coprocessor.
pub struct LdcImm {
    /// Coprocessor index, from 0 to 15.
    coproc: u8,
    /// Coprocessor destination register.
    crd: u8,
    /// Base register.
    rn: RegisterIndex,
    /// D field.
    d: bool,
    /// True to load with indexing.
    index: bool,
    /// True to add offset, false to subtract.
    add: bool,
    /// True to write new offset value back to Rn.
    wback: bool,
    /// Immediate offset applied to Rn.
    imm32: u32,
    /// Encoding.
    encoding: Encoding,
    /// Raw encoding.
    /// Required since transmitted directly to the coprocessor.
    ins: u32,
}

impl Instruction for LdcImm {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "1110110xxxx1xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "1111110xxxx1xxxxxxxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        let rn = ins.reg4(16);
        other(rn.is_pc())?; // LDC (literal)
        let index = ins.bit(24);
        let add = ins.bit(23);
        let d = ins.bit(22);
        let wback = ins.bit(21);
        undefined(!index && !add && !d && !wback)?;
        other(!index && !add && d && !wback)?; // MRRC, MRRC2
        Ok(Self {
            coproc: ins.imm4(8) as u8,
            crd: ins.imm4(12) as u8,
            rn,
            d,
            index,
            add,
            wback,
            imm32: ins.imm8(0) << 2,
            encoding,
            ins,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let Some(coprocessor) = proc.coproc_accepted(self.coproc, self.ins) else {
            proc.generate_coprocessor_exception();
            return Ok(Effect::None);
        };

        let rn = proc[self.rn];
        let offset_addr = rn.add_or_sub(self.imm32, self.add);
        let mut address = if self.index { offset_addr } else { rn };

        loop {
            coprocessor
                .borrow_mut()
                .send_loaded_word(proc.read_u32_aligned(address)?, self.ins);
            address = address.wrapping_add(4);
            if coprocessor.borrow().done_loading(self.ins) {
                return Ok(Effect::None);
            }
            if self.wback {
                proc.set(self.rn, offset_addr);
            }
        }
    }

    fn name(&self) -> String {
        match (self.encoding, self.d) {
            (T1, true) => "ldcl",
            (T1, false) => "ldc",
            (T2, true) => "ldc2l",
            (T2, false) => "ldc2",
            _ => panic!(),
        }
        .into()
    }

    fn args(&self, _pc: u32) -> String {
        let last_arg = if !self.index && !self.wback && self.add {
            format!("[{}], {{{}}}", self.rn, self.imm32 >> 2)
        } else {
            indexing_args(self.rn, self.imm32, false, self.index, self.add, self.wback)
        };
        format!("p{}, c{}, {}", self.coproc, self.crd, last_arg)
    }
}

/// LDC or LDC2 (literal) instruction.
///
/// Load Coprocessor.
pub struct LdcLit {
    /// Coprocessor index, from 0 to 15.
    coproc: u8,
    /// Coprocessor destination register.
    crd: u8,
    /// D field.
    d: bool,
    /// True to load with indexing.
    index: bool,
    /// True to add offset, false to subtract.
    add: bool,
    /// Immediate offset applied to PC.
    imm32: u32,
    /// Encoding.
    encoding: Encoding,
    /// Raw encoding.
    /// Required since transmitted directly to the coprocessor.
    ins: u32,
}

impl Instruction for LdcLit {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "1110110xxxx11111xxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "1111110xxxx11111xxxxxxxxxxxxxxxx",
            },
        ]
    }

    fn try_decode(encoding: Encoding, ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!((encoding == T1) || (encoding == T2));
        let index = ins.bit(24);
        let add = ins.bit(23);
        let d = ins.bit(22);
        let wback = ins.bit(21);
        undefined(!index && !add && !d && !wback)?;
        other(!index && !add && d && !wback)?;
        unpredictable(wback || !index)?;
        Ok(Self {
            coproc: ins.imm4(8) as u8,
            crd: ins.imm4(12) as u8,
            d,
            index,
            add,
            imm32: ins.imm8(0) << 2,
            encoding,
            ins,
        })
    }

    fn execute(&self, proc: &mut ArmProcessor) -> Result<Effect, RunError> {
        let Some(coprocessor) = proc.coproc_accepted(self.coproc, self.ins) else {
            proc.generate_coprocessor_exception();
            return Ok(Effect::None);
        };

        let pc = proc.pc().align(4);
        let offset_addr = pc.add_or_sub(self.imm32, self.add);
        let mut address = if self.index { offset_addr } else { pc };

        loop {
            coprocessor
                .borrow_mut()
                .send_loaded_word(proc.read_u32_aligned(address)?, self.ins);
            address = address.wrapping_add(4);
            if coprocessor.borrow().done_loading(self.ins) {
                return Ok(Effect::None);
            }
        }
    }

    fn name(&self) -> String {
        match (self.encoding, self.d) {
            (T1, true) => "ldcl",
            (T1, false) => "ldc",
            (T2, true) => "ldc2l",
            (T2, false) => "ldc2",
            _ => panic!(),
        }
        .into()
    }

    fn args(&self, _pc: u32) -> String {
        format!(
            "p{}, c{}, {}",
            self.coproc,
            self.crd,
            indexing_args(
                RegisterIndex::Pc,
                self.imm32,
                false,
                self.index,
                self.add,
                false
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{LdcImm, LdcLit};
    use crate::{
        core::{ArmProcessor, Config, Coprocessor},
        instructions::{Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };
    use rand::Rng;
    use std::{cell::RefCell, rc::Rc};

    struct TestCoproc {
        data: Vec<u32>,
    }

    impl Coprocessor for TestCoproc {
        fn accepted(&self, ins: u32) -> bool {
            assert_eq!(ins, 0xaabbccdd);
            true
        }

        fn done_loading(&self, _ins: u32) -> bool {
            self.data.len() == 4
        }

        fn done_storing(&self, _ins: u32) -> bool {
            unimplemented!()
        }

        fn get_one_word(&mut self, _ins: u32) -> u32 {
            unimplemented!()
        }

        fn get_two_words(&mut self, _ins: u32) -> (u32, u32) {
            unimplemented!()
        }

        fn get_word_to_store(&mut self, _ins: u32) -> u32 {
            unimplemented!()
        }

        fn internal_operation(&mut self, _ins: u32) {
            unimplemented!()
        }

        fn send_loaded_word(&mut self, word: u32, ins: u32) {
            assert_eq!(ins, 0xaabbccdd);
            self.data.push(word);
        }

        fn send_one_word(&mut self, _word: u32, _ins: u32) {
            unimplemented!()
        }

        fn send_two_words(&mut self, _word1: u32, _word2: u32, _ins: u32) {
            unimplemented!()
        }
    }

    #[test]
    fn test_ldc_imm() {
        let mut proc = ArmProcessor::new(Config::v7m());
        let mut rng = rand::rng();
        let cp: u8 = rng.random_range(0..16);
        let coprocessor = Rc::new(RefCell::new(TestCoproc { data: Vec::new() }));
        let rn = RegisterIndex::new_general_random();
        proc.set_coprocessor(cp as usize, coprocessor.clone());
        proc.map(
            0x1000,
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        )
        .unwrap();
        proc.set(rn, 0x1000 - 100);
        let mut expected = proc.registers.clone();
        expected.set(rn, 0x1000);
        LdcImm {
            coproc: cp,
            crd: 0,
            rn,
            d: false,
            index: true,
            add: true,
            wback: true,
            imm32: 100,
            encoding: DontCare,
            ins: 0xaabbccdd,
        }
        .execute(&mut proc)
        .unwrap();
        assert_eq!(proc.registers, expected);
        assert_eq!(
            &coprocessor.borrow().data,
            &[0x03020100, 0x07060504, 0x0b0a0908, 0x0f0e0d0c]
        );
    }

    #[test]
    fn test_ldc_lit() {
        let mut proc = ArmProcessor::new(Config::v7m());
        let mut rng = rand::rng();
        let cp: u8 = rng.random_range(0..16);
        let coprocessor = Rc::new(RefCell::new(TestCoproc { data: Vec::new() }));
        proc.set_coprocessor(cp as usize, coprocessor.clone());
        proc.map(
            0x1000,
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        )
        .unwrap();
        proc.set_pc(0x1003 - 100); // +3 to test alignment
        LdcLit {
            coproc: cp,
            crd: 0,
            d: false,
            index: true,
            add: true,
            imm32: 100,
            encoding: DontCare,
            ins: 0xaabbccdd,
        }
        .execute(&mut proc)
        .unwrap();
        assert_eq!(
            &coprocessor.borrow().data,
            &[0x03020100, 0x07060504, 0x0b0a0908, 0x0f0e0d0c]
        );
    }
}
