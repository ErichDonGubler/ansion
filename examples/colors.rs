#[macro_use]
extern crate ansion;
extern crate failure;

use {
    ansion::{
        ansi_terminal,
        AnsiColor,
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
    use AnsiColor::*;
    let mut t = ansi_terminal()?;
    t.set_mode(TerminalModeOptions::raw())?;
    out!(
        t,
        ("\n\n\n"),
        CursorUp(3),
        Rgb(255, 0, 0),
        ("Hay sup!\r\n"),
        White,
        ("Does this work?\r\n"),
    );
    sleep(Duration::from_millis(1000));
    out!(
        t,
        Green,
        ("Looks like it!"),
    );

    Ok(())
}
