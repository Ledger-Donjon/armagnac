// This file is compiled to generate encodings that we then try to decode
// during testing.
//
// The resulting object is dissassembled using llvm-objdump, and the
// dissassembly is parsed by parse.py to generate the test vector file used when
// running cargo test.
//
// The test vector file is tracked by git, so CI does not need to have llvm or
// run the parsing script.
//
// This is not a real program, don't try to understand what it does!

.syntax unified
.thumb
.org 0x1000

// ADC (immediate)
// T1
adc     r0, r1, #0
adc     r3, r3, #0
adc     r1, r2, #0xaa00aa00
adc     r11, r12, #0x1fe
adcs    r0, r1, #5

// ADC (register)
// T1
adcs    r0, r1
adcs    r1, r2
adcs    r6, r7
// T2
adc     r0, r1
adc     r1, r2
adc     r11, r12
adc     r1, r2, r3, lsl #10
adc     r2, r3, r4, lsr #31

// ADD (immediate)
// T1
adds    r0, r1, #0
adds    r1, r2, #7
adds    r6, r7, #5
// T2
adds    r0, #0
adds    r1, #0x55
adds    r7, #0xff
// T3
add.w   r0, r1, #0
add.w   r1, r2, #0xaa00aa00
add.w   r11, r12, #0x1fe
adds.w  r2, r3, #0x55
// T4
addw    r0, r1, #0
addw    r1, r2, #0xfff
addw    r11, r12, #0x555

// ADD (register)
// T1
adds    r0, r1, r2
adds    r5, r6, r7
// T2
add     r0, r1
add     r1, r2
add     r12, r7
// T3
add.w   r0, r1, r2, lsl #0
add.w   r1, r2, r3, lsl #31
add.w   r10, r11, r12, lsr #10

// ADD (SP plus immediate)
// T1
add     r0, sp, #0
add     r1, sp, #0x2a8
add     r7, sp, #0x3fc
// T2
add     sp, sp, #0
add     sp, sp, #0x154
add     sp, sp, #0x1fc
// T3
add.w   r0, sp, #0
add.w   r1, sp, #0xaa00aa
add.w   r12, sp, #0x1fe
adds.w  r2, sp, #0x55
// T4
addw    r0, sp, #0
addw    r1, sp, #0xfff
addw    r12, sp, #0x555

// ADD (SP plus register)
// T1
add     r0, sp, r0
add     r1, sp, r1
add     r12, sp, r12
// T2
add     sp, r0
add     sp, r1
add     sp, r12
// T3
add.w   r0, sp, r1
add.w   r1, sp, r2, lsl #1
add.w   r11, sp, r12, lsr #31

// ADR
// T1
adr     r0, label_adr_t1
adr     r1, label_adr_t1
adr     r7, label_adr_t1
nop
nop
nop
label_adr_t1:
// T2
label_adr_t2:
adr     r0, label_adr_t2
adr     r1, label_adr_t2
adr     r14, label_adr_t2
// T3
adr.w   r0, label_adr_t3
adr.w   r1, label_adr_t3
adr.w   r14, label_adr_t3
nop
nop
nop
label_adr_t3:

// AND (immediate)
// T1
and     r0, r1, #0
and     r1, r2, #0xaa00aa00
and     r12, r14, #0x1fe
ands    r2, r3, #5

// AND (register)
// T1
ands    r0, r1
ands    r1, r2
ands    r6, r7
// T2
and.w   r0, r1, r2
and.w   r1, r2, r3, lsl #10
and.w   r11, r12, r14, lsr #31
and.w   r0, r0, r1
ands.w  r2, r3, r4, asr #5

// ASR (immediate)
// T1
asrs    r0, r1, #1
asrs    r1, r2, #10
asrs    r6, r7, #31
// T2
asr.w   r0, r1, #1
asr.w   r1, r2, #10
asr.w   r12, r14, #31
asrs.w  r2, r3, #5

// ASR (register)
// T1
asrs    r0, r1
asrs    r1, r2
asrs    r6, r7
// T2
asr.w   r0, r1, r2
asr.w   r1, r2, r3
asr.w   r11, r12, r14

// B
// T1
label_b:
b       label_b
beq     label_b
bne     label_b
bcs     label_b
bcc     label_b
bmi     label_b
bpl     label_b
bvs     label_b
bvc     label_b
bhi     label_b
bls     label_b
bge     label_b
blt     label_b
ble     label_b
// T2
b       label_far_s12
// T3
b.w     label_b
beq.w   label_b
beq.w   label_b
bne.w   label_b
bcs.w   label_b
bcc.w   label_b
bmi.w   label_b
bpl.w   label_b
bvs.w   label_b
bvc.w   label_b
bhi.w   label_b
bls.w   label_b
bge.w   label_b
blt.w   label_b
ble.w   label_b
// T4
b.w   label_far_s25

// BFC
// T1
bfc     r0, #0, #1
bfc     r1, #0, #5
bfc     r2, #0, #32
bfc     r11, #10, #1
bfc     r11, #10, #5
bfc     r12, #10, #22
bfc     r14, #31, #1

// BFI
// T1
bfi     r0, r1, #0, #1
bfi     r1, r2, #0, #5
bfi     r2, r3, #0, #32
bfi     r11, r12, #10, #1
bfi     r11, r14, #10, #5
bfi     r12, r14, #10, #22
bfi     r14, r11, #31, #1

// BIC (immediate)
// T1
bic     r0, r1, #0
bic     r1, r2, #0xaa00aa00
bic     r2, r3, #0x1fe
bic     r12, r14, #5
bics    r12, r14, #5

// BIC (register)
// T1
bics    r0, r1
bics    r2, r3
bics    r5, r7
// T2
bic.w   r0, r1, r2
bic.w   r1, r2, r3, lsl #10
bic.w   r11, r12, r14, lsl #31
bics.w  r0, r10, r8, lsr #5

