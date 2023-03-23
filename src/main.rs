mod box_cuboid;
mod common;
pub mod lid;
mod vinyl;

use clap::Command;
use common::{args, DrawResult};
use svg;
use svg::Document;

use crate::common::VIEWPORT_OFFSET;
use anyhow::{bail, Result};
use clap::error::ErrorKind;
use common::args::ArgsGlobal;
use env_logger::Builder;
use log::LevelFilter;

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

fn cli_build() -> Command {
    let mut cmd = args::cli_base_args();
    cmd = box_cuboid::cli_build(cmd);
    cmd = vinyl::cli_build(cmd);
    cmd = lid::cli_build(cmd);
    cmd
}

fn execute() -> Result<()> {
    let mut cli = cli_build();
    let matches = cli.clone().try_get_matches().unwrap_or_else(|e| {
        match e.kind() {
            ErrorKind::DisplayHelp
            | ErrorKind::DisplayVersion
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {}
            _ => {
                cli.print_help().unwrap();
                println!("\n(o_O) ОШИБОЧКА!\n");
            }
        }
        e.exit()
    });

    let globs = ArgsGlobal::from_matches(&matches);

    // if globs.width.is_some() && globs.length.is_some() {
    //     let w = globs.width.unwrap();
    //     let l = globs.length.unwrap();
    //     if w > l {
    //         error!("Ширина коробки {}мм должна быть меньше длинны {}мм", w, l);
    //     }
    // }

    let draw_res = match matches.subcommand() {
        Some((vinyl::CLI_SUBCOMMAND, subm)) => vinyl::cli_draw(subm),
        Some((box_cuboid::CLI_SUBCOMMAND, subm)) => box_cuboid::cli_draw(subm),
        Some((lid::CLI_SUBCOMMAND, subm)) => lid::cli_draw(subm),
        _ => {
            log::error!("No subcommand. Should not execute here");
            std::process::exit(42);
        }
    }?;

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
