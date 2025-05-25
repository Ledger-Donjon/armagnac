/// Arm Coprocessor interface definition.
///
/// Systems may include third-party coprocessors, which are accessible using a specific set of Arm
/// instructions. It is possible to implement a coprossessor using this trait and attack it to one
/// of the 16 available slots using [crate::arm::ArmProcessor::set_coprocessor].
pub trait Coprocessor {
    /// Whether the coprocessor accepts the `ins` instruction.
    ///
    /// Corresponds to `Coproc_Accepted()` in the Arm Architecture Reference Manual.
    fn accepted(&self, ins: u32) -> bool;

    /// Determines for an LDC instruction whether enough words have been loaded.
    ///
    /// Corresponds to `Coproc_DoneLoading()` in the Arm Architecture Reference Manual.
    fn done_loading(&self, ins: u32) -> bool;

    /// Determines for an STC instruction whether enough words have been stored.
    ///
    /// Corresponds to `Coproc_DoneStoring()` in the Arm Architecture Reference Manual.
    fn done_storing(&self, ins: u32) -> bool;

    /// Obtain the word for an MRC instruction.
    ///
    /// Corresponds to `Coproc_GetOneWord()` in the Arm Architecture Reference Manual.
    fn get_one_word(&mut self, ins: u32) -> u32;

    /// Obtain two words for a, MRRC instruction.
    ///
    /// Corresponds to `Coproc_GetTwoWords()` in the Arm Architecture Reference Manual.
    fn get_two_words(&mut self, ins: u32) -> (u32, u32);

    /// Obtain the next word to store for an STC instruction from the coprocessor.
    ///
    /// Corresponds to `Coproc_GetWordToStore()` in the Arm Architecture Reference Manual.
    fn get_word_to_store(&mut self, ins: u32) -> u32;

    /// Instruct the coprocessor to perform the internal operation requested by a CDP instruction.
    ///
    /// Corresponds to `Coproc_InternalOperation()` in the Arm Architecture Reference Manual.
    fn internal_operation(&mut self, ins: u32);

    /// Send a loaded word for an LDC instruction to the processor.
    ///
    /// Corresponds to `Coproc_SendLoadedWord()` in the Arm Architecture Reference Manual.
    fn send_loaded_word(&mut self, word: u32, ins: u32);

    /// Send a word for an MCR instruction to the coprocessor.
    ///
    /// Corresponds to `Coproc_SendOneWord()` in the Arm Architecture Reference Manual.
    fn send_one_word(&mut self, word: u32, ins: u32);

    /// Send two words for an MCRR instruction to the coprocessor.
    ///
    /// Corresponds to `Coproc_SendTwoWords()` in the Arm Architecture Reference Manual.
    fn send_two_words(&mut self, word1: u32, word2: u32, ins: u32);
}
