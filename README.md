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

Here is a list of instructions that are not implemented yet for ArmV6-M archiecture version. Unimplemented instructions will raise an error during execution.

- WFE: Wait For Event
- WFI: Wait For Interrupt
- YIELD

## Unimplemented instructions for ArmV7-M

Here is a list of instructions that are not implemented yet for ArmV7-M archiecture version. In particular, there is no support for coprocessor operations. Unimplemented instructions will raise an error during execution.

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
- MRC, MRC2: Move to Register from Coprocessor
- MRRC, MRRC2: Move to two Registers from Compressor
- PKHBT, PKHTB: Pack Halfword
- PLD: Preload Data
- PLI: Preload Instruction
- QASX: Saturating Add and Subtract with Exchange
- QSAX: Saturating Subtract and Add with Exchange
- SADD16: Signed Add 16
- SADD8: Signed Add 8
- SASX: Signed Add and Subtract with Exchange
- SMLAL: Signed Multiply Accumulate Long
- SMULL: Signed Multiply Long
- STC, STC2: Store Coprocessor
- STRBT: Store Register Byte Unprivileged
- STREX: Store Register Exclusive
- STREXB: Store Register Exclusive Byte
- STREXH: Store Register Exclusive Halfword
- STRHT: Store Register Halfword Unprivileged
- STRT: Store Register Unprivileged
- SUB (SP minus register): Subtract
- USAT: Unsigned Saturate
- WFE: Wait For Event
- WFI: Wait For Interrupt
- YIELD

## Unimplemented instructions for ArmV8-M

Here is the list of instructions that are not implemented yet for ArmV8-M architecture version. In particular, there is no support for floating-point arithmetic and coprocessor operations. Unimplemented instructions will raise an error during execution.

