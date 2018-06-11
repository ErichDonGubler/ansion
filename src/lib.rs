#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate winapi;

use std::{
    fmt,
    io,
    io::{
        stdout,
        Write,
    },
};

#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
use windows::*;

/// Represents the terminal as a resource and all valid operations that can be used with it.
pub trait AnsiTerminal {
    fn set_mode(&mut self, options: TerminalModeOptions) -> Result<(), TerminalModeSetError>;

    fn write<T: TerminalOutput>(&mut self, t: &T) -> io::Result<()> {
        t.fmt(&mut stdout())
    }

    fn flush(&mut self) -> io::Result<()> {
        stdout().flush()
    }
}

/// Convenience wrapper around `ansi_terminal_with_config` that defaults to all channels set to
/// cooked mode.
pub fn ansi_terminal() -> Result<impl AnsiTerminal, TerminalSetupError> {
    ansi_terminal_with_config(TerminalModeOptions::cooked())
}

/// Constructs an AnsiTerminal instance according to the `TerminalModeOptions` passed.
pub fn ansi_terminal_with_config(
    options: TerminalModeOptions,
) -> Result<impl AnsiTerminal, TerminalSetupError> {
    {
        #[cfg(windows)]
        WindowsAnsiTerminal::new()
    }.and_then(|mut t| {
        t.set_mode(options)?;
        Ok(t)
    })
}

/// Represents an error encountered while constructing an `AnsiTerminal`.
#[derive(Debug, Fail)]
pub enum TerminalSetupError {
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

/// Represents a abstract, coarse-grained mode that one of the standard streams can be set to.
pub enum TerminalChannelMode {
    Cooked,
    Raw,
}

/// Represents an abstract, coarse-grained set of modes for each standard stream that
/// `AnsiTerminal` manipulates.
pub struct TerminalModeOptions {
    stdin: TerminalChannelMode,
    stdout: TerminalChannelMode,
}

impl TerminalModeOptions {
    pub fn cooked() -> Self {
        use TerminalChannelMode::*;
        TerminalModeOptions {
            stdin: Cooked,
            stdout: Cooked,
        }
    }

    pub fn raw() -> Self {
        use TerminalChannelMode::*;
        TerminalModeOptions {
            stdin: Raw,
            stdout: Raw,
        }
    }
}

/// Represents an error encountered when setting the mode on a standard stream.
#[derive(Debug, Fail)]
pub enum TerminalModeSetError {
    #[fail(display = "unable to set flags on stdin: {}", _0)]
    Stdin(io::Error),
    #[fail(display = "unable to set flags on stdout: {}", _0)]
    Stdout(io::Error),
}

macro_rules! ansi {
    ($($l:expr),*) => { concat!("\x1B", $($l),*) };
}

/// Shamelessly stolen from the Termion codebase. :)
/// See here: https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_sequences
macro_rules! csi {
    ($($l:expr),*) => { ansi!("[", $($l),*) };
}

/// Represents the full set of ANSI escapes that are supported cross-platform by this library.
/// For more information for your platform, please see:
/// * Windows: https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences
pub enum AnsiEscape {
    CursorUp(u16),
    CursorDown(u16),
    CursorForward(u16),
    CursorBack(u16),
    CursorNextLine(u16),
    CursorPreviousLine(u16),
    CursorHorizontalAbsolute(u16),
    CursorPosition(u16, u16),
    SaveCursorPosition,
    RestoreCursorPosition,
    ScrollUp(u16),
    ScrollDown(u16),
    InsertLine(u16),
    DeleteLine(u16),
}

/// Represents something that an `AnsiTerminal` can use to manipulate the standard out stream.
pub trait TerminalOutput {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()>;
}

impl TerminalOutput for AnsiEscape {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        macro_rules! write_csi {
            ($($e: expr),*; $($args: expr),*) => {
                write!(f, csi!($($e),*) $(, $args)*)
            }
        }

        use AnsiEscape::*;
        match self {
            CursorUp(x) => write_csi!("{}A"; x),
            CursorDown(x) => write_csi!("{}B"; x),
            CursorForward(x) => write_csi!("{}C"; x),
            CursorBack(x) => write_csi!("{}D"; x),
            CursorNextLine(x) => write_csi!("{}E"; x),
            CursorPreviousLine(x) => write_csi!("{}F"; x),
            CursorHorizontalAbsolute(x) => write_csi!("{}G"; x),
            CursorPosition(x, y) => write_csi!("{};{}H"; x, y),
            SaveCursorPosition => write_csi!("s";),
            RestoreCursorPosition => write_csi!("u";),
            ScrollUp(x) => write_csi!("{}S"; x),
            ScrollDown(x) => write_csi!("{}T"; x),
            InsertLine(x) => write_csi!("{}L"; x),
            DeleteLine(x) => write_csi!("{}M"; x),
        }
    }
}

impl<'a> TerminalOutput for fmt::Arguments<'a> {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        write!(f, "{}", self)
    }
}

/// A convenience macro that simplifies writing streams of `TerminalOutput` implementors to an
/// `AnsiTerminal`.
#[macro_export]
macro_rules! out {
    (@args $t: expr; ($fmt: expr $(, $args: expr)*), $($tail: tt)*) => {
        $t.write(&format_args!($fmt, $($args),*))?;
        out!(@args $t; $($tail)*);
    };
    (@args $t: expr; ($fmt: expr $(, $args: expr)*)) => {
        $t.write(&format_args!($fmt, $($args),*))?;
    };
    (@args $t: expr; $escape: ident ($($args: tt)*), $($tail: tt)*) => {
        $t.write(&$escape($($args)*))?;
        out!(@args $t; $($tail)*);
    };
    (@args $t: expr; $escape: ident ($($args: tt)*)) => {
        $t.write(&$escape($($args)*))?;
    };
    (@args $t: expr; $escape: ident { $($args: tt)* }, $($tail: tt)*) => {
        $t.write(&$escape { $($args)* })?;
        out!(@args $t; $($tail)*);
    };
    (@args $t: expr; $escape: ident { $($args: tt)* }) => {
        $t.write(&$escape { $($args)* })?;
    };
    (@args $t: expr;) => {};
    ($t: expr, $($tail: tt)*) => {
        out!(@args $t; $($tail)*);
    };
}
