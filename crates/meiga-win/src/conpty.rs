//! Pseudoconsoles: programs that run in a console without a window and talk
//! to meiga through two pipes.
//!
//! meiga writes to the input pipe what the user types, and reads from the
//! output pipe what the program shows: UTF-8 text with VT sequences. Each
//! session has two threads, which sleep while there is nothing to do (ADR
//! 0005 and ADR 0011):
//!
//! - the reader, blocked in `ReadFile` on the output pipe, gives each block
//!   that it reads to an [`Output`], which can answer the program at once
//!   with an [`Input`];
//! - the writer waits at the same time for input to send, for the session to
//!   be closed and for the program to end. Then it closes the pseudoconsole,
//!   and makes sure that the session ends.
//!
//! The program, and the programs that it starts, run in a job of the
//! session, so that they can all be ended if the session gets stuck. A
//! process that exits while it closes sessions ends their writers with it:
//! [`wait_for_closed_sessions`] waits for them before.

use std::ffi::{CStr, c_void};
use std::io;
use std::marker::PhantomData;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use meiga_core::geometry::GridSize;
use windows_sys::Win32::Foundation::{FALSE, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows_sys::Win32::System::Console::{
    COORD, ClosePseudoConsole, CreatePseudoConsole, HPCON, ResizePseudoConsole,
};
use windows_sys::Win32::System::JobObjects::{
    CreateJobObjectW, JOB_OBJECT_LIMIT_BREAKAWAY_OK, JOBOBJECT_BASIC_PROCESS_ID_LIST,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectBasicProcessIdList,
    JobObjectExtendedLimitInformation, QueryInformationJobObject, SetInformationJobObject,
    TerminateJobObject,
};
use windows_sys::Win32::System::LibraryLoader::{
    GetProcAddress, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CREATE_UNICODE_ENVIRONMENT, CreateEventW, CreateProcessW, DeleteProcThreadAttributeList,
    EXTENDED_STARTUPINFO_PRESENT, GetExitCodeProcess, INFINITE, InitializeProcThreadAttributeList,
    LPPROC_THREAD_ATTRIBUTE_LIST, OpenProcess, PROC_THREAD_ATTRIBUTE_JOB_LIST,
    PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, PROCESS_INFORMATION, PROCESS_SYNCHRONIZE,
    STARTF_USESTDHANDLES, STARTUPINFOEXW, STARTUPINFOW, SetEvent, SetThreadPriority,
    THREAD_PRIORITY, THREAD_PRIORITY_BELOW_NORMAL, THREAD_PRIORITY_NORMAL,
    UpdateProcThreadAttribute, WaitForMultipleObjects, WaitForSingleObject,
};
use windows_sys::core::HRESULT;

use crate::{Error, wide};

/// The library of the bundled ConPTY.
const LIBRARY: &str = "conpty.dll";

/// The console host of the bundled ConPTY. When it is not next to the
/// library, `conpty.dll` starts the console host of Windows instead, without
/// a word.
const HOST: &str = "OpenConsole.exe";

/// `PSEUDOCONSOLE_GLYPH_WIDTH_WCSWIDTH`: ConPTY measures the width of the
/// characters with `wcswidth`, as the terminal grid does (ADR 0004). Only
/// the bundled ConPTY knows it: the one of Windows ignores it.
const GLYPH_WIDTH_WCSWIDTH: u32 = 0x10;

/// The size of each read from the output pipe (ADR 0005), and of the buffer
/// of the pipe.
const READ_SIZE: u32 = 64 * 1024;

/// How long the writer waits, once it has closed the pseudoconsole, for one
/// more of the programs that the session had to end, from the last one that
/// ended. Windows asks the programs of a console to end one at a time, and
/// gives each 5 s to handle `CTRL_CLOSE_EVENT` before it ends it itself:
/// while a session ends, a program ends at least every 5 s. A session where
/// none ends for twice as long is stuck, and the writer ends all its
/// programs (ADR 0011 and ADR 0026).
const CLOSE_TIMEOUT_MS: u32 = 10_000;

/// [`CLOSE_TIMEOUT_MS`], to add to a moment.
const CLOSE_TIMEOUT: Duration = Duration::from_millis(CLOSE_TIMEOUT_MS as u64);

/// The most objects that `WaitForMultipleObjects` waits for. It is in the
/// `Win32_System_SystemServices` feature of `windows-sys`, which would bring
/// thousands of declarations for it, as in `event.rs`.
const MAXIMUM_WAIT_OBJECTS: usize = 64;

/// How many of the programs that a session had when it closed the writer
/// waits for one by one: two of the objects that it waits for are the
/// output and the program.
const WATCHED: usize = MAXIMUM_WAIT_OBJECTS - 2;

/// How long the reader waits for the program to end once the output has
/// closed, to tell its exit code. The program has usually ended by then, or
/// ends a moment later. If not, and the session was closed, the writer ends
/// it once no other program ends for [`CLOSE_TIMEOUT_MS`].
const EXIT_TIMEOUT_MS: u32 = CLOSE_TIMEOUT_MS + 1_000;

/// The exit code of the programs that the writer ends, the same as
/// `std::process::Child::kill` gives.
const KILLED_EXIT_CODE: u32 = 1;

/// `ConptyCreatePseudoConsole` and `CreatePseudoConsole`.
type CreateFn = unsafe extern "system" fn(COORD, HANDLE, HANDLE, u32, *mut HPCON) -> HRESULT;

/// `ConptyClosePseudoConsole` and `ClosePseudoConsole`.
type CloseFn = unsafe extern "system" fn(HPCON);

/// `ConptyResizePseudoConsole` and `ResizePseudoConsole`.
type ResizeFn = unsafe extern "system" fn(HPCON, COORD) -> HRESULT;

/// What a function found by `GetProcAddress` looks like before it is cast to
/// its real signature.
type AnyFn = unsafe extern "system" fn() -> isize;

/// Where a ConPTY comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// The ConPTY of Windows Terminal that meiga ships: `conpty.dll` and
    /// `OpenConsole.exe`, in this folder.
    Bundled(PathBuf),
    /// The ConPTY of Windows, in `kernel32.dll`, with its console host,
    /// `conhost.exe`.
    System,
}

