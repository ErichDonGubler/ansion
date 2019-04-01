use {crate::TerminalOutput, std::io};

#[derive(Clone, Debug)]
pub enum SetGraphicsRenditionEscape {
    Reset,
    Bright,
    Underline,
    NoUnderline,
    Negative,
    Positive,
    ForegroundBlack,
    ForegroundRed,
    ForegroundGreen,
    ForegroundYellow,
    ForegroundBlue,
    ForegroundMagenta,
    ForegroundCyan,
    ForegroundWhite,
    ForegroundExtended(ExtendedColor),
    ForegroundDefault,
    BackgroundBlack,
    BackgroundRed,
    BackgroundGreen,
    BackgroundYellow,
    BackgroundBlue,
    BackgroundMagenta,
    BackgroundCyan,
    BackgroundWhite,
    BackgroundExtended(ExtendedColor),
    BackgroundDefault,
    BrightForegroundBlack,
    BrightForegroundRed,
    BrightForegroundGreen,
    BrightForegroundYellow,
    BrightForegroundBlue,
    BrightForegroundMagenta,
    BrightForegroundCyan,
    BrightForegroundWhite,
    BrightBackgroundBlack,
    BrightBackgroundRed,
    BrightBackgroundGreen,
    BrightBackgroundYellow,
    BrightBackgroundBlue,
    BrightBackgroundMagenta,
    BrightBackgroundCyan,
    BrightBackgroundWhite,
}

impl TerminalOutput for SetGraphicsRenditionEscape {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        macro_rules! w {
            ($code: expr) => {
                write!(f, csi!("{}m"), $code)
            };
        }
        use self::SetGraphicsRenditionEscape::*;
        match self {
            Reset => w!(0),
            Bright => w!(1),
            Underline => w!(4),
            NoUnderline => w!(24),
            Negative => w!(7),
            Positive => w!(27),
            ForegroundBlack => w!(30),
            ForegroundRed => w!(31),
            ForegroundGreen => w!(32),
            ForegroundYellow => w!(33),
            ForegroundBlue => w!(34),
            ForegroundMagenta => w!(35),
            ForegroundCyan => w!(36),
            ForegroundWhite => w!(37),
            ForegroundExtended(e) => {
                write!(f, csi!("38;"))?;
                e.write_color_code(f)?;
                write!(f, "m")
            }
            ForegroundDefault => w!(39),
            BackgroundBlack => w!(40),
            BackgroundRed => w!(41),
            BackgroundGreen => w!(42),
            BackgroundYellow => w!(43),
            BackgroundBlue => w!(44),
            BackgroundMagenta => w!(45),
            BackgroundCyan => w!(46),
            BackgroundWhite => w!(47),
            BackgroundExtended(e) => {
                write!(f, csi!("48;"))?;
                e.write_color_code(f)?;
                write!(f, "m")
            }
            BackgroundDefault => w!(49),
            BrightForegroundBlack => w!(90),
            BrightForegroundRed => w!(91),
            BrightForegroundGreen => w!(92),
            BrightForegroundYellow => w!(93),
            BrightForegroundBlue => w!(94),
            BrightForegroundMagenta => w!(95),
            BrightForegroundCyan => w!(96),
            BrightForegroundWhite => w!(97),
            BrightBackgroundBlack => w!(100),
            BrightBackgroundRed => w!(101),
            BrightBackgroundGreen => w!(102),
            BrightBackgroundYellow => w!(103),
            BrightBackgroundBlue => w!(104),
            BrightBackgroundMagenta => w!(105),
            BrightBackgroundCyan => w!(106),
            BrightBackgroundWhite => w!(107),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColorTableValue(pub u8);

impl ColorTableValue {
    fn write_color_code(&self, f: &mut io::Write) -> io::Result<()> {
        let ColorTableValue(v) = self;
        write!(f, "5;{}", v)
    }
}

impl TerminalOutput for ColorTableValue {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        TerminalOutput::fmt(&ExtendedColor::ColorTable(self.clone()), f)
    }
}

#[derive(Clone, Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    fn write_color_code(&self, f: &mut io::Write) -> io::Result<()> {
        let Rgb(r, g, b) = self;
        write!(f, "2;{};{};{}", r, g, b)
    }
}

impl TerminalOutput for Rgb {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        TerminalOutput::fmt(&ExtendedColor::Rgb(self.clone()), f)
    }
}

#[derive(Clone, Debug)]
pub enum ExtendedColor {
    ColorTable(ColorTableValue),
    Rgb(Rgb),
}

impl ExtendedColor {
    fn write_color_code(&self, f: &mut io::Write) -> io::Result<()> {
        use self::ExtendedColor::*;
        match self {
            ColorTable(ctv) => ctv.write_color_code(f),
            Rgb(rgb) => rgb.write_color_code(f),
        }
    }
}

