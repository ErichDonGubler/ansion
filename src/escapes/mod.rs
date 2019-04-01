pub mod cursor;
pub mod formatting;

use {
    self::{cursor::CursorEscape, formatting::SetGraphicsRenditionEscape},
    crate::TerminalOutput,
    std::io,
};

/// Represents the full set of ANSI escapes that are supported cross-platform by this library.
/// For more information for your platform, please see:
/// * Windows: https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences
#[derive(Clone, Debug)]
pub enum AnsiEscape {
    Cursor(CursorEscape),
    ScrollUp(u16),
    ScrollDown(u16),
    InsertLine(u16),
    DeleteLine(u16),
    SetGraphicsRendition(SetGraphicsRenditionEscape),
    SwitchToAlternateScreenBuffer,
    SwitchToMainScreenBuffer,
}

impl TerminalOutput for AnsiEscape {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        macro_rules! write_csi {
            ($($e: expr),*; $($args: expr),*) => {
                write!(f, csi!($($e),*) $(, $args)*)
            }
        }

        use self::AnsiEscape::*;
        match self {
            // FIXME: use [Simple Cursor
            // Positioning](https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#simple-cursor-positioning)
            // for fewer chars printed?
            Cursor(c) => TerminalOutput::fmt(c, f),
            ScrollUp(x) => write_csi!("{}S"; x),
            ScrollDown(x) => write_csi!("{}T"; x),
            InsertLine(x) => write_csi!("{}L"; x),
            DeleteLine(x) => write_csi!("{}M"; x),
            SetGraphicsRendition(sgr) => TerminalOutput::fmt(sgr, f),
            SwitchToAlternateScreenBuffer => write_csi!("?1049h";),
            SwitchToMainScreenBuffer => write_csi!("?1049l";),
        }
    }
}
