[![Latest version](https://img.shields.io/crates/v/armagnac.svg)](https://crates.io/crates/armagnac)
[![Documentation](https://docs.rs/armagnac/badge.svg)](https://docs.rs/armagnac)

# Armagnac

Armagnac is a simple ARM Thumb emulation library written in Rust which can be used to emulate simple embedded systems. The library gives high control on the processor execution, allowing to run instruction by instruction, create hooks, inspect or modify the system state on the fly. Integration of custom peripherals in the memory space is made easy, allowing custom platforms emulation. This library has little dependencies.

The library is in development and is highly experimental. It is not complete as not all instructions have been implemented yet. Implementation has been mostly tested against ArmV7-M firmwares, a bit less against ArmV8-M, and ArmV6-M has not been tested. Expect bugs, rage and frustration.

Currently, emulation speed is typically 8 million instructions per second. There is no virtualization or translation to native code whatsoever. Also, there is no "unsafe" code.

## Basic example

The following basic example runs a tiny assembly program and reads the processor R2 register value at the end.

```rust
use armagnac::arm::{ArmProcessor, ArmVersion};
let mut proc = ArmProcessor::new(ArmVersion::V7M, 0);

// mov r0, #5
// mov r1, #2
// sub r2, r0, r1
proc.map(0x1000, &[0x05, 0x20, 0x02, 0x21, 0x42, 0x1a]);

proc.set_pc(0x1000);
for i in 0..3 {
    proc.stepi().unwrap();
}
assert_eq!(proc.registers.r2, 3);
```

## Limitations

Here is a non-exhaustive list of what is not implemented/supported yet:
- There is not MPU support for ArmV6-M yet, only skeletons for ArmV7-M and ArmV8-M.
- Although some MPU registers are emulated, accesses are currently not verified by the processor.
- There is basic support for exceptions, but priorities are not enforced yet.

## Unimplemented instructions for ArmV6-M

Here is a list of instructions that are not implemented yet for ArmV6-M archiecture version:

- BKPT: Breakpoint
- UDF: Undefined
- WFE: Wait For Event
- WFI: Wait For Interrupt
- YIELD

## Unimplemented instructions for ArmV7-M/ArmV8-M

Here is the list of instructions that are not implemented yet for ArmV7-M and/or ArmV8-M architecture versions. In particular, there is not support for floating-point arithmetic and coprocessor operations. Unimplemented instructions will raise an error during execution.

- BKPT: Breakpoint
- CDP, CDP2: Coprocessor Data Processing
- CLREX: Clear Exclusive
- DBG: Debug Hint
- LDC, LDC2: Load Coprossessor
- LDREX: Load Register Exclusive
- LDREXB: Load Register Exclusive Byte
- LDREXH: Load Register Exclusive Halfword
- LDRSBT: Load Register Signed Byte Unprivileged
- LDRSHT: Load Register Signed Halfword Unprivileged
- MCR, MCR2: Move to Coprocessor from ARM Register
- MCRR, MCRR2: Move to Compressor from two ARM Registers
- MRC, MRC2: Move to ARM Register from Coprocessor
- MRRC, MRRC2: Move to two ARM Registers from Coprocessor
- PKHBT, PKHTB: Pack Halfword
- PLD: Preload Data
- PLI: Preload Instruction
- QASX: Saturating Add and Subtract with Exchange
- QSAX: Saturating Subtract and Add with Exchange
- SADD16: Signed Add 16
- SADD8: Signed Add 8
- SASX: Signed Add and Subtract with Exchange
- SEL: Select Bytes
- SHADD16: Signed Halving Add 16
- SHADD8: Signed Halving Add 8
- SHASX: Signed Halving Add and Subtract with Exchange
- SHSUB16: Signed Halving Subtract 16
- SHSUB8: Signed Halving Subtract 8
- SMLABB, SMLABT, SMLATB, SMLATT: Signed Multiply Accumutate (halfwords)
- SMLAD, SMLADX: Signed Multiply Accumutate Dual
- SMLAL: Signed Multiply Accumulate Long
- SMLALBB, SMLALBT, SMLALTB, SMLALTT: Signed Multiply Accumulate Long (halfwords)
- SMLALD, SMLALDX: Signed Multiply Accumulate Long Dual
- SMLAWB, SMLAWT: Signed Multiply Accumulate (word by halfword)
- SMLSD, SMLSDX: Signed Multiply Subtract Dual
- SMLSLD, SMLSLDX: Signed Multiply Subtract Long Dual
- SMMLA, SMMLAR: Signed Most Significant Word Multiply Accumulate
- SMMLS, SMMLSR: Signed Most Significant Word Multiply Subtract
- SMMUL, SMMULR: Signed Most Significant Word Multiply
- SMUAD, SMUADX: Signed Dual Multiply Add
- SMULBB, SMULBT, SMULTB, SMULTT: Signed Multiply (halfwords)
- SMULL: Signed Multiply Long
- SMULWB, SMULWT: Signed Multiply (word by halfword)
- SMUSD, SMUSDX: Signed Dual Multiply Subtract
- SSAT16: Signed Saturate 16
- SSAX: Signed Subtract and Add with Exchange
- SSUB16: Signed Subtract 16
- SSUB8: Signed Subtract 8
- STC, STC2: Store Coprocessor
- STRBT: Store Register Byte Unprivileged
- STREX: Store Register Exclusive
- STREXB: Store Register Exclusive Byte
- STREXH: Store Register Exclusive Halfword
- STRHT: Store Register Halfword Unprivileged
- STRT: Store Register Unprivileged
- SUB (SP minus register): Subtract
- SXTAB: Signed Extend and Add Byte
- SXTAB16: Signed Extend and Add Byte 16
- SXTAH: Signed Extend and Add Halfword
- SXTB16: Signed Extend Byte 16
- UADD16: Unsigned Add 16
- UADD8: Unsigned Add 8
- UASX: Unsigned Add and Subtract with Exchange
- UHADD16: Unsigned Halving Add 16
- UHADD8: Unsigned Halving Add 8
- UHASX: Unsigned Halfving Add and Subtract with Exchange
- UHSUB16: Unsigned Halving Subtract 16
- UHSUB8: Unsigned Halving Subtract 8
- UMAAL: Unsigned Multiply Accumulate Accumulate Long
- UQADD16: Unsigned Saturating Add 16
- UQADD8: Unsigned Saturating Add 8
- UQASX: Unsigned Saturating Add and Subtract with Exchange
- UQSUB16: Unsigned Saturating Subtract 16
- UQSUB8: Unsigned Saturating Subtract 8
- USAD8: Unsigned Sum of Absolute Differences
- USADA8: Unsigned Sum of Absolute Differences and Accumulate
- USAT: Unsigned Saturate
- USAT16: Unsigned Saturate 16
- USAX: Unsigned Subtract and Add with Exchange
- USUB16: Unsigned Subtract 16
- USUB8: Unsigned Subtract 8
- UXTAB: Unsigned Extend and Add Byte
- UXTAB16: Unsigned Extend and Add Byte 16
- UXTAH: Unsigned Extend and Add Halfword
- UXTB16: Unsigned Extend Byte 16
- VABS: Floating-point Absolute
- VADD: Floating-point Add
- VCMP, VCMPE: Floating-point Compare
- VCVT, VCVTR: Floating-point Convert
- VCVTB, VCVTT: Floating-point Convert Bottom / Top
- VDIV: Floating-point Divide
- VFMA, VFMS: Floating-point Fused Multiply Accumulate
- VFNMA, VFNMS: Floating-point Fused Negate Multiply Accumulate
- VLDM: Floating-point Load Multiple
- VLDR: Floating-point Load Register
- VMLA, VMLS: Floating-point Multiply and Accumulate
- VMOV: Floating-point Move
- VMRS: Move to ARM core register from floating-point Special Register
- VMSR: Move to floating-point Special Register from ARM core register
- VMUL: Floating-point Multiply
- VNEG: Floating-point Negate
- VNMLA, VNMLS, VNMUL: Floating-point Multiply Accumulate Negate
- VPOP: Floating-point Pop Registers
- VPUSH: Floating-point Push Registers
- VSQRT: Floating-point Square Root
- VSTM: Floating-point Store Multiple
- VSTR: Floating-point Store Register
- VSUB: Floating-point Subtract
- WFE: Wait For Event
- WFI: Wait For Interrupt
- YIELD