/// A ConPTY: the functions that create, resize and close pseudoconsoles.
///
/// It is cheap to clone. The library of a bundled ConPTY stays loaded until
/// the process ends.
#[derive(Clone, Debug)]
pub struct Conpty {
    source: Source,
    create: CreateFn,
    close: CloseFn,
    resize: ResizeFn,
    flags: u32,
}

impl Conpty {
    /// The ConPTY next to the executable, if `conpty.dll` and
    /// `OpenConsole.exe` are both there, or else the one of Windows.
    pub fn load() -> Self {
        let exe = std::env::current_exe().ok();
        exe.as_deref()
            .and_then(Path::parent)
            .and_then(|folder| Self::bundled(folder).ok())
            .unwrap_or_else(Self::system)
    }

    /// The ConPTY in `folder`: `conpty.dll` and `OpenConsole.exe`.
    pub fn bundled(folder: &Path) -> Result<Self, Error> {
        let folder =
            std::path::absolute(folder).map_err(|error| Error::new("GetFullPathNameW", error))?;
        for file in [LIBRARY, HOST] {
            let path = folder.join(file);
            if !path.is_file() {
                let message = format!("{} does not exist", path.display());
                return Err(Error::new(
                    "LoadLibraryExW",
                    io::Error::new(io::ErrorKind::NotFound, message),
                ));
            }
        }
        let name: Vec<u16> = folder
            .join(LIBRARY)
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        // SAFETY: the name is a null-terminated full path. The flags look for
        // the libraries that it needs only in its folder and in System32.
        let module = unsafe {
            LoadLibraryExW(
                name.as_ptr(),
                ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
            )
        };
        if module.is_null() {
            return Err(Error::last("LoadLibraryExW"));
        }
        // The module is never freed: the functions below point into it.
        let function = |name: &CStr| {
            // SAFETY: the module is loaded, and the name is null-terminated.
            unsafe { GetProcAddress(module, name.as_ptr().cast()) }
                .ok_or_else(|| Error::last("GetProcAddress"))
        };
        let create = function(c"ConptyCreatePseudoConsole")?;
        let close = function(c"ConptyClosePseudoConsole")?;
        let resize = function(c"ConptyResizePseudoConsole")?;
        // SAFETY: conpty.h, in the NuGet package, declares the three
        // functions with these signatures.
        let (create, close, resize) = unsafe {
            (
                mem::transmute::<AnyFn, CreateFn>(create),
                mem::transmute::<AnyFn, CloseFn>(close),
                mem::transmute::<AnyFn, ResizeFn>(resize),
            )
        };
        Ok(Self {
            source: Source::Bundled(folder),
            create,
            close,
            resize,
            flags: GLYPH_WIDTH_WCSWIDTH,
        })
    }

    /// The ConPTY of Windows: `CreatePseudoConsole`, `ResizePseudoConsole`
    /// and `ClosePseudoConsole`, in `kernel32.dll`.
    pub fn system() -> Self {
        Self {
            source: Source::System,
            create: CreatePseudoConsole,
            close: ClosePseudoConsole,
            resize: ResizePseudoConsole,
            flags: 0,
        }
    }

    /// Where it comes from.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// Runs `command` in a new pseudoconsole of `size`, and gives what it
    /// shows to `output`, on a thread of the session.
    ///
    /// The program runs in a job of the session, as the programs that it
    /// starts, unless they ask to leave it.
    ///
    /// If it fails, `output` is never called.
    pub fn spawn<O: Output>(
        &self,
        command: &Command<'_>,
        size: GridSize,
        output: O,
    ) -> Result<Pty, Error> {
        let size = coord(size, "CreatePseudoConsole")?;
        let mut command_line = null_terminated(command.line, "the command line")?;
        let folder = command
            .folder
            .map(|folder| null_terminated(folder, "the folder"))
            .transpose()?;
        let environment = command.environment.map(environment_block).transpose()?;
        let wake = event(false)?;
        let output_closed = event(true)?;
        let job = job()?;
        let (input_read, input_write) = pipe(0)?;
        let (output_read, output_write) = pipe(READ_SIZE)?;
        let console = self.create(size, &input_read, &output_write)?;
        // ConPTY keeps copies of its ends of the pipes. Without ours, the
        // pipes break when ConPTY ends, so the reader and the writer do not
        // wait for ever.
        drop((input_read, output_write));
        let program = Program {
            command_line: &mut command_line,
            folder: folder.as_deref(),
            environment,
        };
        let (process, process_id) = match start(program, &console, &job) {
            Ok(started) => started,
            Err(error) => {
                // Before Windows 11 24H2, ClosePseudoConsole waits until
                // ConPTY ends, and ConPTY may be waiting to write its
                // output: close the output pipe first.
                drop(output_read);
                drop(console);
                return Err(error);
            }
        };
        // The pseudoconsole stays here until the writer takes it: if the
        // writer does not start, it is closed below, after the programs end.
        let shared = Arc::new(Shared {
            event: wake,
            output_closed,
            process,
            job,
            state: Mutex::new(State {
                console: Some(console),
                ..State::default()
            }),
        });

        let writer = {
            let shared = Arc::clone(&shared);
            thread::Builder::new()
                .name(format!("conpty writer {process_id}"))
                .spawn(move || write_input(&shared, input_write))
        };
        let writer = match writer {
            Ok(writer) => writer,
            Err(error) => {
                // End the programs before their pseudoconsole closes, which must
                // not happen while a program connects to it, and close the
                // output first, as above.
                shared.terminate();
                drop(output_read);
                let console = shared.lock().console.take();
                drop(console);
                return Err(Error::new("CreateThread", error));
            }
        };
        let input = Input { shared };
        let reader = {
            let shared = Arc::clone(&input.shared);
            thread::Builder::new()
                .name(format!("conpty reader {process_id}"))
                .spawn(move || read_output(output_read, shared, output))
        };
        match reader {
            Ok(reader) => Ok(Pty {
                input,
                process_id,
                reader,
                writer: Some(writer),
            }),
            Err(error) => {
                // Nobody reads the output: end the programs, tell the writer
                // not to wait for the output, and close the session, as
                // dropping a `Pty` does. `output` is never called.
                input.shared.terminate();
                input.shared.close_output();
                input.close();
                keep_closing(writer);
                Err(Error::new("CreateThread", error))
            }
        }
    }

