use armagnac::arm::{ArmProcessor, ArmVersion};

/// This test runs a call to memcpy to copy a string to a buffer.
#[test]
fn test_memcpy() {
    let mut proc = ArmProcessor::new(ArmVersion::V7M, 0);
    const ADDR_RAM: u32 = 0x10000000;
    const STACK_SIZE: u32 = 1024;
    proc.map(0, include_bytes!("memcpy.bin")).unwrap();
    proc.map_ram(ADDR_RAM, 1024).unwrap();
    proc.set_sp(ADDR_RAM + STACK_SIZE);
    proc.registers.r0 = ADDR_RAM; // dst buffer argument
    proc.registers.lr = 0xfffffffe;

    // Run maximum 1000 instructions
    for _ in 0..20000 {
        proc.stepi().unwrap();
        // Run util code branches back to initial LR
        if proc.pc() == 0xfffffffe {
            break;
        }
    }

    // Check that the string has been copied to the RAM.
    let expect = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\0";
    assert_eq!(proc.bytes_at(ADDR_RAM, expect.bytes().len() as u32).unwrap().as_slice(), expect.as_bytes());
    // Verify stack has been poped correctly
    assert_eq!(proc.sp(), ADDR_RAM + STACK_SIZE);
}