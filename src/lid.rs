use anyhow::Result;
use clap::{value_parser, Arg, ArgMatches, Command};
use rust_decimal::{prelude::ToPrimitive, Decimal};

use crate::common::{
    args::{cli_help_arg, GlueFlap, Height, Length, Thickness, Width},
    Borders, CutType, DrawResult, Origin, Point, SquareElement, VIEWPORT_OFFSET,
};

const FILE_NAME_DEFAULT: &str = "LaserCutLid.svg";

pub const CLI_SUBCOMMAND: &str = "lid";

pub struct LidHeight;
impl LidHeight {
    const NAME: &'static str = "lid";

    const DEFAULT: &'static str = "35";

    pub fn arg() -> Arg {
        Arg::new(Self::NAME)
            .long("lid")
            .value_parser(value_parser!(Decimal))
            .default_value(Self::DEFAULT)
            .help("Высота бортика крышки (мм).")
    }

    pub fn extract(m: &ArgMatches) -> Option<Decimal> {
        m.get_one(Self::NAME).copied()
    }
}

pub struct LidBorders;
impl LidBorders {
    const NAME: &'static str = "lid-fat";

    pub fn arg() -> Arg {
        Arg::new(Self::NAME)
            .long("lid-fat")
            .required(false)
            .help("Толстые (двойные) борты у крышки.")
    }

    pub fn extract(m: &ArgMatches) -> bool {
        m.get_flag(Self::NAME)
    }
}

pub fn cli_build(root: Command) -> Command {
    let c = Command::new(CLI_SUBCOMMAND)
        .about("Крышка для коробок")
        .arg(cli_help_arg())
        .arg_required_else_help(true)
        .arg(Height::arg().help("Высота крышки (мм)."))
        .arg(Width::arg().help("Наружная ширина коробки (мм)."))
        .arg(Length::arg().help("Наружная длинна (мм). Тут может быть склейка."))
        .arg(Thickness::arg())
        .arg(GlueFlap::arg())
        .arg(
            Arg::new("fat")
                .long("fat")
                .num_args(0)
                .help("Толстые (двойные) борты у крышки."),
        );

    root.subcommand(c)
}

pub fn cli_draw(m: &ArgMatches) -> Result<DrawResult> {
    let lid = LidForBox {
        ltype: LidType::Separated,
        height: Height::extract(m).unwrap().to_f64().unwrap(),
        box_outer_width: Width::extract(m).unwrap().to_f64().unwrap(),
        box_outer_length: Length::extract(m).unwrap().to_f64().unwrap(),
        thickness: Thickness::extract(m).unwrap().to_f64().unwrap(),
        glue_flap: GlueFlap::extract(m).unwrap().to_f64().unwrap(),
        fat_border: m.get_flag("fat"),
        result: DrawResult::empty(FILE_NAME_DEFAULT.to_string()),
    };

    log::info!("Крышка для коробок в работе");

    Ok(lid.draw())
}

pub enum LidType {
    Joined,
    Glued,
    Separated,
}

pub struct LidForBox {
    ltype: LidType,
    height: f64,
    box_outer_width: f64,
    box_outer_length: f64,
    thickness: f64,
    glue_flap: f64,
    fat_border: bool,
    result: DrawResult,
}

impl LidForBox {
    pub fn thick_n(&self, multiply: usize) -> f64 {
        self.thickness * multiply as f64
    }

    fn square_cut(&self) -> SquareElement {
        SquareElement::cut(self.thickness, self.thickness)
    }

