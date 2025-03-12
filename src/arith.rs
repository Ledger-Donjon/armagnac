use std::fmt::Display;

pub enum ArithError {
    Unpredictable,
}

/// Calculate the sum of two 32-bit words and a carry bit, and calculate carry
/// and overflow status bits. Returns the tuple `(result, carry, overflow)`.
///
/// # Arguments
///
/// * `x` - First operand
/// * `y` - Second operand
/// * `carry_in` - Input carry bit
pub fn add_with_carry(x: u32, y: u32, carry_in: bool) -> (u32, bool, bool) {
    let carry_in = if carry_in { 1 } else { 0 };
    let unsigned_sum = x as u64 + y as u64 + carry_in as u64;
    let signed_sum = x as i32 as i64 + y as i32 as i64 + carry_in as i64;
    let result = x.wrapping_add(y).wrapping_add(carry_in as u32);
    let carry_out = if result as u64 == unsigned_sum {
        false
    } else {
        true
    };
    let overflow = if result as i32 as i64 == signed_sum {
        false
    } else {
        true
    };
    (result, carry_out, overflow)
}

/// Returns the arithmetic shift right of `value` and the carry output.
pub fn asr_c(value: u32, shift: u32) -> (u32, bool) {
    assert!((shift > 0) && (shift < 32));
    (
        ((value as i32) >> shift) as u32,
        value & (1 << shift - 1) != 0,
    )
}

/// Returns left shifted value and carry out.
///
/// # Arguments
///
/// * `value` - Value to be shifted
/// * `shift` - Shift count. Must be > 0.
pub fn lsl_c(value: u32, shift: u32) -> (u32, bool) {
    assert!(shift > 0);
    // Max shift by 63 to avoid overflow errors.
    // We cannot use wrapping_shl here since it does not conform to what ARM implements when
    // shift >= 32.
    let extended = (value as u64) << shift.min(63);
    (extended as u32, extended & 1 << 32 != 0)
}

/// Returns right shifted value and carry out.
///
/// # Arguments
///
/// * `value` - Value to be shifted
/// * `shift` - Shift count. Must be > 0.
pub fn lsr_c(value: u32, shift: u32) -> (u32, bool) {
    assert!(shift > 0);
    (value >> shift, value & (1 << shift - 1) != 0)
}

/// Returns right rotated value and carry out.
///
/// # Arguments
///
/// * `value` - Value to be rotated
/// * `shift` - Shift count. Must be > 0.
pub fn ror_c(value: u32, shift: u32) -> (u32, bool) {
    const N: u32 = 32;
    assert!(shift > 0);
    let m = shift % N;
    let result = lsr_c(value, m).0 | lsl_c(value, N - m).0;
    (result, result & (1 << N - 1) != 0)
}

/// Returns `value` shifted to the right with most significant set from the input carry. The output
/// carry is the initial value least significant bit.
pub fn rrx_c(value: u32, carry_in: bool) -> (u32, bool) {
    let mut result = value >> 1;
    if carry_in {
        result |= 0x80000000;
    }
    (result, value & 1 != 0)
}

/// Returns right rotated value and discard the carry.
///
/// # Arguments
///
/// * `value` - Value to be rotated
/// * `shift` - Shift count. 0 is allowed.
pub fn ror(value: u32, shift: u32) -> u32 {
    if shift != 0 {
        ror_c(value, shift).0
    } else {
        value
    }
}