    /// Creates a pseudoconsole that reads its input from `input` and writes
    /// its output to `output`.
    fn create(
        &self,
        size: COORD,
        input: &OwnedHandle,
        output: &OwnedHandle,
    ) -> Result<PseudoConsole, Error> {
        let mut handle: HPCON = 0;
        // SAFETY: the function has the signature of conpty.h. Both handles
        // are synchronous pipes, valid during the call, and `handle` is
        // valid for it to fill in.
        let result = unsafe {
            (self.create)(
                size,
                input.as_raw_handle(),
                output.as_raw_handle(),
                self.flags,
                &mut handle,
            )
        };
        if result < 0 {
            let function = match self.source {
                Source::Bundled(_) => "ConptyCreatePseudoConsole",
                Source::System => "CreatePseudoConsole",
            };
            return Err(Error::hresult(function, result));
        }
        Ok(PseudoConsole {
            handle,
            close: self.close,
            resize: self.resize,
            may_wait: matches!(self.source, Source::System),
        })
    }
}

/// Where a pseudoconsole sends what the program shows.
///
/// Its methods run on the reader thread of the session, one after the
/// other. They should return soon: while they run, nothing is read, and when
/// the output pipe is full, the program waits.
pub trait Output: Send + 'static {
    /// The pseudoconsole wrote `bytes`: UTF-8 text with VT sequences. A
    /// sequence, or a character, can be split between two calls.
    ///
    /// `input` sends to the program the answers to the questions that it
    /// asks in its output, such as the attributes of the terminal, at once,
    /// from this thread.
    fn received(&mut self, bytes: &[u8], input: &Input);

    /// The session ended: the pseudoconsole closed its output, after the
    /// last [`Output::received`]. `exit_code` is the exit code of the
    /// program, or `None` if it is still running some seconds later, which
    /// only happens if ConPTY itself fails.
    fn ended(&mut self, exit_code: Option<u32>);
}

/// The priority of the reader thread of a session, which gives the output of
/// the program to its [`Output`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    /// The one of the threads of a program, such as the one of its windows:
    /// the one with which the thread starts.
    Normal,
    /// One step below (`THREAD_PRIORITY_BELOW_NORMAL`): when the processor
    /// is busy, the threads with the normal priority run first, but Windows
    /// still lets it run from time to time, so it never stops for good.
    BelowNormal,
}

/// A program running in a pseudoconsole.
///
/// Dropping it closes the session without waiting. Its writer thread closes
/// the pseudoconsole, which asks the program to end, and its reader thread
/// calls [`Output::ended`] after the last output. If the session gets stuck,
/// the writer ends the programs of its job. Before the process exits,
/// [`wait_for_closed_sessions`] waits for the writers of the sessions that
/// were closed.
pub struct Pty {
    input: Input,
    process_id: u32,
    /// The reader thread, whose priority can change. Dropping it does not
    /// wait for the thread: it goes on until the output ends.
    reader: JoinHandle<()>,
    /// The writer thread, until the `Pty` is dropped: then it goes to
    /// [`CLOSING`].
    writer: Option<JoinHandle<()>>,
}

/// The writer threads of the sessions that were closed, which end once
/// their sessions have ended: [`wait_for_closed_sessions`] waits for them.
/// The ones that have ended leave the list each time that another one comes.
static CLOSING: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());

/// Keeps the writer thread of a session that was closed.
fn keep_closing(writer: JoinHandle<()>) {
    let mut closing = CLOSING.lock().unwrap_or_else(PoisonError::into_inner);
    closing.retain(|writer| !writer.is_finished());
    closing.push(writer);
}

/// Waits until the sessions that were closed have ended: until the writer of
/// each one has seen ConPTY close the output and the program end or, if the
/// session got stuck, has ended the programs of its job. If a session ended
/// on its own, its programs that are not in its console, such as a window
/// that its shell opened, go on. Call it before the process exits, once no
/// thread closes more sessions: when a process exits, its threads end with
/// it, and a session that got stuck would stay so for ever, with its
/// programs and its ConPTY (ADR 0026).
///
/// It waits as long as the sessions take: a session ends at once, or when
/// its programs handle `CTRL_CLOSE_EVENT`, or [`CLOSE_TIMEOUT_MS`] after the
/// last of them ended, if it gets stuck.
pub fn wait_for_closed_sessions() {
    let writers = mem::take(&mut *CLOSING.lock().unwrap_or_else(PoisonError::into_inner));
    for writer in writers {
        // A writer that panicked has nothing left to wait for: the panic
        // message has already told why.
        let _ = writer.join();
    }
}

impl Pty {
    /// Sends `bytes` to the program, as if they were typed: see
    /// [`Input::write`].
    pub fn write(&self, bytes: &[u8]) {
        self.input.write(bytes);
    }

    /// What sends input to the program, for another thread. It does not
    /// keep the session open: dropping the `Pty` closes it.
    pub fn input(&self) -> Input {
        self.input.clone()
    }

    /// The identifier of the process of the program.
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    /// Changes the size of the pseudoconsole: see [`Input::resize`].
    pub fn resize(&self, size: GridSize) -> Result<(), Error> {
        self.input.resize(size)
    }

