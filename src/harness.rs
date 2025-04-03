use crate::{
    arm::{ArmProcessor, ArmVersion},
    instructions::Mnemonic,
};
use object::{File, Object, ObjectSection, ObjectSymbol};
use std::collections::BTreeMap;

pub const ADDR_RAM: u32 = 0x10000000;
pub const STACK_SIZE: u32 = 1024;

/// Helper for running easily tests from an ELF file.
pub struct ElfHarness {
    /// Processor executing the test methods.
    pub proc: ArmProcessor,
    /// All symbols and their address, extracted from the ELF file.
    pub symbols: BTreeMap<String, u32>,
}

impl ElfHarness {
    pub fn new(elf: &[u8]) -> Self {
        // Parse ELF file to extract all symbols.
        // Keep the symbols in a BTreeMap to simplify and discard object::File.
        let object = File::parse(&*elf).unwrap();
        let symbols = object
            .symbols()
            .into_iter()
            .map(|s| (s.name().unwrap().into(), s.address() as u32))
            .collect();

        let mut proc = ArmProcessor::new(ArmVersion::V7M, 0);
        proc.map_ram(ADDR_RAM, 1024).unwrap();

        // Map program section as read-only RAM memories
        for name in [".text", ".rodata", ".data"] {
            if let Some(section) = object.section_by_name(name) {
                proc.map(
                    section.address() as u32,
                    &section.uncompressed_data().unwrap(),
                )
                .unwrap();
            }
        }

        Self { proc, symbols }
    }

    /// Sets PC at given method entry and execute instructions until the function returns (or a
    /// timeout is reached).
    pub fn call(&mut self, method: &str) {
        let address = self.symbols[method.into()];
        self.proc.set_pc(address & 0xfffffffe); // We drop lsb (thumb mode bit)
        self.proc.registers.lr = 0xfffffffe;
        self.proc.set_sp(ADDR_RAM + STACK_SIZE);

        loop {
            self.proc.stepi().unwrap();
            // Run util code branches back to initial LR
            if self.proc.pc() == 0xfffffffe {
                // Verify stack has been poped correctly
                assert_eq!(self.proc.sp(), ADDR_RAM + STACK_SIZE);
                return;
            }
        }
    }

    /// Similar to [Self::call], with passing a function argument and returning function result
    /// using R0 register.
    pub fn call1(&mut self, method: &str, arg0: u32) -> u32 {
        self.proc.registers.r0 = arg0;
        self.call(method);
        self.proc.registers.r0
    }

    /// Similar to [Self::call], with passing two function arguments and returning function result.
    pub fn call2(&mut self, method: &str, arg0: u32, arg1: u32) -> u32 {
        self.proc.registers.r0 = arg0;
        self.proc.registers.r1 = arg1;
        self.call(method);
        self.proc.registers.r0
    }
}
