//! Turns a screenshot into translatable regions: OCR, paragraph grouping,
//! language detection and the colours to repaint each region with.

use image::{Rgba, RgbaImage};
use polyloupe_ocr::{Rect, TextBlock};
use polyloupe_translate::{Lang, LanguageDetector};

/// A paragraph on screen that can be replaced by its translation.
#[derive(Clone, Debug)]
pub struct Region {
    /// Bounds in screenshot (physical) pixels.
    pub bounds: Rect,
    pub line_height: f32,
    pub original: String,
    pub language: Lang,
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    /// Filled in once translation finishes.
    pub translation: Option<String>,
}

/// Recognize text in `image` and keep the blocks worth translating into `target`.
pub fn find_regions(
    image: &RgbaImage,
    target: Lang,
    detector: &LanguageDetector,
) -> anyhow::Result<Vec<Region>> {
    let lines = polyloupe_ocr::recognize(image)?;
    Ok(polyloupe_ocr::group_into_blocks(lines)
        .into_iter()
        .filter_map(|block| region_for(image, block, target, detector))
        .collect())
}

fn region_for(
    image: &RgbaImage,
    block: TextBlock,
    target: Lang,
    detector: &LanguageDetector,
) -> Option<Region> {
    let original = block.text();
    // Skip numbers, clocks, single-letter labels and the like.
    if original.chars().filter(|c| c.is_alphabetic()).count() < 3 {
        return None;
    }
    let language = detector.detect(&original)?;
    if language == target {
        return None;
    }
    let line_height = block.line_height();
    let pad = (line_height * 0.15).max(2.0);
    let bounds = Rect {
        x: block.bounds.x - pad,
        y: block.bounds.y - pad,
        width: block.bounds.width + 2.0 * pad,
        height: block.bounds.height + 2.0 * pad,
    };
    let background = sample_background(image, &bounds);
    let foreground = pick_foreground(image, &block.bounds, background);
    Some(Region {
        bounds,
        line_height,
        original,
        language,
        background,
        foreground,
        translation: None,
    })
}

/// Median colour of a thin ring just outside `bounds`.
fn sample_background(image: &RgbaImage, bounds: &Rect) -> [u8; 3] {
    let (w, h) = (image.width() as i64, image.height() as i64);
    let mut samples = Vec::new();
    let mut push = |x: f32, y: f32| {
        let (x, y) = (x as i64, y as i64);
        if (0..w).contains(&x) && (0..h).contains(&y) {
            let Rgba([r, g, b, _]) = *image.get_pixel(x as u32, y as u32);
            samples.push([r, g, b]);
        }
    };
    let steps = 64;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = bounds.x + t * bounds.width;
        let y = bounds.y + t * bounds.height;
        push(x, bounds.y - 1.0);
        push(x, bounds.bottom() + 1.0);
        push(bounds.x - 1.0, y);
        push(bounds.right() + 1.0, y);
    }
    if samples.is_empty() {
        return [255, 255, 255];
    }
    samples.sort_by_key(|&[r, g, b]| luminance([r, g, b]).to_bits());
    samples[samples.len() / 2]
}

/// The original text colour when it is readable on `background`, otherwise
/// black or white — whichever contrasts more. For any background one of the
/// two reaches at least 4.58:1, so the result always meets WCAG AA.
fn pick_foreground(image: &RgbaImage, bounds: &Rect, background: [u8; 3]) -> [u8; 3] {
    let mut best = None;
    let mut best_contrast = 0.0;
    let (x0, y0) = (bounds.x.max(0.0) as u32, bounds.y.max(0.0) as u32);
    let x1 = (bounds.right() as u32).min(image.width());
    let y1 = (bounds.bottom() as u32).min(image.height());
    for y in (y0..y1).step_by(2) {
        for x in (x0..x1).step_by(2) {
            let Rgba([r, g, b, _]) = *image.get_pixel(x, y);
            let contrast = contrast_ratio([r, g, b], background);
            if contrast > best_contrast {
                best_contrast = contrast;
                best = Some([r, g, b]);
            }
        }
    }
    match best {
        Some(color) if best_contrast >= 4.5 => color,
        _ if contrast_ratio([0, 0, 0], background)
            >= contrast_ratio([255, 255, 255], background) =>
        {
            [0, 0, 0]
        }
        _ => [255, 255, 255],
    }
}

/// WCAG relative luminance.
pub fn luminance([r, g, b]: [u8; 3]) -> f32 {
    let channel = |c: u8| {
        let c = c as f32 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// WCAG contrast ratio, from 1 to 21.
pub fn contrast_ratio(a: [u8; 3], b: [u8; 3]) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreground_always_meets_wcag_aa() {
        let mut image = RgbaImage::from_pixel(40, 20, Rgba([119, 119, 119, 255]));
        // Low-contrast "text" pixels on a mid-grey background.
        image.put_pixel(20, 10, Rgba([140, 140, 140, 255]));
        let bounds = Rect {
            x: 5.0,
            y: 5.0,
            width: 30.0,
            height: 10.0,
        };
        let background = sample_background(&image, &bounds);
        let foreground = pick_foreground(&image, &bounds, background);
        assert!(contrast_ratio(foreground, background) >= 4.5);
    }

    #[test]
    fn keeps_readable_original_text_colour() {
        let mut image = RgbaImage::from_pixel(40, 20, Rgba([255, 255, 255, 255]));
        for (x, y) in (16..24).flat_map(|x| (8..12).map(move |y| (x, y))) {
            image.put_pixel(x, y, Rgba([20, 40, 120, 255]));
        }
        let bounds = Rect {
            x: 5.0,
            y: 5.0,
            width: 30.0,
            height: 10.0,
        };
        assert_eq!(
            pick_foreground(&image, &bounds, [255, 255, 255]),
            [20, 40, 120]
        );
    }
}
