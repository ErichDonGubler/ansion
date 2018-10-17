use {
    AnsiTerminal,
    TerminalChannelMode,
    TerminalModeOptions,
    TerminalModeSetError,
    TerminalOutput,
    TerminalSetupError,
    escapes::formatting::SetGraphicsRenditionEscape,
    std::{
        io::{
            self,
            stdin,
            stdout,
        },
        os::unix::io::{
            AsRawFd,
            RawFd,
        },
    },
    termios::{
        cfmakeraw,
        tcsetattr,
        Termios,
        TCSANOW,
    },
    try_from::TryFrom,
};

#[derive(Debug)]
pub struct UnixAnsiTerminal {
    stdin: StdInputHandle,
    stdout: StdOutputHandle,
}

#[derive(Debug)]
pub struct TerminalState {
    file_descriptor: RawFd,
    raw_termios: Termios,
    cooked_termios: Termios,
    termios_to_restore: Termios,
}

impl TerminalState {
    fn set_mode(&mut self, mode: TerminalChannelMode) -> io::Result<()> {
        use self::TerminalChannelMode::*;
        let termios = match mode {
            Raw => &self.raw_termios,
            Cooked => &self.cooked_termios,
        };
        tcsetattr(self.file_descriptor, TCSANOW, &termios)
    }
}

impl TryFrom<RawFd> for TerminalState {
    type Err = io::Error;
    fn try_from(fd: RawFd) -> Result<Self, Self::Err> {
        let termios = Termios::from_fd(fd)?;

        let cooked_termios = termios.clone();
        let mut raw_termios = cooked_termios;
        cfmakeraw(&mut raw_termios);
        Ok(TerminalState {
            file_descriptor: fd,
            raw_termios,
            cooked_termios,
            termios_to_restore: termios.clone(),
        })
    }
}

impl Drop for TerminalState {
    fn drop(&mut self) {
        let _ = tcsetattr(self.file_descriptor, TCSANOW, &self.termios_to_restore);
    }
}

#[derive(Debug)]
pub enum Stream {
    Tty(TerminalState),
    NonTty(RawFd),
}

impl From<RawFd> for Stream {
    fn from(fd: RawFd) -> Stream {
        match TerminalState::try_from(fd) {
            Ok(state) => Stream::Tty(state),
            Err(_) => Stream::NonTty(fd),
        }
    }
}

#[derive(Debug)]
pub struct StdInputHandle(pub Stream);
#[derive(Debug)]
pub struct StdOutputHandle(pub Stream);

impl UnixAnsiTerminal {
    pub fn new() -> Result<UnixAnsiTerminal, TerminalSetupError> {
        Ok(UnixAnsiTerminal {
            stdin: StdInputHandle(Stream::from(stdin().as_raw_fd())),
            stdout: StdOutputHandle(Stream::from(stdout().as_raw_fd())),
        })
    }
}

impl AnsiTerminal for UnixAnsiTerminal {
    fn set_mode(&mut self, options: TerminalModeOptions) -> Result<(), TerminalModeSetError> {
        use self::{
            Stream::*,
            TerminalModeSetError::*,
        };

        let TerminalModeOptions {
            stdin: stdin_mode,
            stdout: stdout_mode,
        } = options;

        if let Tty(stdin) = &mut self.stdin.0 {
            stdin.set_mode(stdin_mode).map_err(Stdin)?;
        }
        if let Tty(stdout) = &mut self.stdout.0 {
            stdout.set_mode(stdout_mode).map_err(Stdout)?;
        }
        Ok(())
    }
}

impl Drop for UnixAnsiTerminal {
    fn drop(&mut self) {
        let _ = SetGraphicsRenditionEscape::Reset.fmt(&mut stdout());
    }
}
