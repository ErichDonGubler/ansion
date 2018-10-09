use {
    std::io::{
        self,
        stdout,
    },
    winapi::{
        shared::{
            minwindef::DWORD,
            ntdef::HANDLE,
        },
        um::{
            consoleapi::{
                GetConsoleMode,
                SetConsoleMode,
            },
            processenv::GetStdHandle,
            winbase::{
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
    AnsiTerminal,
    escapes::formatting::SetGraphicsRenditionEscape,
    TerminalModeOptions,
    TerminalModeSetError,
    TerminalOutput,
    TerminalSetupError,
};

#[derive(Debug)]
pub struct WindowsAnsiTerminal {
    stdin: StdInputHandle,
    stdout: StdOutputHandle,
}

impl WindowsAnsiTerminal {
    pub fn new() -> Result<Self, TerminalSetupError> {
        use TerminalSetupError::*;

        let stdin = StdInputHandle::new().unwrap().map_err(Stdin)?;
        let stdout = StdOutputHandle::new().unwrap().map_err(Stdout)?;

        let mut t = WindowsAnsiTerminal { stdin, stdout };
        if let StreamHandle::Console(out) = &mut t.stdout.0 {
            out.set_flags(ENABLE_VIRTUAL_TERMINAL_PROCESSING)
                .map_err(TerminalModeSetError::Stdout)?;
        }
        Ok(t)
    }
}

impl Drop for WindowsAnsiTerminal {
    fn drop(&mut self) {
        let _ = SetGraphicsRenditionEscape::Reset.fmt(&mut stdout());
    }
}

impl AnsiTerminal for WindowsAnsiTerminal {
    fn set_mode(&mut self, options: TerminalModeOptions) -> Result<(), TerminalModeSetError> {
        use TerminalModeSetError::*;

        let mut stdin_flags = 0;
        let mut stdout_flags = 0;

        let WindowsTerminalMode {
            disable_newline_auto_return,
            echo_input,
            line_input,
            processed_input,
            processed_output,
            wrap_at_eol_output,
        } = WindowsTerminalMode::from(options);

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

        use StreamHandle::*;

        if let Console(c) = &mut self.stdin.0 {
            c.set_flags(stdin_flags).map_err(Stdin)?;
        }
        if let Console(c) = &mut self.stdout.0 {
            c.set_flags(stdout_flags).map_err(Stdout)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ConsoleHandle {
    handle: HANDLE,
    state_to_restore: DWORD,
    state: DWORD,
}

impl ConsoleHandle {
    fn set_flags(&mut self, flags: DWORD) -> io::Result<()> {
        self.state |= flags;
        match unsafe { SetConsoleMode(self.handle, self.state) } {
            0 => Err(io::Error::last_os_error()),
            _ => Ok(()),
        }
    }
}

impl Drop for ConsoleHandle {
    fn drop(&mut self) {
        if unsafe { SetConsoleMode(self.handle, self.state_to_restore) } == 0 {
            warn!("could not reset console state: {}", io::Error::last_os_error());
        }
    }
}

#[derive(Debug)]
pub enum StreamHandle {
    NonConsole {
        handle: HANDLE,
    },
    Console(ConsoleHandle),
}

impl StreamHandle {
    unsafe fn from_std_stream(std_handle: DWORD) -> Result<StreamHandle, io::Error> {
        let handle = GetStdHandle(std_handle);

        let mut state_to_restore = 0;
        if GetConsoleMode(handle, &mut state_to_restore as *mut DWORD) == 0 {
            // FIXME: We probably want to make sure that this is a certain error.
            return Ok(StreamHandle::NonConsole { handle });
            // return Err(io::Error::last_os_error());
        }

        Ok(StreamHandle::Console(ConsoleHandle {
            handle,
            state_to_restore,
            state: state_to_restore,
        }))
    }
}

#[derive(Debug)]
pub struct StdInputHandle(StreamHandle);

impl StdInputHandle {
    pub fn new() -> Option<io::Result<Self>> {
        Some(Ok(StdInputHandle(unsafe {
            match StreamHandle::from_std_stream(STD_INPUT_HANDLE) {
                Ok(ch) => ch,
                Err(e) => return Some(Err(e)),
            }
        })))
    }
}

#[derive(Debug)]
pub struct StdOutputHandle(StreamHandle);

impl StdOutputHandle {
    pub fn new() -> Option<io::Result<Self>> {
        Some(Ok(StdOutputHandle(unsafe {
            match StreamHandle::from_std_stream(STD_OUTPUT_HANDLE) {
                Ok(ch) => ch,
                Err(e) => return Some(Err(e)),
            }
        })))
    }
}

pub struct WindowsTerminalMode {
    disable_newline_auto_return: bool,
    echo_input: bool,
    line_input: bool,
    processed_input: bool,
    processed_output: bool,
    wrap_at_eol_output: bool,
}

impl From<TerminalModeOptions> for WindowsTerminalMode {
    fn from(x: TerminalModeOptions) -> Self {
        use TerminalChannelMode::*;

        let TerminalModeOptions { stdin, stdout } = x;

        let stdin_cooked_flag = match stdin {
            Raw => false,
            Cooked => true,
        };

        let stdout_cooked_flag = match stdout {
            Raw => false,
            Cooked => true,
        };

        WindowsTerminalMode {
            disable_newline_auto_return: !stdin_cooked_flag,
            echo_input: stdin_cooked_flag,
            line_input: stdin_cooked_flag,
            processed_input: stdin_cooked_flag,
            processed_output: stdout_cooked_flag,
            wrap_at_eol_output: stdout_cooked_flag,
        }
    }
}
