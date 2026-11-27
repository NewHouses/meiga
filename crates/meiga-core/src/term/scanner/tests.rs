use alacritty_terminal::vte::{Parser, Perform};
use proptest::prelude::*;

use super::*;

/// What the scanner finds in `parts`, read one after the other.
fn scanned(parts: &[&[u8]]) -> Vec<Report> {
    let mut scanner = Scanner::new();
    let mut found = Vec::new();
    for part in parts {
        scanner.scan(part, |report| found.push(report));
    }
    found
}

fn windows(path: &str) -> Report {
    Report::Folder(Folder::Windows(path.to_owned()))
}

fn url(host: &str, path: &str) -> Report {
    Report::Folder(Folder::Url {
        host: host.to_owned(),
        path: path.to_owned(),
    })
}

fn done(code: Option<i32>) -> Report {
    Report::Mark(Mark::Done(code))
}

#[test]
fn it_finds_the_folder_of_osc_7_and_osc_9_9() {
    assert_eq!(
        scanned(&[b"\x1b]7;file://ana-pc/C:/x%20y\x07"]),
        [url("ana-pc", "/C:/x y")]
    );
    assert_eq!(
        scanned(&[b"\x1b]7;file:///home/ana\x1b\\"]),
        [url("", "/home/ana")]
    );
    assert_eq!(
        scanned(&[b"\x1b]9;9;\"C:\\x y\"\x1b\\"]),
        [windows("C:\\x y")]
    );
    assert_eq!(scanned(&[b"\x1b]9;9;C:\\x\x07"]), [windows("C:\\x")]);
}

#[test]
fn it_finds_the_marks_of_osc_133() {
    let marks =
        scanned(&[b"\x1b]133;A\x07\x1b]133;B\x07\x1b]133;C\x07\x1b]133;D;0\x07\x1b]133;D\x1b\\"]);
    let expected = [
        Report::Mark(Mark::Prompt),
        Report::Mark(Mark::Command),
        Report::Mark(Mark::Output),
        done(Some(0)),
        done(None),
    ];
    assert_eq!(marks, expected);
}

#[test]
fn the_exit_code_is_a_number_of_32_bits() {
    let code = |text: &str| scanned(&[format!("\x1b]133;D;{text}\x07").as_bytes()]);
    assert_eq!(code("1"), [done(Some(1))]);
    assert_eq!(code("-1"), [done(Some(-1))]);
    assert_eq!(code("+7"), [done(Some(7))]);
    assert_eq!(code("2147483647"), [done(Some(i32::MAX))]);
    assert_eq!(code("-2147483648"), [done(Some(i32::MIN))]);
    // As Windows gives them, without a sign: the i32 that it keeps.
    assert_eq!(code("3221225477"), [done(Some(-1_073_741_819))]);
    assert_eq!(code("4294967295"), [done(Some(-1))]);
    // Not a number of 32 bits: the command ended, with no code.
    assert_eq!(code("4294967296"), [done(None)]);
    assert_eq!(code("-2147483649"), [done(None)]);
    assert_eq!(code("x"), [done(None)]);
    assert_eq!(code(""), [done(None)]);
    assert_eq!(code(" 1"), [done(None)]);
    // The options after the code are ignored, and an option where the code
    // goes is not a code.
    assert_eq!(code("2;err=x"), [done(Some(2))]);
    assert_eq!(code("aid=7"), [done(None)]);
}

#[test]
fn other_letters_and_options_of_osc_133_are_ignored() {
    assert_eq!(
        scanned(&[b"\x1b]133;A;aid=1;cl=m\x07\x1b]133;B;k=i\x07"]),
        [Report::Mark(Mark::Prompt), Report::Mark(Mark::Command)]
    );
    for text in ["P;k=i", "N", "L", "I", "a", "Ax", "AB", "", ";A"] {
        let osc = format!("\x1b]133;{text}\x07");
        assert_eq!(scanned(&[osc.as_bytes()]), [], "{text:?}");
    }
}

