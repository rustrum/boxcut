use crate::common::{draw_line, CutType, DrawResult, Origin, Point};
use svg::node::element::Path;

#[derive(Debug, Clone, Copy)]
pub struct Square {
    pub h: f64,
    pub w: f64,
}

impl Square {
    pub fn new(w: f64, h: f64) -> Self {
        Self { h, w }
    }

    pub fn add_w(&mut self, delta: f64) {
        self.w += delta;
    }

    pub fn add_h(&mut self, delta: f64) {
        self.h += delta;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Borders {
    pub top: CutType,
    pub right: CutType,
    pub bottom: CutType,
    pub left: CutType,
}

impl Borders {
    pub fn new(top: CutType, right: CutType, bottom: CutType, left: CutType) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn new_cut() -> Self {
        Self::new(CutType::Cut, CutType::Cut, CutType::Cut, CutType::Cut)
    }

    pub fn new_bend() -> Self {
        Self::new(CutType::Bend, CutType::Bend, CutType::Bend, CutType::Bend)
    }
    pub fn nope() -> Self {
        Self::new(CutType::Nope, CutType::Nope, CutType::Nope, CutType::Nope)
    }

    pub fn cut_top(&self) -> bool {
        !self.top.dont_cut()
    }
    pub fn cut_right(&self) -> bool {
        !self.right.dont_cut()
    }
    pub fn cut_bottom(&self) -> bool {
        !self.bottom.dont_cut()
    }
    pub fn cut_left(&self) -> bool {
        !self.left.dont_cut()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SquareElement {
    borders: Borders,
    pub square: Square,
}

impl SquareElement {
    pub fn new(w: f64, h: f64) -> Self {
        Self {
            borders: Borders::nope(),
            square: Square::new(w, h),
        }
    }

    pub fn new_square(side: f64) -> Self {
        Self::new(side, side)
    }

    pub fn with_borders(&self, borders: Borders) -> Self {
        let mut new = self.clone();
        new.borders = borders;
        new
    }
    pub fn borders(&self, top: CutType, right: CutType, bottom: CutType, left: CutType) -> Self {
        let mut new = self.clone();
        new.borders = Borders::new(top, right, bottom, left);
        new
    }

    pub fn border_top(&self, tp: CutType) -> Self {
        let mut new = self.clone();
        new.borders.top = tp;
        new
    }

    pub fn border_right(&self, tp: CutType) -> Self {
        let mut new = self.clone();
        new.borders.right = tp;
        new
    }

    pub fn border_bottom(&self, tp: CutType) -> Self {
        let mut new = self.clone();
        new.borders.bottom = tp;
        new
    }

    pub fn border_left(&self, tp: CutType) -> Self {
        let mut new = self.clone();
        new.borders.left = tp;
        new
    }

    pub fn height(&self, h: f64) -> Self {
        let mut new = self.clone();
        new.square.h = h;
        new
    }

    pub fn width(&self, w: f64) -> Self {
        let mut new = self.clone();
        new.square.w = w;
        new
    }

    pub fn mirror_vertical(&self) -> Self {
        let mut mirrored = self.clone();
        mirrored.borders.left = self.borders.right;
        mirrored.borders.right = self.borders.left;
        mirrored
    }

    pub fn mirror_horisontal(&self) -> Self {
        let mut mirrored = self.clone();
        mirrored.borders.top = self.borders.bottom;
        mirrored.borders.bottom = self.borders.top;
        mirrored
    }

    pub fn draw(&self, offset: Point) -> DrawResult {
        let mut paths = Vec::new();
        let mut from = offset.align_top_left(self.square);
        let mut max = from;

        // TOP
        let to = from.shift_xy(self.square.w, 0.0);
        paths.push(draw_line(from, to, &self.borders.top));
        max.update_max(to);
        from = to;

        // RIGHT
        let to = from.shift_xy(0.0, self.square.h);
        paths.push(draw_line(from, to, &self.borders.right));
        max.update_max(to);
        from = to;

        // BOTTOM
        let to = from.shift_xy(self.square.w * -1.0, 0.0);
        paths.push(draw_line(from, to, &self.borders.bottom));
        max.update_max(to);
        from = to;

        // LEFT
        let to = from.shift_xy(0.0, self.square.h * -1.0);
        paths.push(draw_line(from, to, &self.borders.left));
        max.update_max(to);
        from = to;

        DrawResult::new(paths.into_iter().filter_map(|v| v).collect(), max)
    }
}
