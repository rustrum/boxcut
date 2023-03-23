use anyhow::Result;
use clap::{ArgMatches, Command};
use rust_decimal::prelude::ToPrimitive;

use crate::common::args::{cli_help_arg, GlueFlap, Thickness, Width};
use crate::common::{Borders, CutType, DrawResult, Origin, Point, SquareElement, VIEWPORT_OFFSET};
use crate::lid::LidHeight;

const INNER_H: f64 = 330.0;
const INNER_L: f64 = 330.0;
const STRIPE_H: f64 = 90.0;
const STRIPE_HANDLE_TOP_OFFSET: f64 = 35.0;

const VINYL_FIE_NAME: &str = "LaserCutVinylBox.svg";

pub const CLI_SUBCOMMAND: &str = "vinyl";

pub fn cli_build(root: Command) -> Command {
    let c = Command::new(CLI_SUBCOMMAND)
        .about("Коробка для виниловых пластинок.")
        .arg(cli_help_arg())
        .arg_required_else_help(true)
        .arg(Width::arg())
        .arg(LidHeight::arg())
        .arg(Thickness::arg())
        .arg(GlueFlap::arg());

    root.subcommand(c)
}

pub fn cli_draw(m: &ArgMatches) -> Result<DrawResult> {
    let cfg = VinylBoxCfg::from(m)?;
    log::info!("Коробка для винила в работе.");

    let bx = VinylBox::new(cfg);
    Ok(bx.draw())
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
    pub fn from(m: &ArgMatches) -> Result<Self> {
        let thickn = Thickness::extract(m).unwrap().to_f64().unwrap();
        Ok(Self {
            thickness: Thickness::extract(m).unwrap().to_f64().unwrap(),
            glue_flap: GlueFlap::extract(m).unwrap().to_f64().unwrap(),
            lid_height: LidHeight::extract(m).unwrap().to_f64().unwrap(),
            height: INNER_H + thickn * 3.0,
            length: INNER_L + thickn * 4.0,
            width: Width::extract(m).unwrap().to_f64().unwrap(),
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
        let offset =
            Point::new(cfg.width + cfg.glue_flap, 0.0).shift_xy(VIEWPORT_OFFSET, VIEWPORT_OFFSET);
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
        self.draw_bottom_stripe();

        self.result
    }

    fn square_cut(&self) -> SquareElement {
        SquareElement::cut(self.cfg.thickness, self.cfg.thickness)
    }

    fn square_cut_w(&self) -> SquareElement {
        SquareElement::cut(self.cfg.thick_n(2), self.cfg.thickness)
    }

    fn main_wall_length(&self) -> f64 {
        // задняя стенка минус толщина круговой накладки
        self.cfg.length - self.cfg.thick_n(2)
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

    fn draw_side_walls(&mut self) {
        let offset = self.offset.shift_xy(self.cfg.thickness, self.cfg.thickness);

        let side_wall = SquareElement::new(self.cfg.width, self.cfg.height - self.cfg.thickness)
            .borders(CutType::Cut, CutType::Nope, CutType::Bend, CutType::Bend);

        let flap = SquareElement::new(self.cfg.glue_flap, side_wall.square.h).borders(
            CutType::Cut,
            CutType::Nope,
            CutType::Cut,
            CutType::Cut,
        );

        let flap_bot = SquareElement::new(
            self.cfg.width - self.cfg.thick_n(3),
            (self.cfg.length - self.cfg.thick_n(4)) / 2.0,
        )
        .borders(CutType::Nope, CutType::Cut, CutType::Cut, CutType::Cut);

        let (side_off, handle) = self.handle_hole(true);
        let handle_top_offset = if STRIPE_HANDLE_TOP_OFFSET < self.cfg.lid_height {
            self.cfg.lid_height
        } else {
            STRIPE_HANDLE_TOP_OFFSET
        };

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

        self.result.append(
            handle.draw(
                offset
                    .shift_nx(side_off - self.cfg.thickness)
                    .shift_y(handle_top_offset)
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

        self.result.append(
            handle.draw(
                roffset
                    .shift_x(side_off - self.cfg.thickness)
                    .shift_y(handle_top_offset),
            ),
        );

        let offset_stripe = offset
            .shift_nx(self.cfg.thickness)
            .shift_y(side_wall.square.h)
            .shift_y(flap_bot.square.h);

        self.draw_vertical_half_stripes(self.cfg.length, offset_stripe);
    }

    fn draw_main_walls(&mut self) {
        let offset = self.offset.shift_x(self.cfg.thickness);

        let back_wall = SquareElement::new(
            self.main_wall_length(),
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

    fn draw_bottom_stripe(&mut self) {
        let offset = self.offset.shift_y(5.0);

        let front = SquareElement::cut(
            self.cfg.length - (self.cfg.thick_n(2) + self.cfg.glue_flap * 2.0),
            STRIPE_H,
        );

        self.result
            .append(front.draw(offset.shift_x(self.cfg.glue_flap)));

        /*
        let offset = offset.shift_y(STRIPE_H + 5.0);

        let side = SquareElement::cut(self.cfg.width, STRIPE_H).border_right(CutType::Bend);
        let (side_off, handle) = self.handle_hole(true);

        self.result
            .append(side.draw(offset.origin(Origin::TopRight)));

        self.result.append(
            handle.draw(
                offset
                    .shift_nx(side_off)
                    .shift_y(STRIPE_HANDLE_OFFSET)
                    .origin(Origin::TopRight),
            ),
        );

        self.result
            .append(side.mirror_vertical().draw(offset.shift_x(self.cfg.length)));

        self.result.append(
            handle.draw(
                offset
                    .shift_x(self.cfg.length + side_off)
                    .shift_y(STRIPE_HANDLE_OFFSET),
            ),
        );

        let center = SquareElement::new(self.cfg.length, STRIPE_H)
            .border_top(CutType::Cut)
            .border_bottom(CutType::Cut);

        self.result.append(center.draw(offset));
        */
    }

    fn draw_vertical_half_stripes(&mut self, width: f64, offset: Point) {
        let (side_off, handle) = self.handle_hole(false);

        let top = SquareElement::cut(STRIPE_H, self.cfg.width + self.cfg.thickness)
            .border_bottom(CutType::Bend);
        let center = SquareElement::cut(STRIPE_H, self.cfg.length / 2.0).border_top(CutType::Nope);
        let handle_top_offset = if STRIPE_HANDLE_TOP_OFFSET < self.cfg.lid_height {
            self.cfg.lid_height
        } else {
            STRIPE_HANDLE_TOP_OFFSET
        };

        self.result
            .append(top.draw(offset.origin(Origin::TopRight)));

        self.result
            .append(center.draw(offset.shift_y(top.square.h).origin(Origin::TopRight)));

        self.result.append(top.draw(offset.shift_x(width)));

        self.result
            .append(center.draw(offset.shift_x(width).shift_y(top.square.h)));

        if handle_top_offset < top.square.w {
            // left hole
            self.result.append(
                handle.draw(
                    offset
                        .shift_y(side_off + self.cfg.thickness)
                        .shift_nx(handle_top_offset)
                        .origin(Origin::TopRight),
                ),
            );

            // right hole
            self.result.append(
                handle.draw(
                    offset
                        .shift_y(side_off + self.cfg.thickness)
                        .shift_x(width + handle_top_offset),
                ),
            );
        }
    }

    fn handle_hole(&self, horizontal: bool) -> (f64, SquareElement) {
        let height = 25.0;
        let max_w = 80.0;
        let min_w = 10.0;
        let min_side_offset = 25.0;

        if min_side_offset * min_w < self.cfg.width {
            return (0.0, SquareElement::new(min_w, min_w));
        }

        let mut width = self.cfg.width - min_side_offset * 2.0;
        if width > max_w {
            width = max_w;
        } else if width < min_w {
            width = min_w;
        }

        let side_offset = (self.cfg.width - width) / 2.0;

        if horizontal {
            (side_offset, SquareElement::cut(width, height))
        } else {
            (side_offset, SquareElement::cut(height, width))
        }
    }
}
