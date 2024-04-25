use std::fmt::{self, Display};

/// Possible conditions for conditional execution.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Condition {
    Equal,
    NotEqual,
    CarrySet,
    CarryClear,
    Minus,
    Plus,
    Overflow,
    NoOverflow,
    Higher,
    LowerOrSame,
    GreaterThanOrEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    Always,
}

impl Condition {
    /// Returns inverse condition, or None if current condition is Always (there is no "Never"
    /// condition in ARMv7).
    pub fn inverse(&self) -> Option<Self> {
        Some(match self {
            Condition::Equal => Condition::NotEqual,
            Condition::NotEqual => Condition::Equal,
            Condition::CarrySet => Condition::CarryClear,
            Condition::CarryClear => Condition::CarrySet,
            Condition::Minus => Condition::Plus,
            Condition::Plus => Condition::Minus,
            Condition::Overflow => Condition::NoOverflow,
            Condition::NoOverflow => Condition::Overflow,
            Condition::Higher => Condition::LowerOrSame,
            Condition::LowerOrSame => Condition::Higher,
            Condition::GreaterThanOrEqual => Condition::LessThan,
            Condition::LessThan => Condition::GreaterThanOrEqual,
            Condition::GreaterThan => Condition::LessThanOrEqual,
            Condition::LessThanOrEqual => Condition::GreaterThan,
            Condition::Always => return None,
        })
    }
}

impl TryFrom<u32> for Condition {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Condition::Equal),
            1 => Ok(Condition::NotEqual),
            2 => Ok(Condition::CarrySet),
            3 => Ok(Condition::CarryClear),
            4 => Ok(Condition::Minus),
            5 => Ok(Condition::Plus),
            6 => Ok(Condition::Overflow),
            7 => Ok(Condition::NoOverflow),
            8 => Ok(Condition::Higher),
            9 => Ok(Condition::LowerOrSame),
            10 => Ok(Condition::GreaterThanOrEqual),
            11 => Ok(Condition::LessThan),
            12 => Ok(Condition::GreaterThan),
            13 => Ok(Condition::LessThanOrEqual),
            14 => Ok(Condition::Always),
            _ => Err(()),
        }
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Condition::Equal => "eq",
            Condition::NotEqual => "ne",
            Condition::CarrySet => "cs",
            Condition::CarryClear => "cc",
            Condition::Minus => "mi",
            Condition::Plus => "pl",
            Condition::Overflow => "vs",
            Condition::NoOverflow => "vc",
            Condition::Higher => "hi",
            Condition::LowerOrSame => "ls",
            Condition::GreaterThanOrEqual => "ge",
            Condition::LessThan => "lt",
            Condition::GreaterThan => "gt",
            Condition::LessThanOrEqual => "le",
            Condition::Always => "",
        };
        write!(f, "{}", s)
    }
}
