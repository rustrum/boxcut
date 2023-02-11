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

use crate::common::{draw_square, CutType, Point, VIEWPORT_OFFSET};
use anyhow::Result;
use env_logger::{Builder, Target};
use log::LevelFilter;

#[derive(Subcommand, Debug)]
enum BoxType {
    /// Коробка для виниловых пластинок.
    /// Игнорируется длинна и высота коробки, можно менять только ширину.
    Vinyl(ArgsVinyl),
}

impl BoxType {
    fn draw_with(self, globs: ArgsGlobal) -> Result<DrawResult> {
        log::debug!("{:?}", globs);
        match self {
            BoxType::Vinyl(args) => args.draw_with(globs),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "Laser cut box generator", version)]
struct Args {
    /// Внутреняя высота
    #[clap(global = true, long, short = 'H')]
    height: Option<Decimal>,

    /// Внутреняя длинна (более длинная сторона)
    #[clap(global = true, long, short = 'L')]
    length: Option<Decimal>,

    /// Внутреняя ширина (более короткая сторона)
    #[clap(global = true, long, short = 'W')]
    width: Option<Decimal>,

    /// Толщина картона (2мм по умолчанию)
    #[clap(global = true, long, short = 'T')]
    thickness: Option<Decimal>,

    /// Имя файла (путь к файлу) с результатом.
    /// Если файл существует он будет перезаписан
    #[clap(global = true, long, short = 'F')]
    file: Option<Decimal>,

    /// Рисовать линии изгиба прерывисто.
    /// Что бы не возиться с настройками лазерника и просто резать по всем линиям.
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

    match draw() {
        Err(e) => log::error!("{e}"),
        Ok(d) => write_svg(d),
    };
}

fn draw() -> Result<DrawResult> {
    let args = Args::parse();
    let globs = ArgsGlobal::from(&args)?;
    args.box_type.draw_with(globs)
}

fn write_svg(drawing: DrawResult) {
    log::trace!("DRAW PATHS: \n{:?}", drawing.paths);

    let max = drawing.max.shift_xy(VIEWPORT_OFFSET, VIEWPORT_OFFSET);
    let mut document = Document::new()
        .set("width", format!("{}mm", max.x))
        .set("height", format!("{}mm", max.y))
        .set("viewBox", (0, 0, max.x, max.y));

    for p in drawing.paths {
        document = document.add(p);
    }

    svg::save("image.svg", &document).unwrap();
}