    /// Changes the priority of the reader thread, from any thread. It takes
    /// effect at once, also while the reader is giving output to its
    /// [`Output`]. Once the reader has ended, it does nothing.
    pub fn set_reader_priority(&self, priority: Priority) -> Result<(), Error> {
        let value: THREAD_PRIORITY = match priority {
            Priority::Normal => THREAD_PRIORITY_NORMAL,
            Priority::BelowNormal => THREAD_PRIORITY_BELOW_NORMAL,
        };
        // SAFETY: the handle is the one of the reader thread, which the
        // JoinHandle keeps open, also after the thread ends; it has every
        // access right, THREAD_SET_INFORMATION among them.
        if unsafe { SetThreadPriority(self.reader.as_raw_handle(), value) } == FALSE {
            return Err(Error::last("SetThreadPriority"));
        }
        Ok(())
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        self.input.close();
        if let Some(writer) = self.writer.take() {
            keep_closing(writer);
        }
    }
}

/// Sends input to the program of a session, and changes the size of its
/// pseudoconsole, from any thread. It is cheap to clone.
#[derive(Clone)]
pub struct Input {
    shared: Arc<Shared>,
}

impl Input {
    /// Sends `bytes` to the program, as if they were typed. It does not
    /// wait: the writer thread sends them. Once the session has been closed
    /// or has ended, it does nothing.
    pub fn write(&self, bytes: &[u8]) {
        let wake = {
            let mut state = self.shared.lock();
            if state.closing || state.finished || bytes.is_empty() {
                return;
            }
            state.input.extend_from_slice(bytes);
            // If there was input already, the writer has been woken up for
            // it and has not taken it yet.
            state.input.len() == bytes.len()
        };
        if wake {
            self.shared.wake();
        }
    }

    /// Changes the size of the pseudoconsole. ConPTY reflows its buffer, and
    /// tells the program, which may show itself again for the new size; to
    /// the terminal it writes nothing, but it asks for the position of its
    /// cursor (`CSI 6n`) when the program asks for its own. It does not
    /// wait for ConPTY. Once the session has been closed or has ended, it
    /// does nothing.
    pub fn resize(&self, size: GridSize) -> Result<(), Error> {
        let size = coord(size, "ResizePseudoConsole")?;
        let state = self.shared.lock();
        let Some(console) = state.console.as_ref().filter(|_| !state.closing) else {
            return Ok(());
        };
        // SAFETY: the handle comes from the create function of the same
        // ConPTY, and it stays open while the lock is held: the writer
        // takes it under the lock to close it. The function only writes a
        // message to ConPTY, without waiting for it.
        let result = unsafe { (console.resize)(console.handle, size) };
        if result < 0 {
            return Err(Error::hresult("ResizePseudoConsole", result));
        }
        Ok(())
    }

    /// Closes the session without waiting: the writer closes the
    /// pseudoconsole.
    fn close(&self) {
        self.shared.lock().closing = true;
        self.shared.wake();
    }
}

