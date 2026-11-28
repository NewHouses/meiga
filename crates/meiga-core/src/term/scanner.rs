//! The scanner that reads the output of the program before the parser, to
//! find what the shell says about itself, which `vte` ignores (ADR 0007 and
//! ADR 0032): its folder, with OSC 7 or OSC 9;9, and where it is between its
//! prompt and the output of a command, with OSC 133. It only reads the
//! bytes: the parser gets them unchanged.
//!
//! An OSC starts at `ESC ]`. The scanner looks for its `]` with `memchr`,
//! many bytes at a time, and checks that an ESC comes right before it, so it
//! does not stop at the other sequences, as the colours, nor at the text; in
//! a read with many `]` without an ESC, as a list, it looks for the pair with
//! `memchr::memmem` instead. From there it reads the OSC as `vte` does, also
//! when it comes split between reads, and it ends it where `vte` ends it: at
//! BEL, CAN, SUB or any ESC. The only OSC that it misses is one with bytes
//! between the ESC and the `]` that do not end the escape in `vte`: the
//! controls, which it runs, and DEL and the bytes from 0x80, which it
//! ignores, as `ESC \x80 ]`. No program writes them.

use memchr::memmem::Finder;

use super::shell_state::{Folder, Mark};

/// The most bytes of text that the scanner keeps of an OSC that it reads:
/// the longest path of Windows, 32 767 characters, fits in UTF-8 with its
/// quotes. A longer OSC is ignored whole: the scanner never uses a piece of
/// one.
const MAX_TEXT: usize = 128 * 1024;

/// The memory that the text keeps between two OSCs: after a longer one, it
/// gives the rest back.
const KEPT_CAPACITY: usize = 4 * 1024;

/// How many `]` without an ESC before them a read can have before the scanner
/// looks for the pair `ESC ]` instead ([`Scanner::next_start`]).
const LONE_BRACKETS: usize = 16;

const BEL: u8 = 0x07;
const CAN: u8 = 0x18;
const SUB: u8 = 0x1a;
const ESC: u8 = 0x1b;

/// What the shell said, in the order that it said it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Report {
    /// Its folder: OSC 7 or OSC 9;9.
    Folder(Folder),
    /// Where it is: OSC 133.
    Mark(Mark),
}

/// The OSCs that the scanner reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// OSC 7: the folder as a URL.
    Url,
    /// OSC 9, of ConEmu, whose 9;9 is the folder as a path of Windows.
    ConEmu,
    /// OSC 133: the marks around the prompt and the commands.
    Mark,
}

/// Where the scanner is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// Looking for the next `ESC ]`. `escape`: the last read ended with an
    /// ESC, which the next one can follow with `]`.
    Ground { escape: bool },
    /// After `ESC ]`: the number of the OSC, until its first `;`.
    Number,
    /// The text of an OSC that the scanner reads, until its end.
    Text(Kind),
}