#[test]
fn what_is_not_a_folder_is_ignored() {
    for osc in [
        // Not a URL of a file, or without a path.
        &b"\x1b]7;http://ana-pc/x\x07"[..],
        b"\x1b]7;file:/x\x07",
        b"\x1b]7;file://ana-pc\x07",
        b"\x1b]7;\x07",
        b"\x1b]7\x07",
        // The other OSC 9: progress, a notification.
        b"\x1b]9;4;1;50\x07",
        b"\x1b]9;hello\x07",
        b"\x1b]9;99;x\x07",
        // An empty path, also between quotes.
        b"\x1b]9;9;\x07",
        b"\x1b]9;9;\"\"\x07",
        // Not UTF-8, also once the %XX are decoded.
        b"\x1b]9;9;C:\\\xff\x07",
        b"\x1b]7;file://ana-pc/%ff\x07",
        // Other numbers, as vte reads them: 07 is not 7.
        b"\x1b]07;file://ana-pc/x\x07",
        b"\x1b]9 ;9;C:\\x\x07",
        b"\x1b]1337;CurrentDir=/x\x07",
        b"\x1b]0;title\x07",
    ] {
        assert_eq!(scanned(&[osc]), [], "{osc:?}");
    }
}

#[test]
fn the_url_is_percent_decoded() {
    let folder = |text: &str| scanned(&[format!("\x1b]7;{text}\x07").as_bytes()]);
    assert_eq!(folder("file://h/%41%62%2f"), [url("h", "/Ab/")]);
    // A % without two hexadecimal digits stays.
    assert_eq!(folder("file://h/%4"), [url("h", "/%4")]);
    assert_eq!(folder("file://h/%zz%"), [url("h", "/%zz%")]);
    assert_eq!(folder("file://h/%%41"), [url("h", "/%A")]);
    // The computer too, and the scheme in capitals.
    assert_eq!(folder("FILE://ana%2Dpc/x"), [url("ana-pc", "/x")]);
    // Characters of more than one byte.
    assert_eq!(
        folder("file://h/%C3%A1/\u{e9}"),
        [url("h", "/\u{e1}/\u{e9}")]
    );
}

#[test]
fn only_the_quotes_around_the_path_go() {
    let folder = |path: &str| scanned(&[format!("\x1b]9;9;{path}\x07").as_bytes()]);
    assert_eq!(folder("\"C:\\x\""), [windows("C:\\x")]);
    assert_eq!(folder("\"C:\\x"), [windows("\"C:\\x")]);
    assert_eq!(folder("C:\\x\""), [windows("C:\\x\"")]);
    assert_eq!(folder("\""), [windows("\"")]);
    // Everything after 9;9;, also with ;, which a path of Windows can have.
    assert_eq!(folder("C:\\a;b"), [windows("C:\\a;b")]);
    assert_eq!(folder("\"C:\\a;b\""), [windows("C:\\a;b")]);
}

#[test]
fn an_osc_ends_as_in_vte() {
    // BEL, ST, any ESC, CAN and SUB end it, and it counts.
    for end in [
        &b"\x07"[..],
        b"\x1b\\",
        b"\x1b[0m",
        b"\x1b",
        b"\x18",
        b"\x1a",
    ] {
        let output = [&b"\x1b]133;A"[..], end].concat();
        assert_eq!(
            scanned(&[&output]),
            [Report::Mark(Mark::Prompt)],
            "{output:?}"
        );
    }
    // An ESC that ends one can start the next one.
    assert_eq!(
        scanned(&[b"\x1b]133;C\x1b]133;D;1\x1b]9;9;C:\\x\x07"]),
        [Report::Mark(Mark::Output), done(Some(1)), windows("C:\\x")]
    );
}

#[test]
fn the_controls_that_vte_ignores_in_an_osc_are_not_part_of_it() {
    assert_eq!(
        scanned(&[b"\x1b]1\n33;\rA\x07"]),
        [Report::Mark(Mark::Prompt)]
    );
    assert_eq!(
        scanned(&[b"\x1b]9;9;C:\\\x00x\x7f\x07"]),
        [windows("C:\\x\x7f")]
    );
}

