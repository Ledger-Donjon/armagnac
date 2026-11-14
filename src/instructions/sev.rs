//! Implements SEV (Send Event) instruction.

use super::Encoding::{self, T1, T2};
use super::Instruction;
use super::{
    ArmVersion::{V6M, V7EM, V7M, V8M},
    Pattern,
};
use crate::{
    core::{Effect, Processor, RunError},
    decoder::DecodeError,
    instructions::ItState,
};

/// SEV instruction.
pub struct Sev {}

impl Instruction for Sev {
    fn patterns() -> &'static [Pattern] {
        &[
            Pattern {
                encoding: T1,
                versions: &[V6M, V7M, V7EM, V8M],
                expression: "1011111101000000",
            },
            Pattern {
                encoding: T2,
                versions: &[V7M, V7EM, V8M],
                expression: "111100111010(1)(1)(1)(1)10(0)0(0)00000000100",
            },
        ]
    }

    fn try_decode(encoding: Encoding, _ins: u32, _state: ItState) -> Result<Self, DecodeError> {
        debug_assert!(encoding == T1 || encoding == T2);
        Ok(Self {})
    }

    fn execute(&self, proc: &mut Processor) -> Result<Effect, RunError> {
        proc.registers.event = true;
        Ok(Effect::None)
    }

    fn name(&self) -> String {
        "sev".into()
    }

    fn args(&self, _pc: u32) -> String {
        "".into()
    }
}

#[cfg(test)]
mod tests {
    use super::Sev;
    use crate::{
        core::{Config, Effect, Emulator, ItState, Processor, RunOptions},
        instructions::{Instruction, InstructionSize},
    };
    use std::rc::Rc;

    const SEV_T1: &[u8] = &[0x40, 0xbf];
    const SEV_T2: &[u8] = &[0xaf, 0xf3, 0x04, 0x80];
    const WFE_T1: &[u8] = &[0x20, 0xbf];
    const WFE_T2: &[u8] = &[0xaf, 0xf3, 0x02, 0x80];

    fn processor(config: Config, code: &[u8]) -> Processor {
        let mut proc = Processor::new(config);
        proc.map(0x1000, code).unwrap();
        proc.set_pc(0x1000);
        proc
    }

    fn decode(proc: &Processor, code: &[u8]) -> Rc<dyn Instruction> {
        let (raw, size) = match code {
            [a, b] => (u16::from_le_bytes([*a, *b]) as u32, InstructionSize::Ins16),
            [a, b, c, d] => (
                ((u16::from_le_bytes([*a, *b]) as u32) << 16) | u16::from_le_bytes([*c, *d]) as u32,
                InstructionSize::Ins32,
            ),
            _ => panic!("expected one Thumb instruction"),
        };
        proc.instruction_decoder
            .try_decode(raw, size, ItState::new())
            .unwrap()
    }

    fn check_sev(config: fn() -> Config, code: &[u8]) {
        for initial_event in [false, true] {
            let mut proc = processor(config(), code);
            proc.registers.event = initial_event;
            proc.registers.r0 = 0x12345678;
            proc.registers.psr.set_flags(0b10101);
            let mut expected = proc.registers.clone();
            expected.event = true;
            expected.pc += code.len() as u32;

            proc.run(RunOptions::new().gas(1)).unwrap();

            assert_eq!(proc.registers, expected);
            assert_eq!(proc.cycles, 1);
        }
    }

    #[test]
    fn test_sev_t1() {
        for config in [Config::v6m, Config::v7m, Config::v7em, Config::v8m] {
            check_sev(config, SEV_T1);
        }
    }

    #[test]
    fn test_sev_t2() {
        for config in [Config::v7m, Config::v7em, Config::v8m] {
            check_sev(config, SEV_T2);
        }
    }

    #[test]
    fn test_sev_effect() {
        let mut proc = Processor::new(Config::v7m());
        assert_eq!(Sev {}.execute(&mut proc).unwrap(), Effect::None);
        assert!(proc.registers.event);
    }

    #[test]
    fn test_wfe_without_event() {
        for wfe in [WFE_T1, WFE_T2] {
            let mut proc = processor(Config::v7m(), wfe);
            assert!(!proc.registers.event);
            let expected = proc.registers.clone();
            let ins = decode(&proc, wfe);

            for _ in 0..3 {
                assert_eq!(ins.execute(&mut proc).unwrap(), Effect::WaitForEvent);
                assert_eq!(proc.registers, expected);
            }
        }
    }

    #[test]
    fn test_repeated_sev_supplies_one_event() {
        for sev in [SEV_T1, SEV_T2] {
            for wfe in [WFE_T1, WFE_T2] {
                let mut proc = processor(Config::v7m(), &[sev, wfe].concat());
                let send = decode(&proc, sev);
                let wait = decode(&proc, wfe);
                for _ in 0..2 {
                    assert_eq!(send.execute(&mut proc).unwrap(), Effect::None);
                    assert!(proc.registers.event);
                }

                assert_eq!(wait.execute(&mut proc).unwrap(), Effect::None);
                assert!(!proc.registers.event);
                for _ in 0..3 {
                    assert_eq!(wait.execute(&mut proc).unwrap(), Effect::WaitForEvent);
                    assert!(!proc.registers.event);
                }
            }
        }
    }

    #[test]
    fn test_conditional_sev() {
        for sev in [SEV_T1, SEV_T2] {
            for condition_passed in [false, true] {
                for initial_event in [false, true] {
                    let code = [&[0x08, 0xbf][..], sev, WFE_T1].concat();
                    let mut proc = processor(Config::v7m(), &code);
                    proc.registers.psr.set_z(condition_passed);
                    proc.registers.event = initial_event;
                    let mut expected = proc.registers.clone();
                    expected.pc += 2 + sev.len() as u32;
                    expected.event = initial_event || condition_passed;

                    proc.run(RunOptions::new().gas(2)).unwrap();
                    assert_eq!(proc.registers, expected);
                    assert_eq!(proc.cycles, 2);

                    let wait = decode(&proc, WFE_T1);
                    let effect = if expected.event {
                        Effect::None
                    } else {
                        Effect::WaitForEvent
                    };
                    assert_eq!(wait.execute(&mut proc).unwrap(), effect);
                    assert!(!proc.registers.event);
                }
            }
        }
    }
}
