#[macro_use]
extern crate ansion;
extern crate failure;

use {
    ansion::{
        ansi_terminal,
        AnsiEscape,
        AnsiTerminal,
        TerminalModeOptions,
    },
    failure::Error,
    std::{
        thread::sleep,
        time::Duration,
    },
};

fn main() -> Result<(), Error> {
    use AnsiEscape::*;
    let mut t = ansi_terminal()?;
    t.set_mode(TerminalModeOptions::raw())?;
    out!(t, ("\n\n\n"), CursorUp(3), ("Hay sup!"), CursorDown(3));
    t.flush()?;
    sleep(Duration::from_millis(1000));
    Ok(())
}
