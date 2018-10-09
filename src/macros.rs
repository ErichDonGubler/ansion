#[macro_export]
macro_rules! ansi {
    ($($l: expr),*) => { concat!("\x1B", $($l),*) };
}

/// Shamelessly stolen from the Termion codebase. :)
/// See here: https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_sequences
#[macro_export]
macro_rules! csi {
    ($($l: expr),*) => { ansi!("[", $($l),*) };
}
