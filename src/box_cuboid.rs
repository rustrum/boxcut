use anyhow::Result;
use clap::{ArgMatches, Command};
use rust_decimal::prelude::ToPrimitive;

use crate::common::args::{cli_help_arg, GlueFlap, Height, Length, Thickness, Width};
use crate::common::{Borders, CutType, DrawResult, Origin, Point, SquareElement, VIEWPORT_OFFSET};
use crate::lid::LidHeight;

const BOX_CUBE_FIE_NAME: &str = "LaserCutBoxCube.svg";

pub const CLI_SUBCOMMAND: &str = "box-cuboid";

pub fn cli_build(root: Command) -> Command {
    let c = Command::new(CLI_SUBCOMMAND)
        .about("Коробка-параллелипипед с крышкой.")
        .arg(cli_help_arg())
        .arg_required_else_help(true)
        .arg(Length::arg())
        .arg(Width::arg())
        .arg(Height::arg())
        .arg(LidHeight::arg())
        .arg(GlueFlap::arg())
        .arg(Thickness::arg())
        ;

    root.subcommand(c)
}

pub fn cli_draw(m: &ArgMatches) -> Result<DrawResult> {
    let cfg = BoxCubeCfg::new(m)?;
    log::info!("Коробка-параллелипипед в работе.");

    let bx = BoxCube::new(cfg);
    Ok(bx.draw())
}

#[derive(Debug)]
pub struct BoxCubeCfg {
    thickness: f64,
    glue_flap: f64,
    lid_height: f64,
    height: f64,
    length: f64,
    width: f64,
}

impl BoxCubeCfg {
    pub fn thick_n(&self, multiply: usize) -> f64 {
        self.thickness * multiply as f64
    }
}

impl BoxCubeCfg {
    pub fn new(m: &ArgMatches) -> Result<Self> {
        Ok(Self {
            thickness: Thickness::extract(m).unwrap().to_f64().unwrap(),
            glue_flap: GlueFlap::extract(m).unwrap().to_f64().unwrap(),
            lid_height: LidHeight::extract(m).unwrap().to_f64().unwrap(),
            height: Height::extract(m).unwrap().to_f64().unwrap(),
            length: Length::extract(m).unwrap().to_f64().unwrap(),
            width: Width::extract(m).unwrap().to_f64().unwrap(),
        })
    }
}

struct BoxCube {
    cfg: BoxCubeCfg,
    offset: Point,
    result: DrawResult,
}

impl BoxCube {
    fn new(cfg: BoxCubeCfg) -> Self {
        // Initial offset
        let offset = Point::new(cfg.height, 0.0).shift_xy(VIEWPORT_OFFSET, VIEWPORT_OFFSET);
        Self {
            cfg,
            offset,
            result: DrawResult::empty(BOX_CUBE_FIE_NAME.into()),
        }
    }

    fn draw(mut self) -> DrawResult {
        self.draw_top_lid();
        self.draw_main_walls();

        self.result
    }

    fn square_cut(&self) -> SquareElement {
        SquareElement::cut(self.cfg.thickness, self.cfg.thickness)
    }

    fn square_cut_w(&self) -> SquareElement {
        SquareElement::cut(self.cfg.thick_n(2), self.cfg.thickness)
    }

    fn draw_top_lid(&mut self) {
        let lid_len = self.cfg.length + self.cfg.thick_n(2);
        let lid_width = self.cfg.width + self.cfg.thick_n(2);
        let offset = self.offset.shift_nx(self.cfg.thick_n(1));

        let top_flap = SquareElement::new(
            lid_len - self.cfg.glue_flap * 2.0,
            self.cfg.lid_height - self.cfg.thickness,
        )
        .with_borders(Borders::new_cut())
        .border_bottom(CutType::Bend);

        self.result
            .append(top_flap.draw(offset.shift_x(self.cfg.glue_flap)));

        let top_flap_side_cut = SquareElement::new(self.cfg.glue_flap, top_flap.square.h)
            .with_borders(Borders::nope())
            .border_bottom(CutType::Cut);

        self.result.append(top_flap_side_cut.draw(offset));

        self.result
            .append(top_flap_side_cut.draw(offset.shift_x(lid_len).origin(Origin::TopRight)));

        let offset = offset.shift_y(top_flap.square.h);

        let lid_front_side = SquareElement::new(lid_len, self.cfg.lid_height).borders(
            CutType::Nope,
            CutType::Cut,
            CutType::Bend,
            CutType::Cut,
        );

        self.result.append(lid_front_side.draw(offset));

        let offset = offset.shift_y(lid_front_side.square.h);

        let lid_top_wall = SquareElement::new(lid_len, lid_width).borders(
            CutType::Nope,
            CutType::Bend,
            CutType::Bend,
            CutType::Bend,
        );

        self.result.append(lid_top_wall.draw(offset));

        let side_flap =
            SquareElement::new(self.cfg.lid_height - self.cfg.thickness, self.cfg.glue_flap)
                .borders(CutType::Cut, CutType::Cut, CutType::Bend, CutType::Cut);

        self.result.append(
            side_flap.draw(
                offset
                    .shift_nx(self.cfg.thickness)
                    .shift_y(self.cfg.thickness)
                    .origin(Origin::BottomRight),
            ),
        );

        self.result.append(
            side_flap.mirror_vertical().draw(
                offset
                    .shift_xy(lid_len + self.cfg.thickness, self.cfg.thickness)
                    .origin(Origin::BottomLeft),
            ),
        );

        let lid_side_wall = SquareElement::new(self.cfg.lid_height, lid_width - self.cfg.thickness)
            .borders(CutType::Nope, CutType::Nope, CutType::Cut, CutType::Cut);

        self.result.append(
            lid_side_wall.draw(offset.shift_y(self.cfg.thick_n(1)).origin(Origin::TopRight)),
        );

        self.result.append(
            lid_side_wall
                .mirror_vertical()
                .draw(offset.shift_xy(lid_len, self.cfg.thickness)),
        );

        // Small cut offs
        self.result
            .append(self.square_cut().draw(offset.origin(Origin::TopRight)));

        self.result
            .append(self.square_cut().draw(offset.shift_x(lid_len)));

        self.offset.y = offset.shift_y(lid_top_wall.square.h).y;
    }