/// Sign-extends a word.
///
/// # Arguments
///
/// * `value` - Value to be sign-extended, passed as a row unsigned word.
/// * `width` - Number of bits in the value.
pub fn sign_extend(value: u32, width: u8) -> i32 {
    (value << (32 - width)) as i32 >> (32 - width)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShiftType {
    Lsl,
    Lsr,
    Asr,
    Ror,
    Rrx,
}

impl Display for ShiftType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Lsl => "lsl",
            Self::Lsr => "lsr",
            Self::Asr => "asr",
            Self::Ror => "ror",
            Self::Rrx => "rrx",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Shift {
    pub t: ShiftType,
    pub n: u32,
}

impl Shift {
    pub fn from_bits(t: u32, imm5: u32) -> Self {
        assert!(imm5 < 0x20);
        match t & 3 {
            0 => Shift {
                t: ShiftType::Lsl,
                n: imm5,
            },
            1 => Shift {
                t: ShiftType::Lsr,
                n: if imm5 == 0 { 32 } else { imm5 },
            },
            2 => Shift {
                t: ShiftType::Asr,
                n: if imm5 == 0 { 32 } else { imm5 },
            },
            3 => {
                if imm5 == 0 {
                    Shift {
                        t: ShiftType::Rrx,
                        n: 1,
                    }
                } else {
                    Shift {
                        t: ShiftType::Ror,
                        n: imm5,
                    }
                }
            }
            _ => panic!(),
        }
    }

    /// Return an empty string if the shift is null, or a string representation of the shift
    /// prefixed with ", " to be used in instruction string representation.
    pub fn arg_string(&self) -> String {
        if self.n != 0 {
            format!(", {self}")
        } else {
            "".into()
        }
    }

    pub fn asr(n: u32) -> Self {
        Self {
            t: ShiftType::Asr,
            n,
        }
    }

    pub fn lsl(n: u32) -> Self {
        Self {
            t: ShiftType::Lsl,
            n,
        }
    }

    pub fn lsr(n: u32) -> Self {
        Self {
            t: ShiftType::Lsr,
            n,
        }
    }

    pub fn ror(n: u32) -> Self {
        Self {
            t: ShiftType::Ror,
            n,
        }
    }

    pub fn rrx() -> Self {
        Self {
            t: ShiftType::Rrx,
            n: 1,
        }
    }
}

impl Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} #{}", self.t, self.n)
    }
}

/// Returns shifted value and carry out.
///
/// # Arguments
///
/// * `value` - Value to be shifted
/// * `shift` - Shift type and amount
/// * `carry_in` - Input carry bit
pub fn shift_c(value: u32, shift: Shift, carry_in: bool) -> (u32, bool) {
    if shift.n == 0 {
        (value, carry_in)
    } else {
        match shift.t {
            ShiftType::Lsl => lsl_c(value, shift.n),
            ShiftType::Lsr => lsr_c(value, shift.n),
            ShiftType::Asr => asr_c(value, shift.n),
            ShiftType::Ror => ror_c(value, shift.n),
            ShiftType::Rrx => {
                debug_assert_eq!(shift.n, 1);
                rrx_c(value, carry_in)
            }
        }
    }
}

/// Returns immediate value expansion from thumb instruction encoding.
/// Result is the expansion value and an eventual carry output depending on the type of expansion.
///
/// If instruction is unpredictable, [ArithError::Unpredictable] is returned.
pub fn thumb_expand_imm_optc(imm12: u32) -> Result<(u32, Option<bool>), ArithError> {
    debug_assert!(imm12 < 0x1000);
    if imm12 >> 10 & 3 == 0 {
        let imm8 = imm12 & 0xff;
        match imm12 >> 8 & 3 {
            0 => Ok((imm8, None)),
            1 => {
                if imm8 == 0 {
                    Err(ArithError::Unpredictable)
                } else {
                    Ok((imm8 << 16 | imm8, None))
                }
            }
            2 => {
                if imm8 == 0 {
                    Err(ArithError::Unpredictable)
                } else {
                    Ok((imm8 << 24 | imm8 << 8, None))
                }
            }
            3 => {
                if imm8 == 0 {
                    Err(ArithError::Unpredictable)
                } else {
                    Ok((imm8 << 24 | imm8 << 16 | imm8 << 8 | imm8, None))
                }
            }
            _ => panic!(),
        }
    } else {
        // Rotate
        let (result, carry_out) = ror_c((1 << 7) | (imm12 & 0x7f), (imm12 >> 7) & 0x1f);
        Ok((result, Some(carry_out)))
    }
}

/// Returns immediate value expansion from thumb instruction encoding.
///
/// If instruction is unpredictable, `Err(())` is returned.
pub fn thumb_expand_imm(imm12: u32) -> Result<u32, ArithError> {
    thumb_expand_imm_optc(imm12).map(|x| x.0)
}