// BKPT
// T1
bkpt
bkpt    #5
bkpt    #255

// BL
// T1
label_bl:
bl      label_bl
bl      label_far_s12
bl      label_far_s25

// BLX
// T1
blx     r0
blx     r1
blx     r2
blx     r14

// BX
// T1
bx      r0
bx      r1
bx      r2
bx      r14

// CBNZ
// T1
cbnz    r0, label_cbnz
cbnz    r1, label_cbnz
cbnz    r2, label_cbnz
cbnz    r7, label_cbnz
cbz     r7, label_cbnz
nop // Without this cbz is reduced to nop
label_cbnz:

// CDP
// T1
cdp     p0, #0, c0, c15, c9, #0
cdp     p3, #10, c3, c7, c3, #3
cdp     p5, #15, c9, c6, c4, #0
cdp     p9, #7, c15, c2, c15, #2
cdp     p15, #11, c1, c0, c7, #7
// T2
cdp2    p0, #0, c0, c15, c9, #0
cdp2    p3, #10, c3, c7, c3, #3
cdp2    p5, #15, c9, c6, c4, #0
cdp2    p9, #7, c15, c2, c15, #2
cdp2    p15, #11, c1, c0, c7, #7

// CLREX
// T1
clrex

// CLZ
// T1
clz     r0, r1
clz     r1, r2
clz     r12, r14

// CMN (immediate)
// T1
cmn     r0, #0
cmn     r1, #0xaa00aa00
cmn     r2, #0x1fe
cmn     r14, #5

// CMN (register)
// T1
cmn     r0, r1
cmn     r1, r2
cmn     r2, r3
cmn     r6, r7
// T1
cmn.w   r0, r1
cmn.w   r1, r2, lsl #0
cmn.w   r2, r3, lsl #31
cmn.w   r12, r14, lsr #10

// CMP (immediate)
// T1
cmp     r0, #0
cmp     r1, #10
cmp     r2, #0xaa
cmp     r7, #0xff
// T2
cmp.w   r0, #0
cmp.w   r1, #0xaa00aa00
cmp.w   r2, #0x1fe
cmp.w   r14, #5

// CMP (register)
// T1
cmp     r0, r1
cmp     r1, r2
cmp     r2, r3
cmp     r6, r7
// T2
cmp     r8, r0
cmp     r9, r1
cmp     r2, r10
cmp     r3, r11
cmp     r12, r14
// T3
cmp.w   r0, r1
cmp.w   r1, r2, lsl #0
cmp.w   r2, r3, lsl #31
cmp.w   r12, r14, lsr #10

// CPS
// T1
cpsie   i
cpsie   f
cpsie   fi
cpsid   i
cpsid   f
cpsid   fi

// DMB
// T1
dmb
dmb     #0x0
dmb     #0x5

// DSB
// T1
// dsb #0 is ssbb
dsb
dsb     #0x1
dsb     #0x5

// EOR (immediate)
// T1
eor     r0, r1, #0
eor     r1, r2, #0xaa00aa00
eor     r2, r3, #0x1fe
eor     r12, r14, #5
eors    r3, r4, #10

// EOR (register)
// T1
eors    r0, r1
eors    r1, r2
eors    r2, r3
eors    r6, r7
// T2
eor.w   r0, r1, r2
eor.w   r1, r2, r3, lsl #0
eor.w   r2, r3, r4, lsl #31
eor.w   r11, r12, r14, lsr #10
eors.w  r14, r11, r12, lsr #5

// ISB
// T1
isb
isb     #0x0
isb     #0x5

// IT
// T1
it      eq
nopeq
itt     ne
nopne
nopne
ittt    cs
nopcs
nopcs
nopcs
itttt   cc
nopcc
nopcc
nopcc
nopcc
itett   mi
nopmi
noppl
nopmi
nopmi
ittet   pl
noppl
noppl
nopmi
noppl
ittte   vs
nopvs
nopvs
nopvs
nopvc
iteee   vc
nopvc
nopvs
nopvs
nopvs
itet    hi
nophi
nopls
nophi
itee    ls
nopls
nophi
nophi
itte    ge
nopge
nopge
noplt
ite     lt
noplt
nopge
ite     le
nople
nopgt

