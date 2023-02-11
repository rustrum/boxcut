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

lazy_static! {
    static ref INNER_H: Decimal = Decimal::from(32);
    static ref INNER_L: Decimal = Decimal::from(32);
}

/// SDFDS
#[derive(Args, Debug)]
pub struct ArgsVinyl {
    // /// Толщина коробки (внутри)
    // #[clap(global = true, long)]
    // width: Option<Decimal>,
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
            bail!("Нужно обязательно указать ширину коробки");
        }

        Ok(Self {
            thickness: 2.0,
            glue_flap: 20.0,
            lid_height: 40.0,
            height: 340.0,
            length: 340.0,
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
            result: DrawResult::empty(),
        }
    }

    fn draw(mut self) -> DrawResult {
        self.draw_top_lid();
        self.draw_side_walls();
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

    fn draw_side_walls(&mut self) {
        let height = self.cfg.height + self.cfg.thickness;
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
            CutType::Bend,
            CutType::Bend,
        );

        let flap_bot = SquareElement::new(self.cfg.width, self.cfg.length / 2.0).borders(
            CutType::Nope,
            CutType::Cut,
            CutType::Cut,
            CutType::Cut,
        );

        self.result.append(
            wall.draw(
                self.offset
                    .shift_nx(self.cfg.thickness)
                    .origin(Origin::TopRight),
            ),
        );

        self.result.append(
            flap.draw(
                self.offset
                    .shift_nx(width + self.cfg.thickness)
                    .origin(Origin::TopRight),
            ),
        );

        self.result.append(
            flap_bot.draw(
                self.offset
                    .shift_xy(self.cfg.thick_n(2) * -1.0, height)
                    .origin(Origin::TopRight),
            ),
        );

        let roffset = self.offset.shift_x(self.cfg.length + self.cfg.thickness);

        self.result.append(wall.mirror_vertical().draw(roffset));

        self.result
            .append(flap.mirror_vertical().draw(roffset.shift_x(width)));

        self.result.append(
            flap_bot
                .mirror_vertical()
                .draw(roffset.shift_xy(self.cfg.thickness, height)),
        );
    }

    fn draw_main_walls(&mut self) {
        let wall = SquareElement::new(
            self.cfg.length + self.cfg.thick_n(2),
            self.cfg.height + self.cfg.thickness,
        )
        .borders(CutType::Nope, CutType::Bend, CutType::Bend, CutType::Bend);

        let wall2 = wall
            .height(self.cfg.width + self.cfg.thick_n(2))
            .border_left(CutType::Cut)
            .border_right(CutType::Cut);

        let wall3 = SquareElement::new(self.cfg.length, self.cfg.height + self.cfg.thickness)
            .borders(CutType::Nope, CutType::Cut, CutType::Cut, CutType::Cut);

        self.result
            .append(wall.draw(self.offset.shift_nx(self.cfg.thickness)));
        self.offset = self.offset.shift_y(wall.square.h);

        self.result
            .append(wall2.draw(self.offset.shift_nx(self.cfg.thickness)));
        self.offset = self.offset.shift_y(wall2.square.h);

        self.result.append(wall3.draw(self.offset));
    }
}

/*
     ┌───────┐
   ┌─┼───────┼─┐
   │ │       │ │
 ┌─┴─┼───────┼─┴─┐
 │   │       │   │
 │   │       │   │
 │   │       │   │
 │   │       │   │
 ├──┬┼───────┼┬──┤
 │  ││       ││  │
 └──┴┼───────┼┴──┘
     │       │
     │       │
     │       │
     │       │
     └───────┘

*/