/// What a session, its reader thread and its writer thread share.
struct Shared {
    /// An auto-reset event that wakes the writer up: there is input to send,
    /// or the session was closed.
    event: OwnedHandle,
    /// A manual-reset event that the reader sets when the output closes:
    /// ConPTY has ended.
    output_closed: OwnedHandle,
    /// The program.
    process: OwnedHandle,
    /// The job of the program and of the programs that it starts. They may
    /// leave it if they ask to, with `CREATE_BREAKAWAY_FROM_JOB`.
    job: OwnedHandle,
    state: Mutex<State>,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        // Every change to the state is a single assignment, a swap, an
        // append or a take: a panic cannot leave it half done.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn wake(&self) {
        // SAFETY: the event is valid while `self` exists.
        unsafe { SetEvent(self.event.as_raw_handle()) };
    }

    /// Tells the writer that the output has closed.
    fn close_output(&self) {
        // SAFETY: the event is valid while `self` exists.
        unsafe { SetEvent(self.output_closed.as_raw_handle()) };
    }

    /// Waits, once the pseudoconsole has closed, for the session to end: for
    /// ConPTY to close the output and for the program to end. Meanwhile, the
    /// programs that the session had must keep ending: Windows asks them to
    /// end one at a time. If none of them ends for [`CLOSE_TIMEOUT_MS`],
    /// something keeps the pseudoconsole open: usually a program that was
    /// connecting to it when it closed, and that waits for ConPTY, while
    /// ConPTY waits for it (ADR 0011). Then every program of the session
    /// ends.
    fn wait_for_end(&self) {
        // Only these count: a program that starts later could hide that
        // others end, or, starting others, keep the session waiting.
        let programs = self.programs();
        let (watched, others) = programs.split_at(programs.len().min(WATCHED));
        // The writer waits for these at the same time as for the output and
        // the program, so it knows when each one ends (ADR 0026). The others,
        // if a session had so many, are counted when the time runs out.
        let mut watched: Vec<HANDLE> = watched.iter().map(AsRawHandle::as_raw_handle).collect();
        let mut others_ended = 0;
        let output_closed = self.output_closed.as_raw_handle();
        let process = self.process.as_raw_handle();
        let (mut closed, mut ended) = (false, false);
        let mut deadline = Instant::now() + CLOSE_TIMEOUT;
        let mut handles = Vec::with_capacity(WATCHED + 2);
        loop {
            // The ones that are signalled leave the list: they would stay
            // so, and the wait would return them again and again.
            handles.clear();
            handles.extend((!closed).then_some(output_closed));
            handles.extend((!ended).then_some(process));
            handles.extend_from_slice(&watched);
            let left = deadline.saturating_duration_since(Instant::now());
            // Rounded up, so that the time has run out when it returns.
            let left = u32::try_from(left.as_micros().div_ceil(1000)).unwrap_or(CLOSE_TIMEOUT_MS);
            // SAFETY: the handles are valid while `self` and `programs`
            // exist, and there are at most 64 of them, as the function takes.
            let result = unsafe {
                WaitForMultipleObjects(handles.len() as u32, handles.as_ptr(), FALSE, left)
            };
            if let Some(&handle) = handles.get(result.wrapping_sub(WAIT_OBJECT_0) as usize) {
                if handle == output_closed {
                    closed = true;
                } else if handle == process {
                    ended = true;
                } else {
                    watched.retain(|&program| program != handle);
                    deadline = Instant::now() + CLOSE_TIMEOUT;
                }
                if closed && ended {
                    return;
                }
                continue;
            }
            if result != WAIT_TIMEOUT {
                // The wait failed, and nothing can be waited for any more.
                break;
            }
            let now = others.iter().filter(|program| has_ended(program)).count();
            if now <= others_ended {
                break;
            }
            others_ended = now;
            deadline = Instant::now() + CLOSE_TIMEOUT;
        }
        self.terminate();
    }

    /// The programs that the job has now, opened to wait for them. While
    /// they are open, their identifiers cannot be reused.
    fn programs(&self) -> Vec<OwnedHandle> {
        // The list starts with two counts, and the identifiers follow.
        let header = mem::offset_of!(JOBOBJECT_BASIC_PROCESS_ID_LIST, ProcessIdList)
            / mem::size_of::<usize>();
        let mut capacity = 64;
        // The job can gain programs between two tries.
        for _ in 0..4 {
            let mut list = vec![0_usize; header + capacity];
            // SAFETY: the job is valid while `self` exists, and the buffer
            // has the size that it is given, aligned as the structure.
            let listed = unsafe {
                QueryInformationJobObject(
                    self.job.as_raw_handle(),
                    JobObjectBasicProcessIdList,
                    list.as_mut_ptr().cast(),
                    u32::try_from(mem::size_of_val(list.as_slice())).unwrap_or(u32::MAX),
                    ptr::null_mut(),
                )
            } != 0;
            // SAFETY: the buffer starts with the counts, aligned as the
            // structure. They stay 0 if Windows did not write them.
            let (assigned, returned) = unsafe {
                let counts = list.as_ptr().cast::<JOBOBJECT_BASIC_PROCESS_ID_LIST>();
                (
                    (*counts).NumberOfAssignedProcesses as usize,
                    (*counts).NumberOfProcessIdsInList as usize,
                )
            };
            if listed && returned >= assigned {
                let ids = list.get(header..header + returned).unwrap_or_default();
                return ids.iter().filter_map(|&id| open_to_wait(id)).collect();
            }
            capacity = assigned.max(capacity) * 2;
        }
        Vec::new()
    }

    /// Ends the program and the programs of its job.
    fn terminate(&self) {
        // SAFETY: the job is valid while `self` exists, and CreateJobObjectW
        // gives every access right.
        unsafe { TerminateJobObject(self.job.as_raw_handle(), KILLED_EXIT_CODE) };
    }
}

#[derive(Default)]
struct State {
    /// Input that the writer has not taken yet.
    input: Vec<u8>,
    /// The session was closed.
    closing: bool,
    /// The writer has closed the pseudoconsole: input is no longer sent.
    finished: bool,
    /// The pseudoconsole, until the writer takes it to close it.
    console: Option<PseudoConsole>,
}

/// A pseudoconsole, closed when it is dropped.
struct PseudoConsole {
    handle: HPCON,
    close: CloseFn,
    resize: ResizeFn,
    /// Closing it may wait until ConPTY ends: the close function of Windows
    /// does so before Windows 11 24H2.
    may_wait: bool,
}

impl Drop for PseudoConsole {
    fn drop(&mut self) {
        // SAFETY: the handle comes from the create function of the same
        // ConPTY, and it is closed only here.
        unsafe { (self.close)(self.handle) };
    }
}

/// The writer thread: sends the input to the program, closes the
/// pseudoconsole when the program ends or the session is closed, and makes
/// sure that the session ends.
fn write_input(shared: &Shared, pipe: OwnedHandle) {
    let handles = [shared.event.as_raw_handle(), shared.process.as_raw_handle()];
    let mut pending = Vec::new();
    loop {
        // SAFETY: both handles are valid while this thread runs, because it
        // holds `shared`.
        let woken = unsafe { WaitForMultipleObjects(2, handles.as_ptr(), FALSE, INFINITE) };
        if woken != WAIT_OBJECT_0 {
            // The program ended, or the wait failed and nothing can be
            // waited for any more.
            break;
        }
        let closing = {
            let mut state = shared.lock();
            mem::swap(&mut state.input, &mut pending);
            state.closing
        };
        if closing {
            break;
        }
        if !write_all(&pipe, &pending) {
            // ConPTY closed its end of the input pipe: it has ended.
            break;
        }
        pending.clear();
    }
    let console = {
        let mut state = shared.lock();
        state.finished = true;
        state.console.take()
    };
    drop(pipe);
    if let Some(console) = console {
        close_console(console);
    }
    shared.wait_for_end();
}

/// Closes a pseudoconsole, which asks its programs to end, without waiting
/// for ConPTY to end. The bundled ConPTY, and the one of Windows since
/// Windows 11 24H2, return at once. Before 24H2, ClosePseudoConsole of
/// Windows waits until ConPTY ends, which can take a while: that is why
/// closing a session never calls it on the thread of the window. Only
/// `Conpty::spawn` does, on the thread that calls it, when it fails: by then
/// the session has no programs, and its output is closed, so nothing keeps
/// ConPTY waiting. And ConPTY does not end while a program that was
/// connecting to it when it closed waits for it: the writer would wait for
/// ever, and never end the programs of the session (ADR 0026). So a
/// pseudoconsole that may wait closes on a thread of its own, which ends once
/// ConPTY ends, and nobody waits for it.
fn close_console(console: PseudoConsole) {
    if !console.may_wait {
        drop(console);
        return;
    }
    // If the thread does not start, the closure is dropped here, and the
    // pseudoconsole with it: it closes on this thread, as when it does not
    // wait, and before 24H2 a session that is stuck keeps the writer there
    // for ever.
    let _ = thread::Builder::new()
        .name("conpty close".to_owned())
        .spawn(move || drop(console));
}