impl TerminalOutput for ExtendedColor {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        TerminalOutput::fmt(
            &SetGraphicsRenditionEscape::ForegroundExtended(self.clone()),
            f,
        )
    }
}

#[derive(Clone, Debug)]
pub enum PresetColor {
    DefaultColor,
    Black,
    Blue,
    Green,
    Red,
    Cyan,
    Magenta,
    Yellow,
    White,
}

impl TerminalOutput for PresetColor {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        use self::{PresetColor::*, SetGraphicsRenditionEscape::*};
        TerminalOutput::fmt(
            &match self {
                DefaultColor => ForegroundDefault,
                Black => ForegroundBlack,
                Blue => ForegroundBlue,
                Green => ForegroundGreen,
                Red => ForegroundRed,
                Cyan => ForegroundCyan,
                Magenta => ForegroundMagenta,
                Yellow => ForegroundYellow,
                White => ForegroundWhite,
            },
            f,
        )
    }
}

#[derive(Clone, Debug)]
pub struct PresetColorSpec {
    color: PresetColor,
    bright: bool,
}

impl TerminalOutput for PresetColorSpec {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        use self::{PresetColor::*, SetGraphicsRenditionEscape::*};
        TerminalOutput::fmt(
            &match self {
                PresetColorSpec {
                    color,
                    bright: false,
                } => match color {
                    DefaultColor => ForegroundDefault,
                    Black => ForegroundBlack,
                    Blue => ForegroundBlue,
                    Green => ForegroundGreen,
                    Red => ForegroundRed,
                    Cyan => ForegroundCyan,
                    Magenta => ForegroundMagenta,
                    Yellow => ForegroundYellow,
                    White => ForegroundWhite,
                },
                PresetColorSpec {
                    color,
                    bright: true,
                } => match color {
                    DefaultColor => ForegroundDefault,
                    Black => BrightForegroundBlack,
                    Blue => BrightForegroundBlue,
                    Green => BrightForegroundGreen,
                    Red => BrightForegroundRed,
                    Cyan => BrightForegroundCyan,
                    Magenta => BrightForegroundMagenta,
                    Yellow => BrightForegroundYellow,
                    White => BrightForegroundWhite,
                },
            },
            f,
        )
    }
}

#[derive(Clone, Debug)]
pub enum ColorSpec {
    Preset(PresetColorSpec),
    Extended(ExtendedColor),
}

impl TerminalOutput for ColorSpec {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        use self::ColorSpec::*;
        match self {
            Preset(p) => TerminalOutput::fmt(p, f),
            Extended(e) => TerminalOutput::fmt(e, f),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Style {
    underline: bool,
    negative: bool,
}

impl TerminalOutput for Style {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        use self::SetGraphicsRenditionEscape::*;
        let Style {
            underline,
            negative,
        } = self;
        TerminalOutput::fmt(
            &match underline {
                true => Underline,
                false => NoUnderline,
            },
            f,
        )?;
        TerminalOutput::fmt(
            &match negative {
                true => Negative,
                false => Positive,
            },
            f,
        )?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct FontSpec {
    style: Style,
    foreground_color: ColorSpec,
    background_color: ColorSpec,
}

impl TerminalOutput for FontSpec {
    fn fmt(&self, f: &mut io::Write) -> io::Result<()> {
        // OPT: Extracting out the codes for each abstraction would let us just add another
        // semicolon-delimited item, instead of emitting another entire SGR escape
        let FontSpec {
            style,
            foreground_color,
            background_color,
        } = self;

        TerminalOutput::fmt(style, f)?;
        TerminalOutput::fmt(foreground_color, f)?;
        use self::{ColorSpec::*, PresetColor::*, SetGraphicsRenditionEscape::*};
        TerminalOutput::fmt(
            &match background_color {
                Preset(p) => match p {
                    PresetColorSpec {
                        color,
                        bright: false,
                    } => match color {
                        DefaultColor => BackgroundDefault,
                        Black => BackgroundBlack,
                        Blue => BackgroundBlue,
                        Green => BackgroundGreen,
                        Red => BackgroundRed,
                        Cyan => BackgroundCyan,
                        Magenta => BackgroundMagenta,
                        Yellow => BackgroundYellow,
                        White => BackgroundWhite,
                    },
                    PresetColorSpec {
                        color,
                        bright: true,
                    } => match color {
                        DefaultColor => BackgroundDefault,
                        Black => BrightBackgroundBlack,
                        Blue => BrightBackgroundBlue,
                        Green => BrightBackgroundGreen,
                        Red => BrightBackgroundRed,
                        Cyan => BrightBackgroundCyan,
                        Magenta => BrightBackgroundMagenta,
                        Yellow => BrightBackgroundYellow,
                        White => BrightBackgroundWhite,
                    },
                },
                Extended(e) => BackgroundExtended(e.clone()),
            },
            f,
        )?;
        Ok(())
    }
}