    fn draw_main_walls(&mut self) {
        let vertical_glue_flap = SquareElement::new(
            self.cfg.glue_flap + self.cfg.thickness,
            self.cfg.height - self.cfg.thick_n(2),
        )
        .borders(CutType::Cut, CutType::Nope, CutType::Cut, CutType::Cut);

        let back_wall = SquareElement::new(self.cfg.length - self.cfg.thick_n(2), self.cfg.height)
            .borders(CutType::Nope, CutType::Bend, CutType::Bend, CutType::Bend);

        self.result
            .append(back_wall.draw(self.offset.shift_x(self.cfg.thickness)));

        self.result.append(
            self.square_cut_w().draw(
                self.offset
                    .shift_x(self.cfg.thick_n(1))
                    .origin(Origin::TopRight),
            ),
        );

        self.result.append(
            self.square_cut_w().draw(
                self.offset
                    .shift_x(back_wall.square.w + self.cfg.thick_n(1)),
            ),
        );

        self.result.append(
            vertical_glue_flap.draw(
                self.offset
                    .shift_xy(self.cfg.thickness, self.cfg.thickness)
                    .origin(Origin::TopRight),
            ),
        );

        self.result.append(
            vertical_glue_flap.mirror_vertical().draw(
                self.offset
                    .shift_xy(self.cfg.thickness + back_wall.square.w, self.cfg.thickness),
            ),
        );

        self.result.append(
            self.square_cut_w().draw(
                self.offset
                    .shift_xy(
                        self.cfg.thickness,
                        vertical_glue_flap.square.h + self.cfg.thickness,
                    )
                    .origin(Origin::TopRight),
            ),
        );

        self.result
            .append(self.square_cut_w().draw(self.offset.shift_xy(
                back_wall.square.w + self.cfg.thickness,
                vertical_glue_flap.square.h + self.cfg.thickness,
            )));

        self.offset = self.offset.shift_y(back_wall.square.h);

        let bottom_wall = SquareElement::new(self.cfg.length, self.cfg.width).borders(
            CutType::Nope,
            CutType::Bend,
            CutType::Bend,
            CutType::Bend,
        );

        self.result.append(bottom_wall.draw(self.offset));

        self.draw_side_walls();

        let front_wall = back_wall
            .borders(CutType::Nope, CutType::Cut, CutType::Cut, CutType::Cut)
            .height(self.cfg.height - self.cfg.thickness);

        self.offset = self.offset.shift_y(bottom_wall.square.h);

        self.result.append(
            front_wall
                .border_left(CutType::Bend)
                .border_right(CutType::Bend)
                .draw(self.offset.shift_x(self.cfg.thickness)),
        );

        let offset_flap = self
            .offset
            .shift_y(self.cfg.thickness)
            .shift_x(self.cfg.thickness)
            .origin(Origin::TopRight);

        self.result.append(vertical_glue_flap.draw(offset_flap));

        self.result.append(
            self.square_cut_w()
                .draw(offset_flap.origin(Origin::BottomRight)),
        );

        let offset_flap = offset_flap.shift_x(front_wall.square.w);

        self.result.append(
            vertical_glue_flap
                .mirror_vertical()
                .draw(offset_flap.origin(Origin::TopLeft)),
        );

        self.result.append(
            self.square_cut_w()
                .draw(offset_flap.origin(Origin::BottomLeft)),
        );
    }

    fn draw_side_walls(&mut self) {
        let wall = SquareElement::new(self.cfg.height - self.cfg.thickness, self.cfg.width)
            .borders(CutType::Cut, CutType::Nope, CutType::Cut, CutType::Cut);

        self.result
            .append(wall.draw(self.offset.origin(Origin::TopRight)));

        self.result.append(
            wall.mirror_vertical()
                .draw(self.offset.shift_x(self.cfg.length)),
        );
    }
}
