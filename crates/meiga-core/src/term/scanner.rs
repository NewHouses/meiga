//! The scanner that reads the output of the program before the parser, to
//! find the sequences that `alacritty_terminal` ignores and meiga needs
//! (ADR 0007). It only reads the bytes: the parser gets them unchanged.
//!
//! For now it follows one: win32-input-mode (`CSI ? 9001 h` and `l`), which
//! ConPTY turns on when it starts, so that meiga sends it whole key events
//! (subphase 1.7). It reads the control sequences as `vte` does, so that it
//! sees a change of mode only when `vte` would see one, also when the
//! sequence comes split between two reads. It only follows byte by byte the
//! sequences that start with `ESC [ ?`, the private modes: it skips the
//! text, and the rest of the sequences, looking for the next ESC with
//! `memchr`, many bytes at a time.

/// The private mode of win32-input-mode.
const WIN32_INPUT_MODE: u16 = 9001;

/// The most numbers that a control sequence can have, subparameters
/// included. As in `vte`, a sequence with more is ignored.
const MAX_VALUES: u8 = 32;

/// The most intermediate bytes, the `?` of a private mode included. As in
/// `vte`, a sequence with more is ignored.
const MAX_INTERMEDIATES: u8 = 2;

const ESC: u8 = 0x1b;

/// Where the scanner is, as the states of `vte` that matter here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    /// Text, or a sequence that does not matter: until the next ESC.
    #[default]
    Ground,
    /// After ESC.
    Escape,
    /// After `ESC [`.
    CsiEntry,
    /// Reading the numbers of a private sequence, after `ESC [ ?`.
    CsiParam,
    /// After another intermediate byte of a private sequence, such as `$`.
    CsiIntermediate,
    /// A private sequence that `vte` ignores, until its final byte.
    CsiIgnore,
}

