use crate::condition::Condition;

#[derive(Clone, Copy)]
pub struct ItState(pub u8);

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ItThenElse {
    Then,
    Else,
}

/// ARMv7-M processor IT block state.
///
/// The state is stored in the same encoding as specified in the Architecture Reference Manual.
impl ItState {
    pub fn new() -> Self {
        Self(0)
    }

    /// Try to create a new IT state, fails if any condition combination lead to the invalid
    /// condition code 0xf.
    pub fn try_new(value: u8) -> Result<Self, ()> {
        let first_cond = value >> 4;
        let mask = value & 0xf;
        match (first_cond, mask) {
            (0b1111, _) => Err(()),
            (0b1110, 0b0001 | 0b0010 | 0b0100 | 0b1000) => Ok(Self(value)),
            (0b1110, _) => Err(()),
            (_, _) => Ok(Self(value)),
        }
    }

    /// Advance after normal execution of an IT block instruction.
    pub fn advance(&mut self) {
        if self.0 & 7 == 0 {
            self.0 = 0
        } else {
            self.0 = self.0 & 0xe0 | (self.0 << 1) & 0x1f;
        }
    }

    /// Returns true if execution is currently in an IT block.
    pub fn in_it_block(&self) -> bool {
        return self.0 & 0xf != 0;
    }

    /// Returns true if execution is currently in an IT block, and on the last instruction.
    pub fn last_in_it_block(&self) -> bool {
        return self.0 & 0xf == 8;
    }

    /// Returns true if execution is currently in an IT block, and not in the last instruction.
    pub fn in_it_block_not_last(&self) -> bool {
        self.in_it_block() & !self.last_in_it_block()
    }

    /// Returns current condition of the IT block, or None if currently not in an IT block.
    pub fn current_condition(&self) -> Option<Condition> {
        if self.in_it_block() {
            // Internal value is private and should not have forbidden values, so unwrap here
            // should be fine.
            Some(((self.0 >> 4) as u32).try_into().unwrap())
        } else {
            None
        }
    }

    /// Translates the current IT state to a serie of [ItThenElse::Then] or [ItThenElse::Else].
    pub fn to_then_else(&self) -> Vec<ItThenElse> {
        let mut result = Vec::new();
        let mut mask = self.0 & 0xf;
        while mask & 7 != 0 {
            if mask >> 3 & 1 == self.0 >> 4 & 1 {
                result.push(ItThenElse::Then)
            } else {
                result.push(ItThenElse::Else)
            }
            mask = (mask << 1) & 0xf;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        condition::Condition,
        it_state::{ItState, ItThenElse},
    };

    #[test]
    fn test_try_new() {
        assert!(ItState::try_new(0).is_ok());
        assert!(ItState::try_new(0b11110000).is_err());
        assert!(ItState::try_new(0b11101100).is_err());
        assert!(ItState::try_new(0b11100110).is_err());
        assert!(ItState::try_new(0b11100011).is_err());
        assert!(ItState::try_new(0b11101000).is_ok());
        assert!(ItState::try_new(0b11100100).is_ok());
        assert!(ItState::try_new(0b11100010).is_ok());
        assert!(ItState::try_new(0b11100001).is_ok());
    }

    #[test]
    fn test_advance() {
        let mut state = ItState::try_new(0b00100001).unwrap();
        assert_eq!(state.current_condition(), Some(Condition::CarrySet));
        state.advance();
        assert_eq!(state.current_condition(), Some(Condition::CarrySet));
        state.advance();
        assert_eq!(state.current_condition(), Some(Condition::CarrySet));
        state.advance();
        assert_eq!(state.current_condition(), Some(Condition::CarrySet));
        state.advance();
        assert_eq!(state.current_condition(), None);

        let mut state = ItState::try_new(0b00100100).unwrap();
        assert_eq!(state.current_condition(), Some(Condition::CarrySet));
        state.advance();
        assert_eq!(state.current_condition(), Some(Condition::CarrySet));
        state.advance();
        assert_eq!(state.current_condition(), None);

        let mut state = ItState::try_new(0b10101011).unwrap();
        assert_eq!(
            state.current_condition(),
            Some(Condition::GreaterThanOrEqual)
        );
        state.advance();
        assert_eq!(state.current_condition(), Some(Condition::LessThan));
        state.advance();
        assert_eq!(
            state.current_condition(),
            Some(Condition::GreaterThanOrEqual)
        );
        state.advance();
        assert_eq!(state.current_condition(), Some(Condition::LessThan));
        state.advance();
        assert_eq!(state.current_condition(), None);
    }

    #[test]
    fn test_in_it_block() {
        let mut state = ItState::try_new(0b00100001).unwrap();
        assert_eq!(state.in_it_block(), true);
        assert_eq!(state.last_in_it_block(), false);
        assert_eq!(state.in_it_block_not_last(), true);
        state.advance();
        assert_eq!(state.in_it_block(), true);
        assert_eq!(state.last_in_it_block(), false);
        assert_eq!(state.in_it_block_not_last(), true);
        state.advance();
        assert_eq!(state.in_it_block(), true);
        assert_eq!(state.last_in_it_block(), false);
        assert_eq!(state.in_it_block_not_last(), true);
        state.advance();
        assert_eq!(state.in_it_block(), true);
        assert_eq!(state.last_in_it_block(), true);
        assert_eq!(state.in_it_block_not_last(), false);
        state.advance();
        assert_eq!(state.in_it_block(), false);
        assert_eq!(state.last_in_it_block(), false);
        assert_eq!(state.in_it_block_not_last(), false);
    }

    #[test]
    fn test_to_then_else() {
        let state = ItState::try_new(0b10101011).unwrap();
        assert_eq!(
            state.to_then_else(),
            vec![ItThenElse::Else, ItThenElse::Then, ItThenElse::Else]
        );
    }
}
