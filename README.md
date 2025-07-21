[![Latest version](https://img.shields.io/crates/v/armagnac.svg)](https://crates.io/crates/armagnac)
[![Documentation](https://docs.rs/armagnac/badge.svg)](https://docs.rs/armagnac)

# Armagnac

Armagnac is a simple ARM Thumb emulation library written in Rust which can be used to emulate simple embedded systems. The library gives high control on the processor execution, allowing to run instruction by instruction, create hooks, inspect or modify the system state on the fly. Integration of custom peripherals in the memory space is made easy, allowing custom platforms emulation. This library has little dependencies.

The library is in development and is highly experimental. It is not complete as not all instructions have been implemented yet for ArmV7-M and ArmV8-M (see below for more details). Implementation has been mostly tested against ArmV7-M firmwares, a bit less against ArmV8-M, and ArmV6-M has not been tested. Expect bugs, rage and frustration.

Currently, emulation speed is typically 8 million instructions per second. There is no virtualization or translation to native code whatsoever. Also, there is no "unsafe" code.

## Basic example

The following basic example runs a tiny assembly program and reads the processor R2 register value at the end.

```rust
use armagnac::core::{Processor, Config};
let mut proc = Processor::new(Config::v7m());

// mov r0, #5
// mov r1, #2
// sub r2, r0, r1
proc.map(0x1000, &[0x05, 0x20, 0x02, 0x21, 0x42, 0x1a]);

proc.set_pc(0x1000);
proc.run(RunOptions::new().gas(3)).unwrap();
assert_eq!(proc.registers.r2, 3);
```

## Limitations

Here is a non-exhaustive list of what is not implemented/supported yet:
- There is not MPU support for ArmV6-M yet, only skeletons for ArmV7-M and ArmV8-M.
- Although some MPU registers are emulated, accesses are currently not verified by the processor.
- There is basic support for exceptions, but priorities are not enforced yet.
- Only Thumb mode is supported at the moment.
- All exceptions are considered WFI wakeup events.
- No global monitor is implemented, synchronization and semaphores accross multiple
  processors cannot be emulated.

### Unimplemented instructions for ArmV7-M

Here is a list of instructions that are not implemented yet for ArmV7-M archiecture version. Unimplemented instructions will raise an error during execution.

- CSDB: Consumption of Speculative Data Barrier
- DBG: Debug Hint
- PLD: Preload Data
- PLI: Preload Instruction
- PSSBB: Physical Speculative Store Bypass Barrier

### Unimplemented instructions for ArmV7E-M

In addition to ArmV7-M, Here is the list of instructions specific to ArmV7E-M that are not implemented yet.

- PKHBT, PKHTB: Pack Halfword
- QASX: Saturating Add and Subtract with Exchange
- QSAX: Saturating Subtract and Add with Exchange
- SASX: Signed Add and Subtract with Exchange
- SEL: Select Bytes
- SHADD16: Signed Halving Add 16
- SHADD8: Signed Halving Add 8
- SHASX: Signed Halving Add and Subtract with Exchange
- SHSAX: Signed Halving Subtract and Add with Exchange
- SHSUB16: Signed Halving Subtract 16
- SHSUB8: Signed Halving Subtract 8
- SMLABB, SMLABT, SMLATB, SMLATT: Signed Multiply Accumutate (halfwords)
- SMLAD, SMLADX: Signed Multiply Accumutate Dual
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
- SMULWB, SMULWT: Signed Multiply (word by halfword)
- SMUSD, SMUSDX: Signed Dual Multiply Subtract
- SSAT16: Signed Saturate 16
- SSAX: Signed Subtract and Add with Exchange
- SSUB16: Signed Subtract 16
- SSUB8: Signed Subtract 8
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
- UHSAX: Unsigned Halfving Subtract and Add with Exchange
- UHSUB16: Unsigned Halving Subtract 16
- UHSUB8: Unsigned Halving Subtract 8
- UMAAL: Unsigned Multiply Accumulate Accumulate Long
- UQADD16: Unsigned Saturating Add 16
- UQADD8: Unsigned Saturating Add 8
- UQASX: Unsigned Saturating Add and Subtract with Exchange
- UQSAX: Unsigned Saturating Subtract and Add with Exchange
- UQSUB16: Unsigned Saturating Subtract 16
- UQSUB8: Unsigned Saturating Subtract 8
- USAD8: Unsigned Sum of Absolute Differences
- USADA8: Unsigned Sum of Absolute Differences and Accumulate
- USAX: Unsigned Subtract and Add with Exchange
- USUB16: Unsigned Subtract 16
- USUB8: Unsigned Subtract 8
- UXTAB: Unsigned Extend and Add Byte
- UXTAB16: Unsigned Extend and Add Byte 16
- UXTAH: Unsigned Extend and Add Halfword
- UXTB16: Unsigned Extend Byte 16

### Unimplemented instructions for ArmV8-M

Here is the list of instructions that are not implemented yet for ArmV8-M architecture version. In particular, there is no support for floating-point arithmetic and coprocessor operations. Unimplemented instructions will raise an error during execution.

- ADD (immediate, to PC): Add to PC
- ASRS: Arithmetic Shift Right, Setting flags
- BXAUT: Branch Exchange after Authentication
- CSDB: Consumption of Speculative Data Barrier
- DBG: Debug Hint
- LDA: Load-Acquire Word
- LDAB: Load-Acquire Byte
- LDAEX: Load-Acquire Exclusive Word
- LDAEXB: Load-Acquire Exclusive Byte
- LDAEXH: Load-Acquire Exclusive Halfword
- LDAH: Load-Acquire Halfword
- LSLS: Logical Shift Left, Setting flags
- LSRS: Logical Shift Right, Setting flags
- PLD: Preload Data
- PLI: Preload Instruction
- PSSBB: Physical Speculative Store Bypass Barrier
- SG: Secure Gateway
- SSAT16: Signed Saturate 16
- STL: Store-Release Word
- STLB: Store-Release Byte
- STLEX: Store-Release Exclusive Word
- STLEXB: Store-Release Exclusive Byte
- STLEXH: Store-Release Exclusive Halfword
- STLH: Store-Release Halfword
- SUB (immediate, from PC): Subtract
- TT, TTT, TTA, TTAT: Test Target (Alternate Domain, Unprivileged)
- VLLDM: Floating-point Lazy Load Multiple
- VLSTM: Floating-point Lazy Store Multiple
- VSBC: Whole Vector Subtract With Carry
- WLS, DLS, WLSTP, DLSTP: While Loop Start, Do Loop Start, While Loop Start with Tail Predication, Do Loop Start with Tail Predication

### Unimplemented instructions for ArmV8-M Floating-point extension

- FLDMDBX, FLDMIAX
- FSTMDBX, FSTMIAX
- VABS: Floating-point Absolute
- VADD: Floating-point Add
- VCMP, VCMPE: Floating-point Compare
- VCVT, VCVTA, VCVTB, VCVTM, VCVTN, VCVTP, VCVTR, VCVTT: Floating-point Convert
- VDIV: Floating-point Divide
- VFMA, VFMS: Floating-point Fused Multiply Accumulate
- VFMS: Floating-point Fused Multiply Subtract
- VFNMA, VFNMS: Floating-point Fused Negate Multiply Accumulate
- VINS: Floating-point move Insertion
- VLDM: Floating-point Load Multiple
- VLDR: Floating-point Load Register
- VMAXNM, VMAXNMMA: Vector Maximum, Vector Maximum Absolute
- VMINNM, VMINNMA: Vector Minimum, Vector Minimum Absolute
- VMLA, VMLS: Floating-point Multiply and Accumulate
- VMLS: Floating-point Multiply Subtract
- VMOV: Floating-point Move
- VMOVX: Floating-point Move extraction
- VMRS: Move to ARM core register from floating-point Special Register
- VMSR: Move to floating-point Special Register from ARM core register
- VMUL: Floating-point Multiply
- VNEG: Floating-point Negate
- VNMLA: Floating-point Multiply Accumulate and Negate
- VNMLS: Floating-point Multiply Subtract and Negate
- VNMUL: Floating-point Multiply and Negate
- VPOP: Floating-point Pop Registers
- VPUSH: Floating-point Push Registers
- VRINTA: Floating-point Round to Nearest Integer with Ties to Away
- VRINTM: Floating-point Round to Integer towards -Infinity
- VRINTN: Floating-point Round to Nearest Integer with Ties to Even
- VRINTP: Floating-point Round to Integer toward +Infinity
- VRINTR: Floating-point Round to Integer
- VRINTX: Floating-point Round to Integer, raising Inexact exception
- VRINTZ: Floating-point Round to Integer towards Zero
- VSEL: Floating-point Conditional Select
- VSQRT: Floating-point Square Root
- VSTM: Floating-point Store Multiple
- VSTR: Floating-point Store Register
- VSUB: Floating-point Subtract

### Unimplemented instructions for ArmV8-M DSP extension

- PKHBT, PKHTB: Pack Halfword
- QASX: Saturating Add and Subtract with Exchange
- QSAX: Saturating Subtract and Add with Exchange
- SASX: Signed Add and Subtract with Exchange
- SEL: Select Bytes
- SHADD16: Signed Halving Add 16
- SHADD8: Signed Halving Add 8
- SHASX: Signed Halving Add and Subtract with Exchange
- SHSAX: Signed Halving Subtract and Add with Exchange
- SHSUB16: Signed Halving Subtract 16
- SHSUB8: Signed Halving Subtract 8
- SMLABB, SMLABT, SMLATB, SMLATT: Signed Multiply Accumutate (halfwords)
- SMLAD, SMLADX: Signed Multiply Accumutate Dual
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
- SMULWB, SMULWT: Signed Multiply (word by halfword)
- SMUSD, SMUSDX: Signed Dual Multiply Subtract
- SSAX: Signed Subtract and Add with Exchange
- SSUB16: Signed Subtract 16
- SSUB8: Signed Subtract 8
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
- UQSAX: Unsigned Saturating Subtract and Add with Exchange
- UQSUB16: Unsigned Saturating Subtract 16
- UQSUB8: Unsigned Saturating Subtract 8
- USAD8: Unsigned Sum of Absolute Differences
- USADA8: Unsigned Sum of Absolute Differences and Accumulate
- USAX: Unsigned Subtract and Add with Exchange
- USUB16: Unsigned Subtract 16
- USUB8: Unsigned Subtract 8
- UXTAB: Unsigned Extend and Add Byte
- UXTAB16: Unsigned Extend and Add Byte 16
- UXTAH: Unsigned Extend and Add Halfword
- UXTB16: Unsigned Extend Byte 16

### Unimplemented instructions for ArmV8-M Custom Datapath extension

- CX1: Custom Instruction Class 1
- CX1D: Custom Instruction Class 1 dual
- CX2: Custom Instruction Class 2
- CX2D: Custom Instruction Class 2 dual
- CX3: Custom Instruction Cl- VDUP: Vector Duplicate
- CX3D: Custom Instruction Class 3 dual
- VCX1: Custom Extension Instruction Class 1
- VCX2: Custom Extension Instruction Class 2
- VCX3: Custom Extension Instruction Class 3

### Unimplemented instructions for ArmV8.1-M

- CINC: Conditional Increment
- CINV: Conditional Invert
- CLRM: Clear Multiple
- CNEG: Conditional Negate
- CSEL: Conditional Select
- CSET: Conditional Set
- CSETM: Conditional Set Mask
- CSINC: Conditional Select Increment
- CSINV: Conditional Select Invert
- CSNEG: Conditional Select Negation
- ESB: Error Synchronization Barrier
- VSCCLRM: Floating-point Secure Context Clear Multiple

### Unimplemented instructions for ArmV8.1-M MVE

- ASRL: Arithmetic Shift Right Long
- LCTP: Loop Clear with Tail Predication
- LSLL: Logical Shift Left Long
- LSRL: Logical Shift Right Long
- SQRSHR: Signed Saturating Rounding Shift Right
- SQRSHRL: Signed Saturating Rounding Shift Right Long
- SQSHL: Signed Saturating Shift Left
- SQSHLL: Signed Saturating Shift Left Long
- SRSHR: Signed Rounding Shift Right
- SRSHRL: Signed Rounding Shift Right Long
- UQRSHL: Unsigned Saturating Rounding Shift Left
- UQRSHLL: Unsigned Saturating Rounding Shift Left Long
- UQSHL: Unsigned Saturating Shift Left
- UQSHLL: Unsigned Saturating Shift Left Long
- URSHR: Unsigned Rounding Shift Right
- URSHRL: Unsigned Rounding Shift Right Long
- VABAV: Vector Absolute Difference and Accumulate Across Vector
- VABD: Vector Absolute Difference
- VADC: Whole Vector Add With Carry
- VADDLV: Vector Add Long Across Vector
- VADDV: Vector Add Across Vector
- VAND: Vector Bitwise And
- VBIC: Vector Bitwise Clear
- VBRSR: Vector Bit Reverse and Shift Right
- VCADD: Vector Complex Add with Rotate
- VCLS: Vector Count Leading Sign-bits
- VCLZ: Vector Count Leading Zeros
- VCMLA: Vector Complex Multiply Accumulate
- VCMUL: Vector Complex Multiply
- VCTP: Create Vector Tail Predicate
- VDDUP, VDWDUP: Vector Decrement and Duplicate, Vector Decrement with Wrap and Duplicate
- VDUP: Vector Duplicate
- VEOR: Vector Bitwise Exclusive Or
- VEOR: Vector Bitwise Exclusive Ortions for ArmV8.1-M
- VFMAS: Vector Fused Multiply Accumulate Scalar
- VHADD: Vector Halving Add
- VHCADD: Vector Halving Complex Add with Rotate
- VHSUB: Vector Halving Subtract
- VIDUP, VIWDUP: Vector Increment and Duplicate
- VLD2: Vector Deinterleaving Load - Stride 2
- VLD4: Vector Deinterleaving Load - Stride 4
- VLDRB, VLDRH, VLDRW, VLDRD: Vector Load Register
- VMAX, VMAXA: Vector Maximum, Vector Maximum Absolute
- VMAXNMV, VMAXNMAV: Vector Maximum Across Vector, Vector Maximum Absolute Across Vector
- VMAXXV, VMAXAV: Vector Maximum Across Vector, Vector Maximum Absolute Across Vector
- VMIN, VMINA: Vector Minimum, Vector Minimum Absolute
- VMINNMV, VMINNMAV: Vector Minimum Across Vector, Vector Minimum Absolute Across Vector
- VMINV, VMINAV: Vector Minimum Across Vector, Vector Minimum Absolute Across Vector
- VMLADAV: Vector Multiply Add Dual Accumulate Accross Vector
- VMLALDAV: Vector Multiply Add Long Dual Accumulate Across Vector
- VMLALV: Vector Multiply Accumulate Long Across Vector
- VMLAS: Vector Multiply Accumulate Scalar
- VMLAV: Vector Multiply Accumulate Across Vector
- VMLSDAV: Vector Multiply Subtract Dual Accumulate Across Vector
- VMLSLDAV: Vector Multiply Subtract Long Dual Accumulate Across Vector
- VMOVL: Vector Move Long
- VMOVN: Vector Move and Narrow
- VMULH, VRMULH: Vector Multiply Returning High Half, Vector Rounding Multiply Returning High Half
- VMULL: Vector Multiply Long
- VMVN: Vector Bitwise NOT
- VORN: Vector Bitwise Or Not
- VPNOT: Vector Predicate NOT
- VPSEL: Vector Predicate Select
- VPST: Vector Predicate Set Then
- VPT: Vector Predicate Then
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
- VRMLALDAVH: Vector Rounding Multiply Add Long Dual Accumulate Across Vector Returning High 64 bits
- VRMLALVH: Vector Multiply Accumulate Long Across Vector Returning High 64 bits
- VRMLSLDAVH: Vector Rounding Multiply Subtract Long Dual Accumulate Across Vector Returning High 64 bits
- VRSHL: Vector Rounding Shift Left
- VRSHR: Vector Rounding Shift Right
- VRSHRN: Vector Rounding Shift Right and Narrow
- VSHL: Vector Shift Left
- VSHLC: Whole Vector Left Shift with Carry
- VSHLL: Vector Shift Left Long
- VSHR: Vector Shift Right
- VSHRN: Vector Shift Right and Narrow
- VSLI: Vector Shift Left and Insert
- VSRI: Vector Shift Right and Insert
- VST2: Vector Interleaving Store - Stride 2
- VST4: Vector Interleaving Store - Stride 4
- VSTRB, VSTRH, VSTRW: Vector Store Register

### Unimplemented instructions for ArmV8.1-M PACBTI

- AUT: Authenticate link register
- AUTG: Authenticate general value
- BTI: Branch target identification
- PAC: Pointer Authentication Code for the link register
- PACBTI: Pointer Authentication Code for the link register with BTI clearing
- PACG: Pointer Authentication Code for a general value

### Unimplemented instructions for ArmV8.1-M Low Overhead Branch extension

- BF, BFX, BFL, BFLX, BFCSEL: Branch Future, Branch Future and Exchange, Branch Future with Link, Branch Future with Link and Exchange, Branch Future Conditional Select
- LE, LETP: Loop End, Loop End with Tail Predication