/// Finds what the shell says in the output of the program.
#[derive(Clone, Debug)]
pub struct Scanner {
    finder: Finder<'static>,
    state: State,
    /// The number of the OSC, while it can still be one of those that the
    /// scanner reads: 7, 9 or 133.
    number: [u8; 3],
    number_len: usize,
    /// The text of the OSC, without the bytes that `vte` ignores in it.
    text: Vec<u8>,
    /// The text is longer than [`MAX_TEXT`]: the OSC is ignored.
    too_long: bool,
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            finder: Finder::new(b"\x1b]"),
            state: State::Ground { escape: false },
            number: [0; 3],
            number_len: 0,
            text: Vec::new(),
            too_long: false,
        }
    }

    /// Reads the next bytes of the output, and gives `report` what the shell
    /// said in them, in order.
    pub fn scan(&mut self, bytes: &[u8], mut report: impl FnMut(Report)) {
        let mut rest = bytes;
        while !rest.is_empty() {
            rest = match self.state {
                State::Ground { escape } => self.find_start(rest, escape),
                State::Number => self.read_number(rest),
                State::Text(kind) => self.read_text(rest, kind, &mut report),
            };
        }
    }

    /// Looks for the next `ESC ]` of an OSC that can be one that the scanner
    /// reads, and returns what comes after it.
    #[inline]
    fn find_start<'a>(&mut self, bytes: &'a [u8], escape: bool) -> &'a [u8] {
        if escape && bytes[0] == b']' {
            self.start();
            return &bytes[1..];
        }
        let mut from = 0;
        let mut lone = 0;
        while let Some(at) = self.next_start(bytes, from, &mut lone) {
            from = at + 2;
            match bytes.get(from) {
                // The number of an OSC that the scanner does not read, as the
                // 8 of the links: it goes on as text, until the next ESC ].
                // An ESC that ends it right away can start that one.
                Some(&byte)
                    if !matches!(
                        byte,
                        b'1' | b'7' | b'9' | 0x00..=0x06 | 0x08..=0x17 | 0x19 | 0x1c..=0x1f
                    ) => {}
                _ => {
                    self.start();
                    return &bytes[from..];
                }
            }
        }
        self.state = State::Ground {
            escape: bytes.last() == Some(&ESC),
        };
        &[]
    }

    /// Where the next `ESC ]` of `bytes` is, from `from`. `memchr` finds a
    /// `]` faster than `memmem` finds the pair, and `]` is rare in most
    /// output, also in the one full of colours, where ESC is not; after
    /// [`LONE_BRACKETS`] of them without an ESC before them, as in lists, the
    /// rest goes to `memmem`. `lone` counts them.
    #[inline]
    fn next_start(&self, bytes: &[u8], mut from: usize, lone: &mut usize) -> Option<usize> {
        while *lone < LONE_BRACKETS {
            let at = from + memchr::memchr(b']', &bytes[from..])?;
            // A `]` at the start goes with an ESC of the last read, which
            // `escape` follows.
            if at > 0 && bytes[at - 1] == ESC {
                return Some(at - 1);
            }
            *lone += 1;
            from = at + 1;
        }
        self.finder.find(&bytes[from..]).map(|at| from + at)
    }

    /// `ESC ]`: an OSC starts.
    fn start(&mut self) {
        self.state = State::Number;
        self.number_len = 0;
    }

    /// Reads the number of the OSC, and returns what comes after the part of
    /// `bytes` that it read.
    fn read_number<'a>(&mut self, bytes: &'a [u8]) -> &'a [u8] {
        for (at, &byte) in bytes.iter().enumerate() {
            match byte {
                // The controls that `vte` ignores in an OSC.
                0x00..=0x06 | 0x08..=0x17 | 0x19 | 0x1c..=0x1f => {}
                // An OSC with only a number: none that the scanner reads.
                BEL | CAN | SUB => {
                    self.state = State::Ground { escape: false };
                    return &bytes[at + 1..];
                }
                // It ends, and a new sequence starts, maybe another OSC.
                ESC => {
                    self.state = State::Ground { escape: false };
                    return &bytes[at..];
                }
                b';' => {
                    self.state = match &self.number[..self.number_len] {
                        b"7" => self.start_text(Kind::Url),
                        b"9" => self.start_text(Kind::ConEmu),
                        b"133" => self.start_text(Kind::Mark),
                        _ => State::Ground { escape: false },
                    };
                    return &bytes[at + 1..];
                }
                _ => {
                    if self.number_len == self.number.len() {
                        self.state = State::Ground { escape: false };
                        return &bytes[at + 1..];
                    }
                    self.number[self.number_len] = byte;
                    self.number_len += 1;
                    // Any other OSC goes on as text, until the next ESC ].
                    if !matches!(
                        &self.number[..self.number_len],
                        b"7" | b"9" | b"1" | b"13" | b"133"
                    ) {
                        self.state = State::Ground { escape: false };
                        return &bytes[at + 1..];
                    }
                }
            }
        }
        &[]
    }

    /// Starts the text of an OSC of `kind`: [`Scanner::end_text`] left the
    /// text of the last one empty.
    fn start_text(&mut self, kind: Kind) -> State {
        self.too_long = false;
        State::Text(kind)
    }

    /// Reads the text of an OSC of `kind`, gives `report` what it says when it
    /// ends, and returns what comes after the part of `bytes` that it read.
    fn read_text<'a>(
        &mut self,
        bytes: &'a [u8],
        kind: Kind,
        report: &mut impl FnMut(Report),
    ) -> &'a [u8] {
        for (at, &byte) in bytes.iter().enumerate() {
            match byte {
                0x00..=0x06 | 0x08..=0x17 | 0x19 | 0x1c..=0x1f => {}
                BEL | CAN | SUB => {
                    self.end_text(kind, report);
                    return &bytes[at + 1..];
                }
                ESC => {
                    self.end_text(kind, report);
                    return &bytes[at..];
                }
                _ => {
                    if self.text.len() == MAX_TEXT {
                        self.too_long = true;
                    } else {
                        self.text.push(byte);
                    }
                }
            }
        }
        &[]
    }

    /// The OSC ended: what it says goes to `report`, unless it was too long.
    fn end_text(&mut self, kind: Kind, report: &mut impl FnMut(Report)) {
        self.state = State::Ground { escape: false };
        if !self.too_long
            && let Some(said) = read(kind, &self.text)
        {
            report(said);
        }
        self.text.clear();
        self.text.shrink_to(KEPT_CAPACITY);
    }
}