/// Writes all of `bytes` to `pipe`, or returns false if the pipe broke.
fn write_all(pipe: &OwnedHandle, mut bytes: &[u8]) -> bool {
    while !bytes.is_empty() {
        let length = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
        let mut written = 0;
        // SAFETY: the bytes are valid for `length` bytes, and the pipe is
        // synchronous, so WriteFile returns when they have been written.
        if unsafe {
            WriteFile(
                pipe.as_raw_handle(),
                bytes.as_ptr(),
                length,
                &mut written,
                ptr::null_mut(),
            )
        } == 0
            || written == 0
        {
            return false;
        }
        bytes = bytes.get(written as usize..).unwrap_or_default();
    }
    true
}

/// The reader thread: gives what the program shows to `output`, until
/// ConPTY ends and closes its end of the output pipe.
fn read_output<O: Output>(pipe: OwnedHandle, shared: Arc<Shared>, mut output: O) {
    let input = Input { shared };
    let mut buffer = vec![0; READ_SIZE as usize];
    loop {
        let mut read = 0;
        // SAFETY: the buffer has READ_SIZE bytes, and the pipe is
        // synchronous, so ReadFile returns when it has read.
        if unsafe {
            ReadFile(
                pipe.as_raw_handle(),
                buffer.as_mut_ptr(),
                READ_SIZE,
                &mut read,
                ptr::null_mut(),
            )
        } == 0
        {
            // ERROR_BROKEN_PIPE: ConPTY has ended. Any other error ends the
            // session too.
            break;
        }
        // Reading 0 bytes means that ConPTY wrote 0 bytes, not that it ended.
        if let Some(bytes) = buffer
            .get(..read as usize)
            .filter(|bytes| !bytes.is_empty())
        {
            output.received(bytes, &input);
        }
    }
    drop(pipe);
    input.shared.close_output();
    output.ended(exit_code(&input.shared.process));
}

/// The exit code of `process`, or `None` if it is still running after
/// [`EXIT_TIMEOUT_MS`].
fn exit_code(process: &OwnedHandle) -> Option<u32> {
    // SAFETY: the handle is valid while `process` exists.
    if unsafe { WaitForSingleObject(process.as_raw_handle(), EXIT_TIMEOUT_MS) } != WAIT_OBJECT_0 {
        return None;
    }
    let mut code = 0;
    // SAFETY: as above, and `code` is valid for it to fill in.
    (unsafe { GetExitCodeProcess(process.as_raw_handle(), &mut code) } != 0).then_some(code)
}

/// Whether `process` has ended, without waiting.
fn has_ended(process: &OwnedHandle) -> bool {
    // SAFETY: the handle is valid while `process` exists.
    unsafe { WaitForSingleObject(process.as_raw_handle(), 0) == WAIT_OBJECT_0 }
}

/// The process `id`, if it still exists, opened only to wait for it.
fn open_to_wait(id: usize) -> Option<OwnedHandle> {
    let id = u32::try_from(id).ok()?;
    // SAFETY: it asks only for the right to wait for the process, and does
    // not let new processes inherit the handle. It returns null if it fails.
    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, FALSE, id) };
    // SAFETY: OpenProcess has just opened it, and nothing else owns it.
    (!handle.is_null()).then(|| unsafe { OwnedHandle::from_raw_handle(handle) })
}

/// What a session runs: a program, and where and with what it starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command<'a> {
    /// The program and its arguments, quoted as the program expects.
    /// Windows looks for the program as `CreateProcessW` does: in the folder
    /// of meiga, in its current folder, in the folders of the system and in
    /// the `PATH` of meiga, and adds `.exe` if it has no extension.
    pub line: &'a str,
    /// The folder where the program starts, or `None` for the current
    /// folder of meiga.
    pub folder: Option<&'a str>,
    /// The environment of the program, or `None` for the one of meiga: an
    /// environment block, a `NAME=value` string for each variable, in the
    /// order of their names, each one ending with a null character, and
    /// another one at the end.
    pub environment: Option<&'a [u16]>,
}

impl<'a> Command<'a> {
    /// `line`, in the folder and with the environment of meiga.
    pub const fn new(line: &'a str) -> Self {
        Self {
            line,
            folder: None,
            environment: None,
        }
    }
}

/// A program for `CreateProcessW`: its command line, null-terminated and
/// writable, and its folder, null-terminated, and environment block, which
/// ends with two null characters, if they are not the ones of meiga.
struct Program<'a> {
    command_line: &'a mut [u16],
    folder: Option<&'a [u16]>,
    environment: Option<&'a [u16]>,
}

/// Starts `program` in `console`, in `job`. Returns the handle and the
/// identifier of its process.
fn start(
    program: Program<'_>,
    console: &PseudoConsole,
    job: &OwnedHandle,
) -> Result<(OwnedHandle, u32), Error> {
    let jobs = [job.as_raw_handle()];
    let mut attributes = Attributes::new(console, &jobs)?;
    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = mem::size_of::<STARTUPINFOEXW>() as u32;
    // Without standard handles of its own, the program takes the ones of
    // the pseudoconsole, and never the ones of meiga.
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.lpAttributeList = attributes.as_mut_ptr();
    let mut process = PROCESS_INFORMATION::default();
    let flags = if program.environment.is_some() {
        EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT
    } else {
        EXTENDED_STARTUPINFO_PRESENT
    };
    // SAFETY: the command line is null-terminated and writable, and the
    // folder null-terminated, as CreateProcessW requires. The environment
    // block, in UTF-16 as the flag says, ends with two null characters:
    // Windows reads it up to them, inside it. With
    // EXTENDED_STARTUPINFO_PRESENT, it reads a whole STARTUPINFOEXW, whose
    // attribute list outlives the call. No handles are inherited.
    let started = unsafe {
        CreateProcessW(
            ptr::null(),
            program.command_line.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            FALSE,
            flags,
            program
                .environment
                .map_or(ptr::null(), |block| block.as_ptr().cast()),
            program.folder.map_or(ptr::null(), <[u16]>::as_ptr),
            ptr::from_ref(&startup).cast::<STARTUPINFOW>(),
            &mut process,
        )
    };
    if started == 0 {
        return Err(Error::last("CreateProcessW"));
    }
    // SAFETY: CreateProcessW has just opened both handles, and nothing else
    // owns them.
    let (handle, thread) = unsafe {
        (
            OwnedHandle::from_raw_handle(process.hProcess),
            OwnedHandle::from_raw_handle(process.hThread),
        )
    };
    drop(thread);
    Ok((handle, process.dwProcessId))
}

/// The attributes of a new process: the pseudoconsole that it uses, and the
/// jobs that it belongs to.
struct Attributes<'a> {
    /// The memory of the list, in words so that it is aligned as Windows
    /// expects.
    memory: Vec<usize>,
    /// The list points to the jobs, which must outlive it.
    jobs: PhantomData<&'a [HANDLE]>,
}

impl<'a> Attributes<'a> {
    fn new(console: &PseudoConsole, jobs: &'a [HANDLE]) -> Result<Self, Error> {
        let mut size = 0;
        // SAFETY: without a list, it only writes the size that it needs to
        // `size`, and fails with ERROR_INSUFFICIENT_BUFFER.
        unsafe { InitializeProcThreadAttributeList(ptr::null_mut(), 2, 0, &mut size) };
        let mut memory = vec![0_usize; size.div_ceil(mem::size_of::<usize>())];
        let list: LPPROC_THREAD_ATTRIBUTE_LIST = memory.as_mut_ptr().cast();
        // SAFETY: the memory has at least `size` bytes, aligned to a word.
        if unsafe { InitializeProcThreadAttributeList(list, 2, 0, &mut size) } == 0 {
            return Err(Error::last("InitializeProcThreadAttributeList"));
        }
        // From now on, dropping it deletes the list.
        let mut attributes = Self {
            memory,
            jobs: PhantomData,
        };
        // The value of this attribute is the handle of the pseudoconsole
        // itself, not a pointer to it, as Microsoft documents.
        attributes.update(
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
            ptr::without_provenance(console.handle as usize),
            mem::size_of::<HPCON>(),
        )?;
        attributes.update(
            PROC_THREAD_ATTRIBUTE_JOB_LIST,
            jobs.as_ptr().cast(),
            mem::size_of_val(jobs),
        )?;
        Ok(attributes)
    }

    /// Adds an attribute whose value is `size` bytes at `value`, or `value`
    /// itself.
    fn update(&mut self, attribute: u32, value: *const c_void, size: usize) -> Result<(), Error> {
        // SAFETY: the list was initialised in `new`, with room for both
        // attributes. A value that is a pointer stays valid as long as the
        // list: the jobs outlive it, by its lifetime.
        if unsafe {
            UpdateProcThreadAttribute(
                self.as_mut_ptr(),
                0,
                attribute as usize,
                value,
                size,
                ptr::null_mut(),
                ptr::null(),
            )
        } == 0
        {
            return Err(Error::last("UpdateProcThreadAttribute"));
        }
        Ok(())
    }

    fn as_mut_ptr(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.memory.as_mut_ptr().cast()
    }
}

impl Drop for Attributes<'_> {
    fn drop(&mut self) {
        // SAFETY: the list was initialised in `new`, and is deleted only here.
        unsafe { DeleteProcThreadAttributeList(self.as_mut_ptr()) };
    }
}

/// The size of a pseudoconsole as a `COORD`, whose coordinates are 16-bit
/// signed numbers, for `function`. ConPTY needs at least one cell.
fn coord(size: GridSize, function: &'static str) -> Result<COORD, Error> {
    match (i16::try_from(size.columns), i16::try_from(size.rows)) {
        (Ok(columns @ 1..), Ok(rows @ 1..)) => Ok(COORD {
            X: columns,
            Y: rows,
        }),
        _ => {
            let message = format!("{} x {} cells is not a valid size", size.columns, size.rows);
            Err(Error::new(
                function,
                io::Error::new(io::ErrorKind::InvalidInput, message),
            ))
        }
    }
}

/// `text`, which is `what` of a program, as CreateProcessW takes it: in
/// null-terminated UTF-16. A null character would silently cut it.
fn null_terminated(text: &str, what: &str) -> Result<Vec<u16>, Error> {
    if text.contains('\0') {
        return Err(invalid_input(format!("{what} has a null character")));
    }
    Ok(wide(text))
}

/// `block`, if it is an environment block that ends as CreateProcessW
/// expects: with two null characters. Windows reads a block up to the first
/// empty string, so it would read past the end of one without them.
fn environment_block(block: &[u16]) -> Result<&[u16], Error> {
    if block.ends_with(&[0, 0]) {
        Ok(block)
    } else {
        Err(invalid_input(
            "the environment block does not end with two null characters",
        ))
    }
}

/// An error of CreateProcessW for `message`, before calling it.
fn invalid_input(message: impl Into<String>) -> Error {
    Error::new(
        "CreateProcessW",
        io::Error::new(io::ErrorKind::InvalidInput, message.into()),
    )
}

/// An event, not signalled. A wait that sees it resets it, unless it is
/// `manual_reset`: then it stays signalled.
fn event(manual_reset: bool) -> Result<OwnedHandle, Error> {
    // SAFETY: no security attributes and no name. It returns null if it
    // fails.
    let handle = unsafe { CreateEventW(ptr::null(), i32::from(manual_reset), FALSE, ptr::null()) };
    if handle.is_null() {
        return Err(Error::last("CreateEventW"));
    }
    // SAFETY: CreateEventW has just created it, and nothing else owns it.
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}

