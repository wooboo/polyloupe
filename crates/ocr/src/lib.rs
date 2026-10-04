//! Text recognition (OCR) on screenshots.
//!
//! Each platform has its own recognizer; [`group_into_blocks`] then joins the
//! recognized lines into paragraphs so they can be translated with context.

mod blocks;
#[cfg(target_os = "macos")]
mod vision;

pub use blocks::{TextBlock, group_into_blocks};

/// A rectangle in image pixels, origin at the top-left corner.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Rect {
            x,
            y,
            width: self.right().max(other.right()) - x,
            height: self.bottom().max(other.bottom()) - y,
        }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// One line of recognized text.
#[derive(Clone, Debug, PartialEq)]
pub struct TextLine {
    pub text: String,
    pub bounds: Rect,
    pub confidence: f32,
}

/// Recognize the text lines in `image`.
#[cfg(target_os = "macos")]
pub fn recognize(image: &image::RgbaImage) -> anyhow::Result<Vec<TextLine>> {
    vision::recognize(image)
}

#[cfg(not(target_os = "macos"))]
pub fn recognize(_image: &image::RgbaImage) -> anyhow::Result<Vec<TextLine>> {
    anyhow::bail!("text recognition is not implemented on this platform yet")
}
