use armagnac::arm::{ArmProcessor, ArmVersion};
use object::{File, Object, ObjectSection, ObjectSymbol};
use std::collections::BTreeMap;

/// This test runs a call to memcpy to copy a string to a buffer.
#[test]
fn test_memcpy() {
    let mut helper = Helper::new();
    helper.call1("test_memcpy", ADDR_RAM);
    // Check that the string has been copied to the RAM.
    let expect = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\0";
    assert_eq!(
        helper
            .proc
            .bytes_at(ADDR_RAM, expect.bytes().len() as u32)
            .unwrap()
            .as_slice(),
        expect.as_bytes()
    );
}

#[test]
fn test_fibonacci() {
    let mut helper = Helper::new();
    assert_eq!(helper.call1("test_fibonacci", 12), 144);
}

#[test]
fn test_cos() {
    let mut helper = Helper::new();
    let arg = 5.00f32;
    let result = helper.call1("test_cos", arg.to_bits());
    assert_eq!(f32::from_bits(result), arg.cos());
}

#[test]
fn test_sqrt() {
    let mut helper = Helper::new();
    let arg = 5.00f32;
    let result = helper.call1("test_sqrt", arg.to_bits());
    assert_eq!(f32::from_bits(result), arg.sqrt());
}

const ADDR_RAM: u32 = 0x10000000;
const STACK_SIZE: u32 = 1024;

/// Helper for running easily tests from an ELF file.
struct Helper {
    /// Processor executing the test methods.
    proc: ArmProcessor,
    /// All symbols and their address, extracted from the ELF file.
    symbols: BTreeMap<String, u32>,
}

impl Helper {
    fn new() -> Self {
        // Parse ELF file to extract all symbols.
        // Keep the symbols in a BTreeMap to simplify and discard object::File.
        let elf = include_bytes!("tests.elf").to_vec();
        let object = File::parse(&*elf).unwrap();
        let symbols = object
            .symbols()
            .into_iter()
            .map(|s| (s.name().unwrap().into(), s.address() as u32))
            .collect();

        let mut proc = ArmProcessor::new(ArmVersion::V7M, 0);
        //proc.map(0, include_bytes!("trigo.bin")).unwrap();
        proc.map_ram(ADDR_RAM, 1024).unwrap();

        // Map program section as read-only RAM memories
        for name in [".text", ".rodata", ".data"] {
            let section = object.section_by_name(name).unwrap();
            proc.map(
                section.address() as u32,
                &section.uncompressed_data().unwrap(),
            )
            .unwrap();
        }

        Self { proc, symbols }
    }

    /// Sets PC at given method entry and execute instructions until the function returns (or a
    /// timeout is reached).
    fn call(&mut self, method: &str) {
        let address = self.symbols[method.into()];
        self.proc.set_pc(address & 0xfffffffe); // We drop lsb (thumb mode bit)
        self.proc.registers.lr = 0xfffffffe;
        self.proc.set_sp(ADDR_RAM + STACK_SIZE);

        for _ in 0..200000 {
            self.proc.stepi().unwrap();
            // Run util code branches back to initial LR
            if self.proc.pc() == 0xfffffffe {
                // Verify stack has been poped correctly
                assert_eq!(self.proc.sp(), ADDR_RAM + STACK_SIZE);
                return;
            }
        }

        panic!("Test execution limit reached");
    }

    /// Similar to [Self::call], with passing a function argument and returning function result
    /// using R0 register.
    fn call1(&mut self, method: &str, arg0: u32) -> u32 {
        self.proc.registers.r0 = arg0;
        self.call(method);
        self.proc.registers.r0
    }
}