// LDC/LDC2 (immediate)
// T1
ldc     p0, c2, [r0]
ldc     p3, c15, [r7, #4]
ldcl    p1, c8, [r7, #-128]
ldc     p15, c0, [r5, #-1020]!
ldcl    p3, c12, [r1], #16
ldc     p3, c0, [r1], {85}
ldcl    p3, c0, [r1], {255}
// T2
ldc2    p0, c2, [r0]
ldc2    p3, c15, [r7, #4]
ldc2l   p1, c8, [r7, #-128]
ldc2    p15, c0, [r5, #-1020]!
ldc2l   p3, c12, [r1], #16
ldc2    p3, c0, [r1], {85}
ldc2l   p3, c0, [r1], {255}

// LDC/LDC2 (literal)
// T1
ldc     p0, c2, [pc]
ldc     p3, c15, [pc, #4]
ldcl    p1, c8, [pc, #-128]
// T2
ldc2    p0, c2, [pc]
ldc2    p3, c15, [pc, #4]
ldc2l   p1, c8, [pc, #-128]

// LDM
// T1
ldm     r0, {r0}
ldm     r1!, {r2, r3}
ldm     r2, {r2, r4, r5}
ldm     r3!, {r2, r5, r6, r7}
ldm     r7, {r5, r6, r7}
// T2
ldm.w   r0, {r0, r1}
ldm.w   r1!, {r2, r4, r6}
ldm.w   r2, {r1, r3, r7, r10}
ldm.w   r7, {r1, r3, r7, r10, r15}

// LDMDB
// T1
ldmdb   r0, {r1, r2}
ldmdb   r1!, {r2, r4, r6, r8}
ldmdb   r14, {r2, r7, r11, r12, r15}

// LDR (immediate)
// T1
ldr.n   r0, [r1, #0]
ldr.n   r1, [r2, #4]
ldr.n   r2, [r3, #124]
ldr.n   r7, [r7, #4]
// T2
ldr.n   r0, [sp, #0]
ldr.n   r1, [sp, #4]
ldr.n   r2, [sp, #680]
ldr.n   r7, [sp, #1020]
// T3
ldr.w   r0, [r1, #0]
ldr.w   r1, [r2, #4]
ldr.w   r2, [r3, #4095]
ldr.w   r14, [r14, #2730]
// T4
//ldr.w   r0, [r1, #-15] // not supported by clang
//ldr.w   r1, [r2, #-255] // not supported by clang
ldr.w   r2, [r3], #10
ldr.w   r3, [r4], #-31
ldr.w   r4, [r5, #-255]!
ldr.w   r15, [r14, #255]!

// LDR (literal)
// T1
.align  4
ldr.n   r0, label_ldr_lit_a
ldr.n   r1, label_ldr_lit_a
ldr.n   r2, label_ldr_lit_a
ldr.n   r7, label_ldr_lit_a
label_ldr_lit_a:
// T2
ldr.w   r0, label_ldr_lit_a
ldr.w   r1, label_ldr_lit_b
ldr.w   r2, label_ldr_lit_a
ldr.w   r7, label_ldr_lit_b
ldr.w   r15, label_ldr_lit_a
label_ldr_lit_b:

// LDR (register)
// T1
ldr.n   r0, [r1, r2]
ldr.n   r1, [r2, r3]
ldr.n   r7, [r7, r7]
// T2
ldr.w   r0, [r1, r2]
ldr.w   r3, [r2, r1]
ldr.w   r7, [r1, r10, lsl #1]
ldr.w   r10, [r8, r9, lsl #3]
ldr.w   r14, [r14, r14]

// LDRB (immediate)
// T1
ldrb.n   r0, [r1, #0]
ldrb.n   r1, [r2, #4]
ldrb.n   r2, [r3, #31]
ldrb.n   r7, [r7, #4]
// T2
ldrb.w   r0, [r1, #0]
ldrb.w   r1, [r2, #4]
ldrb.w   r2, [r3, #4095]
ldrb.w   r14, [r14, #2730]
// T3
ldrb.w   r0, [r1, #-15]
ldrb.w   r1, [r2, #-255]
ldrb.w   r2, [r3], #10
ldrb.w   r3, [r4], #-31
ldrb.w   r4, [r5, #-255]!
ldrb.w   r12, [r14, #255]!

// LDRB (literal)
// T1
label_ldrb_lit_a:
ldrb    r0, label_ldrb_lit_a
ldrb    r1, label_ldrb_lit_b
ldrb    r2, label_ldrb_lit_a
ldrb    r7, label_ldrb_lit_b
ldrb    r14, label_ldrb_lit_a
label_ldrb_lit_b:

// LDRB (register)
// T1
ldrb.n  r0, [r1, r2]
ldrb.n  r1, [r2, r3]
ldrb.n  r7, [r7, r7]
// T2
ldrb.w  r0, [r1, r2]
ldrb.w  r3, [r2, r1]
ldrb.w  r7, [r1, r10, lsl #1]
ldrb.w  r10, [r8, r9, lsl #3]
ldrb.w  r14, [r14, r14]

// LDRBT
// T1
ldrbt   r0, [r1, #0]
ldrbt   r1, [r2, #170]
ldrbt   r11, [r12, #255]

// LDRD (immediate)
// T1
ldrd    r0, r1, [r2, #0]
ldrd    r1, r2, [r3, #168]
ldrd    r2, r3, [r4, #1020]
ldrd    r3, r4, [r5, #-1020]
ldrd    r10, r11, [r12], #40
ldrd    r14, r12, [r9, #40]!

// LDRD (literal)
// T1
label_ldrd_lit_a:
ldrd    r0, r1, label_ldrd_lit_a
ldrd    r1, r2, label_ldrd_lit_b
ldrd    r2, r3, label_ldrd_lit_a
ldrd    r10, r14, label_ldrd_lit_b
label_ldrd_lit_b:

// LDREX
// T1
ldrex   r0, [r1]
ldrex   r2, [r6, #680]
ldrex   r9, [r4, #1020]
ldrex   r7, [r10, #4]
ldrex   r10, [r13]

// LDREXB
// T1
ldrexb  r0, [r0]
ldrexb  r7, [r6]
ldrexb  r12, [r7]
ldrexb  r3, [r14]
ldrexb  r2, [r1]

// LDRH (immediate)
// T1
ldrh.n   r0, [r1, #0]
ldrh.n   r1, [r2, #4]
ldrh.n   r2, [r3, #62]
ldrh.n   r7, [r7, #20]
// T2
ldrh.w   r0, [r1, #0]
ldrh.w   r1, [r2, #4]
ldrh.w   r2, [r3, #4095]
ldrh.w   r14, [r14, #2730]
// T3
ldrh.w   r0, [r1, #-15]
ldrh.w   r1, [r2, #-255]
ldrh.w   r2, [r3], #10
ldrh.w   r3, [r4], #-31
ldrh.w   r4, [r5, #-255]!
ldrh.w   r12, [r14, #255]!

// LDRH (literal)
// T1
label_ldrh_lit_a:
ldrh    r0, label_ldrh_lit_a
ldrh    r1, label_ldrh_lit_b
ldrh    r2, label_ldrh_lit_a
ldrh    r7, label_ldrh_lit_b
ldrh    r14, label_ldrh_lit_a
label_ldrh_lit_b:

// LDRH (register)
// T1
ldrh.n  r0, [r1, r2]
ldrh.n  r1, [r2, r3]
ldrh.n  r7, [r7, r7]
// T2
ldrh.w  r0, [r1, r2]
ldrh.w  r3, [r2, r1]
ldrh.w  r7, [r1, r10, lsl #1]
ldrh.w  r10, [r8, r9, lsl #3]
ldrh.w  r14, [r14, r14]

// LDRHT
// T1
ldrht   r0, [r1, #0]
ldrht   r1, [r2, #170]
ldrht   r2, [r3, #255]
ldrht   r12, [r14, #60]

// LDRSB (immediate)
// T1
ldrsb   r0, [r0, #0]
ldrsb   r1, [r2, #196]
ldrsb   r2, [r3, #4095]
ldrsb   r12, [r14, #3000]
// T2
ldrsb   r0, [r1, #-8]
ldrsb   r1, [r2, #-255]
ldrsb   r2, [r3], #14
ldrsb   r3, [r4], #-35
ldrsb   r12, [r14, #255]!
ldrsb   r10, [r12, #-255]!

// LDRSB (literal)
// T1
label_ldrsb_lit_a:
ldrsb   r0, label_ldrsb_lit_a
ldrsb   r1, label_ldrsb_lit_b
ldrsb   r2, label_ldrsb_lit_a
ldrsb   r7, label_ldrsb_lit_b
ldrsb   r14, label_ldrsb_lit_a
label_ldrsb_lit_b:

// LDRSB (register)
// T1
ldrsb.n r0, [r1, r2]
ldrsb.n r1, [r2, r3]
ldrsb.n r7, [r7, r7]
// T2
ldrsb.w r0, [r1, r2]
ldrsb.w r3, [r2, r1]
ldrsb.w r7, [r1, r10, lsl #1]
ldrsb.w r10, [r8, r9, lsl #3]
ldrsb.w r14, [r14, r14]

// LDRSH (immediate)
// T1
ldrsh   r0, [r0, #0]
ldrsh   r1, [r2, #196]
ldrsh   r2, [r3, #4095]
ldrsh   r12, [r14, #3000]
// T2
ldrsh   r0, [r1, #-8]
ldrsh   r1, [r2, #-255]
ldrsh   r2, [r3], #14
ldrsh   r3, [r4], #-35
ldrsh   r12, [r14, #255]!
ldrsh   r10, [r12, #-255]!

// LDRSH (literal)
// T1
label_ldrsh_lit_a:
ldrsh   r0, label_ldrsh_lit_a
ldrsh   r1, label_ldrsh_lit_b
ldrsh   r2, label_ldrsh_lit_a
ldrsh   r7, label_ldrsh_lit_b
ldrsh   r14, label_ldrsh_lit_a
label_ldrsh_lit_b:

// LDRSH (register)
// T1
ldrsh.n r0, [r1, r2]
ldrsh.n r1, [r2, r3]
ldrsh.n r7, [r7, r7]
// T2
ldrsh.w r0, [r1, r2]
ldrsh.w r3, [r2, r1]
ldrsh.w r7, [r1, r10, lsl #1]
ldrsh.w r10, [r8, r9, lsl #3]
ldrsh.w r14, [r14, r14]

// LDRT
// T1
ldrt    r0, [r1, #0]
ldrt    r1, [r2, #170]
ldrt    r11, [r12, #255]

// LSL (immediate)
// T1
lsls.n  r0, r1, #1
lsls.n  r1, r2, #10
lsls.n  r2, r3, #21
lsls.n  r6, r7, #31
// T2
lsl.w   r0, r1, #1
lsl.w   r1, r2, #10
lsl.w   r2, r3, #21
lsl.w   r12, r14, #31
lsls.w  r9, r8, #31

// LSL (register)
// T1
lsls    r0, r1
lsls    r1, r2
lsls    r2, r3
lsls    r7, r7
// T2
lsl.w   r0, r1, r2
lsl.w   r1, r2, r3
lsl.w   r3, r4, r5
lsl.w   r12, r14, r10

// LSR (immediate)
// T1
lsrs.n  r0, r1, #1
lsrs.n  r1, r2, #10
lsrs.n  r2, r3, #21
lsrs.n  r6, r7, #31
// T2
lsr.w   r0, r1, #1
lsr.w   r1, r2, #10
lsr.w   r2, r3, #21
lsr.w   r12, r14, #31
lsrs.w  r9, r8, #31

// LSR (register)
// T1
lsrs    r0, r1
lsrs    r1, r2
lsrs    r2, r3
lsrs    r7, r7
// T2
lsr.w   r0, r1, r2
lsr.w   r1, r2, r3
lsr.w   r3, r4, r5
lsr.w   r12, r14, r10

// MCR
// T1
mcr     p0, #0, r1, c4, c5
mcr     p12, #2, r0, c0, c0, #2
mcr     p3, #4, r14, c7, c1, #7
mcr     p15, #7, r7, c15, c7, #3
mcr     p2, #1, r6, c2, c15
// T2
mcr2    p0, #0, r1, c4, c5
mcr2    p12, #2, r0, c0, c0, #2
mcr2    p3, #4, r14, c7, c1, #7
mcr2    p15, #7, r7, c15, c7, #3
mcr2    p2, #1, r6, c2, c15

// MCRR
// T1
mcrr    p0, #0, r1, r14, c0
mcrr    p5, #3, r7, r1, c9
mcrr    p7, #7, r14, r0, c3
mcrr    p14, #15, r10, r7, c1
mcrr    p2, #6, r7, r9, c15
// T2
mcrr2   p0, #0, r1, r14, c0
mcrr2   p5, #3, r7, r1, c9
mcrr2   p7, #7, r14, r0, c3
mcrr2   p14, #15, r10, r7, c1
mcrr2   p2, #6, r7, r9, c15

// MLA
// T1
mla     r0, r1, r2, r3
mla     r4, r5, r0, r7
mla     r9, r7, r8, r0
mla     r14, r12, r10, r11

// MLS
// T1
mls     r0, r1, r2, r3
mls     r4, r5, r0, r7
mls     r9, r7, r8, r0
mls     r14, r12, r10, r11

// MOV (immediate)
// T1
movs.n  r0, #0
movs.n  r1, #0xaa
movs.n  r2, #0x55
movs.n  r3, #255
movs.n  r7, #42
// T2
mov.w   r0, #0
mov.w   r1, #0xaa00aa00
mov.w   r2, #0x1fe
movs.w  r10, #63
movs.w  r14, #166
// T3
movw    r0, #0
movw    r1, #0xaaaa
movw    r2, #0x5555
movw    r14, #0

// MOV (register)
// T1
mov.n   r0, r1
mov.n   r1, r2
mov.n   r2, r3
mov.n   r12, r14
// T2
movs.n  r0, r1
movs.n  r1, r2
movs.n  r2, r3
movs.n  r6, r7
// T3
mov.w   r0, r1
mov.w   r1, r2
mov.w   r2, r3
mov.w   r12, r14
movs.w  r14, r12

// MOVT
// T1
movt    r0, #0
movt    r1, #0xaaaa
movt    r2, #0x5555
movt    r14, #65535

// MRC
// T1
mrc     p0, #1, r0, c1, c12
mrc     p15, #7, r12, c3, c9, #7
mrc     p7, #0, r7, c15, c6, #1
mrc     p1, #3, r2, c7, c3, #4
mrc     p6, #2, apsr_nzcv, c1, c2, #3
// T2
mrc2    p0, #1, r0, c1, c12
mrc2    p15, #7, r12, c3, c9, #7
mrc2    p7, #0, r7, c15, c6, #1
mrc2    p1, #3, r2, c7, c3, #4
mrc2    p6, #2, apsr_nzcv, c1, c2, #3

// MRRC
// T1
mrrc    p0, #1, r4, r9, c2
mrrc    p1, #15, r0, r2, c7
mrrc    p15, #6, r9, r7, c0
mrrc    p7, #5, r12, r3, c15
mrrc    p6, #7, r3, r14, c1
// T2
mrrc2   p0, #1, r4, r9, c2
mrrc2   p1, #15, r0, r2, c7
mrrc2   p15, #6, r9, r7, c0
mrrc2   p7, #5, r12, r3, c15
mrrc2   p6, #7, r3, r14, c1

// MRS
// T1
mrs     r14, apsr
mrs     r12, iapsr
mrs     r11, eapsr
mrs     r10, xpsr
mrs     r9, ipsr
mrs     r8, epsr
mrs     r7, msp
mrs     r6, psp
mrs     r5, primask
//mrs     r4, basepri
//mrs     r3, basepri_max
//mrs     r2, faultmask
mrs     r1, control

// MSR
// T1
//msr     ipsr, r14
msr     epsr, r11
msr     msp, r10
msr     psp, r8
//msr     primask, r5
//msr     basepri, r4
//msr     basepri_max, r3
//msr     faultmask, r2
msr     control, r1

// MUL
// T1
muls.n  r0, r1
muls.n  r1, r2
muls.n  r2, r3
muls.n  r7, r7
// T2
mul   r0, r1, r2
mul   r1, r2, r3
mul   r2, r3, r4
mul   r14, r12, r10

.org 0x1830
label_far_s12:

// MVN (immediate)
// T1
mvn     r0, #0
mvn     r1, #0xaa00aa00
mvn     r2, #0x1fe
mvns    r3, #42
mvn     r10, #160
mvns    r14, #255

// MVN (register)
// T1
mvns.n  r0, r1
mvns.n  r1, r2
mvns.n  r2, r3
mvns.n  r7, r7
// T2
mvn.w   r0, r1
mvn.w   r1, r2
mvn.w   r2, r3
mvns.w  r3, r4, lsl #1
mvn.w   r4, r5, lsl #31
mvn.w   r9, r7, lsr #10
mvn.w   r9, r7, lsr #5
mvn.w   r2, r12, asr #12
mvn.w   r10, r14

// NOP
// T1
nop
// T2
nop.w

// ORN (immediate)
// T1
orn     r0, r1, #0
orn     r1, r2, #0xaa00aa00
orn     r2, r3, #0x1fe
orn     r7, r10, #42
orn     r12, r14, #255
orns    r12, r14, #255

// ORN (register)
// T1
orn     r0, r1, r2
orn     r1, r4, r10, lsl #1
orn     r2, r6, r3, lsl #31
orns    r7, r0, r14, lsr #10
orns    r0, r14, r8, asr #5

// ORR (immediate)
// T1
orr     r0, r1, #0
orr     r1, r2, #0xaa00aa00
orr     r2, r3, #0x1fe
orr     r7, r10, #42
orr     r12, r14, #255
orrs    r12, r14, #255

// ORR (register)
// T1
orrs.n  r0, r1
orrs.n  r1, r2
orrs.n  r2, r3
orrs.n  r5, r4
orrs.n  r7, r7
// T2
orr.w  r0, r1, r2
orr.w  r1, r6, r3, lsl #1
orr.w  r7, r10, r14, lsl #31
orr.w  r3, r0, r7, lsr #7
orrs.w  r3, r0, r7, asl #3

// POP
// T1
pop.n   {r0}
pop.n   {r0, r1}
pop.n   {r1, r3, r4, r5}
pop.n   {r2, r6, r7}
// T2
pop.w   {r0, r1}
pop.w   {r1, r3, r7, r10}
pop.w   {r0, r1, r9, r10, r14}
// T3
// TODO: fix collision with LDR depending
//pop.w   {r0}
//pop.w   {r1}
//pop.w   {r2}
//pop.w   {r4}
//pop.w   {r14}

// QADD
// T1
qadd    r0, r1, r2
qadd    r1, r2, r3
qadd    r10, r11, r12
qadd    r2, r2, r4

// QADD8
// T1
qadd8   r0, r1, r2
qadd8   r1, r2, r3
qadd8   r10, r11, r12
qadd8   r2, r2, r4

// QADD16
// T1
qadd16  r0, r1, r2
qadd16  r1, r2, r3
qadd16  r10, r11, r12
qadd16  r2, r2, r4

// QDADD
// T1
qdadd   r0, r1, r2
qdadd   r1, r2, r3
qdadd   r10, r11, r12
qdadd   r2, r2, r4

// QDSUB
// T1
qdsub   r0, r1, r2
qdsub   r1, r2, r3
qdsub   r10, r11, r12
qdsub   r2, r2, r4

// QSUB
// T1
qsub    r0, r1, r2
qsub    r1, r2, r3
qsub    r10, r11, r12
qsub    r2, r2, r4

// QSUB8
// T1
qsub8   r0, r1, r2
qsub8   r1, r2, r3
qsub8   r10, r11, r12
qsub8   r2, r2, r4

// QSUB16
// T1
qsub16  r0, r1, r2
qsub16  r1, r2, r3
qsub16  r10, r11, r12
qsub16  r2, r2, r4

// RBIT
// T1
rbit    r0, r1
rbit    r3, r5
rbit    r2, r8
rbit    r10, r2
rbit    r14, r12

// REV
// T1
rev.n   r0, r1
rev.n   r2, r5
rev.n   r1, r7
rev.n   r6, r3
// T2
rev.w   r0, r1
rev.w   r5, r3
rev.w   r10, r7
rev.w   r3, r14

// REV16
// T1
rev16.n r0, r1
rev16.n r2, r5
rev16.n r1, r7
rev16.n r6, r3
// T2
rev16.w r0, r1
rev16.w r5, r3
rev16.w r10, r7
rev16.w r3, r14

// REVSH
// T1
revsh.n r0, r1
revsh.n r2, r5
revsh.n r1, r7
revsh.n r6, r3
// T2
revsh.w r0, r1
revsh.w r5, r3
revsh.w r10, r7
revsh.w r3, r14

// ROR (immediate)
// T1
ror     r0, r1, #1
ror     r5, r3, #5
ror     r10, r14, #31
ror     r2, r12, #7
rors    r1, r7, #30

// ROR (register)
// T1
rors.n  r0, r1
rors.n  r3, r5
rors.n  r7, r0
rors.n  r4, r2
// T2
ror.w   r0, r3, r7
ror.w   r10, r7, r1
ror.w   r5, r14, r2
rors.w  r7, r6, r0

// RRX
// T1
rrx     r0, r3
rrx     r7, r10
rrx     r12, r3
rrx     r3, r14
rrxs    r1, r10

// RSB (immediate)
// T1
rsbs.n  r0, r1, #0
rsbs.n  r3, r5, #0
rsbs.n  r7, r2, #0
rsbs.n  r1, r0, #0
rsbs.n  r2, r3, #0
// T2
rsb.w   r0, r3, #0
rsb.w   r7, r1, #0xaa00aa00
rsb.w   r10, r12, #0x1fe
rsb.w   r14, r0, #7
rsbs.w  r2, r14, #12

// RSB (register)
// T1
rsb     r0, r3, r2
rsb     r7, r1, r10, lsl #1
rsbs    r12, r7, r10, lsl #31
rsb     r5, r14, r3, asr #1
rsbs    r1, r9, r5, asl #4

// SADD8
// T1
sadd8   r0, r3, r12
sadd8   r8, r10, r1
sadd8   r14, r7, r0
sadd8   r2, r2, r8
sadd8   r3, r9, r7

// SADD16
// T1
sadd16  r0, r3, r12
sadd16  r8, r10, r1
sadd16  r14, r7, r0
sadd16  r2, r2, r8
sadd16  r3, r9, r7

// SBC (immediate)
// T1
sbc     r0, r0, #0
sbc     r5, r1, #0
sbc     r10, r3, #0xaa00aa00
sbcs    r1, r14, #0x1fe
sbc     r7, r12, #65
sbcs    r0, r6, #7

// SBC (register)
// T1
sbcs.n  r0, r0
sbcs.n  r3, r2
sbcs.n  r7, r3
sbcs.n  r2, r1
sbcs.n  r6, r5
// T2
sbc.w   r0, r0, r7
sbc.w   r5, r9, r10, lsl #1
sbcs.w  r3, r4, r14, lsl #31
sbc.w   r0, r14, r7, lsr #6
sbcs.w  r12, r11, r0, lsr #22

// SBFX
// T1
sbfx    r0, r14, #0, #1
sbfx    r7, r11, #10, #20
sbfx    r9, r7, #15, #5
sbfx    r10, r2, #18, #7
sbfx    r14, r0, #31, #1

// SDIV
// T1
//sdiv    r7, r5, r12
//sdiv    r0, r7, r0
//sdiv    r10, r12, r3
//sdiv    r2, r3, r7
//sdiv    r14, r1, r9

// SEV
//sev

// SMLAL
// T1
smlal   r0, r5, r7, r14
smlal   r10, r14, r0, r9
smlal   r12, r9, r12, r5
smlal   r7, r1, r11, r10
smlal   r5, r4, r3, r6

// SMULL
// T1
smull   r0, r5, r7, r14
smull   r10, r14, r0, r9
smull   r12, r9, r12, r5
smull   r7, r1, r11, r10
smull   r5, r4, r3, r6

// SSAT
// T1
ssat    r0, #1, r3
ssat    r10, #32, r0
ssat    r14, #5, r10
ssat    r5, #7, r8
ssat    r2, #20, r7

// STC
// T1
stc     p0, c2, [r0]
stc     p3, c15, [r7, #4]
stcl    p1, c8, [r13, #-128]
stc     p15, c0, [r5, #-1020]!
stcl    p3, c12, [r1], #16
stc     p7, c0, [r12], {85}
stcl    p1, c0, [r1], {255}
// T2
stc2    p0, c2, [r0]
stc2    p3, c15, [r7, #4]
stc2l   p1, c8, [r13, #-128]
stc2    p15, c0, [r5, #-1020]!
stc2l   p3, c12, [r1], #16
stc2    p7, c0, [r12], {85}
stc2l   p1, c0, [r1], {255}

// STM
// T1
stm.n   r0!, {r0}
stm.n   r1!, {r1, r3, r4, r7}
stm.n   r3!, {r2, r5}
stm.n   r5!, {r0, r1, r2, r6, r7}
stm.n   r7!, {r0, r2}
// T2
stm.w   r0!, {r7, r8}
stm.w   r2!, {r1, r3, r10, r11, r14}
stm.w   r8!, {r2, r3, r10, r11, r14}
stm.w   r10!, {r3, r7, r8, r11}
stm.w   r14!, {r0, r5, r11}

// STMDB
// T1
stmdb   r0!, {r1, r2}
stmdb   r2!, {r3, r4, r7, r10}
stmdb   r7!, {r0, r1, r9, r12, r14}
stmdb   r9!, {r1, r3, r7}
stmdb   r14!, {r10, r11}

// STR (immediate)
// T1
str.n   r0, [r1]
str.n   r3, [r5, #4]
str.n   r6, [r0, #64]
str.n   r7, [r3, #44]
str.n   r2, [r6, #124]
// T2
str.n   r0, [sp, #0]
str.n   r1, [sp, #100]
str.n   r7, [sp, #96]
str.n   r5, [sp, #128]
str.n   r2, [sp, #1020]
// T3
str.w   r0, [r1, #0]
str.w   r6, [r12, #7]
str.w   r3, [r6, #4095]
str.w   r10, [r7, #2000]
str.w   r14, [r5, #100]
// T4
//str.w   r0, [r2, #-15] // not supported by clang
//str.w   r1, [r6, #-255] // not supported by clang
str.w   r2, [r3], #10
str.w   r10, [r9], #-12
str.w   r7, [r14, #-255]!
str.w   r0, [r1, #255]!

// STR (register)
// T1
str.n   r0, [r1, r5]
str.n   r3, [r6, r1]
str.n   r7, [r2, r7]
str.n   r2, [r3, r4]
str.n   r1, [r0, r0]
// T2
str.w   r0, [r11, r10]
str.w   r10, [r7, r5, lsl #1]
str.w   r7, [r0, r0, lsl #3]
str.w   r5, [r2, r7]
str.w   r14, [r14, r2, lsl #2]

// STRB (immediate)
// T1
strb.n  r1, [r0, #0]
strb.n  r4, [r7, #1]
strb.n  r7, [r3, #12]
strb.n  r2, [r1, #26]
strb.n  r3, [r2, #31]
// T2
strb.w  r0, [r9, #0]
strb.w  r1, [r5, #100]
strb.w  r10, [r1, #125]
strb.w  r14, [r7, #789]
strb.w  r7, [r12, #4095]
// T3
strb    r0, [r6, #-15]
strb    r2, [r1, #-255]
strb    r14, [r0], #12
strb    r6, [r10], #-23
strb    r9, [r14, #-255]!
strb    r4, [r3, #255]!

// STRB (register)
// T1
strb.n  r0, [r1, r5]
strb.n  r3, [r6, r1]
strb.n  r7, [r2, r7]
strb.n  r2, [r3, r4]
strb.n  r1, [r0, r0]
// T2
strb.w  r0, [r11, r10]
strb.w  r10, [r7, r5, lsl #1]
strb.w  r7, [r0, r0, lsl #3]
strb.w  r5, [r2, r7]
strb.w  r14, [r14, r2, lsl #2]

// STRD
// T1
strd    r0, r1, [r2]
strd    r6, r3, [r9, #100]
strd    r8, r6, [r7, #-100]
strd    r10, r12, [r3, #1020]
strd    r14, r5, [r2], #64
strd    r3, r9, [r10], #-208
strd    r2, r6, [r4, #560]!
strd    r4, r0, [r12, #-440]!

// STREX
// T1
strex   r0, r1, [r2]
strex   r5, r4, [r4, #1020]
strex   r10, r12, [r4, #1020]
strex   r9, r2, [r4, #1020]
strex   r12, r7, [r13, #1020]

// STRH (immediate)
// T1
strh.n  r0, [r1]
strh.n  r2, [r7, #4]
strh.n  r5, [r3, #30]
strh.n  r6, [r0, #36]
strh.n  r7, [r2, #62]
// T2
strh.w  r0, [r2]
strh.w  r5, [r9, #8]
strh.w  r9, [r3, #675]
strh.w  r10, [r1, #546]
strh.w  r14, [r12, #4095]
// T3
strh    r8, [r7, #-100]
strh    r14, [r2], #64
strh    r3, [r10], #-208
strh    r2, [r4, #250]!
strh    r4, [r12, #-255]!

// STRH (register)
// T1
strh.n  r0, [r1, r2]
strh.n  r7, [r0, r6]
strh.n  r3, [r7, r0]
strh.n  r5, [r5, r1]
strh.n  r2, [r3, r4]
// T2
strh.w  r0, [r1, r10]
strh.w  r3, [r14, r7, lsl #1]
strh.w  r12, [r1, r10, lsl #2]
strh.w  r7, [r8, r0]
strh.w  r5, [r9, r7, lsl #3]

// SUB (immediate)
// T1
subs.n  r0, r1, #0
subs.n  r2, r3, #2
subs.n  r4, r6, #4
subs.n  r7, r5, #5
subs.n  r1, r0, #7
// T2
subs.n  r0, #0
subs.n  r1, #9
subs.n  r5, #10
subs.n  r7, #123
subs.n  r3, #255
// T3
sub.w   r0, r6, #0
subs.w  r3, r12, #0xaa00aa00
sub.w   r10, r0, #0x1fe
subs.w  r5, r1, #123
sub.w   r14, r3, #5
subs.w  r7, r9, #42
// T4
subw    r0, r5, #0
subw    r8, r6, #12
subw    r10, r14, #466
subw    r3, r0, #123
subw    r14, r7, #4095

// SUB (register)
// T1
subs.n  r0, r1, r3
subs.n  r3, r0, r5
subs.n  r5, r3, r1
subs.n  r6, r7, r7
subs.n  r7, r2, r0
// T2
sub.w   r0, r7, r3
sub.w   r1, r1, r14, lsl #10
sub.w   r5, r10, r2, lsr #31
sub.w   r10, r0, r4, asr #2
sub.w   r14, r5, r9, asl #12

// SUB (SP minus immediate)
// T1
sub.n   sp, sp, #0
sub.n   sp, sp, #4
sub.n   sp, sp, #128
sub.n   sp, sp, #64
sub.n   sp, sp, #508
// T2
sub.w   r0, sp, #0
sub.w   r1, sp, #0xaa00aa
subs.w  r4, sp, #0x1fe
sub.w   r7, sp, #5
subs.w  r14, sp, #15
// T3
subw    r0, sp, #0
subw    r2, sp, #12
subw    r7, sp, #567
subw    r10, sp, #123
subw    r12, sp, #4095

// SUB (SP minus register)
// T1
sub     r0, sp, r10
sub     r7, sp, r2, lsl #1
subs    r10, sp, r9, lsr #31
sub     r2, sp, r3, asr #5
subs    r14, sp, r7, asl #12

// SVC
svc     #0
svc     #10
svc     #62
svc     #133
svc     #255

// SXTB
// T1
sxtb.n  r0, r1
sxtb.n  r3, r0
sxtb.n  r5, r7
sxtb.n  r7, r4
sxtb.n  r2, r3
// T2
sxtb.w  r0, r1
sxtb.w  r2, r7, ror #8
sxtb.w  r14, r9, ror #16
sxtb.w  r7, r10, ror #24
sxtb.w  r5, r14, ror #24

// SXTH
// T1
sxth.n  r0, r1
sxth.n  r3, r0
sxth.n  r5, r7
sxth.n  r7, r4
sxth.n  r2, r3
// T2
sxth.w  r0, r1
sxth.w  r2, r7, ror #8
sxth.w  r14, r9, ror #16
sxth.w  r7, r10, ror #24
sxth.w  r5, r14, ror #24

// TBB/TBH
// T1
tbb     [r0, r1]
tbb     [r1, r3]
tbb     [r7, r10]
tbb     [r10, r14]
tbb     [r14, r2]
tbh     [r0, r1, lsl #1]
tbh     [r2, r8, lsl #1]
tbh     [r7, r10, lsl #1]
tbh     [r11, r5, lsl #1]
tbh     [r14, r2, lsl #1]

// TEQ (immediate)
// T1
teq     r0, #0
teq     r1, #0xaa00aa00
teq     r7, #0x1fe
teq     r9, #7
teq     r14, #65

// TEQ (register)
// T1
teq     r0, r1
teq     r5, r3, lsl #1
teq     r8, r7, lsl #31
teq     r9, r10, asr #5
teq     r12, r14, asl #9

// TST (immediate)
// T1
tst     r0, #0
tst     r1, #0xaa00aa00
tst     r7, #0x1fe
tst     r9, #7
tst     r14, #65

// TST (register)
// T1
tst.n   r0, r1
tst.n   r2, r3
tst.n   r4, r4
tst.n   r6, r5
tst.n   r7, r7
// T2
tst.w   r0, r1
tst.w   r5, r3, lsl #1
tst.w   r8, r7, lsl #31
tst.w   r9, r10, asr #5
tst.w   r12, r14, asl #9

// UBFX
// T1
ubfx    r0, r14, #0, #1
ubfx    r7, r11, #10, #20
ubfx    r9, r7, #15, #5
ubfx    r10, r2, #18, #7
ubfx    r14, r0, #31, #1

// UDF
// T1
udf     #0
udf     #0xa5
udf     #0xaa
udf     #0x55
udf     #0xff

// UDIV
// T1
//udiv    r7, r5, r12
//udiv    r0, r7, r0
//udiv    r10, r12, r3
//udiv    r2, r3, r7
//udiv    r14, r1, r9

// UMLAL
// T1
umlal   r0, r12, r2, r1
umlal   r3, r9, r8, r14
umlal   r7, r5, r7, r3
umlal   r9, r0, r11, r7
umlal   r14, r1, r6, r5

// UMULL
// T1
umull   r0, r12, r2, r1
umull   r3, r9, r8, r14
umull   r7, r5, r7, r3
umull   r9, r0, r11, r7
umull   r14, r1, r6, r5

// USAT
// T1
usat    r0, 1, r10
usat    r4, 5, r7, lsl #2
usat    r8, 8, r14, asr #7
usat    r12, 31, r2, lsl #16
usat    r14, 0, r0, asr #31

// USAT16
// T1
usat16  r0, 1, r10
usat16  r4, 5, r7
usat16  r8, 8, r14
usat16  r12, 15, r2
usat16  r14, 0, r0

// UXTB
// T1
uxtb.n  r0, r7 
uxtb.n  r1, r4
uxtb.n  r2, r2
uxtb.n  r4, r1
uxtb.n  r7, r0
// T2
uxtb.w  r0, r1
uxtb.w  r5, r3, ror #8
uxtb.w  r7, r8, ror #16
uxtb.w  r9, r10, ror #24
uxtb.w  r12, r11, ror #24

// UXTH
// T1
uxth.n  r0, r7 
uxth.n  r1, r4
uxth.n  r2, r2
uxth.n  r4, r1
uxth.n  r7, r0
// T2
uxth.w  r0, r1
uxth.w  r5, r3, ror #8
uxth.w  r7, r8, ror #16
uxth.w  r9, r10, ror #24
uxth.w  r12, r11, ror #24

// YIELD
// T1
yield
// T2
yield.w

// WFE
// T1
wfe
// T2
wfe.w

// WFI
// T1
wfi
// T2
wfi.w

.org 0x1001000
label_far_s25:

