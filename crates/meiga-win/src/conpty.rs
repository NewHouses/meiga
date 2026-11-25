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
//! - the writer creates the pseudoconsole and starts the program in it, so
//!   that the thread that starts a session never waits for Windows to
//!   create their processes (ADR 0028). Then it waits at the same time for
//!   input to send, for the session to be closed and for the program to
//!   end. Then it closes the pseudoconsole, and makes sure that the session
//!   ends.
//!
//! The program, and the programs that it starts, run in a job of the
//! session, so that they can all be ended if the session gets stuck. A
//! process that exits while it closes sessions, or while it starts them,
//! ends their writers with it: [`wait_for_closed_sessions`] waits for them
//! before.

use std::ffi::{CStr, c_void};
use std::io;
use std::marker::PhantomData;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
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
    EXTENDED_STARTUPINFO_PRESENT, GetCurrentThread, GetExitCodeProcess, INFINITE,
    InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST, OpenProcess,
    PROC_THREAD_ATTRIBUTE_JOB_LIST, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, PROCESS_INFORMATION,
    PROCESS_SYNCHRONIZE, STARTF_USESTDHANDLES, STARTUPINFOEXW, STARTUPINFOW, SetEvent,
    SetThreadDescription, SetThreadPriority, THREAD_PRIORITY, THREAD_PRIORITY_BELOW_NORMAL,
    THREAD_PRIORITY_NORMAL, UpdateProcThreadAttribute, WaitForMultipleObjects, WaitForSingleObject,
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

    /// Runs `command` in a new pseudoconsole of `size` of this ConPTY, and
    /// gives what it shows to `output`, on a thread of the session: see
    /// [`spawn`], which uses the ConPTY next to the executable.
    pub fn spawn<O: Output>(
        &self,
        command: &Command<'_>,
        size: GridSize,
        output: O,
    ) -> Result<Pty, Error> {
        spawn_in(Choice::Given(self.clone()), command, size, output)
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

/// The ConPTY that [`spawn`] uses: [`Conpty::load`] gives it to the writer
/// of the first session that needs it, which loads its library, if it is
/// the bundled one, on its own thread (ADR 0028). Then it stays for the
/// rest of the process.
static NEXT_TO_EXE: OnceLock<Conpty> = OnceLock::new();

/// The ConPTY of a session: one that the caller gave, or the one next to
/// the executable, which the writer loads the first time.
enum Choice {
    Given(Conpty),
    NextToExe,
}

impl Choice {
    /// The ConPTY, which the first call with [`Choice::NextToExe`] loads,
    /// on the thread that calls it. If two threads call it at the same
    /// time, the second one waits until the first one has loaded it.
    fn get(&self) -> &Conpty {
        match self {
            Self::Given(conpty) => conpty,
            Self::NextToExe => NEXT_TO_EXE.get_or_init(Conpty::load),
        }
    }
}

/// Runs `command` in a new pseudoconsole of `size`, and gives what it shows
/// to `output`, on a thread of the session. The pseudoconsole is the one of
/// the ConPTY next to the executable, or else the one of Windows, as
/// [`Conpty::load`] says: the writer of the first session loads it.
///
/// It returns at once, with the session that starts, and the program starts
/// on the writer thread of the session (ADR 0028): what is written to it
/// meanwhile waits for it, and the size that it is given is the one that it
/// starts with. If the program cannot start, [`Output::failed`] tells why,
/// instead of [`Output::ended`]. If the session is closed before the writer
/// starts the program, the program does not start, and `output` gets
/// [`Output::ended`], without an exit code; if it is closed while the
/// writer starts it, the program starts, and then the session closes as any
/// other.
///
/// The program runs in a job of the session, as the programs that it
/// starts, unless they ask to leave it.
///
/// It fails at once only if `command` or `size` are not valid, or if
/// Windows cannot give what the session needs before it starts: its pipes,
/// its job, its events and its threads. Then `output` is never called.
pub fn spawn<O: Output>(command: &Command<'_>, size: GridSize, output: O) -> Result<Pty, Error> {
    spawn_in(Choice::NextToExe, command, size, output)
}

/// [`spawn`], with the ConPTY of `conpty`.
fn spawn_in<O: Output>(
    conpty: Choice,
    command: &Command<'_>,
    size: GridSize,
    output: O,
) -> Result<Pty, Error> {
    let size = coord(size, "CreatePseudoConsole")?;
    let command_line = null_terminated(command.line, "the command line")?;
    let folder = command
        .folder
        .map(|folder| null_terminated(folder, "the folder"))
        .transpose()?;
    let environment = command
        .environment
        .map(|block| environment_block(block).map(<[u16]>::to_vec))
        .transpose()?;
    let wake = event(false)?;
    let output_closed = event(true)?;
    let job = job()?;
    let (input_read, input_write) = pipe(0)?;
    let (output_read, output_write) = pipe(READ_SIZE)?;
    let shared = Arc::new(Shared {
        event: wake,
        output_closed,
        process: OnceLock::new(),
        reader: OnceLock::new(),
        job,
        state: Mutex::new(State {
            size,
            ..State::default()
        }),
    });
    let creation = Creation {
        conpty,
        command_line,
        folder,
        environment,
        input: input_read,
        output: output_write,
    };
    // Until the program starts, the threads have no number: the writer
    // gives them the one of its process then.
    let writer = {
        let shared = Arc::clone(&shared);
        thread::Builder::new()
            .name("conpty writer".to_owned())
            .spawn(move || write_input(&shared, input_write, creation))
            // Nothing has been created yet: the pipes close with the
            // closure, and `output` is never called.
            .map_err(|error| Error::new("CreateThread", error))?
    };
    let input = Input { shared };
    let reader = {
        let shared = Arc::clone(&input.shared);
        thread::Builder::new()
            .name("conpty reader".to_owned())
            .spawn(move || read_output(output_read, shared, output))
    };
    match reader {
        Ok(reader) => {
            input.shared.keep_reader(reader);
            Ok(Pty {
                input,
                writer: Some(writer),
            })
        }
        Err(error) => {
            // Nobody reads the output, and `output` is never called: the
            // writer does not start the program, or the session closes as
            // when a `Pty` is dropped, without waiting for the output, and
            // its programs end now if they are already running.
            input.shared.terminate();
            input.shared.close_output();
            input.close();
            keep_closing(writer);
            Err(Error::new("CreateThread", error))
        }
    }
}

/// What the writer needs to create the pseudoconsole and to start the
/// program in it.
struct Creation {
    conpty: Choice,
    /// The command line, null-terminated and writable, as CreateProcessW
    /// takes it.
    command_line: Vec<u16>,
    /// The folder, null-terminated, if it is not the one of meiga.
    folder: Option<Vec<u16>>,
    /// The environment block, which ends with two null characters, if it is
    /// not the one of meiga.
    environment: Option<Vec<u16>>,
    /// The ends of the pipes that ConPTY uses: it reads the input from one,
    /// and writes the output to the other.
    input: OwnedHandle,
    output: OwnedHandle,
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
    /// only happens if ConPTY itself fails, or if the session was closed
    /// before the program started.
    fn ended(&mut self, exit_code: Option<u32>);

    /// The program could not start, because of `error`: the pseudoconsole
    /// or the program could not be created, for example because the
    /// program or its folder are not there. It comes instead of
    /// [`Output::ended`], and also last.
    fn failed(&mut self, error: Error);
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

/// A program running in a pseudoconsole, or starting in it: the writer
/// thread of the session creates the pseudoconsole and the program (ADR
/// 0028).
///
/// Dropping it closes the session without waiting. Its writer thread closes
/// the pseudoconsole, which asks the program to end, and its reader thread
/// calls [`Output::ended`] after the last output. If the writer has not
/// started the program yet, it does not start it; if it is starting it,
/// the program starts, and then closes as any other. If the session gets
/// stuck, the writer ends the programs of its job. Before the process exits,
/// [`wait_for_closed_sessions`] waits for the writers of the sessions that
/// were closed, also of the ones that were starting.
pub struct Pty {
    input: Input,
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
/// session got stuck, has ended the programs of its job, or, if it was
/// closed while its program started, has started it and closed it, or has
/// not started it. If a session ended on its own, its programs that are not
/// in its console, such as a window that its shell opened, go on. Call it
/// before the process exits, once no thread closes more sessions: when a
/// process exits, its threads end with it, and a session that got stuck
/// would stay so for ever, with its programs and its ConPTY (ADR 0026).
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

    /// The identifier of the process of the program, once the writer has
    /// started it, or `None` until then, and for ever if it did not start.
    pub fn process_id(&self) -> Option<u32> {
        self.input.shared.process.get().map(|process| process.id)
    }

    /// Changes the size of the pseudoconsole: see [`Input::resize`].
    pub fn resize(&self, size: GridSize) -> Result<(), Error> {
        self.input.resize(size)
    }

    /// Changes the priority of the reader thread, from any thread. It takes
    /// effect at once, also while the reader is giving output to its
    /// [`Output`], and before the program starts: the reader starts with
    /// the session, and waits for the output. Once the reader has ended, it
    /// does nothing.
    pub fn set_reader_priority(&self, priority: Priority) -> Result<(), Error> {
        let value: THREAD_PRIORITY = match priority {
            Priority::Normal => THREAD_PRIORITY_NORMAL,
            Priority::BelowNormal => THREAD_PRIORITY_BELOW_NORMAL,
        };
        // The session gives it its reader before it gives the `Pty`.
        let Some(reader) = self.input.shared.reader.get() else {
            return Ok(());
        };
        // SAFETY: the handle is the one of the reader thread, which the
        // JoinHandle keeps open, also after the thread ends; it has every
        // access right, THREAD_SET_INFORMATION among them.
        if unsafe { SetThreadPriority(reader.as_raw_handle(), value) } == FALSE {
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
    /// wait: the writer thread sends them, once the program has started if
    /// it is still starting. Once the session has been closed or has ended,
    /// or if the program did not start, it does nothing.
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
    /// wait for ConPTY. While the writer creates the pseudoconsole, it keeps
    /// the size, which the pseudoconsole gets once it is there. Once the
    /// session has been closed or has ended, it does nothing.
    pub fn resize(&self, size: GridSize) -> Result<(), Error> {
        let size = coord(size, "ResizePseudoConsole")?;
        let mut state = self.shared.lock();
        if state.closing || state.finished {
            return Ok(());
        }
        state.size = size;
        let Some(console) = state.console.as_ref() else {
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
    /// pseudoconsole or, if it has not created it yet, does not create it,
    /// and if it has not started the program yet, does not start it.
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
    /// The program, once the writer has started it.
    process: OnceLock<Process>,
    /// The reader thread, whose priority can change, from the moment that
    /// the session starts. Dropping it does not wait for the thread: it goes
    /// on until the output ends.
    reader: OnceLock<JoinHandle<()>>,
    /// The job of the program and of the programs that it starts. They may
    /// leave it if they ask to, with `CREATE_BREAKAWAY_FROM_JOB`.
    job: OwnedHandle,
    state: Mutex<State>,
}

/// The program of a session.
struct Process {
    handle: OwnedHandle,
    id: u32,
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

    /// Keeps the reader thread, which the thread that started the session
    /// created after the writer. If the program has already started, it
    /// gives the reader the number of its process, as [`Shared::started`]
    /// does: under the lock, one of them always sees the other.
    fn keep_reader(&self, reader: JoinHandle<()>) {
        let reader = self.reader.get_or_init(|| reader);
        let _state = self.lock();
        if let Some(process) = self.process.get() {
            name_reader(reader, process.id);
        }
    }

    /// Notes, on the writer thread, that the program started, and gives the
    /// writer and the reader the number of its process in their names,
    /// which only had the kind of the thread until now: the checks and the
    /// profiles of Windows find them by it. The names come first, under the
    /// lock: whoever sees the program through the session, as
    /// [`Pty::process_id`] does, sees the name of the writer, and the one
    /// of the reader unless the reader has not started yet: the standard
    /// library gives it its own name as it starts, and then the reader
    /// takes the number again ([`read_output`]). Windows shows the process
    /// of the program earlier, as soon as it is created.
    fn started(&self, process: Process) -> &Process {
        let _state = self.lock();
        name_thread(
            // SAFETY: the pseudo handle of the calling thread, the writer,
            // which needs no closing.
            unsafe { GetCurrentThread() },
            &format!("conpty writer {}", process.id),
        );
        if let Some(reader) = self.reader.get() {
            name_reader(reader, process.id);
        }
        self.process.get_or_init(|| process)
    }

    /// Notes, on the writer thread, why the program could not start: the
    /// reader tells it, once the output closes. The session is finished
    /// from then on, and takes no more input nor sizes: also if the reader
    /// tells it before the writer gets to close the pseudoconsole.
    fn fail(&self, error: Error) {
        let mut state = self.lock();
        state.failure = Some(error);
        state.finished = true;
    }

    /// Waits, once the pseudoconsole has closed, for the session to end: for
    /// ConPTY to close the output and for `process`, the program, to end.
    /// Meanwhile, the programs that the session had must keep ending:
    /// Windows asks them to end one at a time. If none of them ends for
    /// [`CLOSE_TIMEOUT_MS`], something keeps the pseudoconsole open: usually
    /// a program that was connecting to it when it closed, and that waits
    /// for ConPTY, while ConPTY waits for it (ADR 0011). Then every program
    /// of the session ends.
    fn wait_for_end(&self, process: &Process) {
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
        let process = process.handle.as_raw_handle();
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
    /// Input that the writer has not taken yet: also what was written while
    /// the program started.
    input: Vec<u8>,
    /// The session was closed.
    closing: bool,
    /// The writer has closed the pseudoconsole, or the program did not
    /// start: input is no longer sent.
    finished: bool,
    /// The pseudoconsole, from the moment that the writer creates it until
    /// it takes it to close it.
    console: Option<PseudoConsole>,
    /// The size of the pseudoconsole: the last one that it was given, which
    /// is also the one that it is created with.
    size: COORD,
    /// Why the program could not start, until the reader tells it.
    failure: Option<Error>,
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

/// The writer thread: creates the pseudoconsole and starts the program in
/// it, sends the input to the program, closes the pseudoconsole when the
/// program ends or the session is closed, and makes sure that the session
/// ends. Creating them here, and not on the thread that starts the session,
/// means that the thread of the window never waits for Windows to create a
/// process, which sometimes takes some hundreds of milliseconds (ADR 0028).
fn write_input(shared: &Shared, pipe: OwnedHandle, creation: Creation) {
    let process = create(shared, creation);
    if let Some(process) = process {
        send_input(shared, process, &pipe);
    }
    let console = {
        let mut state = shared.lock();
        state.finished = true;
        state.console.take()
    };
    drop(pipe);
    // Without a program, the pseudoconsole closes at once, and ConPTY ends:
    // the reader reads its output until then.
    if let Some(console) = console {
        close_console(console);
    }
    if let Some(process) = process {
        shared.wait_for_end(process);
    }
}

/// Creates the pseudoconsole, with the last size that the session was
/// given, and starts the program in it, unless the session is closed before.
/// Returns the program, or `None` if it did not start: then, if creating
/// them failed, the reader tells why, once the output closes. The
/// pseudoconsole, if there is one, stays in the state, for the writer to
/// close it.
fn create(shared: &Shared, creation: Creation) -> Option<&Process> {
    let Creation {
        conpty,
        mut command_line,
        folder,
        environment,
        input,
        output,
    } = creation;
    let conpty = conpty.get();
    let size = {
        let state = shared.lock();
        if state.closing {
            return None;
        }
        state.size
    };
    let console = match conpty.create(size, &input, &output) {
        Ok(console) => console,
        Err(error) => {
            // The output closes when `output` does, right after this.
            shared.fail(error);
            return None;
        }
    };
    // ConPTY keeps copies of its ends of the pipes. Without ours, the pipes
    // break when ConPTY ends, so the reader and the writer do not wait for
    // ever.
    drop((input, output));
    let handle = console.handle;
    {
        let mut state = shared.lock();
        // The session may have been given another size meanwhile: from now
        // on, `Input::resize` gives it to the pseudoconsole itself.
        if (state.size.X, state.size.Y) != (size.X, size.Y) {
            // SAFETY: the handle has just been created by the same ConPTY.
            // If it fails, the pseudoconsole keeps the size that it was
            // created with, as when `Input::resize` fails.
            let _ = unsafe { (console.resize)(console.handle, state.size) };
        }
        state.console = Some(console);
        // Closed while the pseudoconsole was created: the program does not
        // start, and the writer closes the pseudoconsole, which has no
        // program yet.
        if state.closing {
            return None;
        }
    }
    let program = Program {
        command_line: &mut command_line,
        folder: folder.as_deref(),
        environment: environment.as_deref(),
    };
    // The pseudoconsole stays open meanwhile: only this thread closes it.
    match start(program, handle, &shared.job) {
        Ok(process) => Some(shared.started(process)),
        Err(error) => {
            // The output closes when the writer closes the pseudoconsole.
            shared.fail(error);
            None
        }
    }
}

/// Sends the input to `process`, the program, until it ends or the session
/// is closed.
fn send_input(shared: &Shared, process: &Process, pipe: &OwnedHandle) {
    let handles = [shared.event.as_raw_handle(), process.handle.as_raw_handle()];
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
        if !write_all(pipe, &pending) {
            // ConPTY closed its end of the input pipe: it has ended.
            break;
        }
        pending.clear();
    }
}

/// Closes a pseudoconsole, which asks its programs to end, without waiting
/// for ConPTY to end. The bundled ConPTY, and the one of Windows since
/// Windows 11 24H2, return at once. Before 24H2, ClosePseudoConsole of
/// Windows waits until ConPTY ends, which can take a while: that is why
/// closing a session never calls it on the thread of the window, and only
/// the writer does. A pseudoconsole whose program did not start has no
/// programs, and its output is read, so nothing keeps ConPTY waiting. But
/// ConPTY does not end while a program that was connecting to it when it
/// closed waits for it: the writer would wait for ever, and never end the
/// programs of the session (ADR 0026). So a pseudoconsole that may wait
/// closes on a thread of its own, which ends once ConPTY ends, and nobody
/// waits for it.
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
/// ConPTY ends and closes its end of the output pipe, or until the writer
/// closes its own end, if it did not create the pseudoconsole. Then it tells
/// `output` how the session ended: why the program could not start, or how
/// it ended.
fn read_output<O: Output>(pipe: OwnedHandle, shared: Arc<Shared>, mut output: O) {
    // The standard library gives this thread its name as it starts, after
    // the writer may have given it the number of the program: then it goes
    // in it again. Under the lock, as the writer gives it.
    {
        let _state = shared.lock();
        if let Some(process) = shared.process.get() {
            name_thread(
                // SAFETY: the pseudo handle of this thread, which needs no
                // closing.
                unsafe { GetCurrentThread() },
                &format!("conpty reader {}", process.id),
            );
        }
    }
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
    // The writer notes a failure before the output can close: before it
    // closes its end of the pipe, or the pseudoconsole.
    let failure = input.shared.lock().failure.take();
    match failure {
        Some(error) => output.failed(error),
        None => {
            let process = input.shared.process.get();
            output.ended(process.and_then(|process| exit_code(&process.handle)));
        }
    }
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

/// Gives `thread` the name `name`, which debuggers, the profiles of Windows
/// and the checks of meiga show. If it fails, the thread keeps the name that
/// it had.
fn name_thread(thread: HANDLE, name: &str) {
    let name = wide(name);
    // SAFETY: the handle is valid during the call, with every access right,
    // and the name is null-terminated.
    unsafe { SetThreadDescription(thread, name.as_ptr()) };
}

/// Gives `reader`, the reader thread of a session, the number of the process
/// of its program.
fn name_reader(reader: &JoinHandle<()>, id: u32) {
    name_thread(reader.as_raw_handle(), &format!("conpty reader {id}"));
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

/// Starts `program` in the pseudoconsole `console`, which stays open
/// meanwhile, in `job`. Returns its process.
fn start(program: Program<'_>, console: HPCON, job: &OwnedHandle) -> Result<Process, Error> {
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
    Ok(Process {
        handle,
        id: process.dwProcessId,
    })
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
    fn new(console: HPCON, jobs: &'a [HANDLE]) -> Result<Self, Error> {
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
            ptr::without_provenance(console as usize),
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
    use std::collections::BTreeMap;
    use std::sync::{Condvar, mpsc};
    use std::{env, fs, process, slice};

    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::System::Threading::GetThreadDescription;
    use windows_sys::core::PWSTR;

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

    // The sessions that the writer creates (ADR 0028), with fake ConPTYs:
    // ones of Windows whose create function waits at a gate until the test
    // opens it, so that the test can act while the writer creates the
    // pseudoconsole.

    /// `E_INVALIDARG`, as a fake function fails.
    const INVALID_ARGUMENT: HRESULT = 0x8007_0057_u32 as HRESULT;

    /// In the flags of a fake ConPTY: its create function fails, once its
    /// gate opens.
    const FAILS: u32 = 0x8000_0000;

    /// What the fake ConPTYs of the tests did, and the gates of their create
    /// functions: by the flags of each one, which tell the tests apart, the
    /// sizes that it created pseudoconsoles with, and whether its gate is
    /// open; and the sizes that pseudoconsoles were given.
    struct Fakes {
        created: BTreeMap<u32, Vec<(i16, i16)>>,
        open: BTreeMap<u32, bool>,
        resized: Vec<(i16, i16)>,
    }

    static FAKES: (Mutex<Fakes>, Condvar) = (
        Mutex::new(Fakes {
            created: BTreeMap::new(),
            open: BTreeMap::new(),
            resized: Vec::new(),
        }),
        Condvar::new(),
    );

    fn fakes() -> MutexGuard<'static, Fakes> {
        FAKES.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Notes the size, waits until the gate of `flags` opens, or 30 s at
    /// most, and then creates a pseudoconsole of Windows, or fails if the
    /// flags have [`FAILS`].
    extern "system" fn gated_create(
        size: COORD,
        input: HANDLE,
        output: HANDLE,
        flags: u32,
        console: *mut HPCON,
    ) -> HRESULT {
        let (fakes, changed) = &FAKES;
        let mut noted = fakes.lock().unwrap_or_else(PoisonError::into_inner);
        noted
            .created
            .entry(flags)
            .or_default()
            .push((size.X, size.Y));
        changed.notify_all();
        let (opened, _) = changed
            .wait_timeout_while(noted, Duration::from_secs(30), |fakes| {
                !fakes.open.get(&flags).copied().unwrap_or(false)
            })
            .unwrap_or_else(PoisonError::into_inner);
        drop(opened);
        if flags & FAILS != 0 {
            return INVALID_ARGUMENT;
        }
        // SAFETY: the arguments are the ones that `Conpty::create` gives,
        // for the same function of Windows.
        unsafe { CreatePseudoConsole(size, input, output, 0, console) }
    }

    /// Gives a pseudoconsole of Windows another size, and notes it.
    extern "system" fn noted_resize(console: HPCON, size: COORD) -> HRESULT {
        fakes().resized.push((size.X, size.Y));
        // SAFETY: `gated_create` created it with the function of Windows.
        unsafe { ResizePseudoConsole(console, size) }
    }

    extern "system" fn failing_resize(_: HPCON, _: COORD) -> HRESULT {
        INVALID_ARGUMENT
    }

    extern "system" fn no_close(_: HPCON) {}

    /// The ConPTY of Windows, with the create function that waits at the
    /// gate of `flags`.
    fn gated(flags: u32) -> Conpty {
        Conpty {
            source: Source::System,
            create: gated_create,
            close: ClosePseudoConsole,
            resize: noted_resize,
            flags,
        }
    }

    /// Opens the gate of `flags`.
    fn open(flags: u32) {
        fakes().open.insert(flags, true);
        FAKES.1.notify_all();
    }

    /// Waits until the create function of `flags` is called, and returns the
    /// sizes that it got.
    fn wait_created(flags: u32) -> Vec<(i16, i16)> {
        let (fakes, changed) = &FAKES;
        let fakes = fakes.lock().unwrap_or_else(PoisonError::into_inner);
        let (fakes, _) = changed
            .wait_timeout_while(fakes, Duration::from_secs(30), |fakes| {
                !fakes.created.contains_key(&flags)
            })
            .unwrap_or_else(PoisonError::into_inner);
        fakes.created.get(&flags).cloned().unwrap_or_default()
    }

    /// What a session of a test tells.
    enum Told {
        Received(Vec<u8>),
        Ended(Option<u32>),
        Failed(Error),
    }

    struct Teller(mpsc::Sender<Told>);

    impl Output for Teller {
        fn received(&mut self, bytes: &[u8], _input: &Input) {
            let _ = self.0.send(Told::Received(bytes.to_vec()));
        }

        fn ended(&mut self, exit_code: Option<u32>) {
            let _ = self.0.send(Told::Ended(exit_code));
        }

        fn failed(&mut self, error: Error) {
            let _ = self.0.send(Told::Failed(error));
        }
    }

    /// Waits until a session tells how it ended, and returns it, with all
    /// that the session showed.
    fn end_of(told: &mpsc::Receiver<Told>) -> (Told, String) {
        let mut shown = Vec::new();
        loop {
            match told.recv_timeout(Duration::from_secs(60)) {
                Ok(Told::Received(bytes)) => shown.extend(bytes),
                Ok(end) => return (end, String::from_utf8_lossy(&shown).into_owned()),
                Err(error) => panic!("{error}: the session did not end"),
            }
        }
    }

    /// A session as `spawn_in` prepares it, before its writer creates it:
    /// what its threads share, with a grid of 80 x 25, what its writer
    /// creates it with, and the ends of the pipes that meiga uses, to write
    /// the input and to read the output.
    fn session(
        conpty: Conpty,
        line: &str,
        folder: Option<&str>,
    ) -> (Shared, Creation, OwnedHandle, OwnedHandle) {
        let (input_read, input_write) = pipe(0).unwrap();
        let (output_read, output_write) = pipe(READ_SIZE).unwrap();
        let shared = Shared {
            event: event(false).unwrap(),
            output_closed: event(true).unwrap(),
            process: OnceLock::new(),
            reader: OnceLock::new(),
            job: job().unwrap(),
            state: Mutex::new(State {
                size: COORD { X: 80, Y: 25 },
                ..State::default()
            }),
        };
        let creation = Creation {
            conpty: Choice::Given(conpty),
            command_line: wide(line),
            folder: folder.map(wide),
            environment: None,
            input: input_read,
            output: output_write,
        };
        (shared, creation, input_write, output_read)
    }

    /// The name of `thread`, as Windows has it.
    fn description(thread: HANDLE) -> String {
        let mut text: PWSTR = ptr::null_mut();
        // SAFETY: the handle is valid, and `text` is valid for it to fill
        // in.
        let result = unsafe { GetThreadDescription(thread, &mut text) };
        assert!(result >= 0, "GetThreadDescription failed: {result:#x}");
        let mut length = 0;
        // SAFETY: Windows gives a null-terminated string, which this reads
        // up to the null character, and then frees.
        unsafe {
            while *text.add(length) != 0 {
                length += 1;
            }
            let name = String::from_utf16_lossy(slice::from_raw_parts(text, length));
            LocalFree(text.cast());
            name
        }
    }

    #[test]
    fn the_writer_creates_the_session_with_what_it_was_given_meanwhile() {
        const FLAGS: u32 = 0x0001_0000;
        let (sender, told) = mpsc::channel();
        // cmd reads a line and shows it twice: "olaola" only comes from cmd.
        let pty = gated(FLAGS)
            .spawn(
                &Command::new("cmd /d /v:on /c \"set /p M=& echo !M!!M!\""),
                GridSize::new(80, 25),
                Teller(sender),
            )
            .unwrap();
        // spawn has returned, and the writer creates the pseudoconsole, with
        // the size that the session started with.
        assert_eq!(wait_created(FLAGS), [(80, 25)]);
        assert_eq!(pty.process_id(), None);
        assert!(pty.input.shared.lock().console.is_none());
        // What the window gives meanwhile waits for the pseudoconsole and
        // for the program, as in a tab that opens.
        pty.input().write(b"ola\r");
        pty.resize(GridSize::new(101, 31)).unwrap();
        open(FLAGS);
        let (end, shown) = end_of(&told);
        assert!(matches!(end, Told::Ended(Some(0))), "{shown:?}");
        assert!(shown.contains("olaola"), "{shown:?}");
        assert!(pty.process_id().is_some());
        // The writer gave the new size to the pseudoconsole once it had it.
        assert!(fakes().resized.contains(&(101, 31)));
    }

    #[test]
    fn a_session_closed_while_its_pseudoconsole_is_created_does_not_start_its_program() {
        const FLAGS: u32 = 0x0002_0000;
        // The program would write this file, if it started.
        let file = env::temp_dir().join(format!("meiga-conpty-closed-{}.txt", process::id()));
        let _ = fs::remove_file(&file);
        let line = format!("cmd /d /c echo started>\"{}\"", file.display());
        let (sender, told) = mpsc::channel();
        let pty = gated(FLAGS)
            .spawn(&Command::new(&line), GridSize::new(80, 25), Teller(sender))
            .unwrap();
        assert_eq!(wait_created(FLAGS).len(), 1);
        let writer = pty.writer.as_ref().map(|writer| writer.thread().id());
        // The tab closes while the writer creates the pseudoconsole.
        drop(pty);
        // Its writer goes with the ones of the other sessions that were
        // closed: before it exits, meiga waits for it too (ADR 0026).
        assert!(
            CLOSING
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .iter()
                .any(|closing| Some(closing.thread().id()) == writer)
        );
        open(FLAGS);
        // The writer closes the pseudoconsole without a program: ConPTY ends,
        // and the session ends without an exit code.
        let (end, shown) = end_of(&told);
        assert!(matches!(end, Told::Ended(None)), "{shown:?}");
        assert!(!file.exists());
    }

    #[test]
    fn a_session_closed_before_its_writer_creates_it_creates_nothing() {
        const FLAGS: u32 = 0x0003_0000;
        open(FLAGS);
        let (shared, creation, _input, _output) = session(gated(FLAGS), "cmd /c exit", None);
        shared.lock().closing = true;
        assert!(create(&shared, creation).is_none());
        assert!(!fakes().created.contains_key(&FLAGS));
        let state = shared.lock();
        assert!(state.console.is_none() && state.failure.is_none());
        assert!(shared.process.get().is_none());
    }

    #[test]
    fn a_pseudoconsole_that_cannot_be_created_tells_why() {
        const FLAGS: u32 = FAILS | 0x0004_0000;
        open(FLAGS);
        let (sender, told) = mpsc::channel();
        let pty = gated(FLAGS)
            .spawn(
                &Command::new("cmd /c exit"),
                GridSize::new(80, 25),
                Teller(sender),
            )
            .unwrap();
        // The reader tells it, once the writer closes its end of the output.
        let (end, shown) = end_of(&told);
        let Told::Failed(error) = end else {
            panic!("the session did not fail: {shown:?}");
        };
        assert_eq!(
            error.os_error().and_then(io::Error::raw_os_error),
            Some(87),
            "{error}"
        );
        assert!(
            error
                .to_string()
                .starts_with("CreatePseudoConsole failed: "),
            "{error}"
        );
        assert!(shown.is_empty(), "{shown:?}");
        assert_eq!(pty.process_id(), None);
        // The function of the bundled ConPTY has another name.
        let bundled = Conpty {
            source: Source::Bundled(PathBuf::from(r"C:\meiga")),
            ..gated(FLAGS)
        };
        let (input, input_write) = pipe(0).unwrap();
        let (output_read, output) = pipe(0).unwrap();
        let Err(error) = bundled.create(COORD { X: 80, Y: 25 }, &input, &output) else {
            panic!("the fake ConPTY created a pseudoconsole");
        };
        assert!(
            error
                .to_string()
                .starts_with("ConptyCreatePseudoConsole failed: "),
            "{error}"
        );
        drop((input_write, output_read));
    }

    #[test]
    fn what_is_not_valid_fails_before_the_session_starts() {
        let conpty = Conpty::system();
        let spawned = |command: Command<'_>, size: GridSize| {
            let (sender, told) = mpsc::channel();
            let result = conpty.spawn(&command, size, Teller(sender));
            // `output` is never called: it goes with the session that did
            // not start.
            assert!(told.recv().is_err());
            result.err().map(|error| error.to_string())
        };
        let folder = Command {
            folder: Some("C:\\x\0y"),
            ..Command::new("cmd /c exit")
        };
        assert_eq!(
            spawned(folder, GridSize::new(80, 25)).as_deref(),
            Some("CreateProcessW failed: the folder has a null character")
        );
        let environment = Command {
            environment: Some(&[65, 61, 49, 0]),
            ..Command::new("cmd /c exit")
        };
        assert_eq!(
            spawned(environment, GridSize::new(80, 25)).as_deref(),
            Some(
                "CreateProcessW failed: the environment block does not end with two null characters"
            )
        );
        let line = spawned(Command::new("cmd\0"), GridSize::new(80, 25));
        assert!(line.is_some_and(|error| error.contains("the command line has a null character")));
        let size = spawned(Command::new("cmd /c exit"), GridSize::new(0, 25));
        assert!(size.is_some_and(|error| error.starts_with("CreatePseudoConsole failed: ")));
    }

    #[test]
    fn a_program_that_cannot_start_leaves_its_pseudoconsole_to_the_writer() {
        const FLAGS: u32 = 0x0005_0000;
        open(FLAGS);
        let (shared, creation, _input, _output) = session(
            gated(FLAGS),
            "cmd /c exit",
            Some(r"C:\meiga\no such folder"),
        );
        assert!(create(&shared, creation).is_none());
        // From then on, the session takes no input nor sizes, which no
        // program would get: also before the writer marks it as finished.
        let input = Input {
            shared: Arc::new(shared),
        };
        input.write(b"ola");
        input.resize(GridSize::new(90, 28)).unwrap();
        let mut state = input.shared.lock();
        assert!(state.finished && state.input.is_empty());
        assert_eq!((state.size.X, state.size.Y), (80, 25));
        let failure = state.failure.take().expect("the writer notes why");
        // ERROR_DIRECTORY.
        assert_eq!(
            failure.os_error().and_then(io::Error::raw_os_error),
            Some(267),
            "{failure}"
        );
        // The writer closes it: it is there, without a program.
        assert!(state.console.is_some());
        assert!(input.shared.process.get().is_none());
    }

    #[test]
    fn the_writer_and_the_reader_get_the_number_of_the_program_when_it_starts() {
        const FLAGS: u32 = 0x0006_0000;
        let (sender, told) = mpsc::channel();
        let pty = gated(FLAGS)
            .spawn(
                &Command::new("cmd /k"),
                GridSize::new(80, 25),
                Teller(sender),
            )
            .unwrap();
        assert_eq!(wait_created(FLAGS).len(), 1);
        let reader = pty
            .input
            .shared
            .reader
            .get()
            .expect("the reader starts with the session")
            .as_raw_handle();
        let writer = pty
            .writer
            .as_ref()
            .expect("the session is open")
            .as_raw_handle();
        // Each thread gets its first name as it starts.
        let named = |thread: HANDLE, name: &str| {
            let deadline = Instant::now() + Duration::from_secs(30);
            while description(thread) != name {
                assert!(
                    Instant::now() < deadline,
                    "{name:?}: {:?}",
                    description(thread)
                );
                thread::sleep(Duration::from_millis(1));
            }
        };
        named(reader, "conpty reader");
        named(writer, "conpty writer");
        open(FLAGS);
        let deadline = Instant::now() + Duration::from_secs(30);
        let id = loop {
            if let Some(id) = pty.process_id() {
                break id;
            }
            assert!(Instant::now() < deadline, "the program did not start");
            thread::sleep(Duration::from_millis(1));
        };
        assert_eq!(description(reader), format!("conpty reader {id}"));
        assert_eq!(description(writer), format!("conpty writer {id}"));
        drop(pty);
        let (end, shown) = end_of(&told);
        assert!(matches!(end, Told::Ended(Some(_))), "{shown:?}");
    }

    #[test]
    fn the_reader_gets_the_number_of_the_program_whichever_comes_first() {
        // The writer may start the program before the thread that starts the
        // session keeps the reader, or after.
        for program_first in [true, false] {
            let (shared, _creation, _input, _output) =
                session(Conpty::system(), "cmd /c exit", None);
            let shared = Arc::new(shared);
            let reader = thread::Builder::new()
                .name("conpty reader".to_owned())
                .spawn(thread::park)
                .unwrap();
            // Once it has its first name, which it gets as it starts.
            let deadline = Instant::now() + Duration::from_secs(30);
            while description(reader.as_raw_handle()) != "conpty reader" {
                assert!(Instant::now() < deadline, "the thread got no name");
                thread::sleep(Duration::from_millis(1));
            }
            let process = Process {
                handle: event(false).unwrap(),
                id: 4242,
            };
            // On another thread, as the writer: it names the thread that
            // calls it.
            let writer = {
                let shared = Arc::clone(&shared);
                move || {
                    shared.started(process);
                }
            };
            if program_first {
                thread::spawn(writer).join().unwrap();
                shared.keep_reader(reader);
            } else {
                shared.keep_reader(reader);
                thread::spawn(writer).join().unwrap();
            }
            let reader = shared.reader.get().unwrap();
            assert_eq!(
                description(reader.as_raw_handle()),
                "conpty reader 4242",
                "program first: {program_first}"
            );
            reader.thread().unpark();
        }
    }

    #[test]
    fn a_reader_that_starts_after_the_program_takes_its_number() {
        let (shared, creation, _input, output) = session(Conpty::system(), "cmd /c exit", None);
        let shared = Arc::new(shared);
        // A program that has ended: an event that is set, so that the reader
        // does not wait for it when the output closes.
        let process = Process {
            handle: event(true).unwrap(),
            id: 4343,
        };
        // SAFETY: the event is valid.
        unsafe { SetEvent(process.handle.as_raw_handle()) };
        {
            let shared = Arc::clone(&shared);
            thread::spawn(move || {
                shared.started(process);
            })
            .join()
            .unwrap();
        }
        let (sender, told) = mpsc::channel();
        let reader = {
            let shared = Arc::clone(&shared);
            thread::Builder::new()
                .name("conpty reader".to_owned())
                .spawn(move || read_output(output, shared, Teller(sender)))
                .unwrap()
        };
        let deadline = Instant::now() + Duration::from_secs(30);
        while description(reader.as_raw_handle()) != "conpty reader 4343" {
            assert!(
                Instant::now() < deadline,
                "{:?}",
                description(reader.as_raw_handle())
            );
            thread::sleep(Duration::from_millis(1));
        }
        // Without the end that ConPTY would write to, the output closes.
        drop(creation);
        let (end, shown) = end_of(&told);
        // The program is not a real one: its exit code means nothing.
        assert!(matches!(end, Told::Ended(_)), "{shown:?}");
        reader.join().unwrap();
    }

    #[test]
    fn the_input_waits_for_the_program_and_stops_with_the_session() {
        let (shared, _creation, _input, _output) = session(Conpty::system(), "cmd /c exit", None);
        let input = Input {
            shared: Arc::new(shared),
        };
        let woken = || {
            // SAFETY: the event is valid while `input` exists. Seeing it
            // resets it.
            unsafe { WaitForSingleObject(input.shared.event.as_raw_handle(), 0) == WAIT_OBJECT_0 }
        };
        // Nothing to send wakes nobody up.
        input.write(b"");
        assert!(input.shared.lock().input.is_empty());
        assert!(!woken());
        // Until the writer takes them, the input waits, which wakes the writer
        // up once, and the size, which the pseudoconsole is created with.
        input.write(b"ola");
        input.write(b" meiga");
        assert!(woken());
        assert!(!woken());
        input.resize(GridSize::new(120, 40)).unwrap();
        {
            let state = input.shared.lock();
            assert_eq!(state.input, b"ola meiga");
            assert_eq!((state.size.X, state.size.Y), (120, 40));
        }
        // A size that a pseudoconsole cannot have is an error.
        assert!(input.resize(GridSize::new(0, 40)).is_err());
        // Once the program did not start, or the session closed, nothing
        // more.
        input.shared.lock().finished = true;
        input.write(b"!");
        input.resize(GridSize::new(10, 10)).unwrap();
        input.shared.lock().finished = false;
        input.close();
        assert!(woken());
        input.write(b"!");
        input.resize(GridSize::new(10, 10)).unwrap();
        let state = input.shared.lock();
        assert_eq!(state.input, b"ola meiga");
        assert_eq!((state.size.X, state.size.Y), (120, 40));
    }

    #[test]
    fn a_size_that_the_pseudoconsole_does_not_take_is_an_error() {
        let (shared, _creation, _input, _output) = session(Conpty::system(), "cmd /c exit", None);
        shared.lock().console = Some(PseudoConsole {
            handle: 1,
            close: no_close,
            resize: failing_resize,
            may_wait: false,
        });
        let input = Input {
            shared: Arc::new(shared),
        };
        let error = input.resize(GridSize::new(90, 30)).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("ResizePseudoConsole failed: "),
            "{error}"
        );
    }

    #[test]
    fn the_conpty_next_to_the_executable_is_loaded_once() {
        let first = Choice::NextToExe.get();
        let second = Choice::NextToExe.get();
        assert!(ptr::eq(first, second));
        // The tests have no conpty.dll next to them.
        assert_eq!(first.source(), &Source::System);
    }
}
