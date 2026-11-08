//! Recordings of what a program writes to the terminal, to replay it in the
//! tests and in the measurements without running the program again.
//!
//! A recording keeps each block as it came from the pseudoconsole, with the
//! moment when it came, so that it can be replayed with the same pieces and
//! the same pauses. The format, with every number in little-endian:
//!
//! | Bytes | Content |
//! |---|---|
//! | 8 | `MEIGAREC` |
//! | 2 | The version of the format: 1. |
//! | 2 + 2 | The columns and rows of the pseudoconsole. |
//!
//! And then, for each block:
//!
//! | Bytes | Content |
//! |---|---|
//! | 8 | Microseconds since the recording started. |
//! | 4 | The length of the block. |
//! | That length | The bytes of the block. |

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::geometry::GridSize;

/// The first bytes of every recording.
pub const MAGIC: [u8; 8] = *b"MEIGAREC";

/// The version of the format that meiga writes and reads.
pub const VERSION: u16 = 1;

const HEADER_LEN: usize = 14;
const BLOCK_HEADER_LEN: usize = 12;

/// Writes a recording.
pub struct Recorder<W: Write> {
    out: W,
    start: Instant,
    blocks: u64,
    bytes: u64,
}

impl Recorder<BufWriter<File>> {
    /// Creates the recording `path`, replacing it if it exists, for a
    /// pseudoconsole of `size`.
    pub fn create(path: &Path, size: GridSize) -> io::Result<Self> {
        Self::new(BufWriter::new(File::create(path)?), size)
    }
}

impl<W: Write> Recorder<W> {
    /// Starts a recording in `out`. The time of each block counts from now.
    pub fn new(mut out: W, size: GridSize) -> io::Result<Self> {
        out.write_all(&MAGIC)?;
        out.write_all(&VERSION.to_le_bytes())?;
        out.write_all(&size.columns.to_le_bytes())?;
        out.write_all(&size.rows.to_le_bytes())?;
        Ok(Self {
            out,
            start: Instant::now(),
            blocks: 0,
            bytes: 0,
        })
    }

    /// Adds a block that has just come.
    pub fn record(&mut self, bytes: &[u8]) -> io::Result<()> {
        let micros = u64::try_from(self.start.elapsed().as_micros()).unwrap_or(u64::MAX);
        let len = u32::try_from(bytes.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "the block is too long"))?;
        self.out.write_all(&micros.to_le_bytes())?;
        self.out.write_all(&len.to_le_bytes())?;
        self.out.write_all(bytes)?;
        self.blocks += 1;
        self.bytes += u64::from(len);
        Ok(())
    }

    /// How many blocks it has recorded.
    pub const fn blocks(&self) -> u64 {
        self.blocks
    }

    /// How many bytes of output it has recorded.
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Writes what is still in memory, and gives back the output.
    pub fn finish(mut self) -> io::Result<W> {
        self.out.flush()?;
        Ok(self.out)
    }
}

/// A block of a recording.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block<'a> {
    /// When it came, since the recording started.
    pub time: Duration,
    pub bytes: &'a [u8],
}

/// Where a block is in the data of the recording.
#[derive(Clone, Copy, Debug)]
struct Span {
    time: Duration,
    start: usize,
    end: usize,
}

/// A recording, read and checked.
#[derive(Clone, Debug)]
pub struct Recording {
    size: GridSize,
    data: Vec<u8>,
    blocks: Vec<Span>,
}

impl Recording {
    /// Reads the recording `path`.
    pub fn read(path: &Path) -> io::Result<Self> {
        Self::from_bytes(std::fs::read(path)?)
    }