- ADD (immediate, to PC): Add to PC
- ASRL: Arithmetic Shift Right Long
- ASRS: Arithmetic Shift Right, Setting flags
- AUT: Authenticate link register
- AUTG: Authenticate general value
- BF, BFX, BFL, BFLX, BFCSEL: Branch Future, Branch Future and Exchange, Branch Future with Link, Branch Future with Link and Exchange, Branch Future Conditional Select
- BTI: Branch target identification
- BXAUT: Branch Exchange after Authentication
- CDP, CDP2: Coprocessor Data Processing
- CINC: Conditional Increment
- CINV: Conditional Invert
- CLREX: Clear Exclusive
- CLRM: Clear Multiple
- CNEG: Conditional Negate
- CSDB: Consumption of Speculative Data Barrier
- CSEL: Conditional Select
- CSET: Conditional Set
- CSETM: Conditional Set Mask
- CSINC: Conditional Select Increment
- CSINV: Conditional Select Invert
- CSNEG: Conditional Select Negation
- CX1: Custom Instruction Class 1
- CX1D: Custom Instruction Class 1 dual
- CX2: Custom Instruction Class 2
- CX2D: Custom Instruction Class 2 dual
- CX3: Custom Instruction Class 3
- CX3D: Custom Instruction Class 3 dual
- DBG: Debug Hint
- ESB: Error Synchronization Barrier
- FLDMDBX, FLDMIAX
- FSTMDBX, FSTMIAX
- LCTP: Loop Clear with Tail Predication
- LDA: Load-Acquire Word
- LDAB: Load-Acquire Byte
- LDAEX: Load-Acquire Exclusive Word
- LDAEXB: Load-Acquire Exclusive Byte
- LDAEXH: Load-Acquire Exclusive Halfword
- LDAH: Load-Acquire Halfword
- LDC, LDC2: Load Coprossessor
- LDREX: Load Register Exclusive
- LDREXB: Load Register Exclusive Byte
- LDREXH: Load Register Exclusive Halfword
- LDRSBT: Load Register Signed Byte Unprivileged
- LDRSHT: Load Register Signed Halfword Unprivileged
- LE, LETP: Loop End, Loop End with Tail Predication
- LSLL: Logical Shift Left Long
- LSLS: Logical Shift Left, Setting flags
- LSRL: Logical Shift Right Long
- LSRS: Logical Shift Right, Setting flags
- MCR, MCR2: Move to Coprocessor from ARM Register
- MCRR, MCRR2: Move to Compressor from two ARM Registers
- MRC, MRC2: Move to ARM Register from Coprocessor
- MRRC, MRRC2: Move to two ARM Registers from Coprocessor
- PAC: Pointer Authentication Code for the link register
- PACBTI: Pointer Authentication Code for the link register with BTI clearing
- PACG: Pointer Authentication Code for a general value
- PKHBT, PKHTB: Pack Halfword
- PLD: Preload Data
- PLI: Preload Instruction
- PSSBB: Physical Speculative Store Bypass Barrier
- QASX: Saturating Add and Subtract with Exchange
- QSAX: Saturating Subtract and Add with Exchange
- SADD16: Signed Add 16
- SADD8: Signed Add 8
- SASX: Signed Add and Subtract with Exchange
- SEL: Select Bytes
- SG: Secure Gateway
- SHADD16: Signed Halving Add 16
- SHADD8: Signed Halving Add 8
- SHASX: Signed Halving Add and Subtract with Exchange
- SHSAX: Signed Halving Subtract and Add with Exchange
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
- SQRSHR: Signed Saturating Rounding Shift Right
- SQRSHRL: Signed Saturating Rounding Shift Right Long
- SQSHL: Signed Saturating Shift Left
- SQSHLL: Signed Saturating Shift Left Long
- SRSHR: Signed Rounding Shift Right
- SRSHRL: Signed Rounding Shift Right Long
- SSAT16: Signed Saturate 16
- SSAX: Signed Subtract and Add with Exchange
- SSBB: Speculative Store Bypass Barrier
- SSUB16: Signed Subtract 16
- SSUB8: Signed Subtract 8
- STC, STC2: Store Coprocessor
- STL: Store-Release Word
- STLB: Store-Release Byte
- STLEX: Store-Release Exclusive Word
- STLEXB: Store-Release Exclusive Byte
- STLEXH: Store-Release Exclusive Halfword
- STLH: Store-Release Halfword
- STRBT: Store Register Byte Unprivileged
- STREX: Store Register Exclusive
- STREXB: Store Register Exclusive Byte
- STREXH: Store Register Exclusive Halfword
- STRHT: Store Register Halfword Unprivileged
- STRT: Store Register Unprivileged
- SUB (SP minus register): Subtract
- SUB (immediate, from PC): Subtract
- SXTAB: Signed Extend and Add Byte
- SXTAB16: Signed Extend and Add Byte 16
- SXTAH: Signed Extend and Add Halfword
- SXTB16: Signed Extend Byte 16
- TT, TTT, TTA, TTAT: Test Target (Alternate Domain, Unprivileged)
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
- UQRSHL: Unsigned Saturating Rounding Shift Left
- UQRSHLL: Unsigned Saturating Rounding Shift Left Long
- UQSAX: Unsigned Saturating Subtract and Add with Exchange
- UQSHL: Unsigned Saturating Shift Left
- UQSHLL: Unsigned Saturating Shift Left Long
- UQSUB16: Unsigned Saturating Subtract 16
- UQSUB8: Unsigned Saturating Subtract 8
- URSHR: Unsigned Rounding Shift Right
- URSHRL: Unsigned Rounding Shift Right Long
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
- VABAV: Vector Absolute Difference and Accumulate Across Vector
- VABD: Vector Absolute Difference
- VABS: Floating-point Absolute
- VADC: Whole Vector Add With Carry
- VADD: Floating-point Add
- VADDLV: Vector Add Long Across Vector
- VADV: Vector Add Across Vector
- VAND: Vector Bitwise And
- VBIC: Vector Bitwise Clear
- VBRSR: Vector Bit Reverse and Shift Right
- VCADD: Vector Complex Add with Rotate
- VCLS: Vector Count Leading Sign-bits
- VCLZ: Vector Count Leading Zeros
- VCMLA: Vector Complex Multiply Accumulate
- VCMP, VCMPE: Floating-point Compare
- VCMUL: Vector Complex Multiply
- VCTP: Create Vector Tail Predicate
- VCVT, VCVTA, VCVTB, VCVTM, VCVTN, VCVTP, VCVTR, VCVTT: Floating-point Convert
- VCX1: Custom Extension Instruction Class 1
- VCX2: Custom Extension Instruction Class 2
- VCX3: Custom Extension Instruction Class 3
- VDDUP, VDWDUP: Vector Decrement and Duplicate, Vector Decrement with Wrap and Duplicate
- VDIV: Floating-point Divide
- VDUP: Vector Duplicate
- VEOR: Vector Bitwise Exclusive Or
- VFMA, VFMS: Floating-point Fused Multiply Accumulate
- VFMAS: Vector Fused Multiply Accumulate Scalar
- VFMS: Floating-point Fused Multiply Subtract
- VFNMA, VFNMS: Floating-point Fused Negate Multiply Accumulate
- VHADD: Vector Halving Add
- VHCADD: Vector Halving Complex Add with Rotate
- VHSUB: Vector Halving Subtract
- VDIP, VIWDUP: Vector Increment and Duplicate
- VINS: Floating-point move Insertion
- VLD2: Vector Deinterleaving Load - Stride 2
- VLD4: Vector Deinterleaving Load - Stride 4
- VLDM: Floating-point Load Multiple
- VLDR: Floating-point Load Register
- VLDRB, VLDRH, VLDRW, VLDRD: Vector Load Register
- VLLDM: Floating-point Lazy Load Multiple
- VLSTM: Floating-point Lazy Store Multiple
- VMAX, VMAXA: Vector Maximum, Vector Maximum Absolute
- VMAXNM, VMAXNMMA: Vector Maximum, Vector Maximum Absolute
- VMAXNMV, VMAXNMAV: Vector Maximum Across Vector, Vector Maximum Absolute Across Vector
- VMAXXV, VMAXAV: Vector Maximum Across Vector, Vector Maximum Absolute Across Vector
- VMIN, VMINA: Vector Minimum, Vector Minimum Absolute
- VMINNM, VMINNMA: Vector Minimum, Vector Minimum Absolute
- VMINNMV, VMINNMAV: Vector Minimum Across Vector, Vector Minimum Absolute Across Vector
- VMINV, VMINAV: Vector Minimum Across Vector, Vector Minimum Absolute Across Vector
- VMLA, VMLS: Floating-point Multiply and Accumulate
- VMLADAV: Vector Multiply Add Dual Accumulate Accross Vector
- VMLALDAV: Vector Multiply Add Long Dual Accumulate Across Vector
- VMLALV: Vector Multiply Accumulate Long Across Vector
- VMLAS: Vector Multiply Accumulate Scalar
- VMLAV: Vector Multiply Accumulate Across Vector
- VMLS: Floating-point Multiply Subtract
- VMLSDAV: Vector Multiply Subtract Dual Accumulate Across Vector
- VMLSLDAV: Vector Multiply Subtract Long Dual Accumulate Across Vector
- VMOV: Floating-point Move
- VMOVL: Vector Move Long
- VMOVN: Vector Move and Narrow
- VMOVX: Floating-point Move extraction
- VMRS: Move to ARM core register from floating-point Special Register
- VMSR: Move to floating-point Special Register from ARM core register
- VMUL: Floating-point Multiply
- VMULH, VRMULH: Vector Multiply Returning High Half, Vector Rounding Multiply Returning High Half
- VMULL: Vector Multiply Long
- VMVN: Vector Bitwise NOT
- VNEG: Floating-point Negate
- VNMLA, VNMLS, VNMUL: Floating-point Multiply Accumulate Negate
- VORN: Vector Bitwise Or Not
- VPNOT: Vector Predicate NOT
- VPOP: Floating-point Pop Registers
- VPSEL: Vector Predicate Select
- VPST: Vector Predicate Set Then
- VPT: Vector Predicate Then
- VPUSH: Floating-point Push Registers
- VQABS: Vector Saturating Absolute
- VQADD: Vector Saturating Add
- VQDMLADH, VQRDMLADH: Vector Saturating Doubling Multiply Add Dual Returning Half High, Vector Saturating Rounding Doubling Multiply Add Dual Returning High Half
- VQDMLAH, VQRDMLAH: Vector Saturating Doubling Multiply Accumulate, Vector Saturating Rounding Doubling Multiply Accumulate
- VQDMLASH, VQRDMLASH: Vector Saturating Doubling Multiply Accumulate Scalar Half High, Vector Saturating Rounding Doubling Multiply Accumulate Scalar High Half
- VQDMLSDH, VQRDMLSDH: Vector Saturating Doubling Multiply Subtract Dual Returning Half High, Vector Saturating Rounding Doubling Multiply Subtract Dual Returning Half High
- VQDMULH, VQRDMULH: Vector Saturating Doubling Multiply Returning High Half, Vector Saturating Rounding Doubling Multiply Returning High Half
- VQDMULL: Vector Multiply Long
- VQMOVN: Vector Saturating Move and Narrow
- VQMOVUN: Vector Saturating Move Unsigned and Narrow
- VQNEG: Vector Saturating Negate
- VQRSHL: Vector Saturating Rounding Shift Left
- VQRSHRN: Vector Saturating Rounding Shift Right and Narrow
- VQRSHRUN: Vector Saturating Rounding Shift Right Unsigned and Narrow
- VQSHL, VQSHLU: Vector Saturating Shift Left, Vector Saturating Shift Left Unsigned
- VQSHRN: Vector Saturating Shift Right and Narrow
- VQSHRUN: Vector Saturating Shift Right Unsigned and Narrow
- VQSUB: Vector Saturating Subtract
- VREV16: Vector Reverse
- VREV32: Vector Reverse
- VREV64: Vector Reverse
- VRHADD: Vector Rounding Halving Add
- VRINT: Vector Round Integer
- VRINTA: Floating-point Round to Nearest Integer with Ties to Away
- VRINTM: Floating-point Round to Integer towards -Infinity
- VRINTN: Floating-point Round to Nearest Integer with Ties to Even
- VRINTP: Floating-point Round to Integer toward +Infinity
- VRINTR: Floating-point Round to Integer
- VRINTX: Floating-point Round to Integer, raising Inexact exception
- VRINTZ: Floating-point Round to Integer towards Zero
- VRMLALDAVH: Vector Rounding Multiply Add Long Dual Accumulate Across Vector Returning High 64 bits
- VRMLALVH: Vector Multiply Accumulate Long Across Vector Returning High 64 bits
- VRMLSLDAVH: Vector Rounding Multiply Subtract Long Dual Accumulate Across Vector Returning High 64 bits
- VRSHL: Vector Rounding Shift Left
- VRSHR: Vector Rounding Shift Right
- VRSHRN: Vector Rounding Shift Right and Narrow
- VSBC: Whole Vector Subtract With Carry
- VSCCLRM: Floating-point Secure Context Clear Multiple
- VSEL: Floating-point Conditional Select
- VSHL: Vector Shift Left
- VSHLC: Whole Vector Left Shift with Carry
- VSHLL: Vector Shift Left Long
- VSHR: Vector Shift Right
- VSHRN: Vector Shift Right and Narrow
- VSLI: Vector Shift Left and Insert
- VSQRT: Floating-point Square Root
- VSRI: Vector Shift Right and Insert
- VST2: Vector Interleaving Store - Stride 2
- VST4: Vector Interleaving Store - Stride 4
- VSTM: Floating-point Store Multiple
- VSTR: Floating-point Store Register
- VSTRB, VSTRH, VSTRW: Vector Store Register
- VSUB: Floating-point Subtract
- WFE: Wait For Event
- WFI: Wait For Interrupt
- WLS, DLS, WLSTP, DLSTP: While Loop Start, Do Loop Start, While Loop Start with Tail Predication, Do Loop Start with Tail Predication
- YIELD