/// A job for the programs of a session. They may leave it if they ask to,
/// and they do not end when it closes: only if the session gets stuck
/// (ADR 0011).
fn job() -> Result<OwnedHandle, Error> {
    // SAFETY: no security attributes and no name. It returns null if it
    // fails.
    let handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
    if handle.is_null() {
        return Err(Error::last("CreateJobObjectW"));
    }
    // SAFETY: CreateJobObjectW has just created it, and nothing else owns it.
    let job = unsafe { OwnedHandle::from_raw_handle(handle) };
    // Without it, the programs that ask to leave the job, with
    // CREATE_BREAKAWAY_FROM_JOB, would not start. It is an extended limit.
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_BREAKAWAY_OK;
    // SAFETY: the job is valid, and the information is the structure of its
    // class, with its size.
    if unsafe {
        SetInformationJobObject(
            job.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            ptr::from_ref(&limits).cast(),
            mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    } == 0
    {
        return Err(Error::last("SetInformationJobObject"));
    }
    Ok(job)
}

/// A synchronous pipe with a buffer of about `size` bytes: its end to read
/// from and its end to write to. Neither is inherited by new processes.
fn pipe(size: u32) -> Result<(OwnedHandle, OwnedHandle), Error> {
    let (mut read, mut write) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: both pointers are valid for CreatePipe to fill in. Without
    // security attributes, the handles are not inherited.
    if unsafe { CreatePipe(&mut read, &mut write, ptr::null(), size) } == 0 {
        return Err(Error::last("CreatePipe"));
    }
    // SAFETY: CreatePipe has just created both handles, and nothing else owns
    // them.
    Ok(unsafe {
        (
            OwnedHandle::from_raw_handle(read),
            OwnedHandle::from_raw_handle(write),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Condvar;

    use super::*;

    #[test]
    fn sizes_must_fit_in_a_coord() {
        let size = coord(GridSize::new(80, 25), "CreatePseudoConsole").unwrap();
        assert_eq!((size.X, size.Y), (80, 25));
        assert!(coord(GridSize::new(32_767, 1), "CreatePseudoConsole").is_ok());
        assert!(coord(GridSize::new(32_768, 25), "CreatePseudoConsole").is_err());
        let Err(error) = coord(GridSize::new(80, 0), "ResizePseudoConsole") else {
            panic!("80 x 0 is not a valid size");
        };
        assert!(
            error.to_string().starts_with("ResizePseudoConsole"),
            "{error}"
        );
    }

    #[test]
    fn command_lines_and_folders_cannot_have_null_characters() {
        assert_eq!(
            null_terminated("cmd", "the command line").unwrap(),
            [99, 109, 100, 0]
        );
        let error = null_terminated("C:\\x\0y", "the folder").unwrap_err();
        assert_eq!(
            error.to_string(),
            "CreateProcessW failed: the folder has a null character"
        );
    }

    #[test]
    fn an_environment_block_ends_with_two_null_characters() {
        assert!(environment_block(&[0, 0]).is_ok());
        assert!(environment_block(&[65, 61, 49, 0, 0]).is_ok());
        assert!(environment_block(&[65, 61, 49, 0]).is_err());
        assert!(environment_block(&[0]).is_err());
        assert!(environment_block(&[]).is_err());
    }

    /// The pseudoconsoles that [`slow_close`] closed, with the thread that
    /// closed each one.
    static CLOSED: Mutex<Vec<(HPCON, thread::ThreadId)>> = Mutex::new(Vec::new());

    /// Lets [`slow_close`] return.
    static RELEASED: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());

    /// Closes as ClosePseudoConsole of Windows does before Windows 11 24H2
    /// in a session that got stuck: it waits until ConPTY ends, here until
    /// the test lets it go, or 5 s at most.
    extern "system" fn slow_close(handle: HPCON) {
        let (released, changed) = &RELEASED;
        let released = released.lock().unwrap_or_else(PoisonError::into_inner);
        let (released, _) = changed
            .wait_timeout_while(released, Duration::from_secs(5), |released| !*released)
            .unwrap_or_else(PoisonError::into_inner);
        drop(released);
        let closer = thread::current().id();
        CLOSED
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((handle, closer));
    }

    extern "system" fn no_resize(_: HPCON, _: COORD) -> HRESULT {
        0
    }

    /// The thread that closed the pseudoconsole `handle`, once it has.
    fn closer_of(handle: HPCON) -> Option<thread::ThreadId> {
        let closed = CLOSED.lock().unwrap_or_else(PoisonError::into_inner);
        closed
            .iter()
            .find(|&&(closed, _)| closed == handle)
            .map(|&(_, closer)| closer)
    }

    #[test]
    fn a_pseudoconsole_that_may_wait_closes_on_a_thread_of_its_own() {
        let console = |handle, may_wait| PseudoConsole {
            handle,
            close: slow_close,
            resize: no_resize,
            may_wait,
        };
        let started = Instant::now();
        close_console(console(1, true));
        let waited = started.elapsed();
        assert!(
            waited < Duration::from_secs(2),
            "the writer waited {waited:?}"
        );
        assert_eq!(closer_of(1), None);
        let (released, changed) = &RELEASED;
        *released.lock().unwrap_or_else(PoisonError::into_inner) = true;
        changed.notify_all();
        let deadline = Instant::now() + Duration::from_secs(5);
        while closer_of(1).is_none() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        let this = thread::current().id();
        assert!(closer_of(1).is_some_and(|closer| closer != this));
        // The bundled ConPTY, and the one of Windows since 24H2, close on
        // the writer.
        close_console(console(2, false));
        assert_eq!(closer_of(2), Some(this));
    }
}