#[test]
fn other_oscs_and_sequences_do_not_hide_the_next_one() {
    let output = concat!(
        "\x1b]0;title\x07\x1b]8;;http://x/\x1b\\link\x1b]8;;\x1b\\",
        "\x1b]52;c;b2xh\x07\x1b]1;icon\x07\x1b]21;dec\x1b\\\x1b]1337;x\x07",
        "\x1b[38;2;1;2;3mtext ] [1]\x1bP1$r\x1b\\\x1b_apc\x1b\\",
        "\x1b]7;file://h/x\x07",
    );
    assert_eq!(scanned(&[output.as_bytes()]), [url("h", "/x")]);
    // A ] without an ESC right before it does not start an OSC.
    assert_eq!(scanned(&[b"]133;A\x07 x]7;file://h/y\x07"]), []);
}

#[test]
fn it_finds_them_when_they_come_split() {
    let output = concat!(
        "pwsh\x1b[0m\x1b]133;D;0\x07\x1b]133;A\x07PS C:\\>\x1b]9;9;\"C:\\x y\"\x1b\\",
        "\x1b]133;B\x07dir\r\n\x1b]133;C\x07\x1b]8;;file:///x\x1b\\x\x1b]8;;\x1b\\",
        "\x1b]0;t\x07\x1b]7;file://h/a%20b\x1b\\\x1b]133;D;1\x1b[m",
    )
    .as_bytes();
    let expected = [
        done(Some(0)),
        Report::Mark(Mark::Prompt),
        windows("C:\\x y"),
        Report::Mark(Mark::Command),
        Report::Mark(Mark::Output),
        url("h", "/a b"),
        done(Some(1)),
    ];
    assert_eq!(scanned(&[output]), expected);
    for at in 0..=output.len() {
        let (first, second) = output.split_at(at);
        assert_eq!(scanned(&[first, second]), expected, "split at {at}");
    }
    let one_by_one: Vec<&[u8]> = output.chunks(1).collect();
    assert_eq!(scanned(&one_by_one), expected);
}

#[test]
fn an_esc_at_the_end_of_a_read_can_start_one_in_the_next() {
    assert_eq!(
        scanned(&[b"text\x1b", b"]133;A\x07"]),
        [Report::Mark(Mark::Prompt)]
    );
    assert_eq!(
        scanned(&[b"\x1b", b"\x1b", b"]133;A\x07"]),
        [Report::Mark(Mark::Prompt)]
    );
    // An empty read changes nothing.
    assert_eq!(
        scanned(&[b"\x1b", b"", b"]133;A\x07"]),
        [Report::Mark(Mark::Prompt)]
    );
    assert_eq!(scanned(&[b"\x1b", b"x]133;A\x07"]), []);
    assert_eq!(scanned(&[b"\x1b", b"[", b"]133;A\x07"]), []);
}

#[test]
fn it_misses_an_osc_with_bytes_that_vte_ignores_after_the_esc() {
    // vte reads them as an OSC; no program writes them.
    assert_eq!(scanned(&[b"\x1b\x80]133;A\x07"]), []);
    assert_eq!(scanned(&[b"\x1b\n]133;A\x07"]), []);
    // Another ESC is not one of them: the OSC starts at the last one.
    assert_eq!(
        scanned(&[b"\x1b\x1b]133;A\x07"]),
        [Report::Mark(Mark::Prompt)]
    );
}

#[test]
fn a_text_longer_than_the_maximum_is_ignored_whole() {
    let path = |len: usize| format!("C:\\{}", "x".repeat(len - 3));
    let osc = |path: &str| format!("\x1b]9;9;{path}\x07");
    // With its 9;, MAX_TEXT bytes fit.
    let fits = path(MAX_TEXT - 2);
    assert_eq!(scanned(&[osc(&fits).as_bytes()]), [windows(&fits)]);
    let longer = path(MAX_TEXT - 1);
    let mut scanner = Scanner::new();
    let mut found = Vec::new();
    // In pieces, and followed by another one, which counts.
    for piece in osc(&longer).as_bytes().chunks(1000) {
        scanner.scan(piece, |report| found.push(report));
    }
    scanner.scan(b"\x1b]133;A\x07", |report| found.push(report));
    assert_eq!(found, [Report::Mark(Mark::Prompt)]);
    // The text gave its memory back.
    assert!(scanner.text.capacity() <= KEPT_CAPACITY);
}

/// What vte reads in an output: the OSCs that it ends, with their
/// parameters, as the scanner has to see them.
#[derive(Default)]
struct Reference {
    found: Vec<Report>,
    /// An OSC had the 16 parameters that vte keeps at most: the text after
    /// them is not in them.
    cut: bool,
}

