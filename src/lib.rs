#[macro_use]
extern crate failure;
#[macro_use]
extern crate winapi;

use {
    std::io,
    winapi::{
        ctypes::c_void,
        shared::minwindef::DWORD,
        um::{
            consoleapi::{
                GetConsoleMode,
                SetConsoleMode,
            },
            processenv::GetStdHandle,
            winbase::{
                STD_ERROR_HANDLE,
                STD_INPUT_HANDLE,
                STD_OUTPUT_HANDLE,
            },
            wincon::{
                DISABLE_NEWLINE_AUTO_RETURN,
                ENABLE_ECHO_INPUT,
                ENABLE_LINE_INPUT,
                ENABLE_PROCESSED_INPUT,
                ENABLE_PROCESSED_OUTPUT,
                ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                ENABLE_WRAP_AT_EOL_OUTPUT,
            },
        },
    },
};

#[derive(Debug)]
struct ConsoleHandle {
    handle: *mut c_void,
    state_to_restore: DWORD,
    state: DWORD,
}

impl ConsoleHandle {
    unsafe fn from_std_stream(std_handle: DWORD) -> Result<ConsoleHandle, io::Error> {
        let handle = GetStdHandle(std_handle);

        let mut state_to_restore = 0;
        if GetConsoleMode(handle, &mut state_to_restore as *mut DWORD) == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(ConsoleHandle {
            handle,
            state_to_restore,
            state: state_to_restore,
        })
    }

    fn flush_console_state(&mut self) -> io::Result<()> {
        match unsafe { SetConsoleMode(self.handle, self.state) } {
            0 => Err(io::Error::last_os_error()),
            _ => Ok(()),
        }
    }

    fn set_flags(&mut self, flags: DWORD) -> io::Result<()> {
        self.state |= flags;
        self.flush_console_state()
    }

    fn unset_flags(&mut self, flags: DWORD) -> io::Result<()> {
        self.state &= !flags;
        self.flush_console_state()
    }
}

impl Drop for ConsoleHandle {
    fn drop(&mut self) {
        if unsafe { SetConsoleMode(self.handle, self.state_to_restore) } == 0 {
            eprintln!("warning: could not reset console state from ANSI processing");
        }
    }
}

#[derive(Debug)]
pub struct StdErrorHandle(ConsoleHandle);

impl StdErrorHandle {
    pub fn new() -> Option<Result<Self, io::Error>> {
        Some(Ok(StdErrorHandle(unsafe {
            match ConsoleHandle::from_std_stream(STD_ERROR_HANDLE) {
                Ok(ch) => ch,
                Err(e) => return Some(Err(e)),
            }
        })))
    }
}

#[derive(Debug)]
pub struct StdInputHandle(ConsoleHandle);

impl StdInputHandle {
    pub fn new() -> Option<io::Result<Self>> {
        Some(Ok(StdInputHandle(unsafe {
            match ConsoleHandle::from_std_stream(STD_INPUT_HANDLE) {
                Ok(ch) => ch,
                Err(e) => return Some(Err(e)),
            }
        })))
    }
}

#[derive(Debug)]
pub struct StdOutputHandle(ConsoleHandle);

impl StdOutputHandle {
    pub fn new() -> Option<io::Result<Self>> {
        Some(Ok(StdOutputHandle(unsafe {
            match ConsoleHandle::from_std_stream(STD_OUTPUT_HANDLE) {
                Ok(ch) => ch,
                Err(e) => return Some(Err(e)),
            }
        })))
    }
}

#[derive(Debug)]
pub struct AnsiTerminal {
    stdin: StdInputHandle,
    stdout: StdOutputHandle,
    stderr: StdErrorHandle,
}

impl AnsiTerminal {
    pub fn new() -> Result<Self, TerminalSetupError> {
        use TerminalSetupError::*;

        let stderr = StdErrorHandle::new().unwrap().map_err(Stderr)?;
        let stdin = StdInputHandle::new().unwrap().map_err(Stdin)?;
        let stdout = StdOutputHandle::new().unwrap().map_err(Stdout)?;

        let mut t = AnsiTerminal {
            stderr,
            stdin,
            stdout,
        };
        t.stdout
            .0
            .set_flags(ENABLE_VIRTUAL_TERMINAL_PROCESSING)
            .map_err(TerminalModeSetError::Stdout)?;
        t.set_mode(TerminalModeOptions::cooked())?;
        Ok(t)
    }

