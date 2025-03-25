use armagnac::harness::{ElfHarness, ADDR_RAM};

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
            .bytes_at(ADDR_RAM, expect.bytes().len() as u32)
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