impl Perform for Reference {
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.len() == 16 {
            self.cut = true;
        }
        let kind = match params.first().copied() {
            Some(b"7") => Kind::Url,
            Some(b"9") => Kind::ConEmu,
            Some(b"133") => Kind::Mark,
            _ => return,
        };
        // An OSC with only its number has no text.
        if params.len() < 2 {
            return;
        }
        if let Some(report) = read(kind, &params[1..].join(&b';')) {
            self.found.push(report);
        }
    }
}

/// The bytes after an ESC that vte ignores, staying after the ESC: the
/// scanner misses an OSC after them.
const fn ignored_after_esc(byte: u8) -> bool {
    matches!(byte, 0x00..=0x17 | 0x19 | 0x1c..=0x1f | 0x7f..=0xff)
}

/// Pieces of output: text, colours, the OSCs that the scanner reads, other
/// OSCs and strings, the starts and the ends of all of them, and any byte.
fn part() -> impl Strategy<Value = Vec<u8>> {
    let pieces: &[&[u8]] = &[
        b"\x1b",
        b"\x1b]",
        b"\x1b]1",
        b"\x1b]13",
        b"\x1b]133",
        b"\x1b]133;",
        b"\x1b]7",
        b"\x1b]7;",
        b"\x1b]9;",
        b"\x1b]9;9;",
        b"\x1b]0;title",
        b"\x1b]8;;http://x/",
        b"\x1b]52;c;b2xh",
        b"\x1b]1337;x",
        b"\x1bP1$r",
        b"\x1b_apc",
        b"\x1b[",
        b"\x1b[0m",
        b"\x1b[38;2;1;2;3m",
        b"\x1b\\",
        b"\x1bc",
        b"]",
        b";",
        b"\x07",
        b"\x18",
        b"\x1a",
        b"\r\n",
    ];
    let texts: &[&str] = &[
        "7;file://ana-pc/C:/x%20y",
        "7;file:///home/ana",
        "7;file://h/%C3%A1;b",
        "7;http://x/y",
        "9;9;\"C:\\x y\"",
        "9;9;C:\\a;b",
        "9;9;",
        "9;4;1;50",
        "9;hello",
        "133;A",
        "133;B;k=i",
        "133;C",
        "133;D",
        "133;D;0",
        "133;D;3221225477",
        "133;D;x;err=1",
        "133;P;k=i",
        "07;file://h/x",
    ];
    let ends: &[&[u8]] = &[
        b"\x07", b"\x1b\\", b"\x1b", b"\x18", b"\x1a", b"\x1b[0m", b"",
    ];
    prop_oneof![
        3 => prop::sample::select(pieces).prop_map(<[u8]>::to_vec),
        3 => (prop::sample::select(texts), prop::sample::select(ends))
            .prop_map(|(text, end)| [b"\x1b]", text.as_bytes(), end].concat()),
        2 => "[ -~\u{e1}\u{4e2d}]{0,12}".prop_map(String::into_bytes),
        2 => any::<u8>().prop_map(|byte| vec![byte]),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2_000))]

    /// The scanner finds what vte reads, in any output and however it is
    /// split, except an OSC with bytes that vte ignores after its ESC.
    #[test]
    fn it_agrees_with_vte(
        parts in prop::collection::vec(part(), 0..40),
        sizes in prop::collection::vec(1usize..24, 1..16),
    ) {
        let mut output = parts.concat();
        // Out of what the scanner misses.
        for at in 1..output.len() {
            if output[at - 1] == ESC && ignored_after_esc(output[at]) {
                output[at] = b'x';
            }
        }
        let mut reference = Reference::default();
        Parser::new().advance(&mut reference, &output);
        prop_assume!(!reference.cut);
        let mut scanner = Scanner::new();
        let mut found = Vec::new();
        let mut rest = output.as_slice();
        for &size in sizes.iter().cycle() {
            if rest.is_empty() {
                break;
            }
            let (piece, after) = rest.split_at(size.min(rest.len()));
            scanner.scan(piece, |report| found.push(report));
            rest = after;
        }
        prop_assert_eq!(found, reference.found, "{:?}", output);
    }
}
