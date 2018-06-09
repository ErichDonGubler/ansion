extern crate ansi_escapes;
extern crate failure;

use {
    ansi_escapes::{
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
    let mut t = AnsiTerminal::new()?;
    t.set_mode(TerminalModeOptions::raw());
    println!("\n\n\n\x1B[3AHay sup!\x1B[3B");
    sleep(Duration::from_millis(1000));
    Ok(())
}
