#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Block,
    Compact,
}

impl Face {
    pub fn glyph_h(self) -> usize {
        match self {
            Face::Block => 7,
            Face::Compact => 5,
        }
    }

    pub fn glyph_w(self) -> usize {
        match self {
            Face::Block => 5,
            Face::Compact => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    pub face: Face,
    pub scale_x: usize,
    pub gap_x: usize,
}

mod block;
mod compact;
mod render;

pub use block::block_glyph;
pub use compact::compact_glyph;
pub use render::{line_width, render_big};
