#[macro_use]
extern crate ansion;
extern crate failure;

use {
    ansion::prelude::*,
    failure::Error,
    std::{
        thread::sleep,
        time::Duration,
    },
};

fn main() -> Result<(), Error> {
    let mut t = ansi_terminal()?;
    t.set_mode(TerminalModeOptions::raw())?;
    out!(t,
        SwitchToAlternateScreenBuffer,
        Hide,
        ("\n\n\n"),
        PreviousLine(3),

        Rgb(255, 0, 0),
        Underline,
            ("Hay sup!"),
        NextLine(1),
        Reset,

        Magenta,
        Negative,
            ("Does this work?\r\n"),
        Positive,
        NextLine(1),
        Reset,
    );
    sleep(Duration::from_millis(3000));
    out!(t,
        SwitchToMainScreenBuffer,
        Show,
        Green,
            ("Looks like it!"),
        Reset,
    );

    Ok(())
}
