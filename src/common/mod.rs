use core::option::Option;

use rust_decimal::Decimal;
use svg::node::element::path::{Data, Parameters};
use svg::node::element::Path;

mod square;

pub use square::*;

pub const VIEWPORT_OFFSET: f64 = 10.0;

pub const DEFAULT_THICKNESS: f64 = 2.0;

pub const DEFAULT_FILE_NAME: &str = "LaserCutBox.svg";

#[derive(Debug, Clone)]
pub struct ArgsGlobal {
    pub height: Option<Decimal>,
    pub length: Option<Decimal>,
    pub width: Option<Decimal>,
    pub thickness: Option<Decimal>,
    pub file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DrawResult {
    pub default_file_name: String,
    pub paths: Vec<Path>,
    pub max: Point,
}

impl DrawResult {
    pub fn empty(default_file_name: String) -> Self {
        Self {
            default_file_name,
            paths: Vec::new(),
            max: Point::new(0.0, 0.0),
        }
    }

    pub fn new(paths: Vec<Path>, max: Point) -> Self {
        Self {
            default_file_name: DEFAULT_FILE_NAME.into(),
            paths,
            max,
        }
    }

    pub fn append(&mut self, other: DrawResult) {
        self.paths.extend(other.paths);
        self.max.update_max(other.max)
    }
}

#[derive(Debug, Clone, Copy)]
/// Origin of the coordinates related to element/shape
pub enum Origin {
    /// Default origin
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub origin: Origin,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            origin: Origin::TopLeft,
        }
    }

    /// Add input offset to the current one
    /// Return new struct
    pub fn shift_with(&self, delta: Self) -> Self {
        Self {
            x: self.x + delta.x,
            y: self.y + delta.y,
            origin: self.origin,
        }
    }

    /// Return new offset with adjustment
    pub fn shift_xy(&self, delta_x: f64, delta_y: f64) -> Self {
        Self {
            x: self.x + delta_x,
            y: self.y + delta_y,
            origin: self.origin,
        }
    }

    pub fn shift_x(&self, delta_x: f64) -> Self {
        Self {
            x: self.x + delta_x,
            y: self.y,
            origin: self.origin,
        }
    }

    pub fn shift_nx(&self, x: f64) -> Self {
        self.shift_x(x * -1.0)
    }

    pub fn shift_y(&self, delta_y: f64) -> Self {
        Self {
            x: self.x,
            y: self.y + delta_y,
            origin: self.origin,
        }
    }

    pub fn shift_ny(&self, y: f64) -> Self {
        self.shift_y(y * -1.0)
    }

    pub fn origin(&self, origin: Origin) -> Self {
        Self {
            x: self.x,
            y: self.y,
            origin,
        }
    }

    /// Align point to TopLeft position related to Suare
    pub fn align_top_left(&self, square: Square) -> Self {
        match self.origin {
            Origin::TopLeft => self.clone(),
            Origin::TopRight => self.shift_x(square.w * -1.0).origin(Origin::TopLeft),
            Origin::BottomRight => self
                .shift_xy(square.w * -1.0, square.h * -1.0)
                .origin(Origin::TopLeft),
            Origin::BottomLeft => self.shift_y(square.h * -1.0).origin(Origin::TopLeft),
        }
    }

    /// Treating Self as max coordinates holder.
    /// Update max coordinates from the input
    pub fn update_max(&mut self, input: Point) {
        if self.x < input.x {
            self.x = input.x;
        }
        if self.y < input.y {
            self.y = input.y;
        }
    }

    pub fn as_parameters(&self) -> Parameters {
        Parameters::from(self)
    }
}

impl From<Point> for Parameters {
    fn from(value: Point) -> Self {
        Parameters::from((value.x, value.y))
    }
}

impl From<&Point> for Parameters {
    fn from(value: &Point) -> Self {
        Parameters::from((value.x, value.y))
    }
}

/// Defines laser cuting type
#[derive(Debug, Clone, Copy)]
pub enum CutType {
    /// Do not cut - do not draw
    Nope,
    /// Cut trhought
    Cut,
    /// Cut for bending
    Bend,
}

impl CutType {
    pub fn dont_cut(&self) -> bool {
        match self {
            CutType::Nope => true,
            _ => false,
        }
    }
}

fn path_for(tp: &CutType, data: Data) -> Path {
    let mut stroke = "white";
    let mut width = 0.2;
    match tp {
        CutType::Nope => {}
        CutType::Cut => {
            stroke = "black";
            // width = 0.1;
        }
        CutType::Bend => {
            stroke = "green";
            // width = 0.1;
        }
    }

    Path::new()
        .set("fill", "none")
        .set("stroke", stroke)
        .set("stroke-width", width)
        .set("d", data)
}

pub fn draw_line(from: Point, to: Point, tp: &CutType) -> Option<Path> {
    if tp.dont_cut() {
        return None;
    }

    let data = Data::new()
        .move_to(from.as_parameters())
        .line_to(to.as_parameters());

    Some(path_for(tp, data))
}

// Depracated
pub fn draw_square(offset: Point, shape: Square, borders: Borders) -> Vec<Path> {
    let mut paths = Vec::new();
    let mut from = offset;

    // TOP
    let to = from.shift_xy(shape.w, 0.0);
    paths.push(draw_line(from, to, &borders.top));
    from = to;

    // RIGHT
    let to = from.shift_xy(0.0, shape.h);
    paths.push(draw_line(from, to, &borders.right));
    from = to;

    // BOTTOM
    let to = from.shift_xy(shape.w * -1.0, 0.0);
    paths.push(draw_line(from, to, &borders.bottom));
    from = to;

    // LEFT
    let to = from.shift_xy(0.0, shape.h * -1.0);
    paths.push(draw_line(from, to, &borders.left));
    from = to;

    paths.into_iter().filter_map(|v| v).collect()
}
