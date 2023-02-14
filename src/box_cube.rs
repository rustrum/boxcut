use anyhow::{bail, Result};
use clap_derive::Args;
use lazy_static::lazy_static;
use log::log;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::common::{
    draw_square, ArgsGlobal, Borders, CutType, DrawResult, Origin, Point, Square, SquareElement,
    VIEWPORT_OFFSET,
};

use svg::node::element::Path;

const BOX_CUBE_FIE_NAME: &str = "LaserCutBoxCube.svg";

#[derive(Args, Debug)]
pub struct ArgsBoxCube {
    /// Длинна лепестка для склеивания мм.
    #[clap(long, default_value = "40.0")]
    flap_glue: Decimal,

    /// Высота бортика крышки мм.
    #[clap(long, default_value = "40.0")]
    lid: Decimal,
}

impl ArgsBoxCube {
    pub fn draw_with(self, globs: ArgsGlobal) -> Result<DrawResult> {
        log::debug!("{:?}", self);

        let cfg = BoxCubeCfg::new(self, globs)?;
        log::info!("Коробка-параллелипипед в работе.");

        let bx = BoxCube::new(cfg);
        Ok(bx.draw())
    }
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
    pub fn new(args: ArgsBoxCube, globs: ArgsGlobal) -> Result<Self> {
        if globs.width.is_none() || globs.height.is_none() || globs.length.is_none() {
            bail!("Нужно указать длинну, ширину и высоту коробки в мм.");
        }

        Ok(Self {
            thickness: globs
                .thickness
                .expect("Толщина материала")
                .to_f64()
                .unwrap(),
            glue_flap: args.flap_glue.to_f64().unwrap(),
            lid_height: args.lid.to_f64().unwrap(),
            height: globs.height.unwrap().to_f64().unwrap(),
            length: globs.length.unwrap().to_f64().unwrap(),
            width: globs.width.unwrap().to_f64().unwrap(),
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
        let offset = Point::new(cfg.width + cfg.glue_flap + cfg.thick_n(5), 0.0)
            .shift_xy(VIEWPORT_OFFSET, VIEWPORT_OFFSET);
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

    fn draw_top_lid(&mut self) {
        let cutty = SquareElement::new(self.cfg.thick_n(2), self.cfg.thick_n(1))
            .with_borders(Borders::new_cut());

        let top_flap = SquareElement::new(self.cfg.glue_flap, self.cfg.lid_height).borders(
            CutType::Cut,
            CutType::Nope,
            CutType::Cut,
            CutType::Cut,
        );

        self.result.append(
            top_flap.draw(
                self.offset
                    .shift_nx(self.cfg.thick_n(4))
                    .origin(Origin::TopRight),
            ),
        );

        self.result.append(
            top_flap
                .mirror_vertical()
                .draw(self.offset.shift_x(self.cfg.length + self.cfg.thick_n(4))),
        );

        self.result.append(
            SquareElement::new(self.cfg.length + self.cfg.thick_n(8), self.cfg.lid_height)
                .borders(CutType::Cut, CutType::Bend, CutType::Bend, CutType::Bend)
                .draw(self.offset.shift_nx(self.cfg.thick_n(4))),
        );

        self.offset = self.offset.shift_y(self.cfg.lid_height);

        let side_lid = SquareElement::new(
            self.cfg.lid_height,
            self.cfg.width + self.cfg.thick_n(3),
        )
        .borders(CutType::Cut, CutType::Nope, CutType::Cut, CutType::Cut);

        self.result.append(
            side_lid.draw(
                self.offset
                    .shift_nx(self.cfg.thick_n(3))
                    .shift_y(self.cfg.thick_n(1))
                    .origin(Origin::TopRight),
            ),
        );

        self.result.append(
            side_lid.mirror_vertical().draw(
                self.offset
                    .shift_xy(self.cfg.length + self.cfg.thick_n(3), self.cfg.thick_n(1)),
            ),
        );

        self.result.append(
            SquareElement::new(
                self.cfg.length + self.cfg.thick_n(6),
                self.cfg.width + self.cfg.thick_n(4),
            )
            .borders(CutType::Nope, CutType::Bend, CutType::Bend, CutType::Bend)
            .draw(self.offset.shift_nx(self.cfg.thick_n(3))),
        );

        // Small cut offs
        self.result.append(
            cutty.draw(
                self.offset
                    .shift_nx(self.cfg.thick_n(3))
                    .origin(Origin::TopRight),
            ),
        );

        self.result
            .append(cutty.draw(self.offset.shift_x(self.cfg.length + self.cfg.thick_n(3))));

        self.result.append(
            cutty.draw(
                self.offset
                    .shift_xy(
                        self.cfg.thickness * -1.0,
                        self.cfg.width + self.cfg.thick_n(4),
                    )
                    .origin(Origin::TopRight),
            ),
        );

        self.result.append(cutty.draw(self.offset.shift_xy(
            self.cfg.length + self.cfg.thick_n(1),
            self.cfg.width + self.cfg.thick_n(4),
        )));

        self.offset = self.offset.shift_y(self.cfg.width + self.cfg.thick_n(5));
    }

    fn draw_main_walls(&mut self) {
        let wall = SquareElement::new(
            self.cfg.length + self.cfg.thick_n(2),
            self.cfg.height + self.cfg.thickness,
        )
        .borders(CutType::Nope, CutType::Bend, CutType::Bend, CutType::Bend);

        let wall_bot = SquareElement::new(
            self.cfg.length + self.cfg.thick_n(4),
            self.cfg.height + self.cfg.thick_n(2),
        )
        .borders(CutType::Nope, CutType::Bend, CutType::Bend, CutType::Bend);

        let wall3 = wall.borders(CutType::Nope, CutType::Cut, CutType::Cut, CutType::Cut);

        let glue_flap = SquareElement::new(
            self.cfg.glue_flap + self.cfg.thickness,
            self.cfg.height,
        )
        .borders(CutType::Cut, CutType::Nope, CutType::Cut, CutType::Cut);

        let cut = SquareElement::cut(self.cfg.thick_n(2), self.cfg.thickness);

        self.result
            .append(wall.draw(self.offset.shift_nx(self.cfg.thickness)));
        self.offset = self.offset.shift_y(wall.square.h);

        let offset_flap = self
            .offset
            .shift_nx(self.cfg.thickness)
            .origin(Origin::BottomRight);

        self.result
            .append(glue_flap.draw(offset_flap.shift_ny(self.cfg.thickness)));

        self.result.append(cut.draw(offset_flap));

        let offset_flap = offset_flap
            .shift_x(wall.square.w)
            .origin(Origin::BottomLeft);

        self.result.append(
            glue_flap
                .mirror_vertical()
                .draw(offset_flap.shift_ny(self.cfg.thickness)),
        );

        self.result.append(cut.draw(offset_flap));

        self.result
            .append(wall_bot.draw(self.offset.shift_nx(self.cfg.thick_n(2))));

        self.draw_side_walls();
        self.offset = self.offset.shift_y(wall_bot.square.h);

        self.result.append(
            wall3
                .border_left(CutType::Bend)
                .border_right(CutType::Bend)
                .draw(self.offset.shift_nx(self.cfg.thickness)),
        );

        let offset_flap = self
            .offset
            .shift_y(self.cfg.thickness)
            .shift_nx(self.cfg.thickness)
            .origin(Origin::TopRight);

        self.result.append(glue_flap.draw(offset_flap));

        self.result
            .append(cut.draw(offset_flap.origin(Origin::BottomRight)));

        let offset_flap = offset_flap.shift_x(wall3.square.w);

        self.result.append(
            glue_flap
                .mirror_vertical()
                .draw(offset_flap.origin(Origin::TopLeft)),
        );

        self.result
            .append(cut.draw(offset_flap.origin(Origin::BottomLeft)));
    }

    fn draw_side_walls(&mut self) {
        let height = self.cfg.height + self.cfg.thick_n(2);
        let width = self.cfg.width + self.cfg.thick_n(3);

        let flap = SquareElement::new(self.cfg.glue_flap + self.cfg.thickness, height).borders(
            CutType::Cut,
            CutType::Nope,
            CutType::Cut,
            CutType::Cut,
        );
        let wall = SquareElement::new(width, height).borders(
            CutType::Cut,
            CutType::Nope,
            CutType::Cut,
            CutType::Bend,
        );

        self.result.append(
            wall.draw(
                self.offset
                    .shift_nx(self.cfg.thick_n(2))
                    .origin(Origin::TopRight),
            ),
        );

        self.result.append(
            flap.draw(
                self.offset
                    .shift_nx(width + self.cfg.thick_n(2))
                    .origin(Origin::TopRight),
            ),
        );

        let roffset = self.offset.shift_x(self.cfg.length + self.cfg.thick_n(2));

        self.result.append(wall.mirror_vertical().draw(roffset));

        self.result
            .append(flap.mirror_vertical().draw(roffset.shift_x(width)));
    }
}
