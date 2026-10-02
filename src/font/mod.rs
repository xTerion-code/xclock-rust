/// Available typefaces.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    /// Large 5×7 block digits (`█`, `▮`).
    Block,
    /// Compact 3×3 plain-ASCII digits (`_`, `|`, `.`).
    Compact,
}

impl Face {
    pub fn glyph_h(self) -> usize {
        match self {
            Face::Block => 7,
            Face::Compact => 3,
        }
    }

    pub fn glyph_w(self) -> usize {
        match self {
            Face::Block => 5,
            Face::Compact => 3,
        }
    }
}

/// How a face is rasterized: which typeface, horizontal pixel doubling
/// (terminal cells are ~2x taller than wide) and inter-glyph gap.
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
