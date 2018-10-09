use {
    std::io,
    TerminalOutput,
};

#[derive(Clone, Debug)]
pub enum CursorEscape {
    Up(u16),
    Down(u16),
    Forward(u16),
    Back(u16),
    NextLine(u16),
    PreviousLine(u16),
    HorizontalAbsolute(u16),
    Position(u16, u16),
    SavePosition,
    RestorePosition,
    EnableBlinking,
    DisableBlinking,
    Show,
    Hide,
}

impl TerminalOutput for CursorEscape {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        macro_rules! write_csi {
            ($($e: expr),*; $($args: expr),*) => {
                write!(f, csi!($($e),*) $(, $args)*)
            }
        }
        use self::CursorEscape::*;
        match self {
            Up(x) => write_csi!("{}A"; x),
            Down(x) => write_csi!("{}B"; x),
            Forward(x) => write_csi!("{}C"; x),
            Back(x) => write_csi!("{}D"; x),
            NextLine(x) => write_csi!("{}E"; x),
            PreviousLine(x) => write_csi!("{}F"; x),
            HorizontalAbsolute(x) => write_csi!("{}G"; x),
            Position(x, y) => write_csi!("{};{}H"; x, y),
            SavePosition => write_csi!("s";),
            RestorePosition => write_csi!("u";),
            EnableBlinking => write_csi!("?12h";),
            DisableBlinking => write_csi!("?12l";),
            Show => write_csi!("?25h";),
            Hide => write_csi!("?25l";),
        }
    }
}