    fn draw(self) -> DrawResult {
        let mut offset = Point::new(VIEWPORT_OFFSET, VIEWPORT_OFFSET);

        offset = offset.shift_x(self.height * if self.fat_border { 2.0 } else { 1.0 });

        self.draw_from(offset).1
    }
    fn draw_from(mut self, offset: Point) -> (Point, DrawResult) {
        let lid_len = self.box_outer_length + self.thick_n(if self.fat_border { 4 } else { 2 });
        let lid_width = self.box_outer_width
            + match self.ltype {
                LidType::Joined => self.thick_n(2),
                LidType::Glued => self.thick_n(3),
                LidType::Separated => self.thick_n(4),
            };

        let long_side_flap = SquareElement::new(lid_len - self.glue_flap * 2.0, self.height)
            .with_borders(Borders::new_cut())
            .border_bottom(CutType::Bend);

        self.result
            .append(long_side_flap.draw(offset.shift_x(self.glue_flap)));

        let glue_flap_side_cut = SquareElement::new(self.glue_flap, long_side_flap.square.h)
            .with_borders(Borders::nope())
            .border_bottom(CutType::Cut);

        self.result.append(glue_flap_side_cut.draw(offset));

        self.result
            .append(glue_flap_side_cut.draw(offset.shift_x(lid_len).origin(Origin::TopRight)));

        let offset = offset.shift_y(long_side_flap.square.h);

        let long_side = SquareElement::new(lid_len, self.height + self.thickness).borders(
            CutType::Nope,
            CutType::Cut,
            CutType::Nope,
            CutType::Cut,
        );

        self.result.append(long_side.draw(offset));

        let mut offset = offset.shift_y(long_side.square.h);

        let top_wall = SquareElement::new(lid_len, lid_width).with_borders(Borders::new_bend());
        self.result.append(top_wall.draw(offset));

        // Small top cuts
        self.result
            .append(self.square_cut().draw(offset.origin(Origin::TopRight)));

        self.result
            .append(self.square_cut().draw(offset.shift_x(lid_len)));

        // Small bottom cuts
        if let LidType::Glued | LidType::Separated = self.ltype {
            self.result.append(
                self.square_cut().draw(
                    offset
                        .origin(Origin::TopRight)
                        .shift_y(top_wall.square.h - self.thickness),
                ),
            );

            self.result.append(
                self.square_cut().draw(
                    offset
                        .shift_x(lid_len)
                        .shift_y(top_wall.square.h - self.thickness),
                ),
            );
        }

        let side_flap = SquareElement::new(self.height, self.glue_flap).borders(
            CutType::Cut,
            CutType::Cut,
            CutType::Bend,
            CutType::Cut,
        );

        self.result.append(
            side_flap.draw(
                offset
                    .shift_nx(self.thickness)
                    .shift_y(self.thickness)
                    .origin(Origin::BottomRight),
            ),
        );

        self.result.append(
            side_flap.mirror_vertical().draw(
                offset
                    .shift_xy(lid_len + self.thickness, self.thickness)
                    .origin(Origin::BottomLeft),
            ),
        );

        let side_wall_h = match self.ltype {
            LidType::Joined | LidType::Glued => lid_width - self.thickness,
            LidType::Separated => lid_width - self.thick_n(2),
        };

        let side_wall = SquareElement::new(self.height + self.thickness, side_wall_h).borders(
            CutType::Nope,
            CutType::Nope,
            if let LidType::Separated = self.ltype {
                CutType::Nope
            } else {
                CutType::Cut
            },
            CutType::Cut,
        );

        self.result
            .append(side_wall.draw(offset.shift_y(self.thick_n(1)).origin(Origin::TopRight)));

        self.result.append(
            side_wall
                .mirror_vertical()
                .draw(offset.shift_xy(lid_len, self.thickness)),
        );

        offset.y += top_wall.square.h;

        // Draw tail for separated lid
        if let LidType::Separated = self.ltype {
            // Flaps for side walls
            let side_flap = side_flap.mirror_horisontal();
            self.result.append(
                side_flap.draw(
                    offset
                        .shift_nx(self.thickness)
                        .shift_ny(self.thickness)
                        .origin(Origin::TopRight),
                ),
            );

            self.result.append(
                side_flap.mirror_vertical().draw(
                    offset
                        .shift_xy(lid_len + self.thickness, -1.0 * self.thickness)
                        .origin(Origin::TopLeft),
                ),
            );

            let long_side = long_side.mirror_horisontal();

            self.result.append(long_side.draw(offset));

            offset.y += long_side.square.h;

            let long_side_flap = long_side_flap.mirror_horisontal();
            let glue_flap_side_cut = glue_flap_side_cut.mirror_horisontal();

            self.result
                .append(long_side_flap.draw(offset.shift_x(self.glue_flap)));

            self.result.append(glue_flap_side_cut.draw(offset));

            self.result
                .append(glue_flap_side_cut.draw(offset.shift_x(lid_len).origin(Origin::TopRight)));

            offset.y += if self.glue_flap > self.height * 2.0 {
                self.glue_flap + self.thickness - self.height
            } else {
                self.height
            }
        }

        (offset, self.result)
    }
}
