use std::{cell::RefCell, rc::Rc};

use armagnac::{
    arm::{
        ArmProcessor, Emulator,
        Event::{self},
    },
    harness::{ElfHarness, ADDR_RAM, STACK_SIZE},
    irq::Irq::SysTick,
    memory::{Env, MemoryInterface},
};

/// This test runs a call to memcpy to copy a string to a buffer.
#[test]
fn test_memcpy() {
    let elf = include_bytes!("tests.elf");
    let mut helper = ElfHarness::new(elf);
    helper.call1("test_memcpy", ADDR_RAM);
    // Check that the string has been copied to the RAM.
    let expect = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\0";
    assert_eq!(
        helper
            .proc
            .read_bytes_iface(ADDR_RAM, expect.bytes().len() as u32)
            .unwrap()
            .as_slice(),
        expect.as_bytes()
    );
}

#[test]
fn test_fibonacci() {
    let elf = include_bytes!("tests.elf");
    let mut helper = ElfHarness::new(elf);
    assert_eq!(helper.call1("test_fibonacci", 12), 144);
}

#[test]
fn test_cos() {
    let elf = include_bytes!("tests.elf");
    let mut helper = ElfHarness::new(elf);
    let arg = 5.00f32;
    let result = helper.call1("test_cos", arg.to_bits());
    assert_eq!(f32::from_bits(result), arg.cos());
}

#[test]
fn test_sqrt() {
    let elf = include_bytes!("tests.elf");
    let mut helper = ElfHarness::new(elf);
    let arg = 5.00f32;
    let result = helper.call1("test_sqrt", arg.to_bits());
    assert_eq!(f32::from_bits(result), arg.sqrt());
}

#[test]
fn test_pow() {
    let elf = include_bytes!("tests.elf");
    let mut helper = ElfHarness::new(elf);
    let arg0 = 5.62f32.to_bits();
    let arg1 = 7.54f32.to_bits();
    assert_eq!(
        f32::from_bits(helper.call2("test_pow", arg0, arg1)),
        449792.0f32
    );
}

#[test]
fn test_bkpt() {
    let elf = include_bytes!("tests.elf");
    let mut helper = ElfHarness::new(elf);
    let arg = 6.00f32;
    helper.proc.registers.r0 = arg.to_bits();
    let address = helper.symbols["test_bkpt".into()];
    helper.proc.set_pc(address & 0xfffffffe); // We drop lsb (thumb mode bit)
    helper.proc.registers.lr = 0xfffffffe;
    helper.proc.set_sp(ADDR_RAM + STACK_SIZE);
    let mut breakpoint = None;

    loop {
        match helper.proc.next_event() {
            Ok(event) => match event {
                Event::Break(value) => {
                    // We have only one BKPT instruction.
                    assert_eq!(breakpoint, None);
                    breakpoint = Some(value)
                }
                _ => {}
            },
            Err(_) => panic!(),
        }
        // Run util code branches back to initial LR
        if helper.proc.pc() == 0xfffffffe {
            assert_eq!(helper.proc.sp(), ADDR_RAM + STACK_SIZE);
            break;
        }
    }

    // Check that we did break once.
    assert_eq!(breakpoint, Some(0xa5));

    // Check result, this won't hurt.
    let result = helper.proc.registers.r0;
    assert_eq!(f32::from_bits(result), arg.cos().sin());
}

#[test]
/// This test calls a simple method which just returns an integer after calling WFE (Wait For
/// Event) or WFI (Wait For Interrupt) instructions.
///
/// The processor execution should halt until an event occurs. To test this, we setup a peripheral
/// which requests an interrupt at cycle 1000. If WFE/WFI works correctly, the execution of the
/// tested functions should not return before cycle 1000.
fn test_wfe_wfi() {
    struct DummyPeripheral {}

    impl MemoryInterface for DummyPeripheral {
        fn size(&self) -> u32 {
            0
        }

        fn update(&mut self, _env: &mut Env) {
            if (_env.cycles % 1000 == 0) && (_env.cycles > 0) {
                _env.request_interrupt(SysTick);
            }
        }
    }

    let elf = include_bytes!("tests.elf");
    let mut helper = ElfHarness::new(elf);
    helper
        .proc
        .map_iface(0x10000000, Rc::new(RefCell::new(DummyPeripheral {})))
        .unwrap();
    let result = helper.call("test_wfe");
    let cycles = helper.proc.cycles;
    assert!((cycles >= 1000) && (cycles < 1100));
    assert_eq!(result, 0xdeadbeef);

    let result = helper.call("test_wfi");
    let cycles = helper.proc.cycles;
    assert!((cycles >= 2000) && (cycles < 2100));
    assert_eq!(result, 0xcafeb105);
}
