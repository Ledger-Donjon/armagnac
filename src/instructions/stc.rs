//! Implements STC and STC2 (Store Coprocessor) instructions.

use super::{
    other, undefined, AddOrSub, DecodeHelper,
    Encoding::{self, T1, T2},
    Instruction, Pattern,
};
use crate::{
    arm::{
        ArmProcessor,
        ArmVersion::{V7EM, V7M, V8M},
        Effect, RunError,
    },
    decoder::DecodeError,
    helpers::BitAccess,
    instructions::{indexing_args, unpredictable},
    it_state::ItState,
    registers::RegisterIndex,
};
use core::panic;

/// STC or STC2 instruction.
///
/// Store Coprocessor.
pub struct Stc {
    /// Coprocessor index, from 0 to 15.
    coproc: u8,
    /// Coprocessor source register.
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

impl Instruction for Stc {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V7M, V7EM, V8M],
                expression: "1110110xxxx0xxxxxxxxxxxxxxxxxxxx",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "1111110xxxx0xxxxxxxxxxxxxxxxxxxx",
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
        other(!index && !add && d && !wback)?; // MCRR, MCRR2
        let rn = ins.reg4(16);
        unpredictable(rn.is_pc())?;
        Ok(Self {
            coproc: ins.imm4(8) as u8,
            crd: ins.imm4(12) as u8,
            rn: ins.reg4(16),
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
            proc.write_u32_aligned(
                address,
                coprocessor.borrow_mut().get_word_to_store(self.ins),
            )?;
            address = address.wrapping_add(4);
            if coprocessor.borrow().done_storing(self.ins) {
                return Ok(Effect::None);
            }
            if self.wback {
                proc.set(self.rn, offset_addr);
            }
        }
    }

    fn name(&self) -> String {
        match (self.encoding, self.d) {
            (T1, true) => "stcl",
            (T1, false) => "stc",
            (T2, true) => "stc2l",
            (T2, false) => "stc2",
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

#[cfg(test)]
pub mod tests {
    use super::Stc;
    use crate::{
        arm::{ArmProcessor, Config},
        coprocessor::Coprocessor,
        instructions::{Encoding::DontCare, Instruction},
        registers::RegisterIndex,
    };
    use rand::Rng;
    use std::{cell::RefCell, rc::Rc};

    struct TestCoproc(Vec<u32>);

    impl Coprocessor for TestCoproc {
        fn accepted(&self, ins: u32) -> bool {
            assert_eq!(ins, 0xaabbccdd);
            true
        }

        fn done_loading(&self, _ins: u32) -> bool {
            unimplemented!()
        }

        fn done_storing(&self, _ins: u32) -> bool {
            self.0.len() == 0
        }

        fn get_one_word(&mut self, _ins: u32) -> u32 {
            unimplemented!()
        }

        fn get_two_words(&mut self, _ins: u32) -> (u32, u32) {
            unimplemented!()
        }

        fn get_word_to_store(&mut self, _ins: u32) -> u32 {
            self.0.remove(0)
        }

        fn internal_operation(&mut self, _ins: u32) {
            unimplemented!()
        }

        fn send_loaded_word(&mut self, _word: u32, _ins: u32) {
            unimplemented!()
        }

        fn send_one_word(&mut self, _word: u32, _ins: u32) {
            unimplemented!()
        }

        fn send_two_words(&mut self, _word1: u32, _word2: u32, _ins: u32) {
            unimplemented!()
        }
    }
    #[test]
    fn test_stc() {
        let mut proc = ArmProcessor::new(Config::v7m());
        let mut rng = rand::rng();
        let cp: u8 = rng.random_range(0..16);
        let coprocessor = Rc::new(RefCell::new(TestCoproc(vec![
            0x00010203, 0x04050607, 0x08090a0b, 0x0c0d0e0f,
        ])));
        let rn = RegisterIndex::new_general_random();
        proc.set_coprocessor(cp as usize, coprocessor.clone());
        proc.map(0x1000, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
            .unwrap();
        proc.set(rn, 0x1000 - 100);
        let mut expected = proc.registers.clone();
        expected.set(rn, 0x1000);
        Stc {
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
            proc.read_bytes_iface(0x1000, 16).unwrap(),
            vec![3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12]
        );
    }
}