    /// Checks that `data` is a whole recording, and finds its blocks.
    pub fn from_bytes(data: Vec<u8>) -> io::Result<Self> {
        let header: &[u8; HEADER_LEN] = data
            .first_chunk()
            .ok_or_else(|| invalid("it is shorter than the header of a recording"))?;
        let [magic @ .., v0, v1, c0, c1, r0, r1] = header;
        if *magic != MAGIC {
            return Err(invalid("it is not a recording of meiga"));
        }
        let version = u16::from_le_bytes([*v0, *v1]);
        if version != VERSION {
            return Err(invalid(&format!(
                "it is a recording of version {version}, and only version {VERSION} can be read"
            )));
        }
        let size = GridSize::new(
            u16::from_le_bytes([*c0, *c1]),
            u16::from_le_bytes([*r0, *r1]),
        );
        let mut blocks = Vec::new();
        let mut at = HEADER_LEN;
        while at < data.len() {
            let block_header: &[u8; BLOCK_HEADER_LEN] = data
                .get(at..)
                .and_then(<[u8]>::first_chunk)
                .ok_or_else(|| invalid(&format!("the block at byte {at} is cut")))?;
            let [time @ .., l0, l1, l2, l3] = block_header;
            let time = Duration::from_micros(u64::from_le_bytes(*time));
            let len = u32::from_le_bytes([*l0, *l1, *l2, *l3]) as usize;
            let start = at + BLOCK_HEADER_LEN;
            let end = start
                .checked_add(len)
                .filter(|&end| end <= data.len())
                .ok_or_else(|| invalid(&format!("the block at byte {at} is cut")))?;
            blocks.push(Span { time, start, end });
            at = end;
        }
        Ok(Self { size, data, blocks })
    }

    /// The size of the pseudoconsole that it was recorded with.
    pub const fn size(&self) -> GridSize {
        self.size
    }

    /// The blocks, in the order in which they came.
    pub fn blocks(&self) -> impl ExactSizeIterator<Item = Block<'_>> + '_ {
        self.blocks.iter().map(|span| Block {
            time: span.time,
            bytes: self.data.get(span.start..span.end).unwrap_or_default(),
        })
    }

    /// How many bytes of output it has, without the headers.
    pub fn output_len(&self) -> usize {
        self.blocks.iter().map(|span| span.end - span.start).sum()
    }

    /// When the last block came.
    pub fn duration(&self) -> Duration {
        self.blocks.last().map_or(Duration::ZERO, |span| span.time)
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recorded(blocks: &[&[u8]]) -> Vec<u8> {
        let mut recorder = Recorder::new(Vec::new(), GridSize::new(120, 30)).unwrap();
        for block in blocks {
            recorder.record(block).unwrap();
        }
        assert_eq!(recorder.blocks(), blocks.len() as u64);
        recorder.finish().unwrap()
    }

    #[test]
    fn a_recording_gives_back_its_blocks_and_size() {
        let data = recorded(&[b"\x1b[1t", b"", b"ola\r\n"]);
        assert_eq!(data.len(), HEADER_LEN + 3 * BLOCK_HEADER_LEN + 9);
        let recording = Recording::from_bytes(data).unwrap();
        assert_eq!(recording.size(), GridSize::new(120, 30));
        let blocks: Vec<&[u8]> = recording.blocks().map(|block| block.bytes).collect();
        assert_eq!(blocks, [&b"\x1b[1t"[..], b"", b"ola\r\n"]);
        assert_eq!(recording.output_len(), 9);
        let times: Vec<Duration> = recording.blocks().map(|block| block.time).collect();
        assert!(times.is_sorted());
        assert_eq!(recording.duration(), times[2]);
    }

    #[test]
    fn a_recording_without_blocks_is_valid() {
        let recording = Recording::from_bytes(recorded(&[])).unwrap();
        assert_eq!(recording.blocks().len(), 0);
        assert_eq!(recording.duration(), Duration::ZERO);
    }

    #[test]
    fn other_data_is_rejected() {
        let error = |data: Vec<u8>| Recording::from_bytes(data).unwrap_err().to_string();
        assert_eq!(
            error(b"MEIGA".to_vec()),
            "it is shorter than the header of a recording"
        );
        assert_eq!(
            error(b"NOTMEIGA\x01\x00\x50\x00\x19\x00".to_vec()),
            "it is not a recording of meiga"
        );
        assert_eq!(
            error(b"MEIGAREC\x02\x00\x50\x00\x19\x00".to_vec()),
            "it is a recording of version 2, and only version 1 can be read"
        );
    }

    #[test]
    fn a_cut_block_is_rejected() {
        let data = recorded(&[b"ola"]);
        for len in [HEADER_LEN + 5, data.len() - 1] {
            let error = Recording::from_bytes(data[..len].to_vec()).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert_eq!(error.to_string(), "the block at byte 14 is cut");
        }
    }
}