/// Returns immediate value expansion from thumb instruction encoding.
///
/// If instruction is unpredictable, `Err(())` is returned.
pub fn thumb_expand_imm_c(imm12: u32, carry_in: bool) -> Result<(u32, bool), ArithError> {
    let (result, carry_out) = thumb_expand_imm_optc(imm12)?;
    Ok((
        result,
        match carry_out {
            Some(c) => c,
            None => carry_in,
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::arith::{
        add_with_carry, asr_c, lsl_c, lsr_c, ror, ror_c, rrx_c, sign_extend, Shift,
    };

    use super::{shift_c, ShiftType};

    #[test]
    fn test_add_with_carry() {
        assert_eq!(add_with_carry(0, 0, false), (0, false, false));
        assert_eq!(add_with_carry(0xffffffff, 1, false), (0, true, false));
        assert_eq!(add_with_carry(0xffffffff, 0, true), (0, true, false));
        assert_eq!(
            add_with_carry(0x7fffffff, 1, false),
            (0x80000000, false, true)
        );
        assert_eq!(
            add_with_carry(0x7fffffff, 0, true),
            (0x80000000, false, true)
        );
    }

    #[test]
    fn test_asr_c() {
        for i in 1..=31 {
            let magic = 0x33e8628f;
            assert_eq!(asr_c(magic, i), (magic >> i, magic & (1 << (i - 1)) != 0));
        }
        for i in 1..=31 {
            let magic = 0xb3e8628f;
            let mask = u32::MAX << (32 - i);
            assert_eq!(
                asr_c(magic, i),
                (magic >> i | mask, magic & (1 << (i - 1)) != 0)
            );
        }
        assert_eq!(asr_c(0x80000000, 8), (0xff800000, false));
        assert_eq!(asr_c(0x80000000, 31), (0xffffffff, false));
        assert_eq!(asr_c(0xc0000000, 31), (0xffffffff, true));
    }

    #[test]
    fn test_lsl_c() {
        for i in 1..=31 {
            assert_eq!(lsl_c(1, i), (1 << i, false));
        }
        assert_eq!(lsl_c(1, 32), (0, true));
        assert_eq!(lsl_c(1, 33), (0, false));
    }

    #[test]
    fn test_lsr_c() {
        assert_eq!(lsr_c(0, 1), (0, false));
        assert_eq!(lsr_c(1, 1), (0, true));
        assert_eq!(lsr_c(2, 1), (1, false));
        for i in 1..=31 {
            assert_eq!(
                lsr_c(0xb3e8628f, i),
                (0xb3e8628f >> i, 0xb3e8628fu32 & (1 << (i - 1)) != 0)
            );
        }
    }

    #[test]
    fn test_ror_c() {
        assert_eq!(ror_c(0, 1), (0, false));
        assert_eq!(ror_c(1, 1), (0x80000000, true));
        assert_eq!(ror_c(1, 31), (2, false));
        assert_eq!(ror_c(0x00a50000, 8), (0x0000a500, false));
        assert_eq!(ror_c(0x00a50000, 16), (0x000000a5, false));
        assert_eq!(ror_c(0x00a50000, 24), (0xa5000000, true));
    }

    #[test]
    fn test_ror() {
        // Only test with 0 shift, allowed for ror (but not for ror_c).
        // For the rest we can count on test_ror_c.
        assert_eq!(ror(0x12345678, 0), 0x12345678);
    }

    #[test]
    fn test_rrx_c() {
        assert_eq!(rrx_c(0x12345678, false), (0x091a2b3c, false));
        assert_eq!(rrx_c(0x12345678, true), (0x891a2b3c, false));
        assert_eq!(rrx_c(0x87654321, true), (0xc3b2a190, true));
    }

    #[test]
    fn test_sign_extend() {
        for width in 1..32 {
            assert_eq!(sign_extend(0, width), 0)
        }
        assert_eq!(sign_extend(1, 1), -1);
        assert_eq!(sign_extend(1, 2), 1);
        assert_eq!(sign_extend(3, 2), -1);
        assert_eq!(sign_extend(3, 3), 3);
        assert_eq!(sign_extend(0x7fffffff, 31), -1);
        assert_eq!(sign_extend(0x7fffffff, 32), 0x7fffffff);
    }

    #[test]
    fn test_shift_c() {
        let value = 0xf0ea918b;
        for i in 1..32 {
            assert_eq!(
                shift_c(
                    value,
                    Shift {
                        t: ShiftType::Lsl,
                        n: i
                    },
                    false
                ),
                lsl_c(value, i)
            );
            assert_eq!(
                shift_c(
                    value,
                    Shift {
                        t: ShiftType::Lsr,
                        n: i
                    },
                    false
                ),
                lsr_c(value, i)
            );
        }
    }
}