/// What the text of an OSC of `kind` says, if it says something that the
/// terminal keeps.
fn read(kind: Kind, text: &[u8]) -> Option<Report> {
    match kind {
        Kind::Url => url_folder(text).map(Report::Folder),
        Kind::ConEmu => conemu_folder(text).map(Report::Folder),
        Kind::Mark => mark(text).map(Report::Mark),
    }
}

/// OSC 7: `file://computer/path`, with the computer and the path
/// percent-encoded. Any computer: what to do with another one is for whoever
/// uses the folder.
fn url_folder(text: &[u8]) -> Option<Folder> {
    const SCHEME: &[u8] = b"file://";
    let (scheme, rest) = text.split_at_checked(SCHEME.len())?;
    if !scheme.eq_ignore_ascii_case(SCHEME) {
        return None;
    }
    let slash = memchr::memchr(b'/', rest)?;
    let (host, path) = rest.split_at(slash);
    Some(Folder::Url {
        host: decode(host)?,
        path: decode(path)?,
    })
}

/// The text of a URL, with each `%XX` as the byte XX: a `%` without two
/// hexadecimal digits stays as it is. `None` if the result is not UTF-8.
fn decode(text: &[u8]) -> Option<String> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut rest = text;
    while let Some((&byte, after)) = rest.split_first() {
        if byte == b'%'
            && let [high, low, ..] = after
            && let (Some(high), Some(low)) = (hex(*high), hex(*low))
        {
            bytes.push(high << 4 | low);
            rest = &after[2..];
        } else {
            bytes.push(byte);
            rest = after;
        }
    }
    String::from_utf8(bytes).ok()
}

const fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// OSC 9;9, of ConEmu: the path of Windows, after `9;`, as it comes or
/// between quotes. The other OSC 9, as the progress (9;4) or the
/// notifications, are not folders.
fn conemu_folder(text: &[u8]) -> Option<Folder> {
    let path = text.strip_prefix(b"9;")?;
    let path = match path {
        [b'"', inside @ .., b'"'] => inside,
        _ => path,
    };
    if path.is_empty() {
        return None;
    }
    String::from_utf8(path.to_vec()).ok().map(Folder::Windows)
}

/// OSC 133: a letter, and for D the exit code, if it is a number. The other
/// letters, and the options after them, are ignored, as the proposal of the
/// semantic prompts asks.
fn mark(text: &[u8]) -> Option<Mark> {
    let mut fields = text.split(|&byte| byte == b';');
    match fields.next()? {
        b"A" => Some(Mark::Prompt),
        b"B" => Some(Mark::Command),
        b"C" => Some(Mark::Output),
        b"D" => Some(Mark::Done(fields.next().and_then(exit_code))),
        _ => None,
    }
}

/// An exit code: a number that fits in an `i32`, or one of 32 bits without
/// sign, as Windows gives them, which ends up as the `i32` that Windows
/// keeps: 3221225477 is -1073741819.
fn exit_code(field: &[u8]) -> Option<i32> {
    let text = std::str::from_utf8(field).ok()?;
    text.parse::<i32>()
        .ok()
        .or_else(|| text.parse::<u32>().ok().map(u32::cast_signed))
}

#[cfg(test)]
mod tests;
