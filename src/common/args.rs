use clap::{command, value_parser, Arg, ArgAction, ArgMatches, Command};
use rust_decimal::Decimal;
use std::path::PathBuf;

pub struct Length;
impl Length {
    const NAME: &'static str = "length";

    pub fn arg() -> Arg {
        Arg::new(Self::NAME)
            .short('l')
            .value_parser(value_parser!(Decimal))
            .required(true)
            .help("Наружная длинна (мм). Более длинная сторона.")
    }

    pub fn extract(m: &ArgMatches) -> Option<Decimal> {
        m.get_one(Self::NAME).copied()
    }
}

pub struct Width;
impl Width {
    const NAME: &'static str = "width";

    pub fn arg() -> Arg {
        Arg::new(Self::NAME)
            .short('w')
            .value_parser(value_parser!(Decimal))
            .required(true)
            .help("Наружная ширина (мм). Более короткая сторона.")
    }

    pub fn extract(m: &ArgMatches) -> Option<Decimal> {
        m.get_one(Self::NAME).copied()
    }
}

pub struct Height;
impl Height {
    const NAME: &'static str = "height";

    pub fn arg() -> Arg {
        Arg::new(Self::NAME)
            .short('h')
            .value_parser(value_parser!(Decimal))
            .required(true)
            .help("Наружная высота (мм).")
    }

    pub fn extract(m: &ArgMatches) -> Option<Decimal> {
        m.get_one(Self::NAME).copied()
    }
}

pub struct GlueFlap;
impl GlueFlap {
    const NAME: &'static str = "glueflap";

    const DEFAULT: &'static str = "40";

    pub fn arg() -> Arg {
        Arg::new(Self::NAME)
            .long("glue-flap")
            .value_parser(value_parser!(Decimal))
            .default_value(Self::DEFAULT)
            .help("Длинна лепестка для склеивания (мм).")
    }

    pub fn extract(m: &ArgMatches) -> Option<Decimal> {
        m.get_one(Self::NAME).copied()
    }
}

pub struct Thickness;
impl Thickness {
    const NAME: &'static str = "thickness";

    const DEFAULT: &'static str = "2.3";

    pub fn arg() -> Arg {
        Arg::new(Self::NAME)
            .short('t')
            .value_parser(value_parser!(Decimal))
            .default_value(Self::DEFAULT)
            .help("Толщина картона (мм).")
    }

    pub fn extract(m: &ArgMatches) -> Option<Decimal> {
        m.get_one(Self::NAME).copied()
    }
}

struct SaveFile;
impl SaveFile {
    const NAME: &'static str = "file";

    fn arg() -> Arg {
        Arg::new(Self::NAME)
            .short('f')
            .value_parser(value_parser!(PathBuf))
            .global(true)
            .long_help("Имя/путь к .SVG файлу с результатом.\nЕсли не указано будет создан файл в текущей папке, существущий перезапишется.")
    }

    fn extract(m: &ArgMatches) -> Option<PathBuf> {
        m.get_one(Self::NAME).cloned()
    }
}

pub fn cli_help_arg() -> Arg {
    Arg::new("help")
        .short('H')
        .long("help")
        .action(ArgAction::Help)
}

pub fn cli_base_args() -> Command {
    command!()
        .arg_required_else_help(true)
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .subcommand_required(true)
        .subcommand_value_name("МОДЕЛЬ")
        .subcommand_help_heading("Типы моделей")
        .arg(cli_help_arg())
        .arg(SaveFile::arg())
}

#[derive(Debug, Clone)]
pub struct ArgsGlobal {
    pub file: Option<String>,
}

impl ArgsGlobal {
    pub fn from_matches(m: &ArgMatches) -> Self {
        Self {
            file: SaveFile::extract(m).map(|v| v.as_os_str().to_str().unwrap().to_string()),
        }
    }
}
