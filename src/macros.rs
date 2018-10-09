#[macro_export]
macro_rules! ansi {
    ($($l: expr),*) => { concat!("\x1B", $($l),*) };
}

#[macro_export]
macro_rules! csi {
    // Shamelessly stolen from the Termion codebase. :)
    // See here: https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_sequences
    ($($l: expr),*) => { ansi!("[", $($l),*) };
}