/// Finds the changes of the modes that `alacritty_terminal` ignores.
#[derive(Clone, Debug)]
pub struct Scanner {
    state: State,
    /// How many intermediate bytes the private sequence has, its `?`
    /// included.
    intermediates: u8,
    /// How many numbers it has, subparameters included.
    values: u8,
    /// The number that it is reading.
    value: u16,
    /// The next number starts a parameter: it is not a subparameter, after
    /// `:`. Only the first number of each parameter is a mode.
    parameter_start: bool,
    /// `vte` will ignore the sequence: it has too many numbers or
    /// intermediates.
    ignoring: bool,
    /// One of the parameters of the sequence is 9001.
    has_win32_input_mode: bool,
    win32_input_mode: bool,
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner {
    pub const fn new() -> Self {
        Self {
            state: State::Ground,
            intermediates: 0,
            values: 0,
            value: 0,
            parameter_start: true,
            ignoring: false,
            has_win32_input_mode: false,
            win32_input_mode: false,
        }
    }

    /// Whether the program turned win32-input-mode on.
    pub const fn win32_input_mode(&self) -> bool {
        self.win32_input_mode
    }

    /// Reads the next bytes of the output.
    pub fn scan(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        loop {
            if self.state == State::Ground {
                // Nothing matters until the next ESC: memchr finds it many
                // bytes at a time.
                let mut escapes = memchr::memchr_iter(ESC, rest);
                let at = loop {
                    let Some(at) = escapes.next() else {
                        return;
                    };
                    // The usual case, as colours and cursor movements: `ESC
                    // [` and a byte after which the sequence cannot change a
                    // private mode. The same as reading them one by one.
                    if let Some(&[b'[', next]) = rest.get(at + 1..at + 3)
                        && Self::not_private(next)
                    {
                        continue;
                    }
                    break at;
                };
                self.state = State::Escape;
                rest = rest.get(at + 1..).unwrap_or_default();
                continue;
            }
            let Some((&byte, after)) = rest.split_first() else {
                return;
            };
            rest = after;
            self.advance(byte);
        }
    }

    /// Whether `byte`, right after `ESC [`, means that the sequence is not a
    /// private one, and cannot change a private mode.
    const fn not_private(byte: u8) -> bool {
        // Only `?` marks a private mode. Control characters, DEL and bytes
        // over 0x7f do not change the sequence: the next byte decides.
        !matches!(byte, b'?' | 0x00..=0x1f | 0x7f..=0xff)
    }

    /// One byte of a sequence, as `vte` reads it.
    fn advance(&mut self, byte: u8) {
        match byte {
            // In any state, ESC starts a new sequence, and CAN and SUB
            // cancel the current one.
            ESC => self.state = State::Escape,
            0x18 | 0x1a => self.state = State::Ground,
            _ => match self.state {
                State::Ground => {}
                State::Escape => self.advance_escape(byte),
                State::CsiEntry => self.advance_csi_entry(byte),
                State::CsiParam => self.advance_csi_param(byte),
                State::CsiIntermediate => self.advance_csi_intermediate(byte),
                State::CsiIgnore => {
                    if (0x40..=0x7e).contains(&byte) {
                        self.state = State::Ground;
                    }
                }
            },
        }
    }

    fn advance_escape(&mut self, byte: u8) {
        match byte {
            // The other control characters, DEL and the bytes over 0x7f do
            // not end the escape: `vte` ignores them, and the next byte
            // chooses the sequence.
            0x00..=0x1f | 0x7f..=0xff => {}
            b'[' => self.state = State::CsiEntry,
            // RIS, the full reset, turns every mode off.
            b'c' => {
                self.win32_input_mode = false;
                self.state = State::Ground;
            }
            // Another escape, or a string (OSC, DCS…) that only an ESC or a
            // control character can end.
            _ => self.state = State::Ground,
        }
    }

    fn advance_csi_entry(&mut self, byte: u8) {
        if Self::not_private(byte) {
            // Colours, cursor movements… The rest of the sequence has no ESC,
            // so it can be read as text: the next ESC ends it, or starts
            // another sequence.
            self.state = State::Ground;
        } else if byte == b'?' {
            self.start_private();
            self.state = State::CsiParam;
        }
    }

    fn advance_csi_param(&mut self, byte: u8) {
        match byte {
            0x20..=0x2f => {
                self.collect();
                self.state = State::CsiIntermediate;
            }
            b'0'..=b'9' => self.digit(byte),
            b':' => self.subparameter(),
            b';' => self.parameter(),
            // A private marker after the numbers.
            0x3c..=0x3f => self.state = State::CsiIgnore,
            0x40..=0x7e => self.dispatch(byte),
            _ => {}
        }
    }

    fn advance_csi_intermediate(&mut self, byte: u8) {
        match byte {
            0x20..=0x2f => self.collect(),
            0x30..=0x3f => self.state = State::CsiIgnore,
            0x40..=0x7e => self.dispatch(byte),
            _ => {}
        }
    }

    /// `ESC [ ?`: the start of a sequence that can set or reset a private
    /// mode. The `?` is its first intermediate byte.
    fn start_private(&mut self) {
        self.intermediates = 1;
        self.values = 0;
        self.value = 0;
        self.parameter_start = true;
        self.ignoring = false;
        self.has_win32_input_mode = false;
    }

    fn collect(&mut self) {
        if self.intermediates == MAX_INTERMEDIATES {
            self.ignoring = true;
        } else {
            self.intermediates += 1;
        }
    }

    fn digit(&mut self, byte: u8) {
        if self.values == MAX_VALUES {
            self.ignoring = true;
            return;
        }
        self.value = self
            .value
            .saturating_mul(10)
            .saturating_add(u16::from(byte - b'0'));
    }

    /// `:`: the number ends, and the next one is a subparameter.
    fn subparameter(&mut self) {
        if self.end_value() {
            self.parameter_start = false;
        }
    }

    /// `;`: the number ends, and the next one starts a parameter.
    fn parameter(&mut self) {
        if self.end_value() {
            self.parameter_start = true;
        }
    }

    /// Adds the number that it was reading. Returns false if there was no
    /// room for it.
    fn end_value(&mut self) -> bool {
        if self.values == MAX_VALUES {
            self.ignoring = true;
            return false;
        }
        self.values += 1;
        if self.parameter_start && self.value == WIN32_INPUT_MODE {
            self.has_win32_input_mode = true;
        }
        self.value = 0;
        true
    }

    /// The final byte of the sequence.
    fn dispatch(&mut self, byte: u8) {
        self.end_value();
        self.state = State::Ground;
        // `vte` only sets or resets private modes with `?` as the only
        // intermediate byte, and when it does not ignore the sequence.
        if self.ignoring || self.intermediates != 1 || !self.has_win32_input_mode {
            return;
        }
        match byte {
            b'h' => self.win32_input_mode = true,
            b'l' => self.win32_input_mode = false,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::vte::ansi::{Handler, PrivateMode, Processor};

    use super::*;

    /// Whether win32-input-mode is on after `parts`, read one after the
    /// other.
    fn scanned(parts: &[&[u8]]) -> bool {
        let mut scanner = Scanner::new();
        for part in parts {
            scanner.scan(part);
        }
        scanner.win32_input_mode()
    }

    #[test]
    fn it_follows_the_mode_on_and_off() {
        assert!(scanned(&[b"\x1b[?9001h"]));
        assert!(!scanned(&[b"\x1b[?9001h", b"text\x1b[?9001l"]));
        assert!(!scanned(&[b"text without sequences"]));
    }

    #[test]
    fn it_finds_the_mode_among_others() {
        assert!(scanned(&[b"\x1b[?1004;9001h"]));
        assert!(!scanned(&[b"\x1b[?1004;9001h\x1b[?9001;1004l"]));
        assert!(!scanned(&[b"\x1b[?1004h"]));
    }

    #[test]
    fn it_follows_the_mode_when_the_sequence_is_split() {
        // What ConPTY writes when it starts.
        let start = b"\x1b[1t\x1b[c\x1b[?1004h\x1b[?9001h\x1b[?25l";
        for at in 0..=start.len() {
            let (first, second) = start.split_at(at);
            assert!(scanned(&[first, second]), "split at {at}");
        }
        let one_by_one: Vec<&[u8]> = start.chunks(1).collect();
        assert!(scanned(&one_by_one));
    }

    #[test]
    fn only_a_private_set_or_reset_changes_the_mode() {
        // Without `?`, with another marker, with another intermediate, and
        // with another final byte.
        assert!(!scanned(&[b"\x1b[9001h"]));
        assert!(!scanned(&[b"\x1b[>9001h"]));
        assert!(!scanned(&[b"\x1b[?9001$h"]));
        assert!(!scanned(&[b"\x1b[??9001h"]));
        assert!(!scanned(&[b"\x1b[?9001p"]));
    }

    #[test]
    fn only_the_first_number_of_a_parameter_is_a_mode() {
        assert!(scanned(&[b"\x1b[?9001:1h"]));
        assert!(!scanned(&[b"\x1b[?1:9001h"]));
    }

    #[test]
    fn control_characters_cancel_or_restart_the_sequence() {
        // CAN cancels it; ESC starts another one; a line feed is executed
        // and the sequence goes on.
        assert!(!scanned(&[b"\x1b[?90\x1801h"]));
        assert!(scanned(&[b"\x1b[?90\x1b[?9001h"]));
        assert!(scanned(&[b"\x1b[?90\n01h"]));
        // An OSC string ends at ESC, which starts the sequence.
        assert!(scanned(&[b"\x1b]0;title\x1b[?9001h"]));
    }

    #[test]
    fn a_full_reset_turns_the_mode_off() {
        assert!(!scanned(&[b"\x1b[?9001h", b"\x1bc"]));
        // ESC ( c designates a character set: it is not a reset.
        assert!(scanned(&[b"\x1b[?9001h", b"\x1b(c"]));
    }

    #[test]
    fn bytes_over_0x7e_do_not_end_an_escape() {
        // vte ignores them after ESC: the next byte still chooses the
        // sequence.
        assert!(scanned(&[b"\x1b\x80[?9001h"]));
        assert!(!scanned(&[b"\x1b[?9001h", "\x1b\u{e1}c".as_bytes()]));
        assert!(!scanned(&[b"\x1b[?9001h", b"\x1b\x7f", b"\xffc"]));
    }

    #[test]
    fn a_sequence_with_too_many_numbers_is_ignored() {
        // 32 numbers fit; with 33, vte ignores the sequence.
        let fits = format!("\x1b[?{}9001h", "1;".repeat(31));
        assert!(scanned(&[fits.as_bytes()]));
        let too_many = format!("\x1b[?{}9001h", "1;".repeat(32));
        assert!(!scanned(&[too_many.as_bytes()]));
    }

    /// Follows win32-input-mode as alacritty_terminal would, if it did not
    /// ignore it: the reference for the scanner.
    #[derive(Default)]
    struct Reference {
        win32_input_mode: bool,
    }

    impl Handler for Reference {
        fn set_private_mode(&mut self, mode: PrivateMode) {
            if mode == PrivateMode::Unknown(WIN32_INPUT_MODE) {
                self.win32_input_mode = true;
            }
        }

        fn unset_private_mode(&mut self, mode: PrivateMode) {
            if mode == PrivateMode::Unknown(WIN32_INPUT_MODE) {
                self.win32_input_mode = false;
            }
        }

        fn reset_state(&mut self) {
            self.win32_input_mode = false;
        }
    }

    #[test]
    fn it_agrees_with_vte_on_random_output() {
        // The bytes that matter for the sequences, and a few others. There
        // is no 2 or 6, so that no synchronized update (?2026) starts.
        const BYTES: &[u8] =
            b"\x1b\x1b[[??;;::0019hhlcp$> \n\x07\x18\x1a]\x7fa\xc3\xa1\x9b\x80\xff";
        // Whole sequences, and their beginnings, so that the mode changes
        // often.
        const SEQUENCES: &[&[u8]] = &[
            b"\x1b[?9001h",
            b"\x1b[?9001l",
            b"\x1b[?1004;9001h",
            b"\x1bc",
            b"\x1b[",
            b"\x1b[?",
            b"9001",
            b"\x1b]0;title\x07",
        ];
        let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut next = move |below: usize| {
            // xorshift64: a fixed sequence, so that a failure repeats.
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % below as u64) as usize
        };
        for case in 0..2_000 {
            // Half of the cases start with the mode on, so that turning it
            // off is checked too.
            let mut output = if case % 2 == 0 {
                b"\x1b[?9001h".to_vec()
            } else {
                Vec::new()
            };
            for _ in 0..=next(100) {
                if next(4) == 0 {
                    output.extend_from_slice(SEQUENCES[next(SEQUENCES.len())]);
                } else {
                    output.push(BYTES[next(BYTES.len())]);
                }
            }
            let mut scanner = Scanner::new();
            let mut parser: Processor = Processor::new();
            let mut reference = Reference::default();
            // In pieces of random length, comparing after each one.
            let mut rest = output.as_slice();
            while !rest.is_empty() {
                let (piece, after) = rest.split_at((1 + next(16)).min(rest.len()));
                rest = after;
                scanner.scan(piece);
                parser.advance(&mut reference, piece);
                assert_eq!(
                    scanner.win32_input_mode(),
                    reference.win32_input_mode,
                    "case {case}: {output:?}"
                );
            }
        }
    }
}
