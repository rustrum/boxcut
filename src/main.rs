mod box_cube;
mod common;
mod vinyl;

use clap::Parser;
use clap_derive::{Parser, Subcommand};
use common::{ArgsGlobal, Borders, DrawResult, Square};
use rust_decimal::Decimal;
use svg;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;
use vinyl::ArgsVinyl;

use crate::box_cube::ArgsBoxCube;
use crate::common::{draw_square, CutType, Point, DEFAULT_FILE_NAME, VIEWPORT_OFFSET};
use anyhow::{bail, Result};
use env_logger::{Builder, Target};
use log::{log, warn, error, LevelFilter};

#[derive(Subcommand, Debug)]
enum BoxType {
    /// Коробка для виниловых пластинок.
    /// Игнорируется длинна и высота коробки, можно менять только ширину.
    Vinyl(ArgsVinyl),

    /// Коробка-параллелипипед с крышкой.
    BoxCube(ArgsBoxCube),
}

impl BoxType {
    fn draw_with(self, globs: ArgsGlobal) -> Result<DrawResult> {
        log::debug!("{:?}", globs);
        match self {
            BoxType::Vinyl(args) => args.draw_with(globs),
            BoxType::BoxCube(args) => args.draw_with(globs),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "Laser cut box generator", version)]
struct Args {
    /// Наружная высота (мм).
    #[clap(global = true, long, short = 'H')]
    height: Option<Decimal>,

    /// Наружная длинна / более длинная сторона (мм).
    #[clap(global = true, long, short = 'L')]
    length: Option<Decimal>,

    /// Наружная ширина / более короткая сторона (мм).
    #[clap(global = true, long, short = 'W')]
    width: Option<Decimal>,

    /// Толщина картона (2мм по умолчанию).
    #[clap(global = true, long, short = 'T')]
    thickness: Option<Decimal>,

    /// Имя/путь к .SVG файлу с результатом,
    /// если не указано будет создан файл в текущей папке,
    /// существущий файл будет перезаписан.
    #[clap(global = true, long, short = 'F')]
    file: Option<String>,

    /// [НЕ работает пока] Рисовать линии изгиба прерывисто,
    /// что бы не возиться с настройками и просто резать.
    #[clap(global = true, long, short = 'D')]
    dashed: Option<bool>,

    /// Тип коробоки
    #[command(subcommand)]
    box_type: BoxType,
}

impl ArgsGlobal {
    fn from(args: &Args) -> Result<Self> {
        Ok(ArgsGlobal {
            height: args.height,
            length: args.length,
            width: args.width,
            thickness: args.thickness,
            file: args.file.clone(),
        })
    }
}

fn main() {
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info);
    builder.format_timestamp(None);
    builder.format_module_path(false);
    builder.format_target(false);
    builder.init();

    if let Err(e) = execute() {
        log::error!("{e}");
        std::process::exit(42);
    }
}

fn execute() -> Result<()> {
    let args = Args::parse();
    let globs = ArgsGlobal::from(&args)?;

    if globs.thickness.is_none() {
        bail!("Нужно указать толщину материала");
    }

    if globs.width.is_some() && globs.length.is_some() {
        let w = globs.width.unwrap();
        let l = globs.length.unwrap();
        if w > l {
            error!("Ширина коробки {}мм должна быть меньше длинны {}мм", w, l);
        }
    }

    let draw_res = args.box_type.draw_with(globs.clone())?;
    write_svg(globs, draw_res)
}

fn write_svg(args: ArgsGlobal, drawing: DrawResult) -> Result<()> {
    log::trace!("DRAW PATHS: \n{:?}", drawing.paths);

    let max = drawing.max.shift_xy(VIEWPORT_OFFSET, VIEWPORT_OFFSET);
    let mut document = Document::new()
        .set("width", format!("{}mm", max.x))
        .set("height", format!("{}mm", max.y))
        .set("viewBox", (0, 0, max.x, max.y));

    log::info!(
        "Размеры листа:\n - Ширина:{}мм\n - Высота:{}мм ",
        max.x,
        max.y
    );

    for p in drawing.paths {
        document = document.add(p);
    }

    let save_path = match args.file {
        None => {
            log::info!(
                "Используется имя файла по умолчанию {}",
                drawing.default_file_name
            );
            drawing.default_file_name
        }
        Some(f) => {
            if f.to_uppercase().ends_with(".SVG") {
                f
            } else {
                bail!("ДА щаз! Имя файла должно заканчиваться на .svg а ты что ввел?");
            }
        }
    };

    if std::path::Path::new(&save_path).exists() {
        log::debug!("Существующий файл будет перезаписан");
    }

    let res = svg::save(&save_path, &document).map_err(anyhow::Error::from);
    if res.is_ok() {
        log::info!("Файл записан: {}", save_path);
    }
    res
}