    pub fn set_mode(&mut self, options: TerminalModeOptions) -> Result<(), TerminalModeSetError> {
        use TerminalModeSetError::*;

        let mut stdin_flags = 0;
        let mut stdout_flags = 0;

        let TerminalModeOptions {
            disable_newline_auto_return,
            echo_input,
            line_input,
            processed_input,
            processed_output,
            wrap_at_eol_output,
        } = options;

        macro_rules! map_option {
            ($flags_set:ident, $option:expr, $flag:expr) => {
                if $option {
                    $flags_set |= $flag;
                }
            };
        }
        map_option!(stdin_flags, echo_input, ENABLE_ECHO_INPUT);
        map_option!(stdin_flags, line_input, ENABLE_LINE_INPUT);
        map_option!(stdin_flags, processed_input, ENABLE_PROCESSED_INPUT);
        map_option!(
            stdout_flags,
            disable_newline_auto_return,
            DISABLE_NEWLINE_AUTO_RETURN
        );
        map_option!(stdout_flags, processed_output, ENABLE_PROCESSED_OUTPUT);
        map_option!(stdout_flags, wrap_at_eol_output, ENABLE_WRAP_AT_EOL_OUTPUT);

        self.stdin.0.set_flags(stdin_flags).map_err(Stdin)?;
        self.stdout.0.set_flags(stdout_flags).map_err(Stdout)?;
        Ok(())
    }
}

pub struct TerminalModeOptions {
    disable_newline_auto_return: bool,
    echo_input: bool,
    line_input: bool,
    processed_input: bool,
    processed_output: bool,
    wrap_at_eol_output: bool,
}

impl TerminalModeOptions {
    pub fn cooked() -> Self {
        TerminalModeOptions {
            disable_newline_auto_return: false,
            echo_input: true,
            line_input: true,
            processed_input: true,
            processed_output: true,
            wrap_at_eol_output: true,
        }
    }

    pub fn raw() -> Self {
        TerminalModeOptions {
            disable_newline_auto_return: true,
            echo_input: false,
            line_input: false,
            processed_input: false,
            processed_output: false,
            wrap_at_eol_output: false,
        }
    }
}

#[derive(Debug, Fail)]
pub enum TerminalSetupError {
    #[fail(display = "unable to get stderr: {}", _0)]
    Stderr(io::Error),
    #[fail(display = "unable to get stdin: {}", _0)]
    Stdin(io::Error),
    #[fail(display = "unable to get stdout: {}", _0)]
    Stdout(io::Error),
    #[fail(display = "unable to set up initial terminal state: {}", _0)]
    CouldNotSetInitialTermState(TerminalModeSetError),
}

impl From<TerminalModeSetError> for TerminalSetupError {
    fn from(e: TerminalModeSetError) -> Self {
        TerminalSetupError::CouldNotSetInitialTermState(e)
    }
}

#[derive(Debug, Fail)]
pub enum TerminalModeSetError {
    #[fail(display = "unable to set flags on stdin: {}", _0)]
    Stdin(io::Error),
    #[fail(display = "unable to set flags on stdout: {}", _0)]
    Stdout(io::Error),
}

/// Shamelessly stolen from the Termion codebase. :)
/// See here: https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_sequences
macro_rules! csi {
    ($( $l:expr ),*) => { concat!("\x1B[", $( $l ),*) };
}

pub enum AnsiCsiEscape {
    CursorUp(u16),
    CursorDown(u16),
    CursorForward(u16),
    CursorBack(u16),
    CursorNextLine(Option<u16>),
    CursorPreviousLine(Option<u16>),
    CursorHorizontalAbsolute(u16),
    CursorPosition(u16, u16),
    SaveCursorPosition,
    RestoreCursorPosition,
    ScrollUp(u16),
    ScrollDown(u16),
}
