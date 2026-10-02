#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Block,
}

impl Face {
    pub fn glyph_h(self) -> usize {
        7
    }

    pub fn glyph_w(self) -> usize {
        5
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    pub face: Face,
    pub scale_x: usize,
    pub gap_x: usize,
}

mod block;
mod render;

pub use block::block_glyph;
pub use render::{line_width, render_big};
