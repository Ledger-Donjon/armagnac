use armagnac::arm::{ArmProcessor, ArmVersion};

#[test]
fn fibonacci() {
    let mut proc = ArmProcessor::new(ArmVersion::V7M, 0);
    const ADDR_RAM: u32 = 0x10000000;
    const STACK_SIZE: u32 = 1024;
    proc.map_ram(ADDR_RAM, 1024).unwrap();
    proc.map(0, include_bytes!("fibonacci.bin")).unwrap();
    proc.set_sp(ADDR_RAM + STACK_SIZE);
    proc.registers.r0 = 12; // Fibonacci input value
    proc.registers.lr = 0xfffffffe;

    // Run maximum 1000 instructions
    for _ in 0..20000 {
        proc.stepi().unwrap();
        // Run util code branches back to initial LR
        if proc.pc() == 0xfffffffe {
            break;
        }
    }

    // Test fibonacci function result
    assert_eq!(proc.registers.r0, 144);
    // Verify stack has been poped correctly
    assert_eq!(proc.sp(), ADDR_RAM + STACK_SIZE);
}
