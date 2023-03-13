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

const INNER_H: f64 = 330.0;
const INNER_L: f64 = 330.0;

const VINYL_FIE_NAME: &str = "LaserCutVinylBox.svg";

#[derive(Args, Debug)]
pub struct ArgsVinyl {
    /// Длинна лепестка для склеивания мм.
    #[clap(long, default_value = "40.0")]
    flap_glue: Decimal,

    /// Высота бортика крышки мм.
    #[clap(long, default_value = "40.0")]
    lid: Decimal,
}

impl ArgsVinyl {
    pub fn draw_with(self, globs: ArgsGlobal) -> Result<DrawResult> {
        log::debug!("{:?}", self);

        let cfg = VinylBoxCfg::new(self, globs)?;
        log::info!("Коробка для винила в работе. Толщина {}mm", cfg.width);

        let bx = VinylBox::new(cfg);
        Ok(bx.draw())
    }
}

#[derive(Debug)]
pub struct VinylBoxCfg {
    thickness: f64,
    glue_flap: f64,
    lid_height: f64,
    height: f64,
    length: f64,
    width: f64,
}

impl VinylBoxCfg {
    pub fn thick_n(&self, multiply: usize) -> f64 {
        self.thickness * multiply as f64
    }
}

impl VinylBoxCfg {
    pub fn new(args: ArgsVinyl, globs: ArgsGlobal) -> Result<Self> {
        if globs.height.is_some() || globs.length.is_some() {
            log::warn!("ВНИМАНИЕ! Введенная длинна и высота коробки игнорируется.")
        }

        if globs.width.is_none() {
            bail!("Нужно обязательно указать ширину коробки в мм.");
        }

        let thickn = globs
            .thickness
            .expect("Толщина материала")
            .to_f64()
            .unwrap();

        Ok(Self {
            thickness: thickn,
            glue_flap: args.flap_glue.to_f64().unwrap(),
            lid_height: args.lid.to_f64().unwrap(),
            height: INNER_H + thickn * 3.0,
            length: INNER_L + thickn * 4.0,
            width: globs.width.unwrap().to_f64().unwrap(),
        })
    }
}

struct VinylBox {
    cfg: VinylBoxCfg,
    offset: Point,
    result: DrawResult,
}

impl VinylBox {
    fn new(cfg: VinylBoxCfg) -> Self {
        // Initial offset
        let offset = Point::new(cfg.width + cfg.glue_flap + cfg.thick_n(5), 0.0)
            .shift_xy(VIEWPORT_OFFSET, VIEWPORT_OFFSET);
        Self {
            cfg,
            offset,
            result: DrawResult::empty(VINYL_FIE_NAME.into()),
        }
    }

    fn draw(mut self) -> DrawResult {
        self.draw_top_lid();
        self.draw_side_walls();
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

        let lid_top_wall = SquareElement::new(lid_len, self.cfg.width + self.cfg.thick_n(2))
            .borders(CutType::Nope, CutType::Bend, CutType::Bend, CutType::Bend);

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

        let lid_side_wall = SquareElement::new(
            self.cfg.lid_height,
            self.cfg.width + self.cfg.thickness,
        )
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

    fn draw_side_walls(&mut self) {
        let offset = self.offset.shift_xy(self.cfg.thickness, self.cfg.thickness);

        let side_wall = SquareElement::new(self.cfg.width, self.cfg.height - self.cfg.thickness)
            .borders(CutType::Cut, CutType::Nope, CutType::Bend, CutType::Bend);

        let flap = SquareElement::new(self.cfg.glue_flap + self.cfg.thickness, side_wall.square.h)
            .borders(CutType::Cut, CutType::Nope, CutType::Cut, CutType::Cut);

        let flap_bot = SquareElement::new(
            self.cfg.width - self.cfg.thick_n(3),
            (self.cfg.length - self.cfg.thick_n(4)) / 2.0,
        )
        .borders(CutType::Nope, CutType::Cut, CutType::Cut, CutType::Cut);

        self.result
            .append(side_wall.draw(offset.origin(Origin::TopRight)));

        self.result
            .append(flap.draw(offset.shift_nx(side_wall.square.w).origin(Origin::TopRight)));

        self.result.append(
            flap_bot.draw(
                offset
                    .shift_nx(self.cfg.thickness)
                    .shift_y(side_wall.square.h)
                    .origin(Origin::TopRight),
            ),
        );

        self.result.append(
            self.square_cut()
                .draw(offset.shift_y(side_wall.square.h).origin(Origin::TopRight)),
        );

        self.result.append(
            self.square_cut_w().draw(
                offset
                    .shift_y(side_wall.square.h)
                    .shift_nx(flap_bot.square.w + self.cfg.thickness)
                    .origin(Origin::TopRight),
            ),
        );

        let roffset = offset.shift_x(self.cfg.length - self.cfg.thick_n(2));

        self.result
            .append(side_wall.mirror_vertical().draw(roffset));

        self.result.append(
            flap.mirror_vertical()
                .draw(roffset.shift_x(side_wall.square.w)),
        );

        self.result.append(
            flap_bot
                .mirror_vertical()
                .draw(roffset.shift_xy(self.cfg.thickness, side_wall.square.h)),
        );

        self.result
            .append(self.square_cut().draw(roffset.shift_y(side_wall.square.h)));

        self.result
            .append(self.square_cut_w().draw(
                roffset.shift_xy(flap_bot.square.w + self.cfg.thickness, side_wall.square.h),
            ));
    }

    fn draw_main_walls(&mut self) {
        let offset = self.offset.shift_x(self.cfg.thickness);

        let back_wall = SquareElement::new(
            self.cfg.length - self.cfg.thick_n(2),
            self.cfg.height + self.cfg.thickness,
        )
        .borders(CutType::Nope, CutType::Bend, CutType::Bend, CutType::Bend);

        self.result
            .append(self.square_cut_w().draw(offset.origin(Origin::TopRight)));
        self.result
            .append(self.square_cut_w().draw(offset.shift_x(back_wall.square.w)));

        self.result.append(back_wall.draw(offset));

        let offset = offset.shift_y(back_wall.square.h);

        let bot_wall = back_wall
            .height(self.cfg.width - self.cfg.thick_n(1))
            .border_left(CutType::Cut)
            .border_right(CutType::Cut);

        self.result.append(bot_wall.draw(offset));

        let offset = offset
            .shift_y(bot_wall.square.h)
            .shift_x(self.cfg.thickness);

        let front_wall = SquareElement::new(
            back_wall.square.w - self.cfg.thick_n(2),
            self.cfg.height,
        )
        .borders(CutType::Nope, CutType::Cut, CutType::Cut, CutType::Cut);

        self.result.append(front_wall.draw(offset));
        self.result
            .append(self.square_cut().draw(offset.origin(Origin::TopRight)));
        self.result
            .append(self.square_cut().draw(offset.shift_x(front_wall.square.w)));

        self.offset.y = offset.shift_y(front_wall.square.h).y;
    }
}